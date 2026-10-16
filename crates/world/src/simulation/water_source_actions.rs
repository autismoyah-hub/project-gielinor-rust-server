use super::*;
use crate::water_sources::{Job, QUEUE};

impl World {
    pub(super) fn start_water_filling(
        &mut self,
        slot: usize,
        loc: crate::locations::Location,
        selected: usize,
        input: i32,
    ) {
        let Some(config) = self.water_sources.as_ref() else {
            return;
        };
        if config.output(loc.id, input).is_none() {
            return;
        }
        let Some(revision) = self.locations.revision(loc) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if p.location_revision != Some(revision) {
            return;
        }
        p.queues.clear_weak();
        let job = Job {
            target: loc,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            slot: selected,
            input,
            queued_at: self.clock.0,
            repeat: false,
        };
        // The reached-location phase follows queues, so one decrement is one
        // subsequent world tick. The first commit uses exactly the selected vessel.
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                1,
                Deferred::WaterFilling(job),
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
    config: &crate::water_sources::Config,
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
    if let Ok(output) = config.fill(
        job.target.id,
        job.input,
        job.slot,
        &mut p.data.inventory,
        catalog,
    ) {
        events.push(WorldEvent::WaterFilled {
            actor: p.id,
            trace: crate::water_sources::Transition {
                target: job.target,
                slot: job.slot,
                input: job.input,
                output,
                queued_at: job.queued_at,
                committed_at: clock,
                repeat: job.repeat,
            },
        });
        events.push(WorldEvent::LocationMessage {
            actor: p.id,
            message: "You fill the container with water.",
        });
        if let Some(slot) =
            p.data.inventory.slots().iter().position(|item| {
                item.is_some_and(|i| i.id == job.input && i.count == 1 && i.vars == 0)
            })
        {
            // Callback-added queues also decrement in this phase: two yields
            // one subsequent tick, without a second commit in this phase.
            if queues
                .add(
                    QUEUE,
                    crate::queues::Category::Weak,
                    2,
                    Deferred::WaterFilling(Job {
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
        w.water_sources = Some(crate::water_sources::Config {
            sources: [(1, crate::water_sources::Source::Sink)].into(),
            vessels: [(
                10,
                crate::water_sources::Vessel {
                    output: 20,
                    well: true,
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
    fn selected_vessel_and_repeats_commit_on_subsequent_ticks_without_xp() {
        let (mut w, actor, loc) = fixture();
        let stats = w.data(actor).unwrap().stats.clone();
        w.submit(actor, action(loc)).unwrap();
        w.tick().unwrap();
        assert_eq!(count(&w, actor, 20), 0);
        w.tick().unwrap();
        assert_eq!(count(&w, actor, 20), 1);
        assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 20);
        w.tick().unwrap();
        assert_eq!(count(&w, actor, 20), 2);
        for _ in 0..12 {
            w.tick().unwrap();
        }
        assert_eq!(count(&w, actor, 10), 0);
        assert_eq!(count(&w, actor, 20), 2);
        assert_eq!(w.data(actor).unwrap().stats, stats);
        let traces = w
            .drain_events()
            .into_iter()
            .filter_map(|event| {
                if let WorldEvent::WaterFilled { trace, .. } = event {
                    Some(trace)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(traces.len(), 2);
        assert_eq!((traces[0].slot, traces[1].slot), (1, 0));
        assert!(!traces[0].repeat);
        assert!(traces[1].repeat);
        for trace in &traces {
            assert_eq!(trace.committed_at - trace.queued_at, 1);
        }
        assert_eq!(traces[0].committed_at, traces[1].queued_at);
    }
    #[test]
    fn selected_slot_target_generation_movement_and_modal_reject_stale_fill() {
        for mutation in 0..5 {
            let (mut w, actor, loc) = fixture();
            w.submit(actor, action(loc)).unwrap();
            w.tick().unwrap();
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
                    w.set_modal_open(actor, true).unwrap();
                    w.set_modal_open(actor, false).unwrap();
                }
                _ => w
                    .players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .begin_held_action(),
            }
            for _ in 0..6 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, actor, 10), 2, "mutation{mutation}");
            assert_eq!(count(&w, actor, 20), 0, "mutation{mutation}");
        }
    }

    #[test]
    fn changes_after_first_commit_cancel_repeat_without_reverting_output() {
        for mutation in 0..5 {
            let (mut w, actor, loc) = fixture();
            w.submit(actor, action(loc)).unwrap();
            w.tick().unwrap();
            w.tick().unwrap();
            assert_eq!(count(&w, actor, 20), 1);
            match mutation {
                0 => {
                    let inv = &mut w.data_mut(actor).unwrap().inventory;
                    let item = inv.get(0);
                    inv.restore_slot(0, None).unwrap();
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
                    w.set_modal_open(actor, true).unwrap();
                    w.set_modal_open(actor, false).unwrap();
                }
                _ => w
                    .players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .begin_held_action(),
            }
            for _ in 0..8 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, actor, 20), 1, "mutation{mutation}");
            assert_eq!(count(&w, actor, 10), 1, "mutation{mutation}");
        }
    }
}
