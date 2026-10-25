use super::*;

impl World {
    pub fn toggle_special_attack(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self.special_attacks.as_ref().ok_or(ActionError::InvalidItem)?;
        config.validate().map_err(|_| ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        let (update, message) = config.toggle(&mut p.data.variables,
            p.data.equipment.get(3).map(|item| item.id), p.protection.processed_clock > 0);
        if let Some(update) = update {
            self.events.push(WorldEvent::CombatVariables { actor, updates: vec![update] });
        }
        if let Some(text) = message {
            self.events.push(WorldEvent::GameMessage { actor, text: text.into() });
        }
        Ok(())
    }
}
