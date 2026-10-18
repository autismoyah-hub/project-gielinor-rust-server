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
#[test]fn both_capes_use_shared_login_timer_and_reactivation_does_not_refresh_decay(){
 for cape in [9780,9781]{for reapply in [false,true]{let(mut w,a)=fixture(cape,99,true);let before=w.data(a).unwrap().clone();w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.data(a).unwrap().stats.current(12),100);
  while w.clock.0<99{if reapply&&w.clock.0==90{w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();}w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.data(a).unwrap().stats.current(12),100);}
  w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.clock.0,100);let mut expected=before.stats.clone();assert_eq!(w.data(a).unwrap().stats,expected,"PlayerMain99 expires before Late increments observedclock100; reactivation must not refresh");
  assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().equipment,before.equipment);
  w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();expected.set_current(12,100);assert_eq!(w.data(a).unwrap().stats,expected);
  while w.clock.0<199{w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.data(a).unwrap().stats.current(12),100);}
  w.tick_with_random(&mut NoRandom).unwrap();expected.set_current(12,99);assert_eq!(w.data(a).unwrap().stats,expected);assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().equipment,before.equipment);
 }}println!("PG_CAPE_EXPIRY4 bothcapes PlayerMain99/observedclock100 expiry,repeatat90doesnotextend;actualreactivationandnextPlayerMain199/observed200expiry;zeroXPitemsRNG PASS");
}
