use super::*;
impl World {
    pub(super) fn search_tower_staff(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self
            .tower_staff
            .as_ref()
            .ok_or(ActionError::InvalidLocation)?;
        let player = self.players.get_mut(actor.slot as usize).unwrap();
        if !crate::tower_layout::contains(config.origin, player.movement.coords) {
            return Ok(());
        }
        let message = config
            .inventory
            .search(&mut player.data, &self.object_types)
            .map_err(|_| ActionError::UnknownTowerBones)?;
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: message.into(),
        });
        Ok(())
    }
}
