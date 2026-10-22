//! Ambient callbacks run between resolved NPC hits and ordinary mode processing.
use super::*;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{npc::{BlockWalk,Restrict},npc_flavour::{Config,Rule}};
    use std::collections::VecDeque;
    struct Draws(VecDeque<(u32,u32)>);
    impl RandomSource for Draws{fn below(&mut self,bound:u32)->u32{let(b,v)=self.0.pop_front().expect("Unexpected random draw");assert_eq!(b,bound);assert!(v<bound);v}}
    fn world()->(World,NpcId){
        let mut w=World::default();w.collision.allocate(3200,3200,0);
        let rule=Rule{text:"Baa!",sound:2053,radius:5,animation:None};w.npc_flavour=Some(Config{types:[(2693,rule),(1178,rule)].into()});
        let mut npc=Npc::new(2693,CoordGrid::new(3201,3201,0).unwrap(),1,Restrict::Normal,BlockWalk::Npc).unwrap();npc.mode=Mode::None;npc.regen_rate=0;npc.ai_timer=1;let n=w.spawn_npc(npc).unwrap();w.drain_events();(w,n)
    }
    #[test]
    fn sheep_flavour_all_interval_draws_and_quarter_chance_follow_source_order(){
        for interval in 0..20 {for chance in 0..4 {
            let(mut w,n)=world();let mut draws=Draws([(20,interval),(4,chance)].into());w.process_npc_flavour(n,&mut draws);assert!(draws.0.is_empty());
            assert_eq!(w.npc(n).unwrap().ai_timer,15+interval as i32);let events=w.drain_events();assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::NpcOverhead{actor,text:"Baa!"}if *actor==n)).count(),usize::from(chance==0));
            assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::AreaSound(s)if s.id==2053&&s.radius==5&&s.loops==1&&s.delay==0&&s.size==0)).count(),usize::from(chance==0));
        }}
    }
    #[test]
    fn sheep_flavour_interval_is_counted_once_per_eligible_npc_phase(){
        let(mut w,n)=world();w.process_npc_flavour(n,&mut Draws([(20,19),(4,3)].into()));
        for _ in 0..33{w.process_npc_flavour(n,&mut Draws(VecDeque::new()));}assert_eq!(w.npc(n).unwrap().ai_timer,1);
        w.process_npc_flavour(n,&mut Draws([(20,0),(4,0)].into()));assert_eq!(w.npc(n).unwrap().ai_timer,15);
        assert_eq!(w.drain_events().iter().filter(|e|matches!(e,WorldEvent::NpcOverhead{..})).count(),1);
    }
    #[test]
    fn sheep_flavour_disabled_hidden_delayed_dead_and_unregistered_do_not_draw(){
        for case in 0..6 {
            let(mut w,n)=world();let npc=w.npc_mut(n).unwrap();match case{0=>npc.ai_timer=-1,1=>npc.visible=false,2=>npc.delayed=true,3=>npc.stats[3]=0,4=>npc.type_id=3227,_=>{w.npc_flavour=None;}}
            let before=w.npc(n).unwrap().ai_timer;w.process_npc_flavour(n,&mut Draws(VecDeque::new()));assert_eq!(w.npc(n).unwrap().ai_timer,before);assert!(w.drain_events().is_empty());
        }
    }
    #[test]
    fn sheep_flavour_shared_tick_publishes_speech_and_tracks_visible_stage(){
        let(mut w,n)=world();w.npc_mut(n).unwrap().transformation=Some(1178);let before=w.npc(n).unwrap().stats;
        let mut draws=Draws([(20,5),(4,0)].into());w.tick_with_random(&mut draws).unwrap();assert!(draws.0.is_empty());assert_eq!(w.npc(n).unwrap().ai_timer,20);
        assert_eq!(w.npc(n).unwrap().type_id,2693);assert_eq!(w.npc(n).unwrap().stats,before);assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::NpcOverhead{actor,text:"Baa!"}if *actor==n)));
    }
}
impl World {
    pub(super) fn npc_flavour_say(&mut self,id:NpcId) {
        let Some(entry)=self.npcs.get(id.slot as usize).filter(|n|n.id==id&&!n.death_queued)else{return};let n=&entry.npc;
        if !n.visible||n.delayed||n.stats[3]<=0{return;}
        let Some(rule)=self.npc_flavour.as_ref().and_then(|c|c.types.get(&n.visible_type())).copied()else{return};
        let coords=n.movement.coords;
        self.events.push(WorldEvent::AreaSound(crate::zones::Sound{coords,id:rule.sound as i32,delay:0,loops:1,radius:rule.radius,size:0}));
        self.events.push(WorldEvent::NpcOverhead{actor:id,text:rule.text});
        if let Some(sequence)=rule.animation{self.events.push(WorldEvent::NpcCombatAnimation{actor:id,sequence});}
    }
    pub(super) fn process_npc_flavour(&mut self,id:NpcId,random:&mut impl RandomSource) {
        let Some(n)=self.npcs.get_mut(id.slot as usize).filter(|n|n.id==id)else{return};
        if n.death_queued||!n.npc.visible||n.npc.delayed||n.npc.stats[3]<=0||n.npc.ai_timer<=0
            ||!self.npc_flavour.as_ref().is_some_and(|c|c.types.contains_key(&n.npc.visible_type())){return;}
        n.npc.ai_timer-=1;if n.npc.ai_timer>0{return;}
        n.npc.ai_timer=15+random.below(20)as i32;
        if random.below(4)==0{self.npc_flavour_say(id);}
    }
}
