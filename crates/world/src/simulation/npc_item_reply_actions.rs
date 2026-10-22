use super::*;
use crate::npc_interaction::Handler;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item,npc::{Mode,Restrict,BlockWalk},npc_options::Definition,stats::Stats,npc_item_reply::Rule};
    fn fixture(calf:bool,item:i32,far:bool)->(World,ActorId,NpcId){
        let mut w=World::default();for x in(3192..3224).step_by(8){for z in(3192..3224).step_by(8){w.collision.allocate(x,z,0);}}
        let raw=if calf{2792}else{2791};w.npc_item_replies.insert(raw,Rule{special_item:1925,special:if calf{"Calves are too young to be milked."}else{"Only dairy cows are suitable for milking."},otherwise:if calf{"The calf doesn't want that."}else{"The cow doesn't want that."}});
        w.npc_options.types.insert(raw,Definition{operations:[None,Some("Attack".into()),None,None,None],..Default::default()});w.npc_handlers.insert((raw,2),Handler::Combat);w.held_use_types.insert(item,crate::held_use::Type{id:item,content:-1});
        let mut n=Npc::new(raw,CoordGrid::new(if far{3208}else{3201},3200,0).unwrap(),2,Restrict::Normal,BlockWalk::Npc).unwrap();n.mode=Mode::None;n.stats[3]=8;n.regen_rate=0;let target=w.spawn_npc(n).unwrap();
        let mut d=PlayerData::new(1,"Cow reply fixture".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());d.inventory.restore_slot(0,Some(Item{id:item,count:1,vars:0})).unwrap();w.queue_login(1,d).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.drain_events();(w,a,target)
    }
    #[test]
    fn held_cow_calf_bucket_and_other_items_reply_once_without_reward_or_attack(){
        for calf in[false,true]{for item in[1925,1513]{for member in[false,true]{
            let(mut w,a,n)=fixture(calf,item,false);w.set_membership(a,member).unwrap();let before=w.data(a).unwrap().clone();let stats=w.npc(n).unwrap().stats;
            w.submit(a,Action::UseNpc{target:n,slot:0,object:item,control:false}).unwrap();let mut replies=Vec::new();
            for _ in 0..5{w.tick().unwrap();replies.extend(w.drain_events().into_iter().filter_map(|e|if let WorldEvent::EngineMessage{message,..}=e{Some(message)}else{None}));}
            let rule=w.npc_item_replies[&w.npc(n).unwrap().type_id];assert_eq!(replies,vec![if item==1925{rule.special}else{rule.otherwise}]);assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(w.npc(n).unwrap().stats,stats);assert!(w.players.get(a.slot as usize).unwrap().aggressive_npc.is_none());
        }}}
    }
    #[test]
    fn held_cow_refusal_rechecks_selected_slot_life_and_visible_identity(){
        for case in 0..6{
            let(mut w,a,n)=fixture(false,1925,true);w.submit(a,Action::UseNpc{target:n,slot:0,object:1925,control:false}).unwrap();w.tick().unwrap();w.drain_events();
            match case{0=>{w.players.get_mut(a.slot as usize).unwrap().data.inventory.restore_slot(0,None).unwrap();},1=>w.npc_mut(n).unwrap().transformation=Some(2792),2=>w.npc_mut(n).unwrap().stats[3]=0,3=>w.npc_mut(n).unwrap().visible=false,4=>{w.npc_item_replies.clear();},_=>w.players.get_mut(a.slot as usize).unwrap().death_queued=true}
            for _ in 0..15{w.tick().unwrap();assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::EngineMessage{message,..}if message.contains("milking"))));}
            assert!(w.players.get(a.slot as usize).unwrap().npc_interaction.is_none());
        }
    }
}
impl World {
    pub(super) fn npc_item_reply_valid(&self,actor:ActorId,target:NpcId,raw:u16,slot:usize,item:i32)->bool {
        let Ok(p)=self.player(actor)else{return false;};
        let Some(n)=self.npcs.get(target.slot as usize).filter(|n|n.id==target&&!n.death_queued)else{return false;};
        !p.saving&&!p.pending_logout&&!p.death_queued&&p.data.stats.current(3)as u8!=0
            &&!p.has_modal&&!p.protection.delayed()&&n.npc.visible&&!n.npc.delayed&&n.npc.stats[3]>0
            &&n.npc.type_id==raw&&n.npc.visible_type()==raw&&self.npc_item_replies.contains_key(&raw)
            &&self.held_use_types.contains_key(&item)
            &&p.data.inventory.get(slot).is_some_and(|i|i.id==item&&i.count>0)
    }
    pub(super) fn request_npc_item_reply(&mut self,actor:ActorId,target:NpcId,slot:i32,item:i32,control:bool)->Result<(),ActionError>{
        let raw=self.npc(target).ok_or(ActionError::InvalidItem)?.type_id;
        let selected=usize::try_from(slot).map_err(|_|ActionError::InvalidItem)?;
        if !self.npc_item_reply_valid(actor,target,raw,selected,item){return Err(ActionError::InvalidItem);}
        // Reuse this attested attackable profile's touch approach, replacing the
        // captured behavior before any interaction processing can run.
        self.request_npc_inner(actor,target,2,control,None)?;
        if let Some(p)=self.players.get_mut(actor.slot as usize).and_then(|p|p.npc_interaction.as_mut()).filter(|p|p.target==target){p.handler=Some(Handler::HeldItemReply{raw,slot:selected,item});}
        Ok(())
    }
    pub(super) fn finish_npc_item_reply(&mut self,actor:ActorId,target:NpcId,raw:u16,slot:usize,item:i32){
        if !self.npc_item_reply_valid(actor,target,raw,slot,item){return;}
        let rule=self.npc_item_replies[&raw];
        self.events.push(WorldEvent::EngineMessage{actor,message:if item==rule.special_item{rule.special}else{rule.otherwise}});
    }
}
