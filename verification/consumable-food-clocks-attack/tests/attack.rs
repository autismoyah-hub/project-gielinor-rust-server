//! Independently authored FOOD/COMBO-only checks; no potion fixture or action.
use gielinor_world::{consumable_clocks::{Kind,Policy,Rules,State},food::Edible,inventory::Item,npc::RandomSource,simulation::{Action,ActorId,PlayerData,World,WorldEvent},stats::Stats,CoordGrid};
struct Fixed;
impl RandomSource for Fixed {fn below(&mut self,_:u32)->u32{panic!("fixed food consumed RNG")}}
fn policy(combo:bool)->Policy{Policy{kind:if combo{Kind::ComboFood}else{Kind::Food},consume_delay:3,combat_delay:if combo{2}else{3},members:combo}}
fn fixture(first:bool,slot:usize)->(World,ActorId){
 let mut w=World::default();w.collision.allocate(3224,3216,0);
 let mut r=Rules::default();
 for (id,combo)in[(2142,false),(2162,true)]{w.foods.insert(id,Edible{id,heal:2,delay:3,replacement:None,message:"Food".into()});r.foods.insert(id,policy(combo));}
 w.consumable_rules=Some(r);w.eating_animation=829;
 let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,50);
 let mut p=PlayerData::new(1,"Independent clock player".into(),CoordGrid::new(3224,3218,0).unwrap(),stats);
 for s in 0..28{p.inventory.restore_slot(s,Some(Item{id:if s==slot{if first{2162}else{2142}}else{1931},count:1,vars:0})).unwrap();}
 w.queue_login(1,p).unwrap();w.tick_with_random(&mut Fixed).unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();(w,a)
}
fn eat(w:&mut World,a:ActorId,slot:usize,id:i32){w.submit(a,Action::EatItemSlot{slot,item:id}).unwrap();}
fn traces(w:&mut World)->usize{w.drain_events().iter().filter(|e|matches!(e,WorldEvent::FoodTransition{..})).count()}
#[test]
fn attack_contributions_survive_setter_and_movement(){
 let mut n=0;
 for slot in [0,13,27]{for combo_first in [false,true]{for active in [0,1,4,10]{for replacement in [0,1,2]{
  let(mut w,a)=fixture(combo_first,slot);let other=(slot+1)%28;let second=if combo_first{2142}else{2162};
  w.data_mut(a).unwrap().inventory.restore_slot(other,Some(Item{id:second,count:1,vars:0})).unwrap();
  let start=w.clock.0;w.set_next_attack_delay(a,active).unwrap();eat(&mut w,a,slot,if combo_first{2162}else{2142});eat(&mut w,a,other,second);w.tick_with_random(&mut Fixed).unwrap();
  let rows:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::FoodTransition{trace,..}=e{Some(trace)}else{None}).collect();let contribution=if combo_first{2}else{5};assert_eq!(rows.len(),if combo_first{1}else{2});
  assert_eq!(rows[0].tick,start);assert_eq!(rows[0].attack_delay,Some(if active>0{active+if combo_first{2}else{3}}else{0}));
  if !combo_first{assert_eq!(rows[1].attack_delay,Some(if active>0{active+5}else{0}));}
  if replacement!=0{w.set_next_attack_delay(a,0).unwrap();}
  if replacement==2{w.submit(a,Action::Walk{destination:CoordGrid::new(3225,3218,0).unwrap(),run:false}).unwrap();w.tick_with_random(&mut Fixed).unwrap();assert_eq!(w.data(a).unwrap().coords,CoordGrid::new(3225,3218,0).unwrap());}
  let end=if active>0{start+active+contribution}else{start};
  while w.clock.0<=end+1{assert_eq!(w.is_attack_delayed(a).unwrap(),w.clock.0<end);w.tick_with_random(&mut Fixed).unwrap();}
  n+=1;
 }}}}
 assert_eq!(n,72);println!("PG_FOOD_CLOCK_ATTACK72 active/ready additive3or5or2 exactremainingtrace zero-setter/realWalk nevererase committedcontribution PASS");
}
