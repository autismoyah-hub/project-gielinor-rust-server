use super::*;
impl World {
    pub(super) fn process_tutorial_fishing(&mut self, slot: usize) {
        let Some(config) = self.tutorial_fishing else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        match config.fish(
            &mut p.data,
            &self.object_types,
            p.protection.processed_clock > 0,
        ) {
            Ok(crate::tutorial_fishing::Outcome::Denied(message)) => self
                .events
                .push(WorldEvent::LocationMessage { actor, message }),
            Ok(crate::tutorial_fishing::Outcome::Caught {
                advancement,
                progression,
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
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: "You manage to catch some shrimp.",
                });
                self.events.push(WorldEvent::TutorialInventoryProgress {
                    actor,
                    effects: progression,
                });
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidStat,
            }),
        }
    }
}
