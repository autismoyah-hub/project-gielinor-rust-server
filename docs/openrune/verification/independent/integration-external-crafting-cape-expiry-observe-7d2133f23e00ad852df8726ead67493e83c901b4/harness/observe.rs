//! Independent public action checks; no developer test modules invoked.
use gielinor_world::{simulation::{World,PlayerData,ActorId,Action,WorldEvent},worn_stat_boost::Rule,inventory::{Item,Template},stats::Stats,npc::RandomSource,CoordGrid};
struct NoRandom;impl RandomSource for NoRandom{fn below(&mut self,_:u32)->u32{panic!("worn cape action drew RNG")}}
fn fixture(cape:i32,current:i8,member:bool)->(World,ActorId){
 let mut world=World::default();world.regeneration=Some(gielinor_world::player_regen::Config{skills:vec![12],hitpoints:3,prayer:5,bracelet:11133,drained_timer:65001,boosted_timer:65002,health_timer:65003,rapid_timer:65004});world.collision.allocate(3216,3216,0);
 for id in [9780,9781]{world.worn_stat_boosts.insert(id,Rule{skill:12,slot:1,button_operation:2,members:true});}
 let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,current);stats.set_fine_xp(12,130344311).unwrap();
 let mut d=PlayerData::new(1,"Independent cape observer".into(),CoordGrid::new(3220,3220,0).unwrap(),stats);
 for slot in 0..28{d.inventory.restore_slot(slot,Some(Item{id:if slot==27{995}else{1931},count:if slot==27{99001}else{1},vars:if slot==6{9}else{0}})).unwrap();}
 d.equipment.restore_slot(1,Some(Item{id:cape,count:1,vars:0})).unwrap();d.equipment.restore_slot(4,Some(Item{id:1129,count:1,vars:7})).unwrap();
 world.queue_login(1,d).unwrap();world.tick_with_random(&mut NoRandom).unwrap();let actor=world.actors().next().unwrap();world.set_membership(actor,member).unwrap();world.drain_events();(world,actor)
}
#[test]fn observe_actual_timer_boundary(){let(mut w,a)=fixture(9780,99,true);w.submit(a,Action::BoostWornItem{slot:1,item:9780}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();for _ in 0..110{let old=w.data(a).unwrap().stats.current(12);w.tick_with_random(&mut NoRandom).unwrap();let now=w.data(a).unwrap().stats.current(12);let events=w.drain_events();if w.clock.0>=95{println!("PG_CAPE_TIMER_OBSERVE clock={} before={} after={} events={:?}",w.clock.0,old,now,events);} }assert_eq!(w.data(a).unwrap().stats.current(12),99);}
