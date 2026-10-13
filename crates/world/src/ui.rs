//! Interface event ranges and button routing from UserInterfaceMap/InterfaceEvents.
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Component(pub u32);
impl Component {
    pub fn new(interface: u16, child: u16) -> Self {
        Self(((interface as u32) << 16) | child as u32)
    }
    pub fn interface(self) -> u16 {
        (self.0 >> 16) as u16
    }
    pub fn child(self) -> u16 {
        self.0 as u16
    }
}
#[derive(Debug, Clone, Copy)]
struct Range {
    component: u16,
    start: i32,
    end: i32,
    events: u64,
}
#[derive(Debug, Clone, Default)]
pub struct Events {
    ranges: BTreeMap<u16, Vec<Range>>,
    count: usize,
}
impl Events {
    /// The -1 marker expands to an unbounded endpoint, including -1..-1.
    pub fn add(
        &mut self,
        component: Component,
        start: i32,
        end: i32,
        events: u64,
    ) -> Result<(), &'static str> {
        if self.count >= 4096 {
            return Err("Interface event range limit");
        }
        self.ranges
            .entry(component.interface())
            .or_default()
            .push(Range {
                component: component.child(),
                start: if start == -1 { i32::MIN } else { start },
                end: if end == -1 { i32::MAX } else { end },
                events,
            });
        self.count += 1;
        Ok(())
    }
    /// Last covering range replaces the whole mask, including an explicit zero.
    pub fn get(&self, component: Component, slot: i32) -> Option<u64> {
        self.ranges
            .get(&component.interface())?
            .iter()
            .rev()
            .find(|range| {
                range.component == component.child() && slot >= range.start && slot <= range.end
            })
            .map(|range| range.events)
    }
    pub fn clear(&mut self, interface: u16) {
        if let Some(ranges) = self.ranges.remove(&interface) {
            self.count -= ranges.len();
        }
    }
    pub fn enabled(&self, component: Component, slot: i32, baked_events: i32, mask: u64) -> bool {
        if let Some(runtime) = self.get(component, slot) {
            return runtime & mask != 0;
        }
        if slot != -1 {
            return false;
        }
        // IF3's baked 24-bit events retain legacy operation bits 1..10.
        let legacy = (1..=10)
            .find(|op| mask == 1u64 << (31 + op))
            .map_or(mask, |op| 1u64 << op);
        (baked_events as i64 as u64) & legacy != 0
    }
}
pub fn operation_mask(op: u8) -> Option<u64> {
    (1..=32).contains(&op).then(|| 1u64 << (31 + op as u32))
}
#[derive(Debug, Clone, Default)]
pub struct Targets(Vec<(Component, u16)>);
impl Targets {
    pub fn entries(&self) -> impl Iterator<Item = (Component, u16)> + '_ {
        self.0.iter().copied()
    }
    pub fn set(&mut self, parent: Component, interface: u16) -> Option<u16> {
        if let Some((_, old)) = self.0.iter_mut().find(|(key, _)| *key == parent) {
            return Some(std::mem::replace(old, interface));
        }
        self.0.push((parent, interface));
        None
    }
    pub fn get(&self, parent: Component) -> Option<u16> {
        self.0
            .iter()
            .find(|(key, _)| *key == parent)
            .map(|(_, value)| *value)
    }
    pub fn contains_interface(&self, interface: u16) -> bool {
        self.0.iter().any(|(_, value)| *value == interface)
    }
    pub fn parent(&self, interface: u16) -> Option<Component> {
        self.0
            .iter()
            .find(|(_, value)| *value == interface)
            .map(|(key, _)| *key)
    }
    pub fn remove(&mut self, parent: Component) -> Option<u16> {
        let index = self.0.iter().position(|(key, _)| *key == parent)?;
        let value = self.0.remove(index).1;
        // Fastutil's default zero is also returned when a stored interface 0 is removed.
        (value != 0).then_some(value)
    }
}
#[derive(Debug, Clone, Default)]
pub struct State {
    pub gameframe: BTreeMap<Component, Component>,
    pub frame_resizable: bool,
    pub frame_width: u16,
    pub frame_height: u16,
    pub top_level: Option<u16>,
    pub overlays: Targets,
    pub modals: Targets,
    pub events: Events,
    pub input_dialog_open: bool,
    pub modal_button_protected: bool,
    /// ResumePauseButtonHandler closes these at the next player main pass.
    pub close_queue: Vec<Component>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRoute {
    Ignored,
    Overlay,
    Modal,
    ProtectedModal,
}
impl State {
    pub fn queue_pause_close(&mut self, component: Component, slot: i32, baked: i32) -> bool {
        if !self.events.enabled(component, slot, baked, 1) {
            return false;
        }
        let Some(parent) = self
            .modals
            .parent(component.interface())
            .or_else(|| self.overlays.parent(component.interface()))
        else {
            return false;
        };
        if !self.close_queue.contains(&parent) {
            self.close_queue.push(parent);
        }
        true
    }
    pub fn translate(&self, component: Component) -> Component {
        self.gameframe.get(&component).copied().unwrap_or(component)
    }
    pub fn drag(
        &mut self,
        source: Component,
        source_slot: i32,
        source_baked: i32,
        target: Component,
        target_slot: i32,
        target_baked: i32,
    ) -> ButtonRoute {
        let modal = self.modals.contains_interface(source.interface());
        if !modal && !self.overlays.contains_interface(source.interface()) {
            return ButtonRoute::Ignored;
        }
        if source.interface() != target.interface()
            && !self.modals.contains_interface(target.interface())
            && !self.overlays.contains_interface(target.interface())
        {
            return ButtonRoute::Ignored;
        }
        if !self
            .events
            .enabled(source, source_slot, source_baked, 1 << 20)
            || !self
                .events
                .enabled(target, target_slot, target_baked, 1 << 20)
        {
            return ButtonRoute::Ignored;
        }
        if !modal {
            return ButtonRoute::Overlay;
        }
        self.input_dialog_open = false;
        if self.modal_button_protected {
            ButtonRoute::ProtectedModal
        } else {
            ButtonRoute::Modal
        }
    }
    /// Remove the selected subinterface followed by its overlay descendants.
    /// Callers choose whether these closures also send packets (replacement opens do not).
    pub fn close_sub(&mut self, target: Component) -> Vec<(Component, u16)> {
        let Some(interface) = self
            .modals
            .remove(target)
            .or_else(|| self.overlays.remove(target))
        else {
            return vec![];
        };
        let mut closed = vec![(target, interface)];
        self.events.clear(interface);
        let children: Vec<_> = self
            .overlays
            .entries()
            .filter(|(parent, _)| parent.interface() == interface)
            .collect();
        for (parent, _) in children {
            closed.extend(self.close_sub(parent));
        }
        closed
    }
    pub fn open_sub(
        &mut self,
        parent: Component,
        interface: u16,
        overlay: bool,
    ) -> Result<Vec<(Component, u16)>, &'static str> {
        if self.modals.0.len() + self.overlays.0.len() >= 128
            && self.modals.get(parent).is_none()
            && self.overlays.get(parent).is_none()
        {
            return Err("Open interface limit");
        }
        let closed = self.close_sub(parent);
        // Opening a replacement page cancels the previous page's queued close.
        self.close_queue.retain(|target| *target != parent);
        if overlay {
            self.overlays.set(parent, interface);
        } else {
            self.modals.set(parent, interface);
        }
        Ok(closed)
    }
    pub fn button(
        &mut self,
        component: Component,
        slot: i32,
        baked_events: i32,
        op: u8,
    ) -> Result<ButtonRoute, &'static str> {
        if !(1..=10).contains(&op) {
            return Err("Unhandled IF3 operation");
        }
        if !self
            .events
            .enabled(component, slot, baked_events, operation_mask(op).unwrap())
        {
            return Ok(ButtonRoute::Ignored);
        }
        let interface = component.interface();
        if self.top_level == Some(interface) || self.overlays.contains_interface(interface) {
            return Ok(ButtonRoute::Overlay);
        }
        if self.modals.contains_interface(interface) {
            self.input_dialog_open = false;
            return Ok(if self.modal_button_protected {
                ButtonRoute::ProtectedModal
            } else {
                ButtonRoute::Modal
            });
        }
        Ok(ButtonRoute::Ignored)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn static_fallback_dynamic_permissions_and_zero_override() {
        let comp = Component::new(149, 0);
        let mut events = Events::default();
        let op = operation_mask(2).unwrap();
        assert!(events.enabled(comp, -1, 4, op));
        assert!(!events.enabled(comp, 0, 4, op));
        events.add(comp, -1, -1, 0).unwrap();
        assert!(!events.enabled(comp, -1, 4, op));
        events.add(comp, 0, 27, op).unwrap();
        assert!(events.enabled(comp, 27, 0, op));
        assert!(!events.enabled(comp, 28, 4, op));
        events.clear(149);
        assert!(events.enabled(comp, -1, 4, op));
    }
    #[test]
    fn only_open_interfaces_dispatch_and_protected_modal_closes_input() {
        let comp = Component::new(12, 4);
        let mut ui = State::default();
        assert_eq!(ui.button(comp, -1, 2, 1), Ok(ButtonRoute::Ignored));
        ui.modals.set(Component::new(548, 20), 12);
        ui.modal_button_protected = true;
        ui.input_dialog_open = true;
        assert_eq!(ui.button(comp, -1, 2, 1), Ok(ButtonRoute::ProtectedModal));
        assert!(!ui.input_dialog_open);
        ui.top_level = Some(12);
        assert_eq!(ui.button(comp, -1, 2, 1), Ok(ButtonRoute::Overlay));
        assert!(ui.button(comp, -1, 2, 11).is_err());
    }
    #[test]
    fn pause_closes_only_enabled_open_interfaces_and_replacement_cancels_close() {
        let parent = Component::new(548, 23);
        let button = Component::new(229, 4);
        let mut ui = State::default();
        assert!(!ui.queue_pause_close(button, -1, 1));
        ui.open_sub(parent, 229, false).unwrap();
        assert!(!ui.queue_pause_close(button, -1, 0));
        ui.events.add(button, -1, -1, 1).unwrap();
        assert!(ui.queue_pause_close(button, -1, 0));
        assert!(ui.queue_pause_close(button, -1, 0));
        assert_eq!(ui.close_queue, [parent]);
        ui.open_sub(parent, 229, false).unwrap();
        assert!(ui.close_queue.is_empty());
        ui.open_sub(parent, 229, true).unwrap();
        assert!(ui.queue_pause_close(button, -1, 1));
        assert_eq!(ui.close_queue, [parent]);
    }
}
