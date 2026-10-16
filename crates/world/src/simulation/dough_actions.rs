use super::*;
use crate::dough::{Job, Outcome, Pending, QUEUE, Request};

impl World {
    pub fn take_dough_request(&mut self, actor: ActorId) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .dough_request
            .take())
    }
    pub fn start_dough(
        &mut self,
        actor: ActorId,
        request: Request,
        index: usize,
        count: i32,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.pending_logout || p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        if !(1..=28).contains(&count)
            || p.movement.coords != request.origin
            || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidItem);
        }
        let recipe = self
            .dough
            .as_ref()
            .and_then(|c| c.recipe(request.water, index))
            .ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                // Input is stamped with the upcoming cycle: queue2 commits
                // one processing-clock interval after selection.
                2,
                Deferred::Dough(Pending {
                    job: Job {
                        recipe,
                        flour_slot: request.flour_slot,
                        water_slot: request.water_slot,
                    },
                    request,
                    remaining: count,
                    queued_at: self.clock.0,
                    index: 0,
                }),
            )
            .map_err(|_| ActionError::QueueFull)?;
        Ok(())
    }
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    pending: Pending,
    catalog: &crate::inventory::Catalog,
    clock: i32,
    events: &mut Vec<WorldEvent>,
) {
    if p.pending_logout
        || p.saving
        || p.has_modal
        || p.location_interaction.is_some()
        || p.npc_interaction.is_some()
        || p.object_interaction.is_some()
        || p.movement.coords != pending.request.origin
        || p.movement.last_movement != pending.request.last_movement
    {
        return;
    }
    let outcome = pending
        .job
        .complete(&mut p.data.inventory, catalog, &p.data.stats);
    if outcome == Outcome::Made {
        events.push(WorldEvent::DoughMade {
            actor: p.id,
            trace: crate::dough::Transition {
                job: pending.job,
                queued_at: pending.queued_at,
                committed_at: clock,
                index: pending.index,
            },
        });
    }
    let message = match outcome {
        Outcome::Made => "You mix the flour and water to make dough.",
        Outcome::Level => "You need a higher Cooking level to make that.",
        Outcome::Capacity => {
            "You need another free inventory space to keep the dough and both containers."
        }
        _ => return,
    };
    events.push(WorldEvent::LocationMessage {
        actor: p.id,
        message,
    });
    if outcome != Outcome::Made || pending.remaining <= 1 {
        return;
    }
    let slot = |id| {
        p.data
            .inventory
            .slots()
            .iter()
            .position(|v| v.is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0))
    };
    let (Some(flour_slot), Some(water_slot)) = (
        slot(pending.job.recipe.flour),
        slot(pending.job.recipe.water),
    ) else {
        return;
    };
    let next = Pending {
        job: Job {
            flour_slot,
            water_slot,
            ..pending.job
        },
        remaining: pending.remaining - 1,
        queued_at: clock,
        index: pending.index + 1,
        ..pending
    };
    // Callback-added queue2 reaches the next commit one world tick later.
    if queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            2,
            Deferred::Dough(next),
        )
        .is_err()
    {
        events.push(WorldEvent::ActionRejected {
            actor: p.id,
            error: ActionError::QueueFull,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dough::{Config, Output},
        held_use,
        inventory::{Inventory, Item, StackType},
    };
    fn fixture() -> (World, ActorId, held_use::Request) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.dough = Some(Config {
            skill: 7,
            tutorial: None,
            flour: 10,
            pot: 12,
            waters: vec![(11, 13)],
            outputs: [
                Output { id: 14, level: 1 },
                Output { id: 15, level: 1 },
                Output { id: 16, level: 35 },
            ],
        });
        w.held_use_types = [10, 11]
            .map(|id| (id, held_use::Type { id, content: 0 }))
            .into();
        let mut data = PlayerData::new(
            1,
            "Dough".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.inventory = Inventory::new(6, StackType::Normal, false);
        for (slot, id) in [(0, 10), (1, 11), (2, 10), (3, 11)] {
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
        let actor = w.actors().next().unwrap();
        (
            w,
            actor,
            held_use::Request {
                selected_id: 11,
                selected_slot: 3,
                target_id: 10,
                target_slot: 2,
            },
        )
    }
    fn choose(w: &mut World, a: ActorId, r: held_use::Request) -> Request {
        w.prepare_held_use(a, r).unwrap();
        w.finish_held_use(a, r).unwrap();
        w.take_dough_request(a).unwrap().unwrap()
    }
    fn count(w: &World, a: ActorId, id: i32) -> usize {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .count()
    }
    #[test]
    fn selected_inputs_then_one_tick_repeats_keep_containers_and_xp() {
        let (mut w, a, r) = fixture();
        let stats = w.data(a).unwrap().stats.clone();
        let request = choose(&mut w, a, r);
        assert_eq!((request.flour_slot, request.water_slot), (2, 3));
        let token = w.start_quantity_dialog(a).unwrap();
        assert!(w.resume_quantity_dialog(a, token).unwrap());
        w.start_dough(a, request, 1, 2).unwrap();
        assert_eq!(count(&w, a, 15), 0);
        w.tick().unwrap();
        assert_eq!(count(&w, a, 15), 0);
        w.tick().unwrap();
        assert_eq!(count(&w, a, 15), 1);
        assert_eq!(w.data(a).unwrap().inventory.get(2).unwrap().id, 12);
        assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().id, 13);
        w.tick().unwrap();
        assert_eq!(count(&w, a, 15), 2);
        assert_eq!(count(&w, a, 12), 2);
        assert_eq!(count(&w, a, 13), 2);
        assert_eq!(w.data(a).unwrap().stats, stats);
        let traces: Vec<_> = w
            .drain_events()
            .into_iter()
            .filter_map(|e| {
                if let WorldEvent::DoughMade { actor, trace } = e {
                    assert_eq!(actor, a);
                    Some(trace)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(traces.len(), 2);
        for (i, trace) in traces.iter().enumerate() {
            assert_eq!(trace.index, i as u32);
            assert_eq!(trace.committed_at - trace.queued_at, 1);
            assert_eq!(
                (trace.job.flour_slot, trace.job.water_slot),
                [(2, 3), (0, 1)][i]
            );
        }
        assert_eq!(traces[1].queued_at, traces[0].committed_at);
        w.tick().unwrap();
        assert_eq!(count(&w, a, 15), 2);
    }
    #[test]
    fn invalid_choice_level_and_stale_input_never_debit() {
        for mode in 0..3 {
            let (mut w, a, r) = fixture();
            let request = choose(&mut w, a, r);
            if mode == 2 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(3, None)
                    .unwrap();
            }
            let before = w.data(a).unwrap().inventory.clone();
            if mode == 0 {
                assert!(w.start_dough(a, request, 3, 2).is_err());
            } else {
                w.start_dough(a, request, if mode == 1 { 2 } else { 0 }, 2)
                    .unwrap();
            }
            for _ in 0..3 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().inventory, before);
        }
    }
    #[test]
    fn interruption_after_first_output_preserves_only_committed_dough() {
        for mode in 0..3 {
            let (mut w, a, r) = fixture();
            let request = choose(&mut w, a, r);
            w.start_dough(a, request, 0, 2).unwrap();
            w.tick().unwrap();
            w.tick().unwrap();
            match mode {
                0 => {
                    w.set_modal_open(a, true).unwrap();
                    w.set_modal_open(a, false).unwrap();
                }
                1 => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .begin_held_action();
                }
                _ => {
                    w.submit(
                        a,
                        Action::MoveClick {
                            destination: CoordGrid::new(3201, 3200, 0).unwrap(),
                            control: false,
                        },
                    )
                    .unwrap();
                }
            }
            for _ in 0..3 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, a, 14), 1, "mode{mode}");
            assert_eq!(count(&w, a, 10), 1);
            assert_eq!(count(&w, a, 11), 1);
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use crate::{
        dough::{Config, Output},
        held_use,
        inventory::{Inventory, Item, StackType},
        variables::{Transmit, Varp},
    };
    #[test]
    fn active_tutorial_blocks_ordinary_choice_until_completed() {
        let mut world = World::default();
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let tutorial = crate::tutorial_progression::Config {
            step: varp(1),
            completed: varp(2),
        };
        world.dough = Some(Config {
            skill: 7,
            tutorial: Some(tutorial),
            flour: 10,
            pot: 12,
            waters: vec![(11, 13)],
            outputs: [
                Output { id: 14, level: 1 },
                Output { id: 15, level: 1 },
                Output { id: 16, level: 35 },
            ],
        });
        world.held_use_types = [10, 11]
            .map(|id| (id, held_use::Type { id, content: 0 }))
            .into();
        let mut data = PlayerData::new(
            1,
            "Dough".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.inventory = Inventory::new(3, StackType::Normal, false);
        for (slot, id) in [(0, 10), (1, 11)] {
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
        let before = data.inventory.clone();
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        let request = held_use::Request {
            selected_id: 10,
            selected_slot: 0,
            target_id: 11,
            target_slot: 1,
        };
        world.prepare_held_use(actor, request).unwrap();
        world.finish_held_use(actor, request).unwrap();
        assert!(world.take_dough_request(actor).unwrap().is_none());
        assert_eq!(world.data(actor).unwrap().inventory, before);
        world
            .data_mut(actor)
            .unwrap()
            .variables
            .set(tutorial.completed, 1, true);
        world.prepare_held_use(actor, request).unwrap();
        world.finish_held_use(actor, request).unwrap();
        let choice = world.take_dough_request(actor).unwrap().unwrap();
        for count in [0, 29] {
            assert!(world.start_dough(actor, choice, 0, count).is_err());
        }
        world
            .data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(1, None)
            .unwrap();
        world.start_dough(actor, choice, 0, 1).unwrap();
        world.tick().unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().inventory.get(0).unwrap().id, 10);
        assert!(
            !world
                .data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == 14)
        );
    }
}
