//! Independent public action checks; no developer test modules invoked.
use gielinor_world::{simulation::{World,PlayerData,ActorId,Action,WorldEvent},worn_stat_boost::Rule,inventory::{Item,Template},stats::Stats,npc::RandomSource,CoordGrid};
struct NoRandom;impl RandomSource for NoRandom{fn below(&mut self,_:u32)->u32{panic!("worn cape action drew RNG")}}
fn fixture(cape:i32,current:i8,member:bool)->(World,ActorId){
 let mut world=World::default();world.collision.allocate(3216,3216,0);
 for id in [9780,9781]{world.worn_stat_boosts.insert(id,Rule{skill:12,slot:1,button_operation:2,members:true});}
 let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,current);stats.set_fine_xp(12,130344311).unwrap();
 let mut d=PlayerData::new(1,"Independent cape observer".into(),CoordGrid::new(3220,3220,0).unwrap(),stats);
 for slot in 0..28{d.inventory.restore_slot(slot,Some(Item{id:if slot==27{995}else{1931},count:if slot==27{99001}else{1},vars:if slot==6{9}else{0}})).unwrap();}
 d.equipment.restore_slot(1,Some(Item{id:cape,count:1,vars:0})).unwrap();d.equipment.restore_slot(4,Some(Item{id:1129,count:1,vars:7})).unwrap();
 world.queue_login(1,d).unwrap();world.tick_with_random(&mut NoRandom).unwrap();let actor=world.actors().next().unwrap();world.set_membership(actor,member).unwrap();world.drain_events();(world,actor)
}
#[test]fn queued_authoritative_death_blocks_cape_even_after_visible_hp_restore(){use gielinor_world::hits::{Target,Hit,Style};for cape in [9780,9781]{for restore in [false,true]{let(mut w,a)=fixture(cape,99,true);w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:20},1).unwrap();w.tick_with_random(&mut NoRandom).unwrap();assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(_))));assert_eq!(w.data(a).unwrap().stats.current(3),0);if restore{w.data_mut(a).unwrap().stats.set_current(3,10);}let before=w.data(a).unwrap().clone();w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();for _ in 0..4{w.tick_with_random(&mut NoRandom).unwrap();let d=w.data(a).unwrap();assert_eq!(d.stats,before.stats);assert_eq!(d.inventory,before.inventory);assert_eq!(d.equipment,before.equipment);}}}println!("PG_CAPE_DEATH4 bothvariants actualDeathRequested then originalzeroHP/restored10HP norespawn Boostrefused wholeStatsbaggear noXPnoRNG PASS");}
