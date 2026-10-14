use super::*;
impl World {
    pub(super) fn process_tutorial_mining(
        &mut self,
        slot: usize,
        ore: crate::tutorial_mining::Ore,
    ) {
        let Some(config) = self.tutorial_mining else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        match config.mine(
            &mut p.data,
            &self.object_types,
            ore,
            p.protection.processed_clock > 0,
        ) {
            Ok(crate::tutorial_mining::Outcome::Denied(message)) => self
                .events
                .push(WorldEvent::LocationMessage { actor, message }),
            Ok(crate::tutorial_mining::Outcome::Mined {
                advancement,
                progression,
                message,
            }) => {
                self.events.push(WorldEvent::PlayerCombatAnimation {
                    actor,
                    sequence: config.animation,
                });
                if self.tutorial_rules_completed_varp.is_some()
                    && p.stat_events
                        .advance(
                            &crate::tutorial_rules::bindings(),
                            config.skill,
                            advancement,
                        )
                        .is_err()
                {
                    p.pending_logout = true;
                    self.events.push(WorldEvent::ActionRejected {
                        actor,
                        error: ActionError::QueueFull,
                    });
                }
                self.events.push(WorldEvent::SkillAdvanced {
                    actor,
                    skill: config.skill,
                    advancement,
                });
                self.events
                    .push(WorldEvent::LocationMessage { actor, message });
                if !progression.is_empty() {
                    self.events.push(WorldEvent::TutorialInventoryProgress {
                        actor,
                        effects: progression,
                    });
                }
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidStat,
            }),
        }
    }
}
