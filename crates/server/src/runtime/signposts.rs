use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_signposts(&mut self, config: crate::content::Signposts) -> io::Result<()> {
        if !self.connections.is_empty() || self.world.has_location_handler(18493, 1) {
            return Err(io::Error::other(
                "Configure signposts before admission without conflicts",
            ));
        }
        self.world.location_handlers.insert(
            (18493, 1),
            gielinor_world::location_content::Handler::Signpost,
        );
        self.signposts = Some(config);
        Ok(())
    }
    pub(super) fn show_signpost(
        &mut self,
        actor: ActorId,
        location: gielinor_world::locations::Location,
        origin: gielinor_world::CoordGrid,
    ) -> io::Result<()> {
        let Some(config) = self.signposts.clone() else {
            return Ok(());
        };
        let Some(connection) = self.connection(actor) else {
            return Ok(());
        };
        let packets = config
            .packets(origin, location.coords)
            .map_err(io::Error::other)?;
        // Replace the old modal first, so its close hook cannot reset the new view.
        self.open_interface(actor, Some(config.parent), config.interface, false)?;
        let active = self.active.get_mut(&actor).unwrap();
        active.signpost = true;
        active.pending_face = None;
        active.face_npc = None;
        active.face_angle = Some(1024);
        active.face_changed = true;
        for packet in packets {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
}
