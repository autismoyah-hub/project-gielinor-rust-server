use super::*;
use crate::inventory::Item;
fn prepared(gear:Option<(usize,i32)>)->(World,ActorId,Arc<Request>){
 let mut w=World::default();w.collision.allocate(3200,3200,0);w.goldsmith_smelting=Some(crate::goldsmith_smelting::Config{ore:444,bar:2357,gauntlets:776,capes:[9795,9796,13280]});w.smithing=Some(Config{queue:900,skill:13,max_level:99,xp_rate:1.0,tutorial:None});let mut p=PlayerData::new(1,"Gold furnace".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());p.stats.set_base(13,99);p.stats.set_current(13,99);for slot in 0..2{p.inventory.restore_slot(slot,Some(Item{id:444,count:1,vars:0})).unwrap();}if let Some((slot,id))=gear{p.equipment.restore_slot(slot,Some(Item{id,count:1,vars:0})).unwrap();}w.queue_login(1,p).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();(w,a,Arc::new(Request{product:2357,produced:1,consumed:vec![(444,1)],xp:22.5,animation:899,cycles:5,impurity:false,members:false,repeat_delay:None,required_level:40,required_tools:vec![]}))
}
fn finish(w:&mut World,n:usize){for _ in 0..n{w.tick().unwrap();}}
fn count(w:&World,a:ActorId,id:i32)->i32{crate::smithing_make::total(&w.data(a).unwrap().inventory,&w.object_types,id)}
#[test]
fn goldsmith_smelting_worn_forms_member_authority_and_fractional_rates_award_once(){
 for gear in [None,Some((9,776)),Some((1,9795)),Some((1,9796)),Some((1,13280))]{for member in [false,true]{for(rate,global)in[(100,1.0),(500,1.0),(250,1.0),(500,2.0)]{let(mut w,a,r)=prepared(gear);w.set_membership(a,member).unwrap();w.data_mut(a).unwrap().xp_rate_hundreds=rate;w.smithing.as_mut().unwrap().xp_rate=global;let equipment=w.data(a).unwrap().equipment.clone();w.start_smelting(a,r,1).unwrap();finish(&mut w,6);let base=if member&&gear.is_some(){56.2}else{22.5};assert_eq!(w.data(a).unwrap().stats.fine_xp(13),(base*rate as f64/100.0*global*10.0)as i32);assert_eq!(count(&w,a,2357),1);assert_eq!(count(&w,a,444),1);assert_eq!(w.data(a).unwrap().equipment,equipment);}}}
}
#[test]
fn goldsmith_smelting_live_equipment_is_rechecked_before_first_and_repeated_reward(){
 for initially_worn in [false,true]{let(mut w,a,r)=prepared(if initially_worn{Some((9,776))}else{None});w.start_smelting(a,r,2).unwrap();w.tick().unwrap();w.data_mut(a).unwrap().equipment.restore_slot(9,if initially_worn{None}else{Some(Item{id:776,count:1,vars:0})}).unwrap();finish(&mut w,4);assert_eq!(count(&w,a,2357),1);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),if initially_worn{225}else{562});w.data_mut(a).unwrap().equipment.restore_slot(9,if initially_worn{Some(Item{id:776,count:1,vars:0})}else{None}).unwrap();finish(&mut w,6);assert_eq!(count(&w,a,2357),2);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),787);assert!(!w.player(a).unwrap().queues.contains(900));}
}
#[test]
fn goldsmith_smelting_carried_wrong_slot_malformed_or_invalid_gear_cannot_boost(){
 for case in 0..7{let(mut w,a,r)=prepared(None);let p=w.data_mut(a).unwrap();match case{0=>{p.inventory.restore_slot(27,Some(Item{id:776,count:1,vars:0})).unwrap();},1=>{p.equipment.restore_slot(1,Some(Item{id:776,count:1,vars:0})).unwrap();},2=>{p.equipment.restore_slot(9,Some(Item{id:9795,count:1,vars:0})).unwrap();},3=>{p.equipment.restore_slot(9,Some(Item{id:776,count:2,vars:0})).unwrap();},4=>{p.equipment.restore_slot(9,Some(Item{id:776,count:1,vars:3})).unwrap();},_=>{p.equipment.restore_slot(9,Some(Item{id:776,count:1,vars:0})).unwrap();}}
 if case==5{w.object_types.dummy.insert(776);}if case==6{w.object_types.stackable.insert(776);}w.start_smelting(a,r,1).unwrap();finish(&mut w,6);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),225,"case{case}");assert_eq!(count(&w,a,2357),1);}
}
#[test]
fn goldsmith_smelting_both_perks_do_not_stack_and_other_or_generic_jobs_keep_base_xp(){
 let(mut w,a,r)=prepared(Some((9,776)));w.data_mut(a).unwrap().equipment.restore_slot(1,Some(Item{id:9795,count:1,vars:0})).unwrap();w.start_smelting(a,r,1).unwrap();finish(&mut w,6);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),562);
 for case in 0..3{let(mut w,a,mut r)=prepared(Some((9,776)));if case==0{w.start_smithing(a,r,1).unwrap();}else{let q=Arc::get_mut(&mut r).unwrap();if case==1{q.product=2353;q.xp=17.5;}else{q.xp=23.0;}w.start_smelting(a,r,1).unwrap();}finish(&mut w,6);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),[225,175,230][case]);}
}
#[test]
fn goldsmith_smelting_failed_product_awards_no_bonus_and_current_membership_can_change(){
 let(mut w,a,r)=prepared(Some((9,776)));w.start_smelting(a,r,1).unwrap();w.tick().unwrap();w.set_membership(a,false).unwrap();finish(&mut w,5);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),225);
 let(mut w,a,r)=prepared(Some((9,776)));w.start_smelting(a,r,1).unwrap();w.tick().unwrap();w.object_types.dummy.insert(2357);finish(&mut w,5);assert_eq!(count(&w,a,2357),0);assert_eq!(count(&w,a,444),2);assert_eq!(w.data(a).unwrap().stats.fine_xp(13),0);
}
