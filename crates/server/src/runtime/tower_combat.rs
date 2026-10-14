use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_tower_combat(&mut self, bosses: [u16; 2], chicken: u16) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure Tower combat before admission"));
        }
        let common = self
            .world
            .tower_tanning
            .as_ref()
            .ok_or_else(|| io::Error::other("Tower diary missing"))?;
        let mut config = gielinor_world::tower_combat::Config {
            bosses: bosses.into(),
            chicken,
            scimitars: [1321].into(),
            shortbows: Default::default(),
            bronze_arrows: Default::default(),
            skills: common.skills,
            bones: common.bones,
        };
        for (&id, ty) in &self.definitions.items {
            if gielinor_world::tower_combat::scimitar(id as i32, Some(&ty.name)) {
                config.scimitars.insert(id as i32);
            }
            if gielinor_world::tower_combat::shortbow(&ty.name) {
                config.shortbows.insert(id as i32);
            }
            if gielinor_world::tower_combat::bronze_arrow(&ty.name) {
                config.bronze_arrows.insert(id as i32);
            }
        }
        self.world.tower_combat = Some(config);
        Ok(())
    }
}
