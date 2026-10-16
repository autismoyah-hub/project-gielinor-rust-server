use super::*;
use crate::potion_upgrades::{Job, Outcome, Pending, QUEUE, Request};
impl World {
    pub fn take_potion_upgrade_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .potion_upgrade_request
            .take())
    }
    pub fn start_potion_upgrade(
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
            .potion_upgrades
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let recipe = *config
            .recipes
            .get(&(request.base, request.secondary))
            .ok_or(ActionError::InvalidItem)?;
        let pending = Pending {
            chemistry:config.chemistry.get(&recipe.output).copied(),
            goggles:config.goggles,
            job: Job {
                recipe,
                base_slot: request.base_slot,
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
        // Input-cycle decrement: queue delay is the validated recipe ticks plus one.
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                i32::from(recipe.ticks)+1,
                Deferred::PotionUpgrade(pending),
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
    worn_changes: &mut Vec<(ActorId, usize, i32)>,
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
    let result = pending.job.complete_with_modifiers(
        &mut p.data.inventory,
        catalog,
        &mut p.data.stats,
        &mut p.data.equipment,
        &mut p.data.chemistry,
        p.member,
        eligible,
        rate_hundreds,
        eligible_goggles,
        pending.chemistry,
        random,
    );
    let outcome=result.outcome;
    let secondary_saved=result.saved;
    if result.chemistry.crumbled {
        worn_changes.push((p.id,2,pending.chemistry.expect("validated chemistry binding").amulet));
        events.push(WorldEvent::LocationMessage{actor:p.id,message:"Your amulet of chemistry has crumbled to dust."});
    }
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
    events.push(WorldEvent::PotionUpgradeMade {
        actor: p.id,
        trace: crate::potion_upgrades::Transition {
            output:result.output,
            chemistry:result.chemistry,
            charges_remaining:p.data.chemistry.remaining(),
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
    if pending.remaining <= 1 || result.chemistry.stop {
        return;
    }
    let slot = |id| {
        p.data
            .inventory
            .slots()
            .iter()
            .position(|v| v.is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0))
    };
    let Some(base_slot)=slot(pending.job.recipe.base) else{return;};
    let secondary_slot=pending.job.secondary_slot;
    let required=i32::from(pending.job.recipe.doses)*i32::from(pending.job.recipe.secondary_per_dose);
    if p.data.inventory.get(secondary_slot).is_none_or(|i|i.id!=pending.job.recipe.secondary || i.count<required || i.vars!=0){return;}

    let next = Pending {
        job: Job {
            base_slot,
            secondary_slot,
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
            i32::from(pending.job.recipe.ticks)+1,
            Deferred::PotionUpgrade(next),
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
    use crate::{held_use,inventory::Item,potion_upgrades::{Config,Recipe},variables::{Transmit,Varp}};
    fn quest()->Varp{Varp{id:80,permanent:true,transmit:Transmit::Always,bit_protect:false,client_code:-1}}
    fn fixture()->(World,ActorId,held_use::Request){
        let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
        let recipe=Recipe{base:3018,secondary:12640,output:12627,doses:3,secondary_per_dose:1,fine_xp_per_dose:255,level:77,ticks:2,allow_goggles:true};
        w.potion_upgrades=Some(Config{recipes:[((3018,12640),recipe)].into(),chemistry:Default::default(),goggles:[29974,29976],quest_progress:80,quest_complete:4,animation:363});
        w.held_use_types=[3018,12640,995].map(|id|(id,held_use::Type{id,content:0})).into();w.object_types.stackable.extend([12640,995]);
        let mut data=PlayerData::new(1,"Upgrades".into(),pos,Stats::default());data.stats.set_base(15,99);data.stats.set_current(15,99);data.stats.set_fine_xp(15,130344311).unwrap();data.xp_rate_hundreds=500;data.variables.set(quest(),4,false);
        for slot in 0..28 {let (id,count)=match slot{0=>(12640,7),2|4=>(3018,1),_=>(995,100)};data.inventory.restore_slot(slot,Some(Item{id,count,vars:0})).unwrap();}
        w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();
        (w,a,held_use::Request{selected_id:12640,selected_slot:0,target_id:3018,target_slot:4})
    }
    fn choose(w:&mut World,a:ActorId,r:held_use::Request){w.prepare_held_use(a,r).unwrap();w.finish_held_use(a,r).unwrap();let request=w.take_potion_upgrade_request(a).unwrap().unwrap();assert_eq!((request.base_slot,request.secondary_slot),(4,0));w.start_potion_upgrade(a,request,0,28).unwrap();}
    #[test]
    fn selected_base_repeat_uses_same_stack_and_two_tick_fractional_xp(){
        let (mut w,a,r)=fixture();let before=w.data(a).unwrap().clone();choose(&mut w,a,r);
        for _ in 0..2{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before.inventory);
        for _ in 0..6{w.tick().unwrap();}
        let data=w.data(a).unwrap();assert_eq!(data.stats.fine_xp(15),130351961);assert_eq!(data.inventory.get(0),Some(Item{id:12640,count:1,vars:0}));
        for slot in 0..28{if slot!=2 && slot!=4 && slot!=0{assert_eq!(data.inventory.get(slot),before.inventory.get(slot));}}
        let traces:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::PotionUpgradeMade{trace,..}=e{Some(trace)}else{None}).collect();assert_eq!(traces.len(),2);
        for (i,t) in traces.iter().enumerate(){assert_eq!(t.committed_at-t.queued_at,2);assert_eq!(t.job.base_slot,[4,2][i]);assert_eq!(t.job.secondary_slot,0);assert_eq!(t.fine_after-t.fine_before,3825);}
    }
    #[test]
    fn repeat_revalidates_live_level_quest_membership_stack_and_movement(){
        for mode in 0..5{let (mut w,a,r)=fixture();choose(&mut w,a,r);for _ in 0..3{w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130348136);
            match mode{0=>w.data_mut(a).unwrap().stats.set_current(15,76),1=>{w.data_mut(a).unwrap().variables.set(quest(),0,false);},2=>w.set_membership(a,false).unwrap(),3=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:12640,count:2,vars:0})).unwrap();},_=>w.players.get_mut(a.slot as usize).unwrap().movement.last_movement+=1}
            let before=w.data(a).unwrap().inventory.clone();w.drain_events();for _ in 0..5{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before);assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130348136);
            assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::PotionUpgradeMade{..})));
        }
    }
    #[test]
    fn three_tick_upgrades_preserve_dose_scaled_reward_and_repeat_boundary(){
        let (mut w,a,r)=fixture();let recipe=w.potion_upgrades.as_mut().unwrap().recipes.get_mut(&(3018,12640)).unwrap();recipe.ticks=3;recipe.level=94;recipe.fine_xp_per_dose=200;
        choose(&mut w,a,r);for _ in 0..3{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130344311);
        w.tick().unwrap();assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130347311);
        for _ in 0..2{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130347311);
        w.tick().unwrap();assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130350311);
        let traces:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::PotionUpgradeMade{trace,..}=e{Some(trace)}else{None}).collect();assert_eq!(traces.len(),2);
        for t in &traces{assert_eq!(t.committed_at-t.queued_at,3);assert_eq!(t.fine_after-t.fine_before,3000);}assert_eq!(traces[1].queued_at,traces[0].committed_at);
    }
}
