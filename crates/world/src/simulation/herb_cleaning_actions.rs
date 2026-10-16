use super::*;
use crate::herb_cleaning::{Job, Outcome, Transition};

impl World {
    pub(super) fn clean_herb(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
    ) -> Result<(), ActionError> {
        let config = self
            .herb_cleaning
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let recipe = *config.recipes.get(&item).ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        p.begin_held_action();
        p.queues.clear_weak();
        let job = Job { recipe, slot };
        let before = p.data.stats.fine_xp(recipe.skill);
        let eligible = p.data.variables.get(config.quest_progress) == config.quest_complete;
        let (outcome, advancement) = job.complete(
            &mut p.data.inventory,
            &self.object_types,
            &mut p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
            eligible,
        );
        if let Some(advancement) = advancement {
            self.events.push(WorldEvent::SkillAdvanced {
                actor,
                skill: recipe.skill,
                advancement,
            });
        }
        let message = match outcome {
            Outcome::Cleaned => {
                self.events.push(WorldEvent::HerbCleaned {
                    actor,
                    trace: Transition {
                        job,
                        clock: self.clock.0,
                        fine_xp_before: before,
                        fine_xp_after: p.data.stats.fine_xp(recipe.skill),
                    },
                });
                "You clean the herb."
            }
            Outcome::Members => "You need to be a member to clean herbs.",
            Outcome::Quest => "You must complete Druidic Ritual before cleaning herbs.",
            Outcome::Level => "You need a higher Herblore level to clean that herb.",
            Outcome::Stale => return Err(ActionError::InvalidItem),
            _ => "You cannot clean that herb right now.",
        };
        self.events
            .push(WorldEvent::LocationMessage { actor, message });
        // Wiki selected Clean recipes specify0ticks. No guessed automatic cadence.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        herb_cleaning::{Config, Recipe},
        inventory::Item,
        variables::{Transmit, Varp},
    };
    fn quest() -> Varp {
        Varp {
            id: 80,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        }
    }
    fn fixture() -> (World, ActorId) {
        let mut w = World::default();
        let pos = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(3200, 3200, 0);
        w.herb_cleaning = Some(Config {
            recipes: [
                (
                    199,
                    Recipe {
                        input: 199,
                        output: 249,
                        skill: 15,
                        level: 3,
                        base_fine_xp: 25,
                    },
                ),
                (
                    201,
                    Recipe {
                        input: 201,
                        output: 251,
                        skill: 15,
                        level: 5,
                        base_fine_xp: 38,
                    },
                ),
            ]
            .into(),
            quest_progress: 80,
            quest_complete: 4,
        });
        let mut data = PlayerData::new(1, "Herbalist".into(), pos, Stats::default());
        data.xp_rate_hundreds = 500;
        data.stats.set_fine_xp(15, 3881).unwrap();
        data.stats.set_base(15, 5);
        data.stats.set_current(15, 5);
        data.variables.set(quest(), 4, false);
        for slot in 0..28 {
            let id = if slot == 0 {
                199
            } else if slot == 1 {
                201
            } else {
                1931
            };
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        w.drain_events();
        (w, a)
    }
    #[test]
    fn manual_clean_is_immediate_selected_atomic_and_does_not_replay() {
        let (mut w, a) = fixture();
        let before = w.data(a).unwrap().inventory.clone();
        w.submit(a, Action::CleanHerb { slot: 1, item: 201 })
            .unwrap();
        let expected_clock = w.clock.0;
        w.tick().unwrap();
        let after = w.data(a).unwrap();
        assert_eq!(after.inventory.get(1).unwrap().id, 251);
        assert_eq!(after.stats.fine_xp(15), 4071);
        for slot in (0..28).filter(|s| *s != 1) {
            assert_eq!(after.inventory.get(slot), before.get(slot));
        }
        let events = w.drain_events();
        let traces = events
            .iter()
            .filter_map(|e| {
                if let WorldEvent::HerbCleaned { trace, .. } = e {
                    Some(trace)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].clock, expected_clock);
        assert_eq!(
            (traces[0].fine_xp_before, traces[0].fine_xp_after),
            (3881, 4071)
        );
        w.submit(a, Action::CleanHerb { slot: 1, item: 201 })
            .unwrap();
        for _ in 0..5 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(a).unwrap().stats.fine_xp(15), 4071);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 199);
        assert!(
            !w.drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::HerbCleaned { .. }))
        );
    }
    #[test]
    fn manual_clean_rechecks_permission_level_and_slot_at_execution() {
        for mode in 0..6 {
            let (mut w, a) = fixture();
            w.submit(a, Action::CleanHerb { slot: 0, item: 199 })
                .unwrap();
            match mode {
                0 => w.set_membership(a, false).unwrap(),
                1 => {
                    w.data_mut(a).unwrap().variables.set(quest(), 3, true);
                }
                2 => w.data_mut(a).unwrap().stats.set_current(15, 2),
                3 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(0, None)
                    .unwrap(),
                4 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 201,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                _ => w.data_mut(a).unwrap().xp_rate_hundreds = 0,
            }
            let before = w.data(a).unwrap().inventory.clone();
            let xp = w.data(a).unwrap().stats.clone();
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().inventory, before, "mode{mode}");
            assert_eq!(w.data(a).unwrap().stats, xp, "mode{mode}");
            assert!(
                !w.drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::HerbCleaned { .. })),
                "mode{mode}"
            );
        }
    }
    #[test]
    fn manual_clean_respects_modal_delay_and_logout() {
        for mode in 0..3 {
            let (mut w, a) = fixture();
            let before = w.data(a).unwrap().inventory.clone();
            let xp = w.data(a).unwrap().stats.clone();
            match mode {
                0 => w.set_modal_open(a, true).unwrap(),
                1 => w.delay_player(a, 3).unwrap(),
                _ => {
                    w.players.get_mut(a.slot as usize).unwrap().pending_logout = true;
                }
            }
            if mode == 2 {
                assert_eq!(
                    w.submit(a, Action::CleanHerb { slot: 0, item: 199 }),
                    Err(ActionError::LoggingOut)
                );
            } else {
                w.submit(a, Action::CleanHerb { slot: 0, item: 199 })
                    .unwrap();
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.data(a).unwrap().stats, xp);
            assert!(
                !w.drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::HerbCleaned { .. }))
            );
        }
    }
}
