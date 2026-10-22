use super::*;
impl World{
 pub(super) fn forging_ring_item(&mut self,actor:ActorId,slot:usize,item:i32,operation:u8,worn:bool)->Result<(),ActionError>{
  let c=self.forging_ring.ok_or(ActionError::InvalidItem)?;let p=self.players.get_mut(actor.slot as usize).filter(|p|p.id==actor).ok_or(ActionError::StaleActor)?;
  if p.saving||p.pending_logout||p.death_queued||p.data.stats.current(3)as u8==0{return Err(ActionError::Protected);}
  p.begin_held_action();p.queues.clear_weak();if p.protection.delayed(){return Err(ActionError::Delayed);}if p.has_modal{return Err(ActionError::Protected);}
  if item!=c.ring||!self.object_types.ordinary_nonstackable(item){return Err(ActionError::InvalidItem);}
  let source=if worn{&p.data.equipment}else{&p.data.inventory};if source.get(slot)!=Some(crate::inventory::Item{id:item,count:1,vars:0}){return Err(ActionError::InvalidItem);}
  let charges=c.remaining(&p.data).ok_or(ActionError::InvalidItem)?;
  let text=if worn&&slot==12&&operation==2{format!("Your ring of forging has {charges} charges remaining.")}
  else if !worn&&operation==6{p.data.inventory.restore_slot(slot,None).map_err(|_|ActionError::InvalidItem)?;p.data.variables.set(c.used,0,false);"You break your ring of forging.".into()}
  else{return Err(ActionError::InvalidItem);};
  self.events.push(WorldEvent::GameMessage{actor,text});Ok(())
 }
}
