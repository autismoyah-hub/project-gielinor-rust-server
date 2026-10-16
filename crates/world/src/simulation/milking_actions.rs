use super::*;
use crate::milking::{Config, Job, QUEUE};

impl World {
    pub(super) fn start_milking(
        &mut self,
        slot: usize,
        loc: crate::locations::Location,
        selected: Option<usize>,
    ) {
        let Some(config) = self.milking.as_ref() else {
            return;
        };
        let Some(revision) = self.locations.revision(loc) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if p.location_revision != Some(revision) {
            return;
        }
        let selected_slot = selected.or_else(|| {
            p.data.inventory.slots().iter().position(|i| {
                i.is_some_and(|i| i.id == config.bucket && i.count == 1 && i.vars == 0)
            })
        });
        let Some(inv_slot) = selected_slot.filter(|&s| {
            p.data
                .inventory
                .get(s)
                .is_some_and(|i| i.id == config.bucket && i.count == 1 && i.vars == 0)
        }) else {
            self.events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "You need an empty bucket to collect milk.",
            });
            return;
        };
        p.queues.clear_weak();
        let job = Job {
            target: loc,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            slot: inv_slot,
            queued_at: self.clock.0,
            item_use: selected.is_some(),
            repeat: false,
        };
        // Wiki first-action durations differ for menu and bucket-on-cow. These
        // are measured from reached admission; native route phase is separate.
        let delay = if selected.is_some() { 4 } else { 3 };
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                delay,
                Deferred::Milking(job),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
        }
    }
}

pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    job: Job,
    config: &Config,
    catalog: &crate::inventory::Catalog,
    revision: Option<u64>,
    clock: i32,
    events: &mut Vec<WorldEvent>,
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
    {
        return;
    }
    if config
        .fill(&mut p.data.inventory, catalog, job.slot)
        .is_err()
    {
        return;
    }
    events.push(WorldEvent::Milking {
        actor: p.id,
        trace: crate::milking::Transition {
            target: job.target,
            slot: job.slot,
            input: config.bucket,
            output: config.milk,
            queued_at: job.queued_at,
            committed_at: clock,
            item_use: job.item_use,
            repeat: job.repeat,
        },
    });
    events.push(WorldEvent::PlayerCombatAnimation {
        actor: p.id,
        sequence: config.animation,
    });
    events.push(WorldEvent::CombatSound {
        actor: p.id,
        sound: config.sound,
    });
    events.push(WorldEvent::LocationMessage {
        actor: p.id,
        message: "You milk the cow.",
    });
    if let Some(slot) = p
        .data
        .inventory
        .slots()
        .iter()
        .position(|i| i.is_some_and(|i| i.id == config.bucket && i.count == 1 && i.vars == 0))
    {
        if queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                // A callback-added queue is visited in this same phase.
                // Nine decrements yield eight subsequent world ticks.
                9,
                Deferred::Milking(Job {
                    slot,
                    queued_at: clock,
                    repeat: true,
                    ..job
                }),
            )
            .is_err()
        {
            events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
        }
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
        w.milking = Some(Config {
            cows: [(1, 1)].into(),
            bucket: 10,
            milk: 20,
            animation: 2305,
            sound: 372,
        });
        w.held_use_types
            .insert(10, crate::held_use::Type { id: 10, content: 0 });
        let mut data = PlayerData::new(
            1,
            "Milker".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
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
    fn count(w: &World, actor: ActorId, id: i32) -> usize {
        w.data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .count()
    }
    #[test]
    fn first_action_and_repeat_use_distinct_delays_without_xp() {
        for selected in [None, Some(1)] {
            let (mut w, actor, loc) = fixture();
            let stats = w.data(actor).unwrap().stats.clone();
            let action = if let Some(slot) = selected {
                Action::UseLocation {
                    coords: loc.coords,
                    id: loc.id,
                    slot: slot as i32,
                    object: 10,
                    control: false,
                }
            } else {
                Action::OperateLocation {
                    coords: loc.coords,
                    id: loc.id,
                    operation: 1,
                    control: false,
                }
            };
            w.submit(actor, action).unwrap();
            // Admit through PlayerMain's reached-location phase, after queues.
            w.tick().unwrap();
            assert_eq!(count(&w, actor, 20), 0);
            let first = if selected.is_some() { 4 } else { 3 };
            for tick in 1..=first + 8 {
                w.tick().unwrap();
                assert_eq!(
                    count(&w, actor, 20),
                    if tick < first {
                        0
                    } else if tick < first + 8 {
                        1
                    } else {
                        2
                    },
                    "tick{tick}"
                );
                if tick == first {
                    assert_eq!(
                        w.data(actor)
                            .unwrap()
                            .inventory
                            .get(selected.unwrap_or(0))
                            .unwrap()
                            .id,
                        20
                    );
                }
            }
            assert_eq!(w.data(actor).unwrap().stats, stats);
            let traces = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::Milking { trace, .. } = e {
                        Some(trace)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(traces.len(), 2);
            assert_eq!(traces[0].committed_at - traces[0].queued_at, first);
            assert_eq!(traces[1].committed_at - traces[1].queued_at, 8);
            assert!(!traces[0].repeat);
            assert!(traces[1].repeat);
        }
    }
    #[test]
    fn moved_input_replaced_cow_and_interruptions_prevent_late_reward() {
        for mutation in 0..5 {
            let (mut w, actor, loc) = fixture();
            w.start_milking(actor.slot as usize, loc, Some(1));
            match mutation {
                0 => {
                    let inv = &mut w.data_mut(actor).unwrap().inventory;
                    let item = inv.get(1);
                    inv.restore_slot(1, None).unwrap();
                    inv.restore_slot(2, item).unwrap();
                }
                1 => {
                    w.locations.delete(loc);
                    w.locations.add(loc).unwrap();
                }
                2 => {
                    w.submit(
                        actor,
                        Action::MoveClick {
                            destination: CoordGrid::new(3199, 3200, 0).unwrap(),
                            control: false,
                        },
                    )
                    .unwrap();
                }
                3 => {
                    w.players
                        .get_mut(actor.slot as usize)
                        .unwrap()
                        .begin_held_action();
                }
                _ => {
                    w.set_modal_open(actor, true).unwrap();
                }
            }
            for _ in 0..14 {
                w.tick().unwrap();
            }
            w.set_modal_open(actor, false).unwrap();
            for _ in 0..4 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, actor, 20), 0, "mutation{mutation}");
            assert_eq!(count(&w, actor, 10), 2);
        }
    }
    #[test]
    fn world_action_routes_reach_handler_and_stale_arrival_cannot_restart() {
        for use_item in [false, true] {
            let (mut w, actor, loc) = fixture();
            assert!(w.has_location_handler(1, 1));
            assert!(!w.has_location_handler(1, 2));
            let action = if use_item {
                Action::UseLocation {
                    coords: loc.coords,
                    id: 1,
                    slot: 1,
                    object: 10,
                    control: false,
                }
            } else {
                Action::OperateLocation {
                    coords: loc.coords,
                    id: 1,
                    operation: 1,
                    control: false,
                }
            };
            w.submit(actor, action).unwrap();
            for _ in 0..7 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, actor, 20), 1);
        }
        let (mut w, actor, loc) = fixture();
        w.locations.delete(loc);
        w.locations.add(loc).unwrap();
        w.start_milking(actor.slot as usize, loc, None);
        for _ in 0..14 {
            w.tick().unwrap();
        }
        assert_eq!(count(&w, actor, 20), 0);
    }
    #[test]
    fn packed_cow_approach_byte_reaches_existing_milk_handler() {
        let (mut w, actor, loc) = fixture();
        // Angle0 with packed23 permits the west approach used by this fixture.
        w.location_approach.insert(loc.id, 23);
        w.submit(
            actor,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        for _ in 0..4 {
            w.tick().unwrap();
        }
        assert_eq!(count(&w, actor, 20), 1);
    }
}
