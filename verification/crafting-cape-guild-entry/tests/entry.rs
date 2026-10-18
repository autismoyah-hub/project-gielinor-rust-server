//! Independent public action checks; no developer test modules invoked.
use gielinor_world::{simulation::{World,PlayerData,ActorId,Action,WorldEvent},worn_stat_boost::Rule,inventory::{Item,Template},stats::Stats,npc::RandomSource,CoordGrid};
struct NoRandom;impl RandomSource for NoRandom{fn below(&mut self,_:u32)->u32{panic!("worn cape action drew RNG")}}
fn fixture(cape:i32,current:i8,member:bool)->(World,ActorId){
 let mut world=World::default();world.collision.allocate(3216,3216,0);
 for id in [9780,9781]{world.worn_stat_boosts.insert(id,Rule{skill:12,slot:1,button_operation:2,members:true});}
 let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,current);stats.set_fine_xp(12,130344311).unwrap();
 let mut d=PlayerData::new(1,"Independent cape observer".into(),CoordGrid::new(3220,3221,0).unwrap(),stats);
 for slot in 0..28{d.inventory.restore_slot(slot,Some(Item{id:if slot==27{995}else{1931},count:if slot==27{99001}else{1},vars:if slot==6{9}else{0}})).unwrap();}
 d.equipment.restore_slot(1,Some(Item{id:cape,count:1,vars:0})).unwrap();d.equipment.restore_slot(4,Some(Item{id:1129,count:1,vars:7})).unwrap();
 world.queue_login(1,d).unwrap();world.tick_with_random(&mut NoRandom).unwrap();let actor=world.actors().next().unwrap();world.set_membership(actor,member).unwrap();world.drain_events();(world,actor)
}
#[test]fn twelve_actual_guild_entry_and_member_boost_permissions_are_separate(){
 use gielinor_world::{guild_passages::{Gate,Kind,Outfit},locations::Location,loc_collision::{self,LocCollision}};
 for cape in [9780,9781]{for member in [false,true]{for current in [39,40,99]{let(mut w,a)=fixture(cape,current,member);
 let loc=Location::new(CoordGrid::new(3220,3220,0).unwrap(),14910,0,0).unwrap();let ty=LocCollision{width:1,length:1,block_walk:2,block_range:true,break_route_finding:false};for id in [14910,1539]{w.location_types.insert(id,ty);}w.locations.load(loc);loc_collision::toggle(&mut w.collision,loc.coords,loc.shape,loc.angle,ty,true).unwrap();
 w.guild_passages.insert(14910,Gate{kind:Kind::Crafting,opened:1539,outfits:vec![Outfit{id:9780,slot:1,members:true},Outfit{id:9781,slot:1,members:true}]});
 let before=w.data(a).unwrap().clone();w.submit(a,Action::OperateLocation{coords:loc.coords,id:14910,operation:1,control:false}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();let d=w.data(a).unwrap();
 assert_eq!(d.coords,if current>=40{CoordGrid::new(3220,3219,0).unwrap()}else{before.coords},"cape{cape} member{member} entrycurrent{current}");assert_eq!(d.stats,before.stats);assert_eq!(d.inventory,before.inventory);assert_eq!(d.equipment,before.equipment);
 let events=w.drain_events();if current>=40{assert!(events.iter().any(|e|matches!(e,WorldEvent::LocationMessage{message,..}if *message=="Welcome to the Crafting Guild.")));}else{assert!(events.iter().any(|e|matches!(e,WorldEvent::LocationMessage{message,..}if *message=="You need level 40 Crafting to enter the guild.")));}
 for _ in 0..4{w.tick_with_random(&mut NoRandom).unwrap();}assert!(!w.protection(a).unwrap().delayed());let before_boost=w.data(a).unwrap().clone();w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();let mut expected=before_boost.stats.clone();if member{expected.set_current(12,current+1);}assert_eq!(w.data(a).unwrap().stats,expected);assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().equipment,before.equipment);
 if !member{assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::LocationMessage{message,..}if *message=="You need membership to use this cape's Boost option.")));}
 if current==39&&member{w.submit(a,Action::OperateLocation{coords:loc.coords,id:14910,operation:1,control:false}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();assert_eq!(w.data(a).unwrap().coords,CoordGrid::new(3220,3219,0).unwrap());assert_eq!(w.data(a).unwrap().stats,expected);}
 }}}println!("PG_CAPE_GUILD12 bothcapes member/F2P current39/40/99 actualOpLocentryindependentofmemberBoost;memberdrained39Boost40thenfreshentry;allXPbaggear noRNG PASS");
}
