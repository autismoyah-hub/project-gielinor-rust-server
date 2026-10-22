use super::*;
#[cfg(test)]
#[path="duck_tests.rs"]
mod tests;
#[derive(Clone,Copy)]
pub(super) enum Callback{AdultSay,DucklingSay,Mourn}
#[derive(Clone,Copy)]
pub(super) struct Job{raw:u16,remaining:i32,processed_cycle:i32,callback:Callback}
impl World{
    fn queue_duck_callback(&mut self,id:NpcId,callback:Callback,delay:i32){
        let Some(n)=self.npcs.get_mut(id.slot as usize).filter(|n|n.id==id)else{return;};
        n.duck_jobs.push_back(Job{raw:n.npc.type_id,remaining:delay,processed_cycle:-1,callback});
    }
    fn duck_say(&mut self,id:NpcId,text:&'static str,sound_at:Option<CoordGrid>){
        let Some(n)=self.npc(id).filter(|n|n.visible)else{return;};
        let Some(config)=self.ducks.as_ref().filter(|c|c.contains(n.type_id))else{return;};
        let sound=config.sound;self.events.push(WorldEvent::NpcOverhead{actor:id,text});
        if let Some(coords)=sound_at{self.events.push(WorldEvent::AreaSound(crate::zones::Sound{coords,id:i32::from(sound),delay:0,loops:1,radius:5,size:0}));}
    }
    pub(super) fn process_duck_callbacks(&mut self,id:NpcId,random:&mut impl RandomSource){
        let Some(config)=self.ducks.as_ref()else{return;};let female=config.female;
        let Some(entry)=self.npcs.get_mut(id.slot as usize).filter(|n|n.id==id&&!n.death_queued&&n.npc.visible&&!n.npc.delayed&&n.npc.stats[3]>0&&config.contains(n.npc.type_id))else{return;};
        let coords=entry.npc.movement.coords;let raw=entry.npc.type_id;
        let adult=config.adults.contains(&raw);
        let expired=if entry.npc.ai_timer>0{entry.npc.ai_timer-=1;entry.npc.ai_timer==0}else{false};
        if expired{
            entry.npc.ai_timer=50+random.below(51)as i32;
            if adult{self.queue_duck_callback(id,Callback::AdultSay,1);}
            else if let Ok(Some(mother))=self.find_npc_line_of_sight(coords,female,10){self.duck_say(mother,"Quack?",Some(coords));self.queue_duck_callback(id,Callback::DucklingSay,2);}
        }
        // Match the source queue iterator's once-per-clock countdown and its
        // treatment of callbacks appended after the current tail was captured.
        loop{
            let mut invoked=false;let mut index=0;
            loop{
                let Some(entry)=self.npcs.get_mut(id.slot as usize).filter(|n|n.id==id)else{return;};
                let len=entry.duck_jobs.len();if index>=len{break;}let has_next=index+1<len;
                let job=&mut entry.duck_jobs[index];if job.processed_cycle!=self.clock.0{job.processed_cycle=self.clock.0;job.remaining-=1;}
                if job.remaining<=0{let job=entry.duck_jobs.remove(index).unwrap();self.run_duck_callback(id,job,random);invoked=true;}else{index+=1;}
                if !has_next{break;}
            }
            let size=self.npcs.get(id.slot as usize).filter(|n|n.id==id).map_or(0,|n|n.duck_jobs.len());if !invoked||size<=1{break;}
        }
    }
    fn run_duck_callback(&mut self,id:NpcId,job:Job,random:&mut impl RandomSource){
        let Some(npc)=self.npc(id).filter(|n|n.type_id==job.raw&&n.visible&&!n.delayed&&n.stats[3]>0)else{return;};let at=npc.movement.coords;
        match job.callback{
            Callback::AdultSay=>self.duck_say(id,"Quack!",Some(at)),
            Callback::DucklingSay=>self.duck_say(id,"Eep!",Some(at)),
            Callback::Mourn=>{
                let stationary=npc.mode==Mode::None;self.duck_say(id,"Cheep cheep!",None);if !stationary{return;}
                if let Some(dest)=self.duck_mourn_square(at,random){if let Some(n)=self.npc_mut(id){let _=n.move_to(dest,Speed::Walk);}}
                self.queue_duck_callback(id,Callback::DucklingSay,2);
            }
        }
    }
    fn duck_mourn_square(&self,center:CoordGrid,random:&mut impl RandomSource)->Option<CoordGrid>{
        if center.x()<9||center.z()<9||center.x()>16374||center.z()>16374{return None;}
        // Bounds iteration is X-fastest. Kotlin Sequence.shuffled selects from
        // the remaining buffer before min-radius/collision filtering.
        let mut squares=Vec::with_capacity(361);for z in center.z()-9..=center.z()+9{for x in center.x()-9..=center.x()+9{squares.push(CoordGrid::new(x,z,center.level()).unwrap());}}
        while !squares.is_empty(){let pick=random.below(squares.len()as u32)as usize;let at=squares.swap_remove(pick);if at.chebyshev_distance(center)>=1&&self.collision.get(at.x(),at.z(),at.level())&0x200000==0{return Some(at);}}
        None
    }
    pub(super) fn duck_death_reactions(&mut self,id:NpcId){
        let Some(config)=self.ducks.as_ref()else{return;};let Some(n)=self.npc(id).filter(|n|config.adults.contains(&n.type_id))else{return;};let at=n.movement.coords;let(male,young)=(config.male,config.ducklings);
        if let Ok(children)=self.find_all_npcs_line_of_sight(at,young,1){for child in children{self.queue_duck_callback(child,Callback::Mourn,2);}}
        if let Ok(Some(male))=self.find_npc_line_of_sight(at,male,1){self.queue_duck_callback(male,Callback::AdultSay,2);}
    }
}
