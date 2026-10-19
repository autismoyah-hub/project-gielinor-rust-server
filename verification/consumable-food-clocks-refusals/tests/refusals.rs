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
fn only_remaining_refusal_cases(){
 for slot in [0,13,27]{for mode in 0..5{
  let(mut w,a)=fixture(true,slot);let original=w.data(a).unwrap().clone();
  match mode{0=>w.set_membership(a,false).unwrap(),1=>{w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:2162,count:2,vars:0})).unwrap();},2=>{w.consumable_rules.as_mut().unwrap().foods.remove(&2162);},3=>{w.consumable_rules.as_mut().unwrap().foods.get_mut(&2162).unwrap().consume_delay=2;},_=>{w.foods.get_mut(&2162).unwrap().heal=-1;}}
  eat(&mut w,a,slot,2162);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(traces(&mut w),0);assert_eq!(w.data(a).unwrap().stats,original.stats);
  w.set_membership(a,true).unwrap();w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:2162,count:1,vars:0})).unwrap();w.foods.get_mut(&2162).unwrap().heal=2;w.consumable_rules.as_mut().unwrap().foods.insert(2162,policy(true));
  eat(&mut w,a,slot,2162);w.tick_with_random(&mut Fixed).unwrap();assert!(w.data(a).unwrap().inventory.get(slot).is_none());assert_eq!(traces(&mut w),1);
 }}
 println!("PG_FOOD_CLOCK_REFUSAL15 member/count/missingrule/mismatcheddelay/invalidheal publish no category lock;nexttick corrected valid action succeeds PASS");
}
