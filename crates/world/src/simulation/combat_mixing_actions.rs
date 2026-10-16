use super::*;
use crate::combat_mixing::{Job, Outcome, Pending, QUEUE, Request};
impl World {
    pub fn take_combat_mixing_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .combat_mixing_request
            .take())
    }
    pub fn start_combat_mixing(
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
            .combat_mixing
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let recipe = *config
            .recipes
            .get(request.recipe_index)
            .ok_or(ActionError::InvalidItem)?;
        let pending = Pending {
            goggles:config.goggles,
            job: Job {
                recipe,
                slots:request.slots,
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
        // Input-cycle decrement: queue delay is the validated recipe ticks plus one.
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                3,
                Deferred::CombatMixing(pending),
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
    let (outcome,secondary_saved)=pending.job.complete(&mut p.data.inventory,catalog,&mut p.data.stats,p.member,eligible,rate_hundreds,eligible_goggles,random);
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
    events.push(WorldEvent::CombatMixingMade {
        actor: p.id,
        trace: crate::combat_mixing::Transition {
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
    let mut slots=[0;4];
    for i in 0..4 {let Some(found)=slot(pending.job.recipe.inputs[i])else{return;};slots[i]=found;}
    let next = Pending {
        job: Job {
            slots,
            ..pending.job
        },
        remaining: pending.remaining - 1,
        queued_at: clock,
        index: pending.index + 1,
        ..pending
    };
    // Callback-added delay preserves the validated recipe cadence per output.
    if queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            3,
            Deferred::CombatMixing(next),
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
    use crate::{held_use,inventory::Item,combat_mixing::{Config,Recipe},variables::{Transmit,Varp}};
    fn quest()->Varp{Varp{id:80,permanent:true,transmit:Transmit::Always,bit_protect:false,client_code:-1}}
    fn fixture(unf:bool,potion:usize,reverse:bool)->(World,ActorId,held_use::Request){
        let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
        let herb=if unf{111}else{269};let recipe=Recipe{inputs:[2436,2440,2442,herb],output:12695};
        w.combat_mixing=Some(Config{recipes:vec![recipe],goggles:[29974,29976],quest_progress:80,quest_complete:4,animation:363});
        w.held_use_types=[2436,2440,2442,herb,1931].map(|id|(id,held_use::Type{id,content:0})).into();
        let mut data=PlayerData::new(1,"Combat mixer".into(),pos,Stats::default());data.stats.set_base(15,99);data.stats.set_current(15,99);data.stats.set_fine_xp(15,130344311).unwrap();data.xp_rate_hundreds=500;data.variables.set(quest(),4,false);
        for slot in 0..28 {let id=if slot<8 {recipe.inputs[slot%4]}else{1931};data.inventory.restore_slot(slot,Some(Item{id,count:1,vars:0})).unwrap();}
        w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();
        let (selected_id,selected_slot,target_id,target_slot)=if reverse{(recipe.inputs[potion],(4+potion) as i32,herb,7)}else{(herb,7,recipe.inputs[potion],(4+potion) as i32)};
        (w,a,held_use::Request{selected_id,selected_slot,target_id,target_slot})
    }
    fn choose(w:&mut World,a:ActorId,r:held_use::Request){w.prepare_held_use(a,r).unwrap();w.finish_held_use(a,r).unwrap();let req=w.take_combat_mixing_request(a).unwrap().unwrap();assert_eq!(req.slots[3],7);w.start_combat_mixing(a,req,0,28).unwrap();}
    #[test]
    fn both_torstols_all_three_native_pairs_both_orders_and_two_tick_repeat(){
        for unf in [false,true]{for potion in 0..3{for reverse in [false,true]{
            let (mut w,a,r)=fixture(unf,potion,reverse);let before=w.data(a).unwrap().inventory.clone();choose(&mut w,a,r);
            for _ in 0..2{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before);
            for _ in 0..6{w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130359311);
            for slot in 8..28{assert_eq!(w.data(a).unwrap().inventory.get(slot),before.get(slot));}
            assert_eq!(w.data(a).unwrap().inventory.slots().iter().flatten().filter(|v|v.id==12695).count(),2);
            let traces:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::CombatMixingMade{trace,..}=e{Some(trace)}else{None}).collect();assert_eq!(traces.len(),2);
            assert_eq!(traces[0].job.slots[potion],4+potion);assert_eq!(traces[0].job.slots[3],7);
            for t in &traces{assert_eq!(t.committed_at-t.queued_at,2);assert_eq!(t.fine_after-t.fine_before,7500);}
            assert_eq!(traces[1].queued_at,traces[0].committed_at);
        }}}
    }
    #[test]
    fn pending_rechecks_each_input_level_membership_quest_and_movement(){
        for mode in 0..8 {
            let(mut w,a,r)=fixture(false,0,false);choose(&mut w,a,r);
            match mode {
                0=>w.data_mut(a).unwrap().stats.set_current(15,89),1=>{w.data_mut(a).unwrap().variables.set(quest(),0,false);},2=>w.set_membership(a,false).unwrap(),3=>w.players.get_mut(a.slot as usize).unwrap().movement.last_movement+=1,
                _=>{let slot=[4,1,2,7][mode-4];w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:1931,count:1,vars:0})).unwrap();}
            }
            let before=w.data(a).unwrap().inventory.clone();for _ in 0..7{w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().inventory,before,"mode{mode}");assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130344311);
            assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::CombatMixingMade{..})));
        }
    }
}
