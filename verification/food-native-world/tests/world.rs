//! Pinned152 typed adapter -> real authoritative World, controlled registered food rules.
use gielinor_cache::{definitions::ItemType,game::{Definitions,GameCache}};
use gielinor_protocol::buttons::Button;
use gielinor_world::{consumable_clocks::{Rules,Policy,Kind},food::Edible,inventory::Item,npc::RandomSource,simulation::{World,PlayerData,WorldEvent},stats::Stats,CoordGrid};
use std::{collections::BTreeMap,path::Path};
struct Fixed;impl RandomSource for Fixed{fn below(&mut self,_:u32)->u32{panic!("fixed food drew RNG")}}
#[test]
fn cache_operation_claims_reach_authoritative_current_inventory_and_member_checks(){
 let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};for id in[2128,2152,1931]{defs.items.insert(id,ItemType::decode(&raw[&id]).unwrap());}
 let mut foods=BTreeMap::new();let mut rules=Rules::default();for(id,heal,kind,combat)in[(2128,1,Kind::Food,3),(2152,3,Kind::ComboFood,2)]{foods.insert(id,Edible{id,heal,delay:3,replacement:None,message:"Food".into()});rules.foods.insert(id,Policy{kind,consume_delay:3,combat_delay:combat,members:true});}
 let mut count=0;let mut successes=0;
 for slot in[0,13,27]{for current in[2128,2152,1931,-1]{for claimed in[2128,2152,1931,-1]{for op in 0..=6{for member in[false,true]{
  let mut w=World::default();w.collision.allocate(3224,3216,0);w.foods=foods.clone();w.member_foods=[2128,2152].into();w.slot_preserving_foods=[2128,2152].into();w.consumable_rules=Some(rules.clone());w.eating_animation=829;
  let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,50);stats.set_fine_xp(3,130344311).unwrap();let mut data=PlayerData::new(1,"Independent typed food World".into(),CoordGrid::new(3224,3218,0).unwrap(),stats);
  for s in 0..28{data.inventory.restore_slot(s,if s==slot&&current==-1{None}else{Some(Item{id:if s==slot||(s==(slot+1)%28&&current!=-1){current}else{1931},count:1,vars:0})}).unwrap();}
  w.queue_login(1,data).unwrap();w.tick_with_random(&mut Fixed).unwrap();let a=w.actors().next().unwrap();w.set_membership(a,member).unwrap();w.drain_events();let before=w.data(a).unwrap().clone();
  let button=Button{component:149<<16,slot:slot as i32,object:claimed,op};if let Some(action)=pg_food_boundary::route(&button,slot,current,&foods,&defs,true){w.submit(a,action).unwrap();}w.tick_with_random(&mut Fixed).unwrap();
  let success=member&&current==claimed&&((current==2128&&op==2)||(current==2152&&op==5));let mut expected_inv=before.inventory;let mut expected_stats=before.stats;if success{expected_inv.restore_slot(slot,None).unwrap();expected_stats.set_current(3,if current==2128{51}else{53});successes+=1;}
  let p=w.data(a).unwrap();assert_eq!(p.inventory.slots(),expected_inv.slots());assert_eq!(p.stats,expected_stats);assert_eq!(p.equipment.slots(),before.equipment.slots());assert!(!w.protection(a).unwrap().delayed());let traces=w.drain_events().iter().filter(|e|matches!(e,WorldEvent::FoodTransition{..})).count();assert_eq!(traces,usize::from(success));count+=1;
 }}}}}
 assert_eq!(count,672);assert_eq!(successes,6);println!("PG_FOOD_TYPED_WORLD672 realcacheop2/op5adapter plus authoritative liveitem/member gate;6validselectedcommits666refusals fullbagduplicate/allStatsfineXPgear noRNG/protection/successtraceonrefusal PASS");
}
