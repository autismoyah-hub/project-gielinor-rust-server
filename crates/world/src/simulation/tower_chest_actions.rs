use super::*;
impl World {
    pub(super) fn claim_tower_chest(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self
            .tower_chest
            .as_ref()
            .ok_or(ActionError::InvalidLocation)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if !crate::tower_layout::contains(config.origin, p.movement.coords) {
            return Ok(());
        }
        let progress = p.data.tower_diary.ok_or(ActionError::UnknownTowerDiary)?;
        let marks = progress.marks(
            config.skills.map(|id| p.data.stats.base(id) as i32),
            p.data.tower_arrival.staff_taken,
        );
        let outcome = config.inventory.claim(
            &mut p.data.inventory,
            &p.data.equipment,
            &p.data.bank,
            &self.object_types,
            marks,
        );
        let bones = config.bones;
        for text in outcome.messages {
            self.events.push(WorldEvent::GameMessage { actor, text });
        }
        if let Some(sync) = outcome.sync {
            self.publish_tower_diary_sync(actor, sync, bones);
        }
        Ok(())
    }
}
