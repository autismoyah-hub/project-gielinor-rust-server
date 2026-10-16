use super::*;
impl World {
    pub fn cancel_chemistry_dialog(&mut self,actor:ActorId)->Result<bool,ActionError> {
        self.player(actor)?;let p=self.players.get_mut(actor.slot as usize).unwrap();
        let cancelled=p.pending_chemistry.take().is_some();if cancelled{p.protection.suspended=false;}Ok(cancelled)
    }
    pub(super) fn chemistry_item(&mut self,actor:ActorId,slot:usize,item:i32,operation:u8,worn:bool)->Result<(),ActionError> {
        let amulet=self.finished_potions.as_ref().and_then(|c|c.chemistry.values().next()).map(|b|b.amulet).ok_or(ActionError::InvalidItem)?;
        let p=self.players.get_mut(actor.slot as usize).ok_or(ActionError::StaleActor)?;
        p.begin_held_action();p.queues.clear_weak();
        if p.protection.delayed(){return Err(ActionError::Delayed);}
        if item!=amulet || (worn && (slot!=2 || operation!=4)) || (if worn{p.data.equipment.get(slot)}else{p.data.inventory.get(slot)})!=Some(crate::inventory::Item{id:item,count:1,vars:0}) {return Err(ActionError::InvalidItem);}
        if ![4,6].contains(&operation){return Err(ActionError::InvalidItem);}
        p.dialog_generation=p.dialog_generation.checked_add(1).ok_or(ActionError::QueueFull)?;
        let token=p.dialog_generation;let breaking=operation==6;
        p.pending_chemistry=Some((token,slot,item,breaking,worn));p.protection.suspended=true;
        self.events.push(WorldEvent::ChemistryDialog{actor,token:Some(token),remaining:p.data.chemistry.remaining(),breaking});Ok(())
    }

    pub(super) fn resume_chemistry(&mut self,actor:ActorId,token:u64,choice:i32)->Result<(),ActionError> {
        let p=self.players.get_mut(actor.slot as usize).ok_or(ActionError::StaleActor)?;
        let Some((current,slot,item,breaking,worn))=p.pending_chemistry else{return Ok(());};
        if current!=token{return Ok(());}
        p.pending_chemistry=None;p.protection.suspended=false;
        self.events.push(WorldEvent::ChemistryDialog{actor,token:None,remaining:p.data.chemistry.remaining(),breaking});
        if p.protection.delayed(){return Err(ActionError::Delayed);}
        if ![1,2].contains(&choice) || (if worn{p.data.equipment.get(slot)}else{p.data.inventory.get(slot)})!=Some(crate::inventory::Item{id:item,count:1,vars:0}) {return Err(ActionError::InvalidItem);}
        if breaking {
            if choice==2{return Ok(());}
            let mut inventory=p.data.inventory.clone();let mut query=crate::inventory::Delete::new(item,1);query.strict_slot=Some(slot);
            inventory.delete(&self.object_types,query).map_err(|_|ActionError::InvalidItem)?;
            p.data.inventory=inventory;p.data.chemistry.break_amulet();
            self.events.push(WorldEvent::LocationMessage{actor,message:"You break the amulet. Your next amulet of chemistry will have five charges."});
        } else {
            p.data.chemistry.set_stop_on_crumble(choice==2);
            self.events.push(WorldEvent::LocationMessage{actor,message:if choice==2{"You will stop making potions when your amulet of chemistry crumbles."}else{"You will continue making potions when your amulet of chemistry crumbles."}});
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    fn fixture()->(World,ActorId){
        let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
        w.finished_potions=Some(crate::finished_potions::Config{chemistry:[(175,crate::chemistry_modifier::Binding{amulet:21163,extra_output:2446})].into(),goggles:[29974,29976],recipes:Default::default(),quest_progress:80,quest_complete:4,animation:363});
        let mut data=PlayerData::new(1,"Chemistry options".into(),pos,Default::default());data.chemistry=crate::chemistry::State::restore(1,0).unwrap();
        data.inventory.restore_slot(3,Some(Item{id:21163,count:1,vars:0})).unwrap();w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.drain_events();(w,a)
    }
    #[test]
    fn options_preserve_charges_and_break_resets_without_replay(){
        let (mut w,a)=fixture();
        for choice in [2,1,2]{
            w.chemistry_item(a,3,21163,4,false).unwrap();let token=w.players.get(a.slot as usize).unwrap().pending_chemistry.unwrap().0;
            w.resume_chemistry(a,token+1,choice).unwrap();assert_eq!(w.data(a).unwrap().chemistry.remaining(),1);
            w.resume_chemistry(a,token,choice).unwrap();assert_eq!(w.data(a).unwrap().chemistry.stop_on_crumble(),choice==2);
            w.resume_chemistry(a,token,3-choice).unwrap();assert_eq!(w.data(a).unwrap().chemistry.stop_on_crumble(),choice==2);
        }
        assert!(w.chemistry_item(a,3,21163,5,false).is_err());
        assert!(w.data(a).unwrap().inventory.get(3).is_some());
        let stats=w.data(a).unwrap().stats.clone();
        w.chemistry_item(a,3,21163,6,false).unwrap();let token=w.players.get(a.slot as usize).unwrap().pending_chemistry.unwrap().0;
        w.resume_chemistry(a,token,2).unwrap();assert_eq!(w.data(a).unwrap().chemistry.remaining(),1);assert!(w.data(a).unwrap().inventory.get(3).is_some());
        w.chemistry_item(a,3,21163,6,false).unwrap();let token=w.players.get(a.slot as usize).unwrap().pending_chemistry.unwrap().0;
        w.resume_chemistry(a,token,1).unwrap();assert!(w.data(a).unwrap().inventory.get(3).is_none());
        assert_eq!(w.data(a).unwrap().chemistry.remaining(),5);assert!(w.data(a).unwrap().chemistry.stop_on_crumble());assert_eq!(w.data(a).unwrap().stats,stats);
        assert!(w.chemistry_item(a,3,21163,6,false).is_err());
    }
    #[test]
    fn stale_slot_cancelled_and_invalid_choice_cannot_change_setting(){
        for mode in 0..4{let (mut w,a)=fixture();w.chemistry_item(a,3,21163,4,false).unwrap();let token=w.players.get(a.slot as usize).unwrap().pending_chemistry.unwrap().0;
            match mode{0=>{w.data_mut(a).unwrap().inventory.restore_slot(3,None).unwrap();},1=>{w.cancel_chemistry_dialog(a).unwrap();},2=>{w.players.get_mut(a.slot as usize).unwrap().begin_held_action();},_=>()}
            let _=w.resume_chemistry(a,token,if mode==3{0}else{2});assert!(!w.data(a).unwrap().chemistry.stop_on_crumble());assert_eq!(w.data(a).unwrap().chemistry.remaining(),1);
        }
    }
    #[test]
    fn worn_options_recheck_neck_and_never_break_equipped_amulet(){
        for removed in [false,true]{let (mut w,a)=fixture();let data=w.data_mut(a).unwrap();data.inventory.restore_slot(3,None).unwrap();data.equipment.restore_slot(2,Some(Item{id:21163,count:1,vars:0})).unwrap();
            assert!(w.chemistry_item(a,2,21163,6,true).is_err());assert!(w.data(a).unwrap().equipment.get(2).is_some());
            w.chemistry_item(a,2,21163,4,true).unwrap();let token=w.players.get(a.slot as usize).unwrap().pending_chemistry.unwrap().0;
            if removed{w.data_mut(a).unwrap().equipment.restore_slot(2,None).unwrap();}
            let result=w.resume_chemistry(a,token,2);assert_eq!(result.is_err(),removed);assert_eq!(w.data(a).unwrap().chemistry.stop_on_crumble(),!removed);assert_eq!(w.data(a).unwrap().chemistry.remaining(),1);
        }
    }
}
