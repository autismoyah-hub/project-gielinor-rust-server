use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn open_counted_assembly(
        &mut self,
        actor: ActorId,
        request: gielinor_world::counted_assembly::Request,
    ) -> io::Result<()> {
        let token = self
            .world
            .start_number_dialog(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let packet = gielinor_protocol::client_script::run(
            108,
            &[gielinor_protocol::client_script::Argument::String(
                "How many would you like to make?".into(),
            )],
        )
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Missing counted actor"))?;
        active.counted_assembly = Some((token, request));
        active.ui.input_dialog_open = true;
        if let Some(connection) = active.connection {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub(super) fn resume_counted_assembly(
        &mut self,
        actor: ActorId,
        count: i32,
    ) -> io::Result<bool> {
        let Some((token, request)) = self
            .active
            .get_mut(&actor)
            .and_then(|a| a.counted_assembly.take())
        else {
            return Ok(false);
        };
        if self
            .world
            .quantity_dialog_token(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            != Some(token)
        {
            return Ok(false);
        }
        self.active.get_mut(&actor).unwrap().ui.input_dialog_open = false;
        if let Err(error) = self
            .world
            .complete_counted_assembly(actor, request, token, count)
        {
            self.events.push(Event::World(
                gielinor_world::simulation::WorldEvent::ActionRejected { actor, error },
            ));
        }
        Ok(true)
    }
}
