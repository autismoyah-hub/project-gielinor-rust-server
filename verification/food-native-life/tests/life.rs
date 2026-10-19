//! Pinned152 typed adapter -> real authoritative World, controlled registered food rules.
use gielinor_cache::{definitions::ItemType,game::{Definitions,GameCache}};
use gielinor_protocol::buttons::Button;
use gielinor_world::{consumable_clocks::{Rules,Policy,Kind},food::Edible,inventory::Item,npc::RandomSource,simulation::{World,PlayerData,WorldEvent},stats::Stats,CoordGrid};
use std::{collections::BTreeMap,path::Path};
struct Fixed;impl RandomSource for Fixed{fn below(&mut self,_:u32)->u32{panic!("fixed food drew RNG")}}
#[test]
fn observe_food_operation_after_zero_hp_and_actual_unresolved_death(){
 use gielinor_world::hits::{Hit,Style,Target};
 let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};for id in[2128,2152,1931]{defs.items.insert(id,ItemType::decode(&raw[&id]).unwrap());}
 let mut foods=BTreeMap::new();let mut rules=Rules::default();for(id,heal,kind,combat)in[(2128,1,Kind::Food,3),(2152,3,Kind::ComboFood,2)]{foods.insert(id,Edible{id,heal,delay:3,replacement:None,message:"Food".into()});rules.foods.insert(id,Policy{kind,consume_delay:3,combat_delay:combat,members:true});}

 let mut violations=0;
 for slot in[0,13,27]{for(id,op,heal)in[(2128,2,1i8),(2152,5,3)]{for phase in 0..3{
  let mut w=World::default();w.collision.allocate(3224,3216,0);w.foods=foods.clone();w.member_foods=[2128,2152].into();w.slot_preserving_foods=[2128,2152].into();w.consumable_rules=Some(rules.clone());w.eating_animation=829;let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,1);stats.set_fine_xp(3,130344311).unwrap();let mut data=PlayerData::new(1,"Independent food life World".into(),CoordGrid::new(3224,3218,0).unwrap(),stats);
  for s in 0..28{data.inventory.restore_slot(s,Some(Item{id:if s==slot||s==(slot+1)%28{id}else{1931},count:1,vars:0})).unwrap();}w.queue_login(1,data).unwrap();w.tick_with_random(&mut Fixed).unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();
  if phase==0{w.data_mut(a).unwrap().stats.set_current(3,0);}else{w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:20},1).unwrap();w.tick_with_random(&mut Fixed).unwrap();assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(x))if *x==a)));assert_eq!(w.data(a).unwrap().stats.current(3),0);if phase==2{w.data_mut(a).unwrap().stats.set_current(3,10);}}
  let before=w.data(a).unwrap().clone();let button=Button{component:149<<16,slot:slot as i32,object:id,op};let action=pg_food_boundary::route(&button,slot,id,&foods,&defs,true).unwrap();w.submit(a,action).unwrap();w.tick_with_random(&mut Fixed).unwrap();let p=w.data(a).unwrap();assert!(p.inventory.get(slot).is_none());assert_eq!(p.inventory.get((slot+1)%28).unwrap().id,id);assert_eq!(p.stats.current(3),before.stats.current(3)+heal);assert_eq!(p.stats.fine_xp(3),before.stats.fine_xp(3));assert_eq!(w.drain_events().iter().filter(|e|matches!(e,WorldEvent::FoodTransition{..})).count(),1);violations+=1;
 }}}
 assert_eq!(violations,18);println!("PG_FOOD_OP_LIFE_VIOLATIONS18 Equaop2/Toadop5 typedadapter plus actualWorld consumes/heals zeroHP/unresolvedDeathRequested/restoredHPwithoutrespawn;diagnosticPASSmeansimplementationV1FAIL");
}
