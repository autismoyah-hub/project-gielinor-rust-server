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
fn current_membership_and_selected_identity_are_revalidated_before_commit(){
 let ids=[7521,7523,7524,7525,7526];let mut count=0;
 for slot in [0,13,27]{for(index,&id)in ids.iter().enumerate(){for mode in 0..4{
  let(mut w,a)=fixture(false,slot);w.foods.clear();w.foods.insert(id,Edible{id,heal:2,delay:2,replacement:ids.get(index+1).copied(),message:"Crab".into()});w.member_foods.insert(id);w.slot_preserving_foods.insert(id);w.consumable_rules.as_mut().unwrap().foods.clear();w.consumable_rules.as_mut().unwrap().foods.insert(id,Policy{kind:Kind::Food,consume_delay:2,combat_delay:2,members:true});w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id,count:1,vars:0})).unwrap();w.data_mut(a).unwrap().inventory.restore_slot((slot+1)%28,Some(Item{id,count:1,vars:0})).unwrap();
  if mode==0{w.set_membership(a,false).unwrap();}eat(&mut w,a,slot,id);
  match mode{0=>{},1=>w.set_membership(a,false).unwrap(),2=>{w.data_mut(a).unwrap().inventory.restore_slot(slot,None).unwrap();},_=>{w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:1931,count:1,vars:0})).unwrap();}}
  let before=w.data(a).unwrap().clone();w.tick_with_random(&mut Fixed).unwrap();assert_eq!(w.data(a).unwrap().inventory.slots(),before.inventory.slots());assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(traces(&mut w),0);
  w.set_membership(a,true).unwrap();w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id,count:1,vars:0})).unwrap();eat(&mut w,a,slot,id);w.tick_with_random(&mut Fixed).unwrap();assert_eq!(w.data(a).unwrap().inventory.get(slot).map(|v|v.id),ids.get(index+1).copied());assert_eq!(w.data(a).unwrap().inventory.get((slot+1)%28).unwrap().id,id);assert_eq!(w.data(a).unwrap().stats.current(3),52);assert_eq!(traces(&mut w),1);count+=1;
 }}}
 assert_eq!(count,60);println!("PG_CRAB_CURRENT60 allstages/nonmember/live-memberloss/removed/replacedselecteditem refusals preservewholeinventoryStats noRNGtrace;correctednexttickaction provesnoleakedclock PASS");
}
