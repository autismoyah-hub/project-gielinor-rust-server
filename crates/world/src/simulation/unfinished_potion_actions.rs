use super::*;
use crate::unfinished_potions::{Job, Outcome, Pending, QUEUE, Request};
impl World {
    pub fn take_unfinished_potion_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .unfinished_potion_request
            .take())
    }
    pub fn start_unfinished_potion(
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
            || p.movement.coords != request.origin
            || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidItem);
        }
        let config = self
            .unfinished_potions
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let recipe = *config
            .recipes
            .get(&(request.herb, request.vessel))
            .ok_or(ActionError::InvalidItem)?;
        if !(1..=2).contains(&recipe.ticks) { return Err(ActionError::InvalidItem); }
        let pending = Pending {
            job: Job {
                recipe,
                herb_slot: request.herb_slot,
                vessel_slot: request.vessel_slot,
            },
            request,
            remaining: count,
            queued_at: self.clock.0,
            index: 0,
            quest_progress: config.quest_progress,
            quest_complete: config.quest_complete,
            animation: config.animation,
        };
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        // Input-cycle decrement requires one extra queue count beyond recipe ticks.
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                i32::from(recipe.ticks) + 1,
                Deferred::UnfinishedPotion(pending),
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
    let eligible = p.data.variables.get(pending.quest_progress) == pending.quest_complete;
    let outcome = pending.job.complete(
        &mut p.data.inventory,
        catalog,
        &mut p.data.stats,
        p.member,
        eligible,
    );
    let message = match outcome {
        Outcome::Made => match pending.job.recipe.vessel {
            227 => "You put the herb into the vial of water.",
            5935 => "You put the herb into the coconut milk.",
            22446 => "You put the herb into the vial of blood.",
            _ => "You mix the herb into the liquid.",
        },
        Outcome::Members => "You need to be a member to make potions.",
        Outcome::Quest => "You must complete Druidic Ritual before making potions.",
        Outcome::Level => "You need a higher Herblore level to make that potion.",
        Outcome::Capacity => "You cannot make that potion in this inventory.",
        _ => return,
    };
    events.push(WorldEvent::LocationMessage {
        actor: p.id,
        message,
    });
    if outcome != Outcome::Made {
        return;
    }
    events.push(WorldEvent::UnfinishedPotionMade {
        actor: p.id,
        trace: crate::unfinished_potions::Transition {
            job: pending.job,
            queued_at: pending.queued_at,
            committed_at: clock,
            index: pending.index,
        },
    });
    events.push(WorldEvent::PlayerCombatAnimation {
        actor: p.id,
        sequence: pending.animation,
    });
    if pending.remaining <= 1 {
        return;
    }
    let slot = |id| {
        p.data
            .inventory
            .slots()
            .iter()
            .position(|v| v.is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0))
    };
    let (Some(herb_slot), Some(vessel_slot)) = (
        slot(pending.job.recipe.herb),
        slot(pending.job.recipe.vessel),
    ) else {
        return;
    };
    let next = Pending {
        job: Job {
            herb_slot,
            vessel_slot,
            ..pending.job
        },
        remaining: pending.remaining - 1,
        queued_at: clock,
        index: pending.index + 1,
        ..pending
    };
    // Preserve the selected recipe cadence on every continuation.
    if queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            i32::from(next.job.recipe.ticks) + 1,
            Deferred::UnfinishedPotion(next),
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
        held_use,
        inventory::Item,
        unfinished_potions::{Config, Recipe},
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
    fn fixture() -> (World, ActorId, held_use::Request) {
        let mut w = World::default();
        let pos = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(3200, 3200, 0);
        w.unfinished_potions = Some(Config {
            recipes: [(
                (249, 227),
                Recipe {
                    herb: 249,
                    vessel: 227,
                    output: 91,
                    level: 3,
                    ticks: 1,
                },
            )]
            .into(),
            quest_progress: 80,
            quest_complete: 4,
            animation: 363,
        });
        w.held_use_types = [249, 227, 995]
            .map(|id| (id, held_use::Type { id, content: 0 }))
            .into();
        let mut data = PlayerData::new(1, "Potion".into(), pos, Stats::default());
        data.stats.set_current(15, 3);
        data.stats.set_fine_xp(15, 3881).unwrap();
        data.variables.set(quest(), 4, false);
        for slot in 0..28 {
            let id = match slot {
                0 | 2 => 249,
                1 | 3 => 227,
                _ => 995,
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
        (
            w,
            a,
            held_use::Request {
                selected_id: 227,
                selected_slot: 3,
                target_id: 249,
                target_slot: 2,
            },
        )
    }
    fn choose(w: &mut World, a: ActorId, r: held_use::Request) -> Request {
        w.prepare_held_use(a, r).unwrap();
        w.finish_held_use(a, r).unwrap();
        w.take_unfinished_potion_request(a).unwrap().unwrap()
    }
    fn products(w: &World, a: ActorId) -> usize {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 91)
            .count()
    }
    #[test]
    fn selected_pair_then_quantity_repeat_are_one_tick_and_zero_xp() {
        let (mut w, a, r) = fixture();
        let stats = w.data(a).unwrap().stats.clone();
        let request = choose(&mut w, a, r);
        assert_eq!((request.herb_slot, request.vessel_slot), (2, 3));
        w.start_unfinished_potion(a, request, 0, 28).unwrap();
        w.tick().unwrap();
        assert_eq!(products(&w, a), 0);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 1);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 249);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 2);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 2);
        assert_eq!(w.data(a).unwrap().stats, stats);
        for slot in 4..28 {
            assert_eq!(
                w.data(a).unwrap().inventory.get(slot),
                Some(Item {
                    id: 995,
                    count: 1,
                    vars: 0
                })
            );
        }
        let traces: Vec<_> = w
            .drain_events()
            .into_iter()
            .filter_map(|e| {
                if let WorldEvent::UnfinishedPotionMade { trace, .. } = e {
                    Some(trace)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(traces.len(), 2);
        for (i, t) in traces.iter().enumerate() {
            assert_eq!(t.committed_at - t.queued_at, 1);
            assert_eq!(t.index, i as u32);
            assert_eq!((t.job.herb_slot, t.job.vessel_slot), [(2, 3), (0, 1)][i]);
        }
        assert_eq!(traces[1].queued_at, traces[0].committed_at);
    }
    #[test]
    fn eligibility_and_selected_slots_are_rechecked_after_quantity_selection() {
        for mode in 0..5 {
            let (mut w, a, r) = fixture();
            let request = choose(&mut w, a, r);
            w.start_unfinished_potion(a, request, 0, 2).unwrap();
            match mode {
                0 => w.set_membership(a, false).unwrap(),
                1 => {
                    w.data_mut(a).unwrap().variables.set(quest(), 3, false);
                }
                2 => w.data_mut(a).unwrap().stats.set_current(15, 2),
                3 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(3, None)
                    .unwrap(),
                _ => {
                    w.object_types.dummy.insert(91);
                }
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..4 {
                w.tick().unwrap();
            }
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone()
                ),
                before
            );
            assert!(
                !w.drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::UnfinishedPotionMade { .. }))
            );
        }
    }
    #[test]
    fn revoked_quest_stops_repeats_and_does_not_revive_after_restoration() {
        let (mut w, a, r) = fixture();
        let request = choose(&mut w, a, r);
        w.start_unfinished_potion(a, request, 0, 28).unwrap();
        w.tick().unwrap();
        w.tick().unwrap();
        assert_eq!(products(&w, a), 1);
        w.data_mut(a).unwrap().variables.set(quest(), 3, false);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 1);
        w.data_mut(a).unwrap().variables.set(quest(), 4, false);
        for _ in 0..3 {
            w.tick().unwrap();
        }
        assert_eq!(products(&w, a), 1);
    }
    #[test]
    fn foreign_choice_and_intervening_held_action_cannot_mix() {
        let (mut w, a, r) = fixture();
        let request = choose(&mut w, a, r);
        assert!(w.start_unfinished_potion(a, request, 1, 1).is_err());
        assert!(w.start_unfinished_potion(a, request, 0, 0).is_err());
        assert!(w.start_unfinished_potion(a, request, 0, 29).is_err());
        w.start_unfinished_potion(a, request, 0, 2).unwrap();
        let other = held_use::Request {
            selected_id: 995,
            selected_slot: 4,
            target_id: 995,
            target_slot: 5,
        };
        w.prepare_held_use(a, other).unwrap();
        w.finish_held_use(a, other).unwrap();
        for _ in 0..4 {
            w.tick().unwrap();
        }
        assert_eq!(products(&w, a), 0);
    }
    #[test]
    fn alternate_vessels_preserve_water_choices_and_recipe_cadence() {
        let recipes = [
            Recipe { herb:2998, vessel:227, output:3002, level:34, ticks:1 },
            Recipe { herb:2998, vessel:5935, output:5942, level:68, ticks:2 },
            Recipe { herb:259, vessel:227, output:101, level:45, ticks:1 },
            Recipe { herb:259, vessel:5935, output:5951, level:79, ticks:1 },
            Recipe { herb:265, vessel:227, output:107, level:66, ticks:1 },
            Recipe { herb:265, vessel:22446, output:22443, level:80, ticks:1 },
        ];
        for recipe in recipes {
            for reverse in [false, true] {
                let (mut w,a,_) = fixture();
                w.unfinished_potions.as_mut().unwrap().recipes = recipes.into_iter()
                    .map(|r| ((r.herb,r.vessel),r)).collect();
                assert_eq!(w.unfinished_potions.as_ref().unwrap().recipes.len(),6);
                for id in [recipe.herb,recipe.vessel] {
                    w.held_use_types.insert(id,held_use::Type{id,content:0});
                }
                let data = w.data_mut(a).unwrap();
                data.stats.set_base(15,99);data.stats.set_current(15,99);
                data.stats.set_fine_xp(15,130344311).unwrap();
                for slot in 0..4 {
                    data.inventory.restore_slot(slot,Some(Item {
                        id:if slot%2==0 {recipe.herb} else {recipe.vessel},count:1,vars:0,
                    })).unwrap();
                }
                let original=data.clone();
                let held=held_use::Request {
                    selected_id:if reverse {recipe.vessel} else {recipe.herb},
                    target_id:if reverse {recipe.herb} else {recipe.vessel},
                    selected_slot:if reverse {3} else {2},target_slot:if reverse {2} else {3},
                };
                let request=choose(&mut w,a,held);
                assert_eq!((request.herb,request.vessel,request.herb_slot,request.vessel_slot),
                    (recipe.herb,recipe.vessel,2,3));
                w.start_unfinished_potion(a,request,0,28).unwrap();
                for _ in 0..(recipe.ticks*2+3) {w.tick().unwrap();}
                let data=w.data(a).unwrap();
                assert_eq!(data.stats,original.stats);
                for slot in [0,2] {assert_eq!(data.inventory.get(slot),Some(Item{id:recipe.output,count:1,vars:0}));}
                for slot in [1,3] {assert_eq!(data.inventory.get(slot),None);}
                for slot in 4..28 {assert_eq!(data.inventory.get(slot),original.inventory.get(slot));}
                let events=w.drain_events();
                let expected_message=match recipe.vessel {
                    227=>"You put the herb into the vial of water.",
                    5935=>"You put the herb into the coconut milk.",
                    22446=>"You put the herb into the vial of blood.",
                    _=>unreachable!(),
                };
                assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::LocationMessage{message,..} if *message==expected_message)).count(),2);
                let traces:Vec<_>=events.into_iter().filter_map(|e| {
                    if let WorldEvent::UnfinishedPotionMade{trace,..}=e {Some(trace)} else {None}
                }).collect();
                assert_eq!(traces.len(),2);
                for (index,t) in traces.iter().enumerate() {
                    assert_eq!(t.committed_at-t.queued_at,i32::from(recipe.ticks));
                    assert_eq!((t.job.recipe.herb,t.job.recipe.vessel,t.job.recipe.output),(recipe.herb,recipe.vessel,recipe.output));
                    assert_eq!((t.job.herb_slot,t.job.vessel_slot),[(2,3),(0,1)][index]);
                    assert_eq!(t.index,index as u32);
                }
                assert_eq!(traces[1].queued_at,traces[0].committed_at);
                for _ in 0..3 {w.tick().unwrap();}
                assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::UnfinishedPotionMade{..})));
            }
        }
    }

}
