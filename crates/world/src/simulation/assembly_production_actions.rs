use super::*;
use crate::assembly_production::{Pending, Request, QUEUE};
impl World {
    pub(super) fn request_assembly_production(
        &mut self,
        actor: ActorId,
        recipe: crate::assembly::PreciseRecipe,
        cycle_ticks: u8,
        held: crate::held_use::Request,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        let request = Request::bind(
            recipe,
            cycle_ticks,
            held,
            &p.data.inventory,
            p.movement.coords,
            p.movement.last_movement,
        )
        .ok_or(ActionError::InvalidItem)?;
        if recipe.recipe.members && !p.member {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: "You need to be a member to make that item.",
            });
            return Ok(());
        }
        self.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .assembly_production_request = Some(request);
        Ok(())
    }
    pub fn take_assembly_production_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .assembly_production_request
            .take())
    }
    pub fn start_assembly_production(
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
        if index != 0
            || !(1..=28).contains(&count)
            || request.recipe.ticks == 0
            || request.cycle_ticks == 0
            || p.movement.coords != request.origin
            || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidItem);
        }
        let jobs = request
            .jobs(&p.data.inventory, count as usize)
            .ok_or(ActionError::InvalidItem)?;
        let first = *jobs.first().ok_or(ActionError::InvalidItem)?;
        let mut inventory = p.data.inventory.clone();
        let mut stats = p.data.stats.clone();
        let (outcome, _) = first.complete_awarding_tenths(
            &mut inventory,
            &self.object_types,
            &mut stats,
            p.data.xp_rate_hundreds,
            p.member,
            request.recipe.xp_tenths,
        );
        if outcome != crate::assembly::Outcome::Made {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: crate::assembly::outcome_message_for_skill(outcome, first.recipe.skill),
            });
            return Ok(());
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        let pending = Pending {
            request,
            jobs: std::sync::Arc::new(jobs),
            index: 0,
            queued_at: self.clock.0,
        };
        enqueue(
            &mut p.queues,
            &pending,
            i32::from(pending.request.recipe.ticks),
        )?;
        animate(actor, pending.request.recipe, &mut self.events);
        Ok(())
    }
}
fn enqueue(
    queues: &mut crate::queues::Scheduler<Deferred>,
    pending: &Pending,
    ticks: i32,
) -> Result<(), ActionError> {
    queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            ticks + 1,
            Deferred::AssemblyProduction(pending.clone()),
        )
        .map_err(|_| ActionError::QueueFull)
}
fn animate(actor: ActorId, recipe: crate::assembly::PreciseRecipe, events: &mut Vec<WorldEvent>) {
    if let Some(sequence) = recipe.animation {
        events.push(WorldEvent::PlayerCombatAnimation { actor, sequence });
    }
    if let Some(spot) = recipe.spot {
        events.push(WorldEvent::PlayerSpotAnimation {
            actor,
            spot,
            delay: 0,
            height: 0,
            slot: 0,
        });
    }
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    mut pending: Pending,
    items: &crate::inventory::Catalog,
    clock: i32,
    events: &mut Vec<WorldEvent>,
) {
    if p.pending_logout
        || p.saving
        || p.has_modal
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || p.movement.coords != pending.request.origin
        || p.movement.last_movement != pending.request.last_movement
        || p.npc_interaction.is_some()
        || p.location_interaction.is_some()
        || p.object_interaction.is_some()
    {
        return;
    }
    let Some(&job) = pending.jobs.get(pending.index) else {
        return;
    };
    let outcome = super::held_use_actions::complete_assembly_reward(
        p,
        job,
        items,
        pending.queued_at,
        clock,
        Some(pending.request.recipe.xp_tenths),
        events,
    );
    if outcome != crate::assembly::Outcome::Made || p.pending_logout {
        return;
    }
    pending.index += 1;
    pending.queued_at = clock;
    if pending.index < pending.jobs.len() {
        if enqueue(queues, &pending, i32::from(pending.request.cycle_ticks)).is_err() {
            events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
        } else {
            animate(p.id, pending.request.recipe, events);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assembly::{PreciseRecipe, Recipe},
        held_use::{Handler, Request as Held, Scope, Type},
        inventory::Item,
    };
    fn fixture(staff: bool) -> (World, ActorId, Request) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 58);
        stats.set_current(12, 58);
        let (first, second, output) = if staff {
            (1391, 575, 1399)
        } else {
            (1759, 1675, 1694)
        };
        let mut data = PlayerData::new(
            1,
            "Pair crafter".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 1511,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        for (slot, id) in [
            (19, first),
            (3, first),
            (11, first),
            (17, second),
            (8, second),
            (22, second),
        ] {
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
        w.set_membership(a, staff).unwrap();
        for id in [first, second] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        let recipe = PreciseRecipe {
            recipe: Recipe {
                first,
                second,
                output,
                skill: 12,
                level: if staff { 58 } else { 1 },
                base_xp: 0,
                members: staff,
            },
            xp_tenths: if staff { 1125 } else { 40 },
            ticks: if staff { 3 } else { 1 },
            animation: None,
            spot: None,
        };
        w.held_use_handlers
            .register(
                Scope::Types,
                first,
                second,
                Handler::AssemblyProduction {
                    recipe,
                    cycle_ticks: 2,
                },
            )
            .unwrap();
        let held = Held {
            selected_slot: 17,
            selected_id: second,
            target_slot: 19,
            target_id: first,
        };
        w.prepare_held_use(a, held).unwrap();
        w.finish_held_use(a, held).unwrap();
        let req = w.take_assembly_production_request(a).unwrap().unwrap();
        assert_eq!((req.first_slot, req.second_slot), (19, 17));
        assert_eq!(req.available(&w.data(a).unwrap().inventory), 3);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
        (w, a, req)
    }
    #[test]
    fn pair_quantity_actual_held_dispatch_captures_selected_pair_and_preserves_fractional_batch_xp()
    {
        for staff in [false, true] {
            let (mut w, a, req) = fixture(staff);
            let start = w.clock.0;
            let first = req.recipe.ticks;
            let expected_fine = if staff { 4218 } else { 150 };
            w.start_assembly_production(a, req, 0, 28).unwrap();
            let mut traces = Vec::new();
            for _ in 0..12 {
                w.tick().unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::AssemblyMade { trace, .. } = e {
                        traces.push(trace);
                    }
                }
            }
            assert_eq!(traces.len(), 3);
            assert_eq!(
                (traces[0].job.first_slot, traces[0].job.second_slot),
                (19, 17)
            );
            assert_eq!(
                (traces[1].job.first_slot, traces[1].job.second_slot),
                (3, 8)
            );
            assert_eq!(
                (traces[2].job.first_slot, traces[2].job.second_slot),
                (11, 22)
            );
            assert_eq!(
                traces.iter().map(|t| t.committed_at).collect::<Vec<_>>(),
                vec![
                    start + i32::from(first),
                    start + i32::from(first) + 2,
                    start + i32::from(first) + 4
                ]
            );
            assert_eq!(traces[0].queued_at, start);
            assert_eq!(traces[1].queued_at, traces[0].committed_at);
            let d = w.data(a).unwrap();
            assert_eq!(d.stats.fine_xp(12), expected_fine);
            assert_eq!(d.inventory.free_space(), 3);
        }
    }
    #[test]
    fn pair_quantity_refuses_invalid_selection_and_live_policy_without_awards() {
        for case in 0..6 {
            let (mut w, a, req) = fixture(true);
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            match case {
                0 => assert!(w.start_assembly_production(a, req, 1, 1).is_err()),
                1 => assert!(w.start_assembly_production(a, req, 0, 0).is_err()),
                2 => assert!(w.start_assembly_production(a, req, 0, 29).is_err()),
                3 => {
                    w.set_membership(a, false).unwrap();
                    w.start_assembly_production(a, req, 0, 3).unwrap();
                }
                4 => {
                    w.data_mut(a).unwrap().xp_rate_hundreds = 0;
                    w.start_assembly_production(a, req, 0, 3).unwrap();
                }
                _ => {
                    w.object_types.dummy.insert(1399);
                    w.start_assembly_production(a, req, 0, 3).unwrap();
                }
            }
            for _ in 0..8 {
                w.tick().unwrap();
            }
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone()
                ),
                before,
                "case{case}"
            );
        }
    }
    #[test]
    fn pair_quantity_later_stale_slot_or_close_member_rate_credit_stops_without_substitution() {
        for case in 0..6 {
            let (mut w, a, req) = fixture(true);
            w.start_assembly_production(a, req, 0, 3).unwrap();
            for _ in 0..4 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 1406);
            match case {
                0 => {
                    let inv = &mut w.data_mut(a).unwrap().inventory;
                    inv.restore_slot(3, None).unwrap();
                    inv.restore_slot(
                        0,
                        Some(Item {
                            id: 1391,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                1 => w.close_dialogs(a).map(|_| ()).unwrap(),
                2 => w.set_membership(a, false).unwrap(),
                3 => w.data_mut(a).unwrap().xp_rate_hundreds = 0,
                4 => {
                    w.object_types.dummy.insert(1399);
                }
                _ => w.data_mut(a).unwrap().stats.set_current(12, 57),
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..10 {
                w.tick().unwrap();
            }
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone()
                ),
                before,
                "case{case}"
            );
        }
    }
}
