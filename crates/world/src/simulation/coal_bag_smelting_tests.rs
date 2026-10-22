use super::*;
use crate::{inventory::Item,smithing_make::Request,variables::{Varp,Varbit,Transmit}};
use std::sync::Arc;
fn prepared(stored:i32,physical:usize)->(World,ActorId,Arc<Request>){
 let mut w=World::default();let c=crate::coal_bag::Config{closed:12019,open:24480,coal:453,capes:[9795,9796,13280],base:Varp{id:65531,permanent:true,transmit:Transmit::Never,bit_protect:true,client_code:-1},stored:Varbit{base:65531,lsb:25,msb:30}};
 w.coal_bag=Some(c);w.object_types.stackable.extend([12019,24480]);w.collision.allocate(3200,3200,0);
 let mut p=PlayerData::new(1,"Furnace coal".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());p.stats.set_current(13,99);p.inventory.restore_slot(27,Some(Item{id:24480,count:1,vars:0})).unwrap();for slot in 0..physical{p.inventory.restore_slot(slot,Some(Item{id:453,count:1,vars:0})).unwrap();}p.inventory.restore_slot(26,Some(Item{id:440,count:1,vars:0})).unwrap();p.variables.set(c.base,0x81000001u32 as i32,false);p.variables.set_bit(c.base,c.stored,stored,false).unwrap();w.queue_login(1,p).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.smithing=Some(crate::smithing_make::Config{queue:900,skill:13,max_level:99,xp_rate:1.0,tutorial:None});w.drain_events();
 (w,a,Arc::new(Request{product:2353,produced:1,consumed:vec![(440,1),(453,2)],xp:17.5,animation:899,cycles:5,impurity:false,members:false,repeat_delay:None,required_level:30,required_tools:vec![]}))
}
fn stock(w:&World,a:ActorId)->i32{w.data(a).unwrap().variables.bit(w.coal_bag.unwrap().stored).unwrap()}
fn total(w:&World,a:ActorId,id:i32)->i32{crate::smithing_make::total(&w.data(a).unwrap().inventory,&w.object_types,id)}
fn finish(w:&mut World){for _ in 0..6{w.tick().unwrap();}}
#[test]
fn coal_bag_smelting_consumes_physical_first_then_stored_and_preserves_packed_neighbors(){
 for stored in [0,1,2,27,36,63]{for physical in 0..=3{
 let(mut w,a,r)=prepared(stored,physical);w.start_smelting(a,r,1).unwrap();finish(&mut w);let made=stored+physical as i32>=2;let from_inventory=if made{(physical as i32).min(2)}else{0};assert_eq!(total(&w,a,453),physical as i32-from_inventory);assert_eq!(stock(&w,a),stored-if made{2-from_inventory}else{0});assert_eq!(total(&w,a,2353),i32::from(made));assert_eq!(total(&w,a,440),i32::from(!made));assert_eq!(w.data(a).unwrap().stats.fine_xp(13),if made{175}else{0});assert_eq!(w.data(a).unwrap().variables.get(65531)as u32&!(63<<25),0x81000001);
 }}
}
#[test]
fn coal_bag_smelting_rechecks_live_bag_membership_level_and_cache_at_delayed_completion(){
 for change in 0..7{let(mut w,a,r)=prepared(4,0);w.start_smelting(a,r,1).unwrap();w.tick().unwrap();match change{0=>{w.data_mut(a).unwrap().inventory.restore_slot(27,None).unwrap();},1=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:12019,count:1,vars:0})).unwrap();},2=>w.set_membership(a,false).unwrap(),3=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:24480,count:2,vars:0})).unwrap();},4=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:24480,count:1,vars:1})).unwrap();},5=>w.data_mut(a).unwrap().stats.set_current(13,29),_=>{w.object_types.dummy.insert(453);}}
 let before=w.data(a).unwrap().clone();finish(&mut w);assert_eq!(w.data(a).unwrap().inventory,before.inventory,"case{change}");assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(stock(&w,a),4);assert!(!w.player(a).unwrap().queues.contains(900));}
}
#[test]
fn coal_bag_smelting_output_or_late_ingredient_failure_rolls_back_every_input_and_xp(){
 for change in 0..3{let(mut w,a,mut r)=prepared(3,1);match change{0=>{Arc::get_mut(&mut r).unwrap().produced=30;},1=>{Arc::get_mut(&mut r).unwrap().consumed.push((999,1));},_=>{Arc::get_mut(&mut r).unwrap().xp=f64::INFINITY;}}
 let before=w.data(a).unwrap().clone();w.start_smelting(a,r,1).unwrap();finish(&mut w);assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(stock(&w,a),3);}
}
#[test]
fn coal_bag_smelting_repeats_with_virtual_inputs_and_cancels_through_existing_weak_queue(){
 for cancel in [false,true]{let(mut w,a,r)=prepared(4,0);w.data_mut(a).unwrap().inventory.restore_slot(25,Some(Item{id:440,count:1,vars:0})).unwrap();w.start_smelting(a,r,5).unwrap();for _ in 0..5{w.tick().unwrap();}assert_eq!(total(&w,a,2353),1);assert_eq!(stock(&w,a),2);
 if cancel{w.submit(a,Action::CoalBagItem{slot:27,item:24480,operation:4}).unwrap();}for _ in 0..15{w.tick().unwrap();}assert_eq!(total(&w,a,2353),if cancel{1}else{2});assert_eq!(stock(&w,a),if cancel{2}else{0});assert_eq!(w.data(a).unwrap().stats.fine_xp(13),if cancel{175}else{350});assert!(!w.player(a).unwrap().queues.contains(900));}
}
#[test]
fn coal_bag_smelting_selection_counts_virtual_coal_but_generic_forging_does_not(){
 let(mut w,a,r)=prepared(2,0);let types=[(440,"Iron ore"),(453,"Coal"),(2353,"Steel bar"),(24480,"Open coal bag")].into_iter().map(|(id,name)|(id,crate::smelting::ItemType{name:name.into(),certificate:false,placeholder:false})).collect();let recipe=crate::smelting::Recipe{bar:2353,label:"Steel".into(),level:30,xp:17.5,ingredients:vec![(440,1),(453,2)],impurity:false,members:false,repeat_delay:None};let p=w.data(a).unwrap();assert!(matches!(recipe.select(&p.inventory,&w.object_types,&types,&p.stats,13,899,1).unwrap(),crate::smelting::Selection::Denied(_)));assert!(matches!(recipe.select_with_coal_bag(&p.inventory,&w.object_types,&types,&p.stats,13,899,1,w.coal_bag,&p.variables,true).unwrap(),crate::smelting::Selection::Start{..}));w.start_smithing(a,r,1).unwrap();finish(&mut w);assert_eq!(stock(&w,a),2);assert_eq!(total(&w,a,2353),0);assert_eq!(total(&w,a,440),1);
}

#[test]
fn coal_bag_smelting_dead_or_departing_players_cannot_admit_or_complete_a_furnace_reward(){
 for before_queue in [false,true]{for change in 0..3{let(mut w,a,r)=prepared(4,0);if !before_queue{w.start_smelting(a,r.clone(),1).unwrap();w.tick().unwrap();}let p=w.players.get_mut(a.slot as usize).unwrap();match change{0=>p.death_queued=true,1=>p.data.stats.set_current(3,0),_=>p.pending_logout=true};let before=p.data.clone();if before_queue{assert!(w.start_smelting(a,r,1).is_err());}else{finish(&mut w);}if let Ok(p)=w.data(a){assert_eq!(p.inventory,before.inventory);assert_eq!(p.stats,before.stats);assert_eq!(p.variables.get(65531),before.variables.get(65531));}assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::SmithingProductMade{..}|WorldEvent::SkillAdvanced{..})));}}
}
