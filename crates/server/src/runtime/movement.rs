use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn move_click(
        &mut self,
        actor: ActorId,
        destination: gielinor_world::CoordGrid,
        control: bool,
    ) -> io::Result<()> {
        // Mandatory character creation remains modal until Confirm is accepted.
        if self.active.get(&actor).is_some_and(|active|active.design.hidden) {
            return Ok(());
        }
        let err = |e| io::Error::other(format!("{e:?}"));
        let Some(input) = self
            .world
            .prepare_movement_input(actor, destination, control)
            .map_err(err)?
        else {
            if let Some(connection) = self.connection(actor) {
                let packet = gielinor_protocol::game::clear_map_flag();
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
            self.events
                .push(Event::World(WorldEvent::ClearMapFlag(actor)));
            return Ok(());
        };
        let aborted = self.world.close_dialogs(actor).map_err(err)?;
        let pause = self.world.cancel_dialogue(actor).map_err(err)?;
        let active = self.active.get_mut(&actor).unwrap();
        active.drop_dialog = None;
        active.ui.input_dialog_open = false;
        if aborted || pause {
            if let Some(connection) = active.connection {
                self.transport.send(connection, 117, &[])?;
            }
        }
        let parents: Vec<_> = active
            .ui
            .modals
            .entries()
            .map(|(parent, _)| parent)
            .collect();
        for parent in parents {
            self.close_interface(actor, parent)?;
        }
        if let Err(error) = self.world.finish_movement_input(input) {
            self.events
                .push(Event::World(WorldEvent::ActionRejected { actor, error }));
        }
        Ok(())
    }
}
