//! Independently authored FOOD/COMBO-only checks; no potion fixture or action.
use gielinor_world::{consumable_clocks::{Kind,Policy,Rules,State},food::Edible,inventory::Item,npc::RandomSource,simulation::{Action,ActorId,PlayerData,World,WorldEvent},stats::Stats,CoordGrid};
struct Fixed;
impl RandomSource for Fixed {fn below(&mut self,_:u32)->u32{panic!("fixed food consumed RNG")}}
fn policy(combo:bool)->Policy{Policy{kind:if combo{Kind::ComboFood}else{Kind::Food},consume_delay:3,combat_delay:if combo{2}else{3},members:combo}}
#[test]
fn independent_timeline_boundary_and_attack_matrix(){
 let mut count=0;
 for now in [0,1,9,99,1000] {for active in [-1,0,1,2,9] {for first in [false,true] {for second in [false,true] {for delta in 0..=5 {
  let attack=now+active;let p=State::default().plan(now,policy(first),attack).unwrap();
  let end=if active>0{attack+if first{2}else{3}}else{attack};assert_eq!(p.action_delay,end);
  let ready=delta>=3||(!first&&second);
  let next=p.state.plan(now+delta,policy(second),0);
  assert_eq!(next.is_ok(),ready,"now={now} first={first} second={second} delta={delta}");
  if let Ok(q)=next{let expected=if end>now+delta{end+if second{2}else{3}}else{0};assert_eq!(q.action_delay,expected);}
  for at in now..=now+16{assert_eq!(p.state.remaining_attack_delay(at,0),(if active>0{end}else{-1}).saturating_sub(at).max(0));}
  count+=1;
 }}}}}
 assert_eq!(count,600);println!("PG_FOOD_CLOCK_TIMELINE600 two-category expiry/equality and active-only additive attack checks PASS");
 for kind in [false,true]{for now in [-1,i32::MAX-1,i32::MAX]{let s=State::default();assert!(s.plan(now,policy(kind),0).is_err());assert_eq!(s,State::default());}}
}
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
fn queued_orders_expiry_and_failed_attempts_preserve_native_clocks(){
 let mut n=0;
 for slot in [0,13,27]{for first in [false,true]{for second in [false,true]{for gap in 0..=4{
  let(mut w,a)=fixture(first,slot);let other=(slot+1)%28;let id=if first{2162}else{2142};let id2=if second{2162}else{2142};
  w.data_mut(a).unwrap().inventory.restore_slot(other,Some(Item{id:id2,count:1,vars:0})).unwrap();let original=w.data(a).unwrap().clone();
  eat(&mut w,a,slot,id);
  if gap==0{eat(&mut w,a,other,id2);w.tick_with_random(&mut Fixed).unwrap();}else{w.tick_with_random(&mut Fixed).unwrap();for _ in 1..gap{w.tick_with_random(&mut Fixed).unwrap();}eat(&mut w,a,other,id2);w.tick_with_random(&mut Fixed).unwrap();}
  let allowed=gap>=3||(!first&&second);let d=w.data(a).unwrap();assert!(d.inventory.get(slot).is_none());assert_eq!(d.inventory.get(other).is_none(),allowed,"first={first} second={second} gap={gap}");
  assert_eq!(d.stats.current(3),50+if allowed{4}else{2});for s in 0..23{assert_eq!(d.stats.fine_xp(s),original.stats.fine_xp(s));}assert_eq!(d.equipment.slots(),original.equipment.slots());
  assert_eq!(traces(&mut w),if allowed{2}else{1});assert!(!w.protection(a).unwrap().delayed());n+=1;
 }}}}
 assert_eq!(n,60);println!("PG_FOOD_CLOCK_WORLD60 authoritative queue category order/exactexpiry/fullbag/XP/equipment/noRNG PASS");
 for slot in [0,13,27]{for mode in 0..5{
  let(mut w,a)=fixture(true,slot);let original=w.data(a).unwrap().clone();
  match mode{0=>w.set_membership(a,false).unwrap(),1=>{w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:2162,count:1,vars:7})).unwrap();},2=>{w.consumable_rules.as_mut().unwrap().foods.remove(&2162);},3=>{w.consumable_rules.as_mut().unwrap().foods.get_mut(&2162).unwrap().consume_delay=2;},_=>{w.foods.get_mut(&2162).unwrap().heal=-1;}}
  eat(&mut w,a,slot,2162);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(traces(&mut w),0);assert_eq!(w.data(a).unwrap().stats,original.stats);
  w.set_membership(a,true).unwrap();w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:2162,count:1,vars:0})).unwrap();w.foods.get_mut(&2162).unwrap().heal=2;w.consumable_rules.as_mut().unwrap().foods.insert(2162,policy(true));
  eat(&mut w,a,slot,2162);w.tick_with_random(&mut Fixed).unwrap();assert!(w.data(a).unwrap().inventory.get(slot).is_none());assert_eq!(traces(&mut w),1);
 }}
 println!("PG_FOOD_CLOCK_REFUSAL15 member/vars/missingrule/mismatcheddelay/invalidheal publish no category lock;nexttick corrected valid action succeeds PASS");
}
