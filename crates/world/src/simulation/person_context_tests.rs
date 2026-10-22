use super::*;
use crate::{dialogue::{Conversation,Event,Expression,Page,Speaker},npc::{Mode,Restrict,BlockWalk,RandomSource},npc_options::Definition,npc_interaction::Handler,stats::Stats};
struct NoDraw;impl RandomSource for NoDraw{fn below(&mut self,_:u32)->u32{panic!("Invalid person context must not consume random state")}}
fn fixture()->(World,ActorId,NpcId){
 let mut w=World::default();w.collision.allocate(3200,3200,0);w.npc_options.types.insert(1,Definition{operations:[Some("Talk-to".into()),None,None,None,None],..Default::default()});w.npc_handlers.insert((1,1),Handler::Person);
 let mut npc=Npc::new(1,CoordGrid::new(3201,3200,0).unwrap(),1,Restrict::Normal,BlockWalk::Npc).unwrap();npc.mode=Mode::None;let target=w.spawn_npc(npc).unwrap();w.queue_login(1,PlayerData::new(1,"Person context fixture".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default())).unwrap();w.tick().unwrap();let actor=w.actors().next().unwrap();w.drain_events();(w,actor,target)
}
#[test]
fn person_context_dead_actor_cannot_start_a_greeting_through_the_shared_npc_action(){
 for queued in[false,true]{let(mut w,a,n)=fixture();if queued{w.players.get_mut(a.slot as usize).unwrap().death_queued=true;}else{w.players.get_mut(a.slot as usize).unwrap().data.stats.set_current(3,0);}
  w.submit(a,Action::OperateNpc{target:n,operation:1,control:false}).unwrap();w.tick_with_random(&mut NoDraw).unwrap();assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:Event::Page{..},..})),"Dead actor received greeting");
 }
}
#[test]
fn person_context_dead_actor_cannot_roll_give_flyer_or_trigger_retaliation(){
 for queued in[false,true]{for effect in 0..3{let(mut w,a,n)=fixture();w.person_flyer=Some(343);let before=w.data(a).unwrap().inventory.clone();
  let conversation=match effect{0=>Conversation::person(),1=>Conversation::response(true,0).unwrap(),_=>Conversation::response(false,17).unwrap()};w.show_dialogue(a,crate::dialogue::Pending{token:0,npc:n,conversation}).unwrap();let token=w.players.get(a.slot as usize).unwrap().dialogue.as_ref().unwrap().token;w.drain_events();
  if queued{w.players.get_mut(a.slot as usize).unwrap().death_queued=true;}else{w.players.get_mut(a.slot as usize).unwrap().data.stats.set_current(3,0);}
  w.resume_dialogue(a,token,-1,&mut NoDraw).unwrap();assert!(w.players.get(a.slot as usize).unwrap().dialogue.is_none());assert!(w.npc(n).unwrap().aggressive_player.is_none());assert!(w.npc(n).unwrap().retaliate_due.is_empty());assert_eq!(w.data(a).unwrap().inventory,before);assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:Event::Page{..},..})));
  let p=w.players.get_mut(a.slot as usize).unwrap();p.data.stats.set_current(3,1);p.death_queued=false;w.resume_dialogue(a,token,-1,&mut NoDraw).unwrap();assert_eq!(w.data(a).unwrap().inventory,before);assert!(w.npc(n).unwrap().retaliate_due.is_empty());
 }}
}
#[test]
fn person_context_healthy_flyer_still_commits_once_after_acknowledgement(){
 let(mut w,a,n)=fixture();w.person_flyer=Some(343);w.show_dialogue(a,crate::dialogue::Pending{token:0,npc:n,conversation:Conversation::response(true,0).unwrap()}).unwrap();let token=w.players.get(a.slot as usize).unwrap().dialogue.as_ref().unwrap().token;
 assert!(w.data(a).unwrap().inventory.slots().iter().all(Option::is_none));w.resume_dialogue(a,token,-1,&mut NoDraw).unwrap();let after=w.data(a).unwrap().inventory.clone();assert_eq!(after.slots().iter().flatten().filter(|i|i.id==343).map(|i|i.count).sum::<i32>(),1);w.resume_dialogue(a,token,-1,&mut NoDraw).unwrap();assert_eq!(w.data(a).unwrap().inventory,after);
}
#[test]
fn person_context_unsigned_positive_hp_retains_the_existing_greeting(){
 let(mut w,a,n)=fixture();w.players.get_mut(a.slot as usize).unwrap().data.stats.set_current(3,128u8 as i8);w.submit(a,Action::OperateNpc{target:n,operation:1,control:false}).unwrap();w.tick_with_random(&mut NoDraw).unwrap();assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:Event::Page{page:Page::Chat{speaker:Speaker::Player,expression:Expression::Happy,..},..},..})));
}
