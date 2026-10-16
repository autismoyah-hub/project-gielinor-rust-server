use super::*;
use crate::finished_potions::{Job, Outcome, Pending, QUEUE, Request};
impl World {
    pub fn take_finished_potion_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .finished_potion_request
            .take())
    }
    pub fn start_finished_potion(
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
            .finished_potions
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let recipe = *config
            .recipes
            .get(&(request.unfinished, request.secondary))
            .ok_or(ActionError::InvalidItem)?;
        let pending = Pending {
            goggles:config.goggles,
            job: Job {
                recipe,
                unfinished_slot: request.unfinished_slot,
                secondary_slot: request.secondary_slot,
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
        // Wiki recipe2ticks: input-cycle decrement means queue3 commits two clocks later.
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                3,
                Deferred::FinishedPotion(pending),
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
    random: &mut impl RandomSource,
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
    let fine_before = p.data.stats.fine_xp(15);
    let rate_hundreds = p.data.xp_rate_hundreds;
    let eligible_goggles=p.data.equipment.get(0).is_some_and(|item| item.count==1 && item.vars==0 && pending.goggles.contains(&item.id));
    let (outcome,secondary_saved) = pending.job.complete_with_goggles(
        &mut p.data.inventory,
        catalog,
        &mut p.data.stats,
        p.member,
        eligible,
        rate_hundreds,
        eligible_goggles,
        random,
    );
    let message = match outcome {
        Outcome::Made(_) => "You mix the ingredients into your potion.",
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
    let Outcome::Made(advancement) = outcome else {
        return;
    };
    events.push(WorldEvent::SkillAdvanced {
        actor: p.id,
        skill: 15,
        advancement,
    });
    events.push(WorldEvent::FinishedPotionMade {
        actor: p.id,
        trace: crate::finished_potions::Transition {
            secondary_saved,
            job: pending.job,
            queued_at: pending.queued_at,
            committed_at: clock,
            index: pending.index,
            fine_before,
            fine_after: p.data.stats.fine_xp(15),
            rate_hundreds,
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
    let (Some(unfinished_slot), Some(secondary_slot)) = (
        slot(pending.job.recipe.unfinished),
        slot(pending.job.recipe.secondary),
    ) else {
        return;
    };
    let next = Pending {
        job: Job {
            unfinished_slot,
            secondary_slot,
            ..pending.job
        },
        remaining: pending.remaining - 1,
        queued_at: clock,
        index: pending.index + 1,
        ..pending
    };
    // Callback-added queue3 preserves two processing-clock intervals per output.
    if queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            3,
            Deferred::FinishedPotion(next),
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
        finished_potions::{Config, Recipe},
        held_use,
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
    fn fixture() -> (World, ActorId, held_use::Request) {
        let mut w = World::default();
        let pos = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(3200, 3200, 0);
        w.finished_potions = Some(Config {
            goggles:[29974,29976],
            recipes: [(
                (93, 235),
                Recipe {
                    unfinished: 93,
                    secondary: 235,
                    output: 175,
                    level: 5,
                    boostable: true,
                    base_fine_xp: 375,
                },
            )]
            .into(),
            quest_progress: 80,
            quest_complete: 4,
            animation: 363,
        });
        w.held_use_types = [93, 235, 995]
            .map(|id| (id, held_use::Type { id, content: 0 }))
            .into();
        let mut data = PlayerData::new(1, "Finished".into(), pos, Stats::default());
        data.stats.set_base(15, 5);
        data.stats.set_current(15, 5);
        data.stats.set_fine_xp(15, 3881).unwrap();
        data.xp_rate_hundreds = 500;
        data.variables.set(quest(), 4, false);
        for slot in 0..28 {
            let id = match slot {
                0 | 2 => 93,
                1 | 3 => 235,
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
                selected_id: 235,
                selected_slot: 3,
                target_id: 93,
                target_slot: 2,
            },
        )
    }
    fn choose(w: &mut World, a: ActorId, r: held_use::Request) -> Request {
        w.prepare_held_use(a, r).unwrap();
        w.finish_held_use(a, r).unwrap();
        w.take_finished_potion_request(a).unwrap().unwrap()
    }
    fn products(w: &World, a: ActorId) -> usize {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 175)
            .count()
    }
    #[test]
    fn selected_pair_and_repeats_take_two_ticks_with_exact_fractional_rewards() {
        let (mut w, a, r) = fixture();
        let before = w.data(a).unwrap().stats.clone();
        let request = choose(&mut w, a, r);
        assert_eq!((request.unfinished_slot, request.secondary_slot), (2, 3));
        w.start_finished_potion(a, request, 0, 28).unwrap();
        w.tick().unwrap();
        w.tick().unwrap();
        assert_eq!(products(&w, a), 0);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(15), 5756);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 93);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 1);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 2);
        w.tick().unwrap();
        assert_eq!(products(&w, a), 2);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(15), 7631);
        for s in 4..28 {
            assert_eq!(
                w.data(a).unwrap().inventory.get(s),
                Some(Item {
                    id: 995,
                    count: 1,
                    vars: 0
                })
            );
        }
        for skill in 0..25 {
            if skill != 15 {
                assert_eq!(
                    w.data(a).unwrap().stats.fine_xp(skill),
                    before.fine_xp(skill)
                );
            }
        }
        let events = w.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 15, .. }))
                .count(),
            2
        );
        let traces: Vec<_> = events
            .into_iter()
            .filter_map(|e| {
                if let WorldEvent::FinishedPotionMade { trace, .. } = e {
                    Some(trace)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(traces.len(), 2);
        for (i, t) in traces.iter().enumerate() {
            assert_eq!(t.committed_at - t.queued_at, 2);
            assert_eq!(t.index, i as u32);
            assert_eq!(
                (t.job.unfinished_slot, t.job.secondary_slot),
                [(2, 3), (0, 1)][i]
            );
            assert_eq!(t.rate_hundreds, 500);
            assert_eq!(t.fine_after - t.fine_before, 1875);
        }
        assert_eq!(traces[1].queued_at, traces[0].committed_at);
        assert_eq!(traces[1].fine_before, traces[0].fine_after);
    }
    #[test]
    fn live_eligibility_credit_and_rate_failures_conserve_items_and_stats() {
        for mode in 0..6 {
            let (mut w, a, r) = fixture();
            let request = choose(&mut w, a, r);
            w.start_finished_potion(a, request, 0, 2).unwrap();
            match mode {
                0 => w.set_membership(a, false).unwrap(),
                1 => {
                    w.data_mut(a).unwrap().variables.set(quest(), 3, false);
                }
                2 => w.data_mut(a).unwrap().stats.set_current(15, 4),
                3 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(3, None)
                    .unwrap(),
                4 => {
                    w.object_types.dummy.insert(175);
                }
                _ => w.data_mut(a).unwrap().xp_rate_hundreds = 0,
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone()
                ),
                before
            );
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::FinishedPotionMade { .. } | WorldEvent::SkillAdvanced { skill: 15, .. }
            )));
        }
    }
    #[test]
    fn repeats_use_current_rate_and_do_not_restart_after_quest_revocation() {
        let (mut w, a, r) = fixture();
        let request = choose(&mut w, a, r);
        w.start_finished_potion(a, request, 0, 28).unwrap();
        for _ in 0..3 {
            w.tick().unwrap();
        }
        assert_eq!(products(&w, a), 1);
        w.data_mut(a).unwrap().xp_rate_hundreds = 100;
        w.tick().unwrap();
        w.tick().unwrap();
        assert_eq!(products(&w, a), 2);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(15), 6131);
        let (mut w, a, r) = fixture();
        let request = choose(&mut w, a, r);
        w.start_finished_potion(a, request, 0, 28).unwrap();
        for _ in 0..3 {
            w.tick().unwrap();
        }
        w.data_mut(a).unwrap().variables.set(quest(), 3, false);
        w.tick().unwrap();
        w.tick().unwrap();
        assert_eq!(products(&w, a), 1);
        w.data_mut(a).unwrap().variables.set(quest(), 4, false);
        for _ in 0..3 {
            w.tick().unwrap();
        }
        assert_eq!(products(&w, a), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(15), 5756);
    }
    #[test]
    fn invalid_selection_and_intervening_held_action_never_award() {
        let (mut w, a, r) = fixture();
        let before = w.data(a).unwrap().stats.clone();
        let request = choose(&mut w, a, r);
        assert!(w.start_finished_potion(a, request, 1, 1).is_err());
        assert!(w.start_finished_potion(a, request, 0, 0).is_err());
        assert!(w.start_finished_potion(a, request, 0, 29).is_err());
        w.start_finished_potion(a, request, 0, 2).unwrap();
        let other = held_use::Request {
            selected_id: 995,
            selected_slot: 4,
            target_id: 995,
            target_slot: 5,
        };
        w.prepare_held_use(a, other).unwrap();
        w.finish_held_use(a, other).unwrap();
        for _ in 0..5 {
            w.tick().unwrap();
        }
        assert_eq!(products(&w, a), 0);
        assert_eq!(w.data(a).unwrap().stats, before);
    }
    #[test]
    fn goggles_variants_are_rechecked_on_each_repeat_and_preserve_saved_slot() {
        struct Save { calls: usize }
        impl RandomSource for Save {
            fn below(&mut self, bound:u32)->u32 {assert_eq!(bound,10);self.calls+=1;0}
        }
        for hat in [29974,29976] {
            let (mut w,a,r)=fixture();
            w.data_mut(a).unwrap().equipment.restore_slot(0,Some(Item{id:hat,count:1,vars:0})).unwrap();
            let request=choose(&mut w,a,r);w.start_finished_potion(a,request,0,28).unwrap();
            let mut random=Save{calls:0};
            for _ in 0..3 {w.tick_with_random(&mut random).unwrap();}
            assert_eq!(random.calls,1);assert_eq!(products(&w,a),1);
            assert_eq!(w.data(a).unwrap().inventory.get(3),Some(Item{id:235,count:1,vars:0}));
            assert_eq!(w.data(a).unwrap().stats.fine_xp(15),5756);
            w.data_mut(a).unwrap().equipment.restore_slot(0,None).unwrap();
            for _ in 0..2 {w.tick_with_random(&mut random).unwrap();}
            assert_eq!(products(&w,a),2);assert_eq!(random.calls,1);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(15),7631);
            let saved=w.drain_events().iter().filter_map(|e|match e{WorldEvent::FinishedPotionMade{trace,..}=>Some(trace.secondary_saved),_=>None}).collect::<Vec<_>>();
            assert_eq!(saved,[true,false]);
            for _ in 0..4 {w.tick_with_random(&mut random).unwrap();}
            assert_eq!(random.calls,1);assert_eq!(products(&w,a),2);
        }
    }

}
