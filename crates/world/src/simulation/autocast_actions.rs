use super::*;
pub(super) fn open_picker(
    data: &PlayerData,
    picker: &mut Option<bool>,
    config: &crate::combat_wear::Config,
    defensive: bool,
) -> Option<i32> {
    let item = data.equipment.get(3)?.id;
    let weapon = config.autocast.weapons.get(&item)?;
    if !matches!(weapon.category, 18 | 21) {
        return None;
    }
    *picker = Some(defensive);
    Some(item)
}
impl World {
    pub fn choose_autocast(&mut self, actor: ActorId, defensive: bool) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self.combat_wear.as_ref().ok_or(ActionError::InvalidItem)?;
        let player = self.players.get_mut(actor.slot as usize).unwrap();
        if player.saving {
            return Err(ActionError::LoggingOut);
        }
        if player.protection.access_protected(player.has_modal) {
            player.queues.remove_all(4);
            player
                .queues
                .add(
                    4,
                    crate::queues::Category::Normal,
                    1,
                    Deferred::AutocastOpen(defensive),
                )
                .map_err(|_| ActionError::QueueFull)?;
        } else {
            player.begin_held_action();
            if let Some(weapon) = open_picker(
                &player.data,
                &mut player.autocast_picker_defensive,
                config,
                defensive,
            ) {
                self.events
                    .push(WorldEvent::AutocastPickerOpen { actor, weapon });
            }
        }
        Ok(())
    }
    pub fn select_autocast(&mut self, actor: ActorId, choice: i32) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self.combat_wear.as_ref().ok_or(ActionError::InvalidItem)?;
        let selection = self
            .autocast_selection
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let player = self.players.get_mut(actor.slot as usize).unwrap();
        if player.saving {
            return Err(ActionError::LoggingOut);
        }
        let vars = player.data.variables.clone();
        let result = selection
            .select(
                &config.autocast,
                &mut player.data.variables,
                player.data.equipment.get(3).map(|v| v.id),
                choice,
                player.autocast_picker_defensive.unwrap_or(false),
                player.data.stats.current(selection.magic_stat) as u8 as i32,
                player.protection.processed_clock > 0,
                |spell| {
                    config.runes.missing_message(
                        &vars,
                        &player.data.inventory,
                        &player.data.equipment,
                        spell,
                    )
                },
            )
            .map_err(|_| ActionError::InvalidItem)?;
        if result.restore_tab {
            player.autocast_picker_defensive = None;
        }
        self.events.push(WorldEvent::AutocastSelected {
            actor,
            events: result.events,
            restore_tab: result.restore_tab,
        });
        Ok(())
    }
}
