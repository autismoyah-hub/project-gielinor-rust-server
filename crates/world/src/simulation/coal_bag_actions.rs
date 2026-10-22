use super::*;
impl World {
    pub(super) fn coal_bag_item(&mut self,actor:ActorId,slot:usize,item:i32,operation:u8)->Result<(),ActionError>{
        let config=self.coal_bag.ok_or(ActionError::InvalidItem)?;
        let p=self.players.get_mut(actor.slot as usize).filter(|p|p.id==actor).ok_or(ActionError::StaleActor)?;
        if p.saving||p.pending_logout||p.death_queued||p.data.stats.current(3)as u8==0{return Err(ActionError::Protected);}
        p.begin_held_action();p.queues.clear_weak();
        if p.protection.delayed(){return Err(ActionError::Delayed);}
        if p.has_modal{return Err(ActionError::Protected);}
        if !p.member||!config.contains(item)||!matches!(operation,2..=5)||p.data.inventory.get(slot)!=Some(crate::inventory::Item{id:item,count:1,vars:0}){return Err(ActionError::InvalidItem);}
        let current=p.data.variables.bit(config.stored).map_err(|_|ActionError::InvalidItem)?;
        let capacity=config.capacity(&p.data.equipment);
        let mut inventory=p.data.inventory.clone();let mut variables=p.data.variables.clone();let mut next=current;let mut message=None;
        match operation {
            2=>{
                let free=(capacity-current).max(0);
                let coal=inventory.slots().iter().flatten().filter(|v|v.id==config.coal).count()as i32;
                if free==0{message=Some("Your coal bag is full.".to_owned());}
                else if coal==0{message=Some("You don't have any coal to fill your coal bag with.".to_owned());}
                else{let amount=free.min(coal);inventory.delete(&self.object_types,crate::inventory::Delete::new(config.coal,amount)).map_err(|_|ActionError::InvalidItem)?;next+=amount;message=Some(format!("You fill the coal bag with {amount} coal."));}
            }
            3=>{inventory.restore_slot(slot,Some(crate::inventory::Item{id:if item==config.closed{config.open}else{config.closed},count:1,vars:0})).map_err(|_|ActionError::InvalidItem)?;}
            4=>{message=Some(if current<=0{"Your coal bag is empty.".to_owned()}else{format!("Your coal bag contains {current} of {capacity} coal.")});}
            5=>{
                if current<=0{message=Some("Your coal bag is empty.".to_owned());}
                else{let mut query=crate::inventory::Insert::new(config.coal,current);query.strict=false;let amount=inventory.insert(&self.object_types,query).map_err(|_|ActionError::InvalidItem)?.completed;
                    if amount==0{message=Some("You don't have any free space in your inventory.".to_owned());}else{next-=amount;message=Some(format!("You empty {amount} coal from the coal bag."));}}
            }
            _=>unreachable!(),
        }
        let update=if next!=current{variables.set_bit(config.base,config.stored,next,p.protection.processed_clock>0).map_err(|_|ActionError::InvalidItem)?}else{None};
        p.data.inventory=inventory;p.data.variables=variables;
        if let Some(update)=update{self.events.push(WorldEvent::SessionVariables{actor,updates:vec![update]});}
        if let Some(text)=message{self.events.push(WorldEvent::GameMessage{actor,text});}
        Ok(())
    }
}
#[cfg(test)]
#[path="coal_bag_tests.rs"]
mod tests;
