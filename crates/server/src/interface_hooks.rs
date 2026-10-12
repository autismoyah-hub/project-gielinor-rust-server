//! Interface open listeners shared by login and subsequent opens.
use gielinor_cache::{enums::EnumType, symbols::Symbols};
use gielinor_protocol::{
    client_script::{self, Argument},
    framing::Packet,
    interfaces,
};
use gielinor_world::ui::{operation_mask, Component, State};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum Action {
    Events {
        component: Component,
        start: i32,
        end: i32,
        mask: u64,
    },
    Script {
        id: i32,
        arguments: Vec<Argument>,
    },
}
#[derive(Default)]
pub struct Hooks {
    pub opens: BTreeMap<u16, Vec<Action>>,
}
impl Hooks {
    /// InvOpenScript enables item operations and targeting on every inventory
    /// slot. Without this hook a native client cannot dispatch equip/eat/drop.
    pub fn inventory(&mut self, component: Component, capacity: usize) -> Result<(), String> {
        if !(1..=65535).contains(&capacity) {
            return Err("Invalid inventory event capacity".into());
        }
        let operations = [2, 3, 4, 6, 7, 10]
            .into_iter()
            .map(|op| operation_mask(op).unwrap())
            .fold(0, |a, b| a | b);
        let targets =
            (11..=17).map(|bit| 1u64 << bit).fold(0, |a, b| a | b) | (1 << 20) | (1 << 21);
        self.opens
            .entry(component.interface())
            .or_default()
            .push(Action::Events {
                component,
                start: 0,
                end: capacity as i32 - 1,
                mask: operations | targets,
            });
        Ok(())
    }
    /// Resolve content names rather than embedding revision-specific component ids.
    pub fn standard(
        interfaces: &Symbols,
        components: &Symbols,
        enums: &Symbols,
        tables: &BTreeMap<u32, EnumType>,
    ) -> Result<Self, String> {
        let symbol = |symbols: &Symbols, name: &str| {
            symbols
                .get(name)
                .copied()
                .ok_or_else(|| format!("Missing interface listener symbol {name}"))
        };
        let component = |name: &str| symbol(components, name).map(|id| Component(id as u32));
        let mut hooks = Self::default();
        let mut insert = |name: &str, actions| -> Result<(), String> {
            let id =
                u16::try_from(symbol(interfaces, name)?).map_err(|_| "Invalid interface id")?;
            hooks.opens.insert(id, actions);
            Ok(())
        };
        let events = |component, start, end, ops: u8| Action::Events {
            component,
            start,
            end,
            mask: (1..=ops)
                .map(|op| operation_mask(op).unwrap())
                .fold(0, |a, b| a | b),
        };
        insert(
            "prayerbook",
            vec![events(component("prayerbook:filtermenu")?, 0, 4, 1)],
        )?;
        insert(
            "account_summary_sidepanel",
            vec![events(
                component("account_summary_sidepanel:summary_click_layer")?,
                3,
                7,
                4,
            )],
        )?;
        let emotes = tables
            .get(&(symbol(enums, "emote_names")? as u32))
            .ok_or("Missing emote enum")?;
        let first = *emotes.values.keys().next().ok_or("Empty emote enum")?;
        let last = *emotes.values.keys().next_back().unwrap();
        insert(
            "emote",
            vec![events(component("emote:contents")?, first, last, 3)],
        )?;
        let mut settings = Vec::new();
        for (name, start, end) in [
            ("music_bobble_container", 0, 21),
            ("sound_bobble_container", 0, 21),
            ("areasounds_bobble_container", 0, 21),
            ("master_bobble_container", 0, 21),
            ("attack_priority_player_buttons", 1, 5),
            ("attack_priority_npc_buttons", 1, 4),
            ("display_dynamic_setting_1_buttons", 1, 3),
            ("brightness_bobble_container", 0, 21),
            ("zoom_slider", 0, 21),
        ] {
            settings.push(events(
                component(&format!("settings_side:{name}"))?,
                start,
                end,
                1,
            ));
        }
        settings.push(Action::Script {
            id: 605,
            arguments: [128, 896, 128, 896].map(Argument::Int).to_vec(),
        });
        insert("settings_side", settings)?;
        Ok(hooks)
    }
    /// Prepare all output before publishing event permissions, so failure is atomic.
    pub fn opened(&self, interface: u16, ui: &mut State) -> Result<Vec<Packet>, String> {
        let mut next = ui.events.clone();
        let mut packets = Vec::new();
        for action in self.opens.get(&interface).into_iter().flatten() {
            match action {
                Action::Events {
                    component,
                    start,
                    end,
                    mask,
                } => {
                    let packet = interfaces::events(component.0, *start, *end, *mask)
                        .map_err(|e| format!("{e:?}"))?;
                    next.add(*component, *start, *end, *mask)
                        .map_err(str::to_owned)?;
                    packets.push(packet);
                }
                Action::Script { id, arguments } => {
                    packets.push(client_script::run(*id, arguments).map_err(|e| format!("{e:?}"))?)
                }
            }
        }
        ui.events = next;
        Ok(packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_open_matches_native_kotlin_capture_and_bounds_permissions() {
        let component = Component::new(149, 0);
        let mut hooks = Hooks::default();
        hooks.inventory(component, 28).unwrap();
        let mut ui = State::default();
        let packets = hooks.opened(149, &mut ui).unwrap();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].opcode, 108);
        // Captured from isolated Kotlin A006, InvOpenScript, revision 239.1.
        assert_eq!(
            packets[0].payload,
            [0x00, 0x9b, 0, 0, 0x02, 0x6e, 0xf8, 0, 0, 0x33, 0, 0, 0x95, 0, 0, 0]
        );
        for slot in [0, 27] {
            for op in 1..=10 {
                assert_eq!(
                    ui.events
                        .enabled(component, slot, 0, operation_mask(op).unwrap()),
                    [2, 3, 4, 6, 7, 10].contains(&op)
                );
            }
        }
        assert!(!ui
            .events
            .enabled(component, 28, 0, operation_mask(3).unwrap()));
        assert!(!ui
            .events
            .enabled(component, -1, 0, operation_mask(3).unwrap()));
    }
}
