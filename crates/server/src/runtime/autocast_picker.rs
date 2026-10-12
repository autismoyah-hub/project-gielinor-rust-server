use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_autocast_picker(
        &mut self,
        presentation: crate::autocast_picker::Presentation,
        selection: gielinor_world::autocast_select::Config,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || self.world.combat_wear.is_none() {
            return Err(io::Error::other(
                "Configure autocast picker after combat equipment and before admission",
            ));
        }
        for component in [
            presentation.parent,
            presentation.spells,
            presentation.normal,
            presentation.defensive,
        ] {
            if !self.components.contains_key(&component.0) {
                return Err(io::Error::other(format!(
                    "Missing autocast component {}",
                    component.0
                )));
            }
        }
        self.autocast_picker = Some(presentation);
        self.world.autocast_selection = Some(selection);
        Ok(())
    }
    /// Kotlin Player.ifClose: abort dialogs, clear weak queues, then close modals.
    pub fn close_dialogs_and_modals(&mut self, actor: ActorId) -> io::Result<()> {
        let aborted = self
            .world
            .close_dialogs(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        active.drop_dialog = None;
        active.ui.input_dialog_open = false;
        if aborted {
            if let Some(connection) = active.connection {
                self.transport.send(connection, 117, &[])?;
            }
        }
        let parents: Vec<_> = self.active[&actor]
            .ui
            .modals
            .entries()
            .map(|(parent, _)| parent)
            .collect();
        for parent in parents {
            self.close_interface(actor, parent)?;
        }
        Ok(())
    }
    pub(super) fn autocast_button(
        &mut self,
        actor: ActorId,
        button: gielinor_protocol::buttons::Button,
    ) -> io::Result<()> {
        let Some(config) = self.autocast_picker else {
            return Ok(());
        };
        if button.op != 1 {
            return Ok(());
        }
        let result =
            if button.component == config.normal.0 || button.component == config.defensive.0 {
                self.close_dialogs_and_modals(actor)?;
                self.world
                    .choose_autocast(actor, button.component == config.defensive.0)
            } else if button.component == config.spells.0 {
                self.world.select_autocast(actor, button.slot)
            } else {
                return Ok(());
            };
        result.map_err(|e| io::Error::other(format!("{e:?}")))
    }
    pub(super) fn open_autocast_picker(&mut self, actor: ActorId, weapon: i32) -> io::Result<()> {
        if self.connection(actor).is_none() {
            return Ok(());
        }
        let Some(config) = self.autocast_picker else {
            return Ok(());
        };
        self.set_varp(
            actor,
            config.layout,
            crate::autocast_picker::Presentation::layout_weapon(weapon),
        )?;
        self.open_interface(actor, Some(config.parent), config.interface, true)?;
        self.set_interface_events(actor, config.spells, 0, 58, (1 << 1) | (1 << 17))
    }
    pub(super) fn selected_autocast(
        &mut self,
        actor: ActorId,
        events: &[gielinor_world::autocast::Event],
        restore: bool,
    ) -> io::Result<()> {
        let Some(connection) = self.connection(actor) else {
            return Ok(());
        };
        for event in events {
            let packet = match event {
                gielinor_world::autocast::Event::Variable(u) => {
                    gielinor_protocol::varp::update(u.id, u.value)
                }
                gielinor_world::autocast::Event::Message(message) => {
                    gielinor_protocol::chat::game(0, None, message)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?
                }
            };
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        if restore {
            if let Some(config) = self.autocast_picker {
                self.close_dialogs_and_modals(actor)?;
                self.open_interface(actor, Some(config.parent), config.combat_interface, true)?;
            }
        }
        Ok(())
    }
}
