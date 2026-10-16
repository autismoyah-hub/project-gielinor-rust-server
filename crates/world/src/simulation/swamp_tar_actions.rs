use super::*;
use crate::swamp_tar::{Job,Outcome,Pending,QUEUE};
fn refusal(outcome:Outcome)->Option<&'static str> {
    match outcome {
        Outcome::Members=>Some("You need to be a member to make herb tar."),
        Outcome::Quest=>Some("You must complete Druidic Ritual before making herb tar."),
        Outcome::Level=>Some("You need a higher Herblore level to make that tar."),
        Outcome::Tool=>Some("You need a pestle and mortar to make herb tar."),
        Outcome::Capacity=>Some("You cannot make that tar in this inventory."),
        _=>None,
    }
}
impl World {
    pub(super) fn start_swamp_tar(&mut self,actor:ActorId,job:Job)->Result<(),ActionError> {
        let config=self.swamp_tar.as_ref().ok_or(ActionError::InvalidItem)?;
        let p=self.player(actor)?;
        let outcome=job.complete(&mut p.data.inventory.clone(),&self.object_types,&mut p.data.stats.clone(),p.member,p.data.variables.get(config.quest_progress)==config.quest_complete,p.data.xp_rate_hundreds);
        if !matches!(outcome,Outcome::Made(_)) {
            if let Some(message)=refusal(outcome) {self.events.push(WorldEvent::LocationMessage{actor,message});}
            return Ok(());
        }
        let pending=Pending{job,origin:p.movement.coords,last_movement:p.movement.last_movement,queued_at:self.clock.0,index:0,quest_progress:config.quest_progress,quest_complete:config.quest_complete,animation:config.animation};
        let p=self.players.get_mut(actor.slot as usize).unwrap();
        p.queues.clear_weak();
        // Input-cycle decrement: four queue counts yield three clock ticks.
        p.queues.add(QUEUE,crate::queues::Category::Weak,4,Deferred::SwampTar(pending)).map_err(|_|ActionError::QueueFull)?;
        Ok(())
    }
}
pub(super) fn complete(p:&mut Player,queues:&mut crate::queues::Scheduler<Deferred>,pending:Pending,catalog:&crate::inventory::Catalog,clock:i32,events:&mut Vec<WorldEvent>) {
    if p.pending_logout || p.saving || p.has_modal || p.location_interaction.is_some() || p.npc_interaction.is_some() || p.object_interaction.is_some() || p.movement.coords!=pending.origin || p.movement.last_movement!=pending.last_movement {return;}
    let fine_before=p.data.stats.fine_xp(15);let rate_hundreds=p.data.xp_rate_hundreds;
    let eligible=p.data.variables.get(pending.quest_progress)==pending.quest_complete;
    let outcome=pending.job.complete(&mut p.data.inventory,catalog,&mut p.data.stats,p.member,eligible,rate_hundreds);
    let Outcome::Made(advancement)=outcome else {
        if let Some(message)=refusal(outcome) {events.push(WorldEvent::LocationMessage{actor:p.id,message});}
        return;
    };
    events.push(WorldEvent::SkillAdvanced{actor:p.id,skill:15,advancement});
    events.push(WorldEvent::SwampTarMade{actor:p.id,trace:crate::swamp_tar::Transition{job:pending.job,queued_at:pending.queued_at,committed_at:clock,index:pending.index,fine_before,fine_after:p.data.stats.fine_xp(15),rate_hundreds}});
    events.push(WorldEvent::PlayerCombatAnimation{actor:p.id,sequence:pending.animation});
    events.push(WorldEvent::LocationMessage{actor:p.id,message:"You mix the herb into the swamp tar."});
    let recipe=pending.job.recipe;
    let Some(herb_slot)=p.data.inventory.slots().iter().position(|i|i.is_some_and(|i|i.id==recipe.herb && i.count==1 && i.vars==0)) else {return;};
    // Keep the originally selected tar stack; never consume a replacement stack elsewhere.
    if p.data.inventory.get(pending.job.tar_slot).is_none_or(|i|i.id!=recipe.tar || i.count<crate::swamp_tar::BATCH || i.vars!=0) {return;}
    let next=Pending{job:Job{herb_slot,..pending.job},queued_at:clock,index:pending.index+1,..pending};
    if queues.add(QUEUE,crate::queues::Category::Weak,4,Deferred::SwampTar(next)).is_err() {events.push(WorldEvent::ActionRejected{actor:p.id,error:ActionError::QueueFull});}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{held_use,inventory::Item,swamp_tar::{Config,Recipe},variables::{Transmit,Varp}};
    fn quest()->Varp {Varp{id:80,permanent:true,transmit:Transmit::Always,bit_protect:false,client_code:-1}}
    fn fixture()->(World,ActorId) {
        let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
        let recipe=Recipe{herb:251,tar:1939,output:10143,tool:233,level:31,base_fine_xp:425};
        w.swamp_tar=Some(Config{recipes:[((251,1939),recipe)].into(),quest_progress:80,quest_complete:4,animation:363});
        w.held_use_types=[251,1939,233,995].map(|id|(id,held_use::Type{id,content:0})).into();
        w.object_types.stackable.extend([1939,10143,995]);
        let mut data=PlayerData::new(1,"Tar".into(),pos,Stats::default());
        data.stats.set_base(15,99);data.stats.set_current(15,99);data.stats.set_fine_xp(15,130344311).unwrap();data.variables.set(quest(),4,false);data.xp_rate_hundreds=500;
        for slot in 0..28 {
            let (id,count)=match slot {0=>(1939,45),1=>(233,1),2|4=>(251,1),_=>(995,100)};
            data.inventory.restore_slot(slot,Some(Item{id,count,vars:0})).unwrap();
        }
        w.queue_login(1,data).unwrap();w.tick().unwrap();let actor=w.actors().next().unwrap();w.set_membership(actor,true).unwrap();w.drain_events();(w,actor)
    }
    fn choose(w:&mut World,a:ActorId,reverse:bool) {
        let r=held_use::Request{selected_id:if reverse {1939}else{251},target_id:if reverse {251}else{1939},selected_slot:if reverse {0}else{4},target_slot:if reverse {4}else{0}};
        w.prepare_held_use(a,r).unwrap();w.finish_held_use(a,r).unwrap();
    }
    #[test]
    fn native_order_selected_herb_then_repeat_three_ticks_and_exact_fine_xp() {
        for reverse in [false,true] {
            let (mut w,a)=fixture();let original=w.data(a).unwrap().clone();choose(&mut w,a,reverse);
            for _ in 0..2 {w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().inventory,original.inventory);
            for _ in 0..6 {w.tick().unwrap();}
            let data=w.data(a).unwrap();assert_eq!(data.stats.fine_xp(15),130348561);
            assert_eq!(data.inventory.get(0),Some(Item{id:1939,count:15,vars:0}));
            assert_eq!(data.inventory.get(4),Some(Item{id:10143,count:30,vars:0}));assert_eq!(data.inventory.get(2),None);
            for slot in 1..28 {if slot!=2 && slot!=4 {assert_eq!(data.inventory.get(slot),original.inventory.get(slot));}}
            let traces:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::SwampTarMade{trace,..}=e{Some(trace)}else{None}).collect();
            assert_eq!(traces.len(),2);
            for (i,t) in traces.iter().enumerate(){assert_eq!(t.committed_at-t.queued_at,3);assert_eq!(t.job.herb_slot,[4,2][i]);assert_eq!(t.job.tar_slot,0);assert_eq!(t.fine_after-t.fine_before,2125);assert_eq!(t.index,i as u32);}
            assert_eq!(traces[1].queued_at,traces[0].committed_at);
            for _ in 0..5 {w.tick().unwrap();}
            assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::SwampTarMade{..})));
        }
    }
    #[test]
    fn repeat_rechecks_tool_level_quest_membership_stack_and_movement() {
        for mode in 0..6 {
            let (mut w,a)=fixture();choose(&mut w,a,false);for _ in 0..4 {w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130346436);
            match mode {
                0=>{w.data_mut(a).unwrap().inventory.restore_slot(1,None).unwrap();},
                1=>w.data_mut(a).unwrap().stats.set_current(15,30),
                2=>{w.data_mut(a).unwrap().variables.set(quest(),0,false);},
                3=>w.set_membership(a,false).unwrap(),
                4=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1939,count:14,vars:0})).unwrap();},
                _=>{w.players.get_mut(a.slot as usize).unwrap().movement.last_movement+=1;},
            }
            let before=w.data(a).unwrap().inventory.clone();w.drain_events();
            for _ in 0..5 {w.tick().unwrap();}
            assert_eq!(w.data(a).unwrap().inventory,before,"mode{mode}");assert_eq!(w.data(a).unwrap().stats.fine_xp(15),130346436,"mode{mode}");
            assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::SwampTarMade{..})),"mode{mode}");
        }
    }
}
