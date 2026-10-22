use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_ordinary_bank_booths(&mut self, names:&crate::startup::Names)->io::Result<()> {
        if !self.connections.is_empty() || self.bank_presentation.is_none()
            || self.npc_bank_parents.is_none() || !self.world.ordinary_bank_booths.is_empty() {
            return Err(io::Error::other("Configure ordinary booths once with shared banking before admission"));
        }
        let booths=crate::content::ordinary_bank_booths(names.group("loc"),names.group("content"),&self.definitions).map_err(io::Error::other)?;
        for &id in &booths {
            if self.world.location_banks.contains_key(&id) || self.world.has_location_handler(id,2) {
                return Err(io::Error::other("Conflicting ordinary bank booth handler"));
            }
        }
        self.world.ordinary_bank_booths=booths;
        Ok(())
    }

    pub fn configure_crafting_guild_bank(
        &mut self,
        names: &crate::startup::Names,
    ) -> io::Result<()> {
        if !self.connections.is_empty()
            || self.bank_presentation.is_none()
            || self.npc_bank_parents.is_none()
        {
            return Err(io::Error::other(
                "Configure location bank with shared banking before admission",
            ));
        }
        let (id, gate) = crate::content::crafting_guild_bank(
            names.group("loc"),
            names.group("stat"),
            names.group("varbit"),
            &self.definitions,
        )
        .map_err(io::Error::other)?;
        if self.world.location_banks.contains_key(&id)
            || self.world.location_handlers.contains_key(&(id, 1))
        {
            return Err(io::Error::other("Conflicting Guild bank handler"));
        }
        self.world.location_banks.insert(id, gate);
        Ok(())
    }
    pub(super) fn open_location_bank(
        &mut self,
        actor: ActorId,
        request: gielinor_world::bank_access::Request,
    ) -> io::Result<()> {
        if self.world.location_bank_request_valid(actor, request) {
            if let Some((main, side)) = self.npc_bank_parents {
                match request.kind() {
                    gielinor_world::bank_access::Kind::Full => self.open_bank(actor, main, side)?,
                    gielinor_world::bank_access::Kind::DepositBox => {
                        self.open_deposit_box(actor)?
                    }
                }
            }
        }
        Ok(())
    }
}
