//! Two protected actor continuations; NPC identity and ordinary inventory stay authoritative.
use super::*;
use crate::{npc_interaction::Handler,shearing::{Config,REGROWTH}};
#[cfg(test)]
#[path="shearing_tests.rs"]
mod tests;
#[derive(Clone,Copy)]
pub(super) struct Job {target:NpcId,raw:u16,sheared:u16,origin:CoordGrid,last_movement:i32,due:i32,face:bool}
#[derive(Clone,Copy)]
pub(super) struct Hold {owner:ActorId,release_at:Option<i32>}
#[derive(Clone,Copy)]
pub(super) struct Regrowth {original:u16,sheared:u16,due:i32}
fn has_shears(p:&Player,c:&Config)->bool {p.data.inventory.slots().iter().flatten().any(|i|i.id==c.shears&&i.count>0&&i.vars==0)}
fn alive(p:&Player)->bool {!p.saving&&!p.pending_logout&&!p.has_modal&&!p.death_queued&&p.data.stats.current(3)as u8!=0}
fn near(p:&Player,n:&Npc)->bool {
    let a=p.movement.coords;let b=n.movement.coords;
    a.level()==b.level()&&(a.x()-b.x()).abs()+(a.z()-b.z()).abs()==1
}
fn release(n:&mut NpcEntry) {
    if let Some(h)=n.shear_hold.take(){if n.npc.dialogue_face==Some(h.owner){n.npc.face_player(None);}}
}
impl World {
    pub(super) fn request_held_npc(&mut self,actor:ActorId,target:NpcId,slot:i32,item:i32,control:bool)->Result<(),ActionError> {
        let raw=self.npc(target).ok_or(ActionError::InvalidItem)?.type_id;
        if !self.shearing.as_ref().is_some_and(|c|c.targets.contains_key(&raw)) {return self.request_banker_note(actor,target,slot,item,control);}
        let c=self.shearing.as_ref().unwrap();let p=self.player(actor)?;
        let selected=usize::try_from(slot).map_err(|_|ActionError::InvalidItem)?;
        if item!=c.shears||!self.held_use_types.contains_key(&item)||!p.data.inventory.get(selected).is_some_and(|i|i.id==item&&i.count>0&&i.vars==0){return Err(ActionError::InvalidItem);}
        self.request_npc_inner(actor,target,1,control,None)?;
        let p=self.players.get_mut(actor.slot as usize).unwrap();
        if let Some(pending)=p.npc_interaction.as_mut().filter(|p|p.target==target&&matches!(p.handler,Some(Handler::Shear{raw:r,..})if r==raw)) {pending.handler=Some(Handler::Shear{raw,selected:Some(selected)});}
        Ok(())
    }
    pub(super) fn sheep_admission(&self,actor:ActorId,target:NpcId,raw:u16,selected:Option<usize>)->bool {
        let Ok(p)=self.player(actor)else{return false};let Some(c)=&self.shearing else{return false};
        let Some(n)=self.npcs.get(target.slot as usize).filter(|n|n.id==target)else{return false};
        alive(p)&&!n.death_queued&&n.npc.visible&&!n.npc.delayed&&n.npc.stats[3]>0&&n.npc.type_id==raw
            &&n.npc.visible_type()==raw
            &&c.targets.contains_key(&raw)&&n.shear_hold.is_none_or(|h|h.owner==actor)
            &&selected.is_none_or(|slot|p.data.inventory.get(slot).is_some_and(|i|i.id==c.shears&&i.count>0&&i.vars==0))
            &&self.npc_handlers.get(&(raw,1))==Some(&Handler::Shear{raw,selected:None})
    }
    pub(super) fn start_shearing(&mut self,slot:usize,target:NpcId,raw:u16,selected:Option<usize>) {
        let actor=self.players.get(slot).unwrap().id;
        if !self.sheep_admission(actor,target,raw,selected){return;}
        let c=self.shearing.as_ref().unwrap();let p=self.players.get(slot).unwrap();
        if !has_shears(p,c){self.events.push(WorldEvent::EngineMessage{actor,message:"You need a set of shears to do this."});return;}
        let n=self.npcs.get(target.slot as usize).unwrap();
        if !near(p,&n.npc)||self.clock.0<0||self.clock.0.checked_add(2+REGROWTH).is_none(){return;}
        let job=Job{target,raw,sheared:c.targets[&raw],origin:p.movement.coords,last_movement:p.movement.last_movement,due:self.clock.0+1,face:true};
        let sequence=c.sequence;let sound=c.sound;let coords=n.npc.movement.coords;
        self.npcs.get_mut(target.slot as usize).unwrap().shear_hold=Some(Hold{owner:actor,release_at:None});
        let p=self.players.get_mut(slot).unwrap();p.pending_shearing=Some(job);p.protection.processed_clock=self.clock.0;p.protection.delay(1);p.protection.suspended=true;
        self.events.push(WorldEvent::ResetFaceEntity{actor});self.events.push(WorldEvent::FaceLocation{actor,coords,width:1,length:1});
        self.events.push(WorldEvent::PlayerAnimation{actor,sequence});self.events.push(WorldEvent::PlayerDelayedSound{actor,sound,delay:10});
    }
    pub(super) fn cancel_shearing(&mut self,slot:usize) {
        let p=self.players.get_mut(slot).unwrap();let Some(job)=p.pending_shearing.take()else{return};p.protection.suspended=false;
        let actor=p.id;
        if let Some(n)=self.npcs.get_mut(job.target.slot as usize).filter(|n|n.id==job.target&&n.shear_hold.is_some_and(|h|h.owner==actor)){release(n);}
    }
    pub(super) fn resume_shearing(&mut self,slot:usize) {
        let Some(p)=self.players.get(slot)else{return};let Some(job)=p.pending_shearing else{return};let actor=p.id;
        let valid=self.sheep_admission(actor,job.target,job.raw,None)
            &&self.shearing.as_ref().is_some_and(|c|c.targets.get(&job.raw)==Some(&job.sheared)&&has_shears(p,c))
            &&p.movement.coords==job.origin&&p.movement.last_movement==job.last_movement
            &&self.npcs.get(job.target.slot as usize).is_some_and(|n|near(p,&n.npc)&&n.shear_hold.is_some_and(|h|h.owner==actor));
        if !valid{self.cancel_shearing(slot);return;}
        if self.clock.0<job.due{return;}
        if job.face {
            let n=self.npcs.get_mut(job.target.slot as usize).unwrap();n.npc.movement.abort();n.npc.movement.interacting=false;n.npc.face_player(Some(actor));n.npc.dialogue_face=Some(actor);
            let p=self.players.get_mut(slot).unwrap();p.pending_shearing=Some(Job{due:self.clock.0+1,face:false,..job});p.protection.processed_clock=self.clock.0;p.protection.delay(1);return;
        }
        let Some(regrow_at)=self.clock.0.checked_add(REGROWTH)else{self.cancel_shearing(slot);return};
        let c=self.shearing.as_ref().unwrap();let wool=c.wool;let p=self.players.get(slot).unwrap();
        let mut candidate=p.data.inventory.clone();
        let in_inventory=match candidate.insert(&self.object_types,crate::inventory::Insert::new(wool,1)) {
            Ok(v)if v.completed==1=>true,
            Err(crate::inventory::Error::NotEnoughSpace)=>false,
            _=>{self.cancel_shearing(slot);return;}
        };
        if !in_inventory {
            let Some(observer)=p.observer_id else{self.cancel_shearing(slot);return;};
            let duration=p.loot_drop_duration.unwrap_or(200);
            if duration<=0||self.clock.0.checked_add(duration.max(100)).is_none(){self.cancel_shearing(slot);return;}
            let object=crate::ground::Object{coords:p.movement.coords,item:wool as u16,count:1,scope:crate::ground::Scope::Private,created:self.clock.0,receiver:Some(observer),owner:Some(observer)};
            if let Err(error)=self.objects.add_batch(&[object],&self.object_types,self.clock,duration,100){self.events.push(WorldEvent::GroundError(error));self.cancel_shearing(slot);return;}
        }
        let p=self.players.get_mut(slot).unwrap();if in_inventory{p.data.inventory=candidate;}p.pending_shearing=None;p.protection.suspended=false;
        let n=self.npcs.get_mut(job.target.slot as usize).unwrap();n.npc.transformation=Some(job.sheared);n.fleece_regrowth=Some(Regrowth{original:job.raw,sheared:job.sheared,due:regrow_at});n.shear_hold.as_mut().unwrap().release_at=Some(self.clock.0+1);
        self.invalidate_sheep_target(job.target);
        self.events.push(WorldEvent::EngineMessage{actor,message:"You get some wool."});
        self.events.push(WorldEvent::SheepSheared{actor,npc:job.target,from:job.raw,to:job.sheared,wool,in_inventory,regrow_at});
    }
    pub(super) fn process_shearing_npcs(&mut self) {
        let clock=self.clock.0;
        let mut restored=Vec::new();
        for(_,n)in self.npcs.iter_mut() {
            if let Some(r)=n.fleece_regrowth.filter(|r|clock>=r.due) {if n.npc.type_id==r.original&&n.npc.transformation==Some(r.sheared){n.npc.transformation=None;restored.push(n.id);}n.fleece_regrowth=None;}
            if let Some(h)=n.shear_hold {
                let valid=h.release_at.map_or_else(||self.players.get(h.owner.slot as usize).is_some_and(|p|p.id==h.owner&&alive(p)&&p.pending_shearing.is_some_and(|j|j.target==n.id&&j.raw==n.npc.type_id)),|at|clock<at);
                if !valid{release(n);}
            }
        }
        for target in restored{self.invalidate_sheep_target(target);}
    }
    /// Model the source's type-dependent interaction UID without respawning
    /// the authoritative NPC or resetting its base stats and AI state.
    fn invalidate_sheep_target(&mut self,target:NpcId) {
        for(_,p)in self.players.iter_mut() {
            if p.npc_interaction.is_some_and(|i|i.target==target) {
                p.npc_interaction=None;p.movement.interacting=false;p.movement.abort();
                self.events.push(WorldEvent::ClearMapFlag(p.id));
            }
        }
    }
}
