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
#[test]fn actual_pending_rune_work_is_cancelled_only_after_valid_cape_admission(){
 use gielinor_world::{locations::Location,loc_collision::{self,LocCollision},runecrafting::{Config,Recipe}};
 for cape in [9780,9781]{for case in 0..6{let(mut w,a)=fixture(cape,if case==1{100}else{99},true);w.collision.allocate(3224,3216,0);w.collision.allocate(3216,3224,0);w.collision.allocate(3224,3224,0);
 let loc=Location::new(CoordGrid::new(3221,3220,0).unwrap(),34760,10,0).unwrap();let ty=LocCollision{width:3,length:3,block_walk:2,block_range:true,break_route_finding:false};w.location_types.insert(34760,ty);w.locations.load(loc);loc_collision::toggle(&mut w.collision,loc.coords,loc.shape,loc.angle,ty,true).unwrap();
 w.runecrafting=Some(Config{altars:[(34760,Recipe{output:556,level:1,xp_tenths:50,interval:11,name:"Independent air",members:false,pure_only:false})].into(),blocked_held:Default::default(),blocked_worn:Default::default(),outfit:Default::default(),animation:791,spot:186});w.object_types.stackable.insert(556);
 w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1436,count:1,vars:0})).unwrap();w.submit(a,Action::OperateLocation{coords:loc.coords,id:34760,operation:1,control:false}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id,1436);
 match case{2=>w.set_membership(a,false).unwrap(),3=>w.data_mut(a).unwrap().stats.set_base(12,98),4=>w.data_mut(a).unwrap().equipment.restore_slot(1,None).unwrap(),5=>{let mut item=w.data(a).unwrap().equipment.get(1).unwrap();item.vars=7;w.data_mut(a).unwrap().equipment.restore_slot(1,Some(item)).unwrap();},_=>{}}
 let before=w.data(a).unwrap().clone();w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();let d=w.data(a).unwrap();let mut bag=before.inventory.clone();let mut stats=before.stats.clone();
 if case<2{stats.set_current(12,100);}else{bag.restore_slot(0,Some(Item{id:556,count:1,vars:0})).unwrap();stats.set_fine_xp(20,before.stats.fine_xp(20)+50*before.xp_rate_hundreds/100).unwrap();}
 assert_eq!(d.inventory,bag,"cape{cape} case{case}");assert_eq!(d.stats,stats,"cape{cape} case{case}");assert_eq!(d.equipment,before.equipment);
 for _ in 0..4{w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.data(a).unwrap().inventory,bag);assert_eq!(w.data(a).unwrap().stats,stats);}
 }}println!("PG_CAPE_PENDING12 realqueuedRuneCraft validBoost/validrepeatcancel;F2P/base98/missing/varsrefusedBoost preservesRunecompletion;wholeStatsbaggear noRNG PASS");
}
