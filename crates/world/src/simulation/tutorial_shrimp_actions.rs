use super::*;
impl World {
    pub(super) fn process_tutorial_shrimp(&mut self, slot: usize) {
        let Some(config) = self.tutorial_shrimp else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        match config.cook(
            &mut p.data,
            &self.object_types,
            p.protection.processed_clock > 0,
        ) {
            Ok(None) => {}
            Ok(Some(result)) => {
                if let Some(config) = &self.cinderworks {
                    if let Some(text) = config.credit(&mut p.data, p.movement.coords, crate::tower_run::Service::Kitchen) {
                        self.events.push(WorldEvent::CinderworksProgress {actor,run:p.data.tower_run});
                        self.events.push(WorldEvent::GameMessage {actor, text:text.into()});
                    }
                }
                self.events.push(WorldEvent::PlayerCombatAnimation {
                    actor,
                    sequence: config.animation,
                });
                if self.tutorial_rules_completed_varp.is_some()
                    && p.stat_events
                        .advance(
                            &crate::tutorial_rules::bindings(),
                            config.skill,
                            result.advancement,
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
                    advancement: result.advancement,
                });
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: "You manage to cook some shrimp.",
                });
                self.events.push(WorldEvent::TutorialInventoryProgress {
                    actor,
                    effects: result.progression,
                });
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidStat,
            }),
        }
    }
}
