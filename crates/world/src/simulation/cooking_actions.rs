use super::*;
use crate::cooking::{Job, QUEUE};
impl World {
    pub(super) fn start_cooking(
        &mut self,
        slot: usize,
        target: crate::locations::Location,
        selected: usize,
        raw: i32,
    ) {
        let Some(c) = self.cooking.as_ref() else {
            return;
        };
        let Some(recipe) = c.recipes.get(&raw) else {
            return;
        };
        let Some(revision) = self.locations.revision(target) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if target.id != c.fire
            || p.location_revision != Some(revision)
            || c.tutorial.is_some_and(|v| v.active(&p.data.variables))
        {
            return;
        }
        if (p.data.stats.current(c.skill) as u8) < recipe.level || recipe.ticks == 0 {
            return;
        }
        if p.data
            .inventory
            .get(selected)
            .is_none_or(|v| v.id != raw || v.count != 1 || v.vars != 0)
        {
            return;
        }
        p.queues.clear_weak();
        let job = Job {
            target,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            slot: selected,
            raw,
            queued_at: self.clock.0,
        };
        // Location reach follows queue processing: raw delay equals subsequent clocks.
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                i32::from(recipe.ticks),
                Deferred::Cooking(job),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: c.animation,
        });
    }
}
pub(super) fn complete(
    p: &mut Player,
    job: Job,
    c: &crate::cooking::Config,
    items: &crate::inventory::Catalog,
    revision: Option<u64>,
    clock: i32,
    tutorial_stats: bool,
    events: &mut Vec<WorldEvent>,
    random: &mut impl RandomSource,
) {
    if p.location_interaction.is_some()
        || p.npc_interaction.is_some()
        || p.object_interaction.is_some()
        || p.pending_logout
        || p.saving
        || p.has_modal
        || p.movement.coords != job.origin
        || p.movement.last_movement != job.last_movement
        || revision != Some(job.revision)
        || job.target.id != c.fire
        || c.tutorial.is_some_and(|v| v.active(&p.data.variables))
    {
        return;
    }
    let Some(recipe) = c.recipes.get(&job.raw) else {
        return;
    };
    if let Ok(outcome) = recipe.complete(
        job.slot,
        &mut p.data.inventory,
        items,
        &mut p.data.stats,
        c.skill,
        c.max_level,
        p.data.xp_rate_hundreds,
        random,
    ) {
        if let Some(advancement) = outcome.advancement {
            if tutorial_stats
                && p.stat_events
                    .advance(&crate::tutorial_rules::bindings(), c.skill, advancement)
                    .is_err()
            {
                p.pending_logout = true;
                events.push(WorldEvent::ActionRejected {
                    actor: p.id,
                    error: ActionError::QueueFull,
                });
            }
            events.push(WorldEvent::SkillAdvanced {
                actor: p.id,
                skill: c.skill,
                advancement,
            });
        }
        events.push(WorldEvent::CookingMade {
            actor: p.id,
            trace: crate::cooking::Transition {
                job,
                outcome,
                committed_at: clock,
            },
        });
        events.push(WorldEvent::LocationMessage {
            actor: p.id,
            message: if outcome.burned {
                "You accidentally burn the food."
            } else {
                "You successfully cook the food."
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, loc_collision::LocCollision, locations::Location};
    fn fixture() -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 1, 10, 0).unwrap();
        let ty = LocCollision {
            width: 1,
            length: 2,
            block_walk: 2,
            block_range: false,
            break_route_finding: false,
        };
        w.location_types.insert(1, ty);
        w.locations.load(loc);
        crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        w.cooking = Some(crate::cooking::Config {
            fire: 1,
            animation: 897,
            skill: 7,
            max_level: 99,
            tutorial: None,
            recipes: [(
                10,
                crate::cooking::Recipe {
                    raw: 10,
                    cooked: 20,
                    burnt: 30,
                    level: 1,
                    stop_burn: 34,
                    low: 128,
                    high: 512,
                    xp: 30,
                    ticks: 1,
                },
            )]
            .into(),
        });
        w.held_use_types
            .insert(10, crate::held_use::Type { id: 10, content: 0 });
        let mut data = PlayerData::new(
            1,
            "Milker".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            {
                let mut s = Stats::default();
                s.set_current(7, 34);
                s
            },
        );
        for slot in 0..2 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 10,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .location_revision = w.locations.revision(loc);
        w.drain_events();
        (w, actor, loc)
    }

    fn action(loc: Location) -> Action {
        Action::UseLocation {
            coords: loc.coords,
            id: loc.id,
            slot: 1,
            object: 10,
            control: false,
        }
    }
    #[test]
    fn actual_location_use_one_or_four_ticks_exact_slot_and_no_repeat() {
        for ticks in [1, 4] {
            let (mut w, actor, loc) = fixture();
            w.cooking
                .as_mut()
                .unwrap()
                .recipes
                .get_mut(&10)
                .unwrap()
                .ticks = ticks;
            w.submit(actor, action(loc)).unwrap();
            w.tick().unwrap();
            assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 10);
            for _ in 1..ticks {
                w.tick().unwrap();
                assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 10);
            }
            w.tick().unwrap();
            assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 20);
            assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 10);
            assert_eq!(w.data(actor).unwrap().stats.fine_xp(7), 300);
            for _ in 0..8 {
                w.tick().unwrap();
            }
            let traces = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::CookingMade { trace, .. } = e {
                        Some(trace)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(traces.len(), 1);
            assert_eq!(
                traces[0].committed_at - traces[0].job.queued_at,
                i32::from(ticks)
            );
            assert!(!traces[0].outcome.burned);
        }
    }
    #[test]
    fn late_input_level_fire_movement_modal_and_held_interrupt_do_not_commit() {
        for mode in 0..6 {
            let (mut w, actor, loc) = fixture();
            w.submit(actor, action(loc)).unwrap();
            w.tick().unwrap();
            match mode {
                0 => {
                    let inv = &mut w.data_mut(actor).unwrap().inventory;
                    let item = inv.get(1);
                    inv.restore_slot(1, None).unwrap();
                    inv.restore_slot(2, item).unwrap();
                }
                1 => w.data_mut(actor).unwrap().stats.set_current(7, 0),
                2 => {
                    w.locations.delete(loc);
                    w.locations.add(loc).unwrap();
                }
                3 => {
                    w.submit(
                        actor,
                        Action::MoveClick {
                            destination: CoordGrid::new(3199, 3200, 0).unwrap(),
                            control: false,
                        },
                    )
                    .unwrap();
                }
                4 => w.set_modal_open(actor, true).unwrap(),
                _ => w
                    .players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .begin_held_action(),
            }
            w.tick().unwrap();
            assert_eq!(w.data(actor).unwrap().stats.fine_xp(7), 0);
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::CookingMade { .. })));
        }
    }
    #[test]
    fn active_tutorial_keeps_existing_handler_completed_accounts_use_ordinary_queue() {
        for completed in [false, true] {
            let (mut w, actor, loc) = fixture();
            let varp = |id| crate::variables::Varp {
                id,
                permanent: true,
                transmit: crate::variables::Transmit::Never,
                bit_protect: false,
                client_code: 0,
            };
            let progression = crate::tutorial_progression::Config {
                step: varp(100),
                completed: varp(101),
            };
            w.cooking.as_mut().unwrap().tutorial = Some(progression);
            w.tutorial_shrimp = Some(crate::tutorial_shrimp::Config {
                progression,
                location: 1,
                raw: 10,
                cooked: 20,
                animation: 897,
                skill: 7,
                max_level: 99,
                global_xp_rate: 1.0,
            });
            // Set trusted tutorial state via the same Variables API as progression.
            progression.set(
                &mut w.data_mut(actor).unwrap().variables,
                crate::tutorial_steps::Step::SurvivalCook,
                None,
                false,
            );
            if completed {
                w.data_mut(actor)
                    .unwrap()
                    .variables
                    .set(progression.completed, 1, false);
            }
            w.submit(actor, action(loc)).unwrap();
            w.tick().unwrap();
            if completed {
                assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 10);
                w.tick().unwrap();
                assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 20);
                assert!(w
                    .drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::CookingMade { .. })));
            } else {
                assert_eq!(w.data(actor).unwrap().stats.fine_xp(7), 300);
                assert!(!w
                    .drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::CookingMade { .. })));
                assert_eq!(
                    progression.current(&w.data(actor).unwrap().variables),
                    crate::tutorial_steps::Step::SurvivalGate
                );
            }
        }
    }
}
