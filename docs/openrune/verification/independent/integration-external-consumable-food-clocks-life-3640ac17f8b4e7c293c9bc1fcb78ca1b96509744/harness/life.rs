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
fn both_real_death_hooks_clear_category_and_retained_attack_contribution(){
 use gielinor_world::hits::{Hit,Style,Target,PlayerQueues};
 let mut count=0;
 for slot in [0,13,27]{for combo in [false,true]{for mode in 0..3{
  let(mut w,a)=fixture(combo,slot);let id=if combo{2162}else{2142};let other=(slot+1)%28;
  // Long bounded sentinel makes reset distinguishable from natural3tick expiry.
  w.foods.get_mut(&id).unwrap().delay=30;w.consumable_rules.as_mut().unwrap().foods.get_mut(&id).unwrap().consume_delay=30;
  w.data_mut(a).unwrap().inventory.restore_slot(other,Some(Item{id,count:1,vars:0})).unwrap();
  w.set_next_attack_delay(a,100).unwrap();let start=w.clock.0;eat(&mut w,a,slot,id);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(traces(&mut w),1);assert!(w.is_attack_delayed(a).unwrap());
  let mut death=false;
  if mode!=0{
   if mode==2{w.player_hit_queues=Some(PlayerQueues{standard:10,death:11});}
   w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:100},1).unwrap();
   for _ in 0..8{w.tick_with_random(&mut Fixed).unwrap();if w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(x))if *x==a)){death=true;break;}}
   assert!(death,"mode={mode}");assert_eq!(w.data(a).unwrap().stats.current(3),0);
  }else{for _ in 0..8{w.tick_with_random(&mut Fixed).unwrap();}w.drain_events();}
  assert!(w.clock.0<start+30);assert!(w.is_attack_delayed(a).unwrap(),"generic action clock intentionally survives death");
  w.set_next_attack_delay(a,0).unwrap();assert_eq!(w.is_attack_delayed(a).unwrap(),!death,"retained contribution reset distinct from generic action clock");
  // Trusted HP restoration only; this is not a real respawn/native claim.
  w.data_mut(a).unwrap().stats.set_current(3,50);eat(&mut w,a,other,id);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(w.data(a).unwrap().inventory.get(other).is_none(),death);assert_eq!(traces(&mut w),if death{1}else{0});count+=1;
 }}}
 assert_eq!(count,18);println!("PG_FOOD_CLOCK_DEATH18 twoactualDeathRequestedpaths and matchedalivecontrols reset30ticksentinel category/retainedattack beforeexpiry;generic actionclock unchanged PASS");
}
