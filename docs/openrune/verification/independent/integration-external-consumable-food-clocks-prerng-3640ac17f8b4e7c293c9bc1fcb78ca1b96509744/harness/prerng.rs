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
fn failed_native_admission_precedes_random_healing(){
 let mut count=0;
 for slot in [0,13,27]{for mode in 0..7{
  let(mut w,a)=fixture(false,slot);let other=(slot+1)%28;
  if mode==0{w.data_mut(a).unwrap().inventory.restore_slot(other,Some(Item{id:2142,count:1,vars:0})).unwrap();eat(&mut w,a,other,2142);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(traces(&mut w),1);}
  w.food_random_healing.insert(2142,gielinor_world::food::RandomHealing{minimum:1,maximum:9});
  match mode{0=>{},1=>w.delay_player(a,5).unwrap(),2=>{w.data_mut(a).unwrap().inventory.restore_slot(slot,None).unwrap();},3=>{w.food_boosts.insert(2142,vec![gielinor_world::food::StatBoost{skill:0,amount:-1}]);},4=>{w.object_types.dummy.insert(2142);},5=>{w.foods.get_mut(&2142).unwrap().replacement=Some(2142);},_=>{w.clock.0=i32::MAX-1;}}
  let before=w.data(a).unwrap().clone();eat(&mut w,a,slot,2142);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(w.data(a).unwrap().inventory.slots(),before.inventory.slots());assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(traces(&mut w),0);count+=1;
 }}
 assert_eq!(count,21);println!("PG_FOOD_CLOCK_PRERNG21 cooldown/protection/empty/lateboost/catalog/remainder/overflow reject before anyrandomheal/inventory/stat/successtrace PASS");
}
