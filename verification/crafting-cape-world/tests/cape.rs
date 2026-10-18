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
#[test]fn all_drained_and_overboosted_values_both_capes_live_member_and_personal_rates(){
 let mut n=0;for cape in [9780,9781]{for current in [0,1,50,98,99,100,104,120]{for member in [false,true]{for rate in [100,125,500]{
  let(mut w,a)=fixture(cape,current,member);w.data_mut(a).unwrap().xp_rate_hundreds=rate;let before=w.data(a).unwrap().clone();
  for press in 0..3{w.submit(a,Action::BoostWornItem{slot:1,item:cape}).unwrap();w.tick_with_random(&mut NoRandom).unwrap();let d=w.data(a).unwrap();let mut expected=before.stats.clone();
   let boosted=if member&&current<100{(current+press+1).min(100)}else{current};expected.set_current(12,boosted);
   assert_eq!(d.stats,expected,"cape{cape} current{current} member{member} rate{rate} press{press}");assert_eq!(d.inventory,before.inventory);assert_eq!(d.equipment,before.equipment);assert_eq!(d.xp_rate_hundreds,rate);
   let events=w.drain_events();assert!(!events.iter().any(|e|matches!(e,WorldEvent::SkillAdvanced{..})));
   if !member{assert!(events.iter().any(|e|matches!(e,WorldEvent::LocationMessage{message,..}if *message=="You need membership to use this cape's Boost option.")));}
  }n+=1;
 }}}}assert_eq!(n,96);println!("PG_CAPE_WORLD96 both variants eight current values liveMember/F2P three rates;three explicit requests each;wholebaggearStats noXP noRNG PASS");
}
#[test]fn forty_eight_live_changes_between_submit_and_dispatch_fail_closed(){
 for cape in [9780,9781]{for case in 0..24{let(mut w,a)=fixture(cape,99,true);let slot=if case==1{4}else{1};let item=if case==2{9782}else{cape};w.submit(a,Action::BoostWornItem{slot,item}).unwrap();
  match case{
   0=>w.set_membership(a,false).unwrap(),1|2=>{},3=>w.data_mut(a).unwrap().stats.set_base(12,98),
   4=>w.data_mut(a).unwrap().equipment.restore_slot(1,None).unwrap(),
   5..=8=>{let mut i=w.data(a).unwrap().equipment.get(1).unwrap();match case{5=>i.id=if cape==9780{9781}else{9780},6=>i.vars=1,7=>i.count=2,_=>i.count=0}w.data_mut(a).unwrap().equipment.restore_slot(1,Some(i)).unwrap();},
   9=>{w.object_types.dummy.insert(cape);},10=>{w.object_types.stackable.insert(cape);},
   11=>{w.object_types.notes.insert(cape,Template{link:cape,template:799});},12=>{w.object_types.placeholders.insert(cape,Template{link:cape,template:14401});},13=>{w.object_types.transforms.insert(cape,Template{link:cape,template:1});},
   14=>w.set_modal_open(a,true).unwrap(),15=>w.delay_player(a,8).unwrap(),16=>w.data_mut(a).unwrap().stats.set_current(3,0),
   17=>{w.worn_stat_boosts.remove(&cape);},
   18=>w.worn_stat_boosts.get_mut(&cape).unwrap().slot=14,19=>w.worn_stat_boosts.get_mut(&cape).unwrap().skill=24,20=>w.worn_stat_boosts.get_mut(&cape).unwrap().button_operation=3,
   21=>w.data_mut(a).unwrap().stats.set_base(12,100),
   22=>{w.data_mut(a).unwrap().equipment.restore_slot(1,Some(Item{id:9782,count:1,vars:0})).unwrap();},
   23=>w.request_logout(a).unwrap(),_=>unreachable!()
  }
  let before=w.data(a).unwrap().clone();w.tick_with_random(&mut NoRandom).unwrap();let d=w.data(a).unwrap();assert_eq!(d.stats,before.stats,"cape{cape} latecase{case}");assert_eq!(d.inventory.slots(),before.inventory.slots());assert_eq!(d.equipment.slots(),before.equipment.slots());
 }}println!("PG_CAPE_WORLD48 late member/base/claimedslot/identity/countvars/catalog/modal/delay/HP/config/logout refusal;wholephysicalbaggearStats PASS");
}
