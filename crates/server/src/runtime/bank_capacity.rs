use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn ensure_bank_capacity(&mut self, actor: ActorId) -> io::Result<()> {
        let bank = self
            .bank_presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Banking not configured"))?;
        let capacity = self
            .world
            .banking
            .as_ref()
            .ok_or_else(|| io::Error::other("Banking not configured"))?
            .capacity;
        if capacity
            .get(
                &self
                    .world
                    .data(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                    .variables,
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            == 0
        {
            let vars = &mut self
                .world
                .data_mut(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .variables;
            if let Some(update) = capacity
                .set(vars, bank.default_capacity)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
            {
                if let Some(connection) = self.connection(actor) {
                    let packet = gielinor_protocol::varp::update(update.id, update.value);
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
        }
        Ok(())
    }
}
