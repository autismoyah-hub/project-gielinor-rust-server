use super::*;
use crate::grinding::{Job, Outcome, Pending, QUEUE, Recipe};

impl World {
    /// Account-authorized metadata, refreshed from the login host; never a client action.
    pub fn set_membership(&mut self, actor: ActorId, member: bool) -> Result<(), ActionError> {
        self.player(actor)?;
        self.players.get_mut(actor.slot as usize).unwrap().member = member;
        Ok(())
    }
    pub(super) fn start_grinding(
        &mut self,
        actor: ActorId,
        recipe: Recipe,
        tool_slot: usize,
        input_slot: usize,
    ) -> Result<(), ActionError> {
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.queues.clear_weak();
        let job = Job {
            recipe,
            tool_slot,
            input_slot,
        };
        if !apply(
            p,
            job,
            &self.object_types,
            self.clock.0,
            self.clock.0,
            0,
            &mut self.events,
        ) {
            return Ok(());
        }
        if let Some(input_slot) = next_input(p, job, &self.object_types) {
            let pending = Pending {
                job: Job { input_slot, ..job },
                origin: p.movement.coords,
                last_movement: p.movement.last_movement,
                queued_at: self.clock.0,
                index: 1,
            };
            // Input is stamped with the upcoming processing cycle. Its queue
            // decrements in that cycle, so three yields a two-clock gap.
            // The first output is synchronized in that initial cycle.
            p.queues
                .add(
                    QUEUE,
                    crate::queues::Category::Weak,
                    3,
                    Deferred::Grinding(pending),
                )
                .map_err(|_| ActionError::QueueFull)?;
        }
        Ok(())
    }
}
fn next_input(p: &Player, job: Job, catalog: &crate::inventory::Catalog) -> Option<usize> {
    let input=job.recipe.input;
    if catalog.stackable.contains(&input) {
        return p.data.inventory.get(job.input_slot)
            .filter(|i| i.id==input && i.count>0 && i.vars==0).map(|_|job.input_slot);
    }

    p.data
        .inventory
        .slots()
        .iter()
        .position(|i| i.is_some_and(|i| i.id == input && i.count == 1 && i.vars == 0))
}
fn apply(
    p: &mut Player,
    job: Job,
    catalog: &crate::inventory::Catalog,
    queued_at: i32,
    clock: i32,
    index: u32,
    events: &mut Vec<WorldEvent>,
) -> bool {
    match job.complete(&mut p.data.inventory, catalog, p.member) {
        Outcome::Made => {
            events.push(WorldEvent::Grinding {
                actor: p.id,
                trace: crate::grinding::Transition {
                    job,
                    queued_at,
                    committed_at: clock,
                    index,
                },
            });
            if let Some(sequence) = job.recipe.animation {
                events.push(WorldEvent::PlayerCombatAnimation {
                    actor: p.id,
                    sequence,
                });
            }
            events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "You grind the ingredient.",
            });
            true
        }
        Outcome::Members => {
            events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "You need membership to use this recipe.",
            });
            false
        }
        _ => false,
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
        || p.movement.coords != pending.origin
        || p.movement.last_movement != pending.last_movement
    {
        return;
    }
    if !apply(
        p,
        pending.job,
        catalog,
        pending.queued_at,
        clock,
        pending.index,
        events,
    ) {
        return;
    }
    if let Some(input_slot) = next_input(p, pending.job, catalog) {
        let next = Pending {
            job: Job {
                input_slot,
                ..pending.job
            },
            queued_at: clock,
            index: pending.index + 1,
            ..pending
        };
        // Callback-added queues decrement in this phase;4 gives3subsequent ticks.
        if queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                4,
                Deferred::Grinding(next),
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
    use crate::{
        held_use::{Request, Scope, Type},
        inventory::{Inventory, Item, StackType},
    };
    fn fixture(members: bool) -> (World, ActorId, Request) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.held_use_types = [10, 11, 12].map(|id| (id, Type { id, content: 0 })).into();
        let recipe = Recipe {
            tool: 10,
            input: 11,
            output: 12,
            members,
            animation: Some(364),
        };
        w.held_use_handlers
            .register(
                Scope::Types,
                10,
                11,
                crate::held_use::Handler::Grinding(recipe),
            )
            .unwrap();
        let mut data = PlayerData::new(
            1,
            "Grinder".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.inventory = Inventory::new(4, StackType::Normal, false);
        for slot in 0..4 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if slot == 0 { 10 } else { 11 },
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
            Request {
                selected_slot: 3,
                selected_id: 11,
                target_slot: 0,
                target_id: 10,
            },
        )
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
    fn start(w: &mut World, actor: ActorId, request: Request) {
        w.prepare_held_use(actor, request).unwrap();
        w.finish_held_use(actor, request).unwrap();
    }
    #[test]
    fn selected_shard_stack_repeats_without_touching_other_stack_or_awarding_xp() {
        for stale in [false,true] {
            let (mut w,a,r)=fixture(true);w.set_membership(a,true).unwrap();w.object_types.stackable.extend([11,12]);
            w.data_mut(a).unwrap().inventory.restore_slot(2,None).unwrap();
            w.data_mut(a).unwrap().inventory.restore_slot(3,Some(Item{id:11,count:3,vars:0})).unwrap();
            let stats=w.data(a).unwrap().stats.clone();start(&mut w,a,r);
            assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().count,2);
            if stale {w.data_mut(a).unwrap().inventory.restore_slot(3,None).unwrap();}
            for _ in 0..10{w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().inventory.get(1).unwrap(),Item{id:11,count:1,vars:0});
            assert_eq!(w.data(a).unwrap().inventory.get(2).unwrap(),Item{id:12,count:if stale{1}else{3},vars:0});
            assert_eq!(w.data(a).unwrap().stats,stats);
        }
    }
    #[test]
    fn reverse_tool_use_is_immediate_then_two_then_three_ticks_with_no_xp() {
        let (mut w, actor, request) = fixture(false);
        let stats = w.data(actor).unwrap().stats.clone();
        start(&mut w, actor, request);
        assert_eq!(count(&w, actor, 12), 1);
        assert_eq!(w.data(actor).unwrap().inventory.get(3).unwrap().id, 12);
        for tick in 1..=9 {
            w.tick().unwrap();
            assert_eq!(
                count(&w, actor, 12),
                if tick < 3 {
                    1
                } else if tick < 6 {
                    2
                } else {
                    3
                },
                "tick{tick}"
            );
            assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 10);
        }
        assert_eq!(w.data(actor).unwrap().stats, stats);
        let traces: Vec<_> = w
            .drain_events()
            .into_iter()
            .filter_map(|event| {
                if let WorldEvent::Grinding { actor: who, trace } = event {
                    assert_eq!(who, actor);
                    Some(trace)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(traces.len(), 3);
        for (i, trace) in traces.iter().enumerate() {
            assert_eq!(trace.index, i as u32);
            assert_eq!(trace.committed_at - trace.queued_at, [0, 2, 3][i]);
            assert_eq!(trace.job.input_slot, [3, 1, 2][i]);
            assert_eq!(trace.job.tool_slot, 0);
            if i > 0 {
                assert_eq!(trace.queued_at, traces[i - 1].committed_at);
            }
        }
    }
    #[test]
    fn membership_is_required_at_admission_and_each_continuation() {
        let (mut w, actor, request) = fixture(true);
        let before = w.data(actor).unwrap().inventory.clone();
        start(&mut w, actor, request);
        assert_eq!(w.data(actor).unwrap().inventory, before);
        w.set_membership(actor, true).unwrap();
        start(&mut w, actor, request);
        assert_eq!(count(&w, actor, 12), 1);
        w.set_membership(actor, false).unwrap();
        for _ in 0..6 {
            w.tick().unwrap();
        }
        assert_eq!(count(&w, actor, 12), 1);
        assert_eq!(count(&w, actor, 11), 2);
        w.set_membership(actor, true).unwrap();
        for _ in 0..6 {
            w.tick().unwrap();
        }
        assert_eq!(count(&w, actor, 12), 1);
    }
    #[test]
    fn stale_tool_input_and_interruption_stop_after_committed_first_output() {
        for mutation in 0..5 {
            let (mut w, actor, request) = fixture(false);
            start(&mut w, actor, request);
            match mutation {
                0 => {
                    let inv = &mut w.data_mut(actor).unwrap().inventory;
                    let a = inv.get(0);
                    let b = inv.get(2);
                    inv.restore_slot(0, b).unwrap();
                    inv.restore_slot(2, a).unwrap();
                }
                1 => {
                    w.data_mut(actor)
                        .unwrap()
                        .inventory
                        .restore_slot(1, None)
                        .unwrap();
                }
                2 => {
                    w.submit(
                        actor,
                        Action::MoveClick {
                            destination: CoordGrid::new(3201, 3200, 0).unwrap(),
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
            assert_eq!(count(&w, actor, 12), 1, "mutation{mutation}");
            assert_eq!(count(&w, actor, 10), 1);
            assert_eq!(
                w.drain_events()
                    .into_iter()
                    .filter(|e| matches!(e, WorldEvent::Grinding { .. }))
                    .count(),
                1
            );
        }
    }
}
