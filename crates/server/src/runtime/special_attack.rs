use super::*;
use gielinor_world::ui::{operation_mask, Component, State};

/// Static bar/orb targets use child=-1. Interaction Op1 is retained in the low
/// word; canonical operation_mask supplies op1 in the distinct operation word.
pub(super) fn opened(buttons: &[u32], interface: u16, ui: &mut State) -> io::Result<Vec<Packet>> {
    let mask = (1 << 1) | operation_mask(1).unwrap();
    let mut packets = Vec::new();
    for &component in buttons.iter().filter(|&&id| id >> 16 == u32::from(interface)) {
        let packet = gielinor_protocol::interfaces::events(component, -1, -1, mask)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        ui.events.add(Component(component), -1, -1, mask).map_err(io::Error::other)?;
        packets.push(packet);
    }
    Ok(packets)
}

impl<B: AccountBackend> Runtime<B> {
    pub fn configure_special_attacks(&mut self, config: crate::special_attack_config::Config) -> io::Result<()> {
        if !self.connections.is_empty() || self.world.special_attacks.is_some()
            || config.buttons.iter().any(|id| !self.components.contains_key(id))
        {
            return Err(io::Error::other("Configure known special controls once before admission"));
        }
        config.world.validate().map_err(io::Error::other)?;
        self.special_buttons = config.buttons;
        self.world.special_attacks = Some(config.world);
        Ok(())
    }
}

#[cfg(test)]
mod special_attack_event_tests {
    use super::*;
    use gielinor_protocol::buffer::{Order, Reader};
    use gielinor_world::ui::ButtonRoute;

    #[test]
    fn static_bar_and_orb_packets_grant_op1_at_the_real_minus_one_child() {
        for (interface, child) in [(593, 39), (160, 36)] {
            let component = Component((u32::from(interface) << 16) | child);
            let mut ui = State::default();
            ui.overlays.set(Component::new(548, 24), interface);
            assert_eq!(ui.button(component, -1, 0, 1).unwrap(), ButtonRoute::Ignored);
            let packets = opened(&[component.0], interface, &mut ui).unwrap();
            assert_eq!(packets.len(), 1);
            assert_eq!(packets[0].opcode, 108);
            let mut reader = Reader::new(&packets[0].payload);
            assert_eq!(reader.u16_add(false).unwrap(), 65535, "client last-child sentinel=-1");
            let native_operations = reader.u32().unwrap();
            assert_eq!(native_operations, 1, "native op1 is upper-word bit0, not bit1");
            assert_eq!(reader.u32_order(Order::Middle).unwrap(), 2, "legacy interaction Op1 flag");
            assert_eq!(reader.u16().unwrap(), 65535, "client first-child sentinel=-1");
            assert_eq!(reader.u32_order(Order::InverseMiddle).unwrap(), component.0);
            reader.finish().unwrap();
            assert_eq!(ui.button(component, -1, 0, 1).unwrap(), ButtonRoute::Overlay);
            for op in 2..=10 {
                assert_eq!(ui.button(component, -1, 0, op).unwrap(), ButtonRoute::Ignored);
            }
            assert_eq!(ui.button(Component(component.0 + 1), -1, 0, 1).unwrap(), ButtonRoute::Ignored);
            if interface == 593 {
                assert_eq!(ui.button(Component((593 << 16) | 38), -1, 0, 1).unwrap(), ButtonRoute::Ignored,
                    "the symbol's parent container is not the operative bar control");
            }
            if interface == 160 {
                assert_eq!(ui.button(Component((160 << 16) | 35), -1, 0, 1).unwrap(), ButtonRoute::Ignored,
                    "the orb sprite is not the operative control");
            }
        }
    }
}
