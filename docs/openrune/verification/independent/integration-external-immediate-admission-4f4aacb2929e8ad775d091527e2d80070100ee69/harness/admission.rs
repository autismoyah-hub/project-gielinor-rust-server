//! Separate public World admission campaign; completed resolver matrix is not repeated.
use gielinor_world::{grinding,tool_crafting::{Recipe,Failure},held_use::{Handler,Scope,Type,Request},inventory::{Item,StackType},simulation::{World,PlayerData,ActorId,WorldEvent},stats::Stats,CoordGrid};
use std::sync::Arc;
fn fixture(limestone:bool,reverse:bool,mode:usize)->(World,ActorId,Request){
 let(tool,raw,out,level,xp)=if limestone{(1755,3211,3420,12,60)}else{(946,771,772,31,0)};
 let mut w=World::default();w.collision.allocate(3200,3200,0);let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();let mut d=PlayerData::new(1,"Independent admission".into(),CoordGrid::new(3200,3200,0).unwrap(),stats);d.xp_rate_hundreds=125;
 for s in 0..28{d.inventory.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}for(s,id)in[(0,tool),(1,tool),(17,raw),(26,raw)]{d.inventory.restore_slot(s,Some(Item{id,count:1,vars:0})).unwrap();}
 w.queue_login(1,d).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();for id in[tool,raw]{w.held_use_types.insert(id,Type{id,content:-1});}
 let mut r=Recipe{exchange:grinding::Recipe{tool,input:raw,output:out,members:true,animation:Some(if limestone{1309}else{10937})},level,xp_tenths:xp,ticks:0,name:"independent",action:"craft",failure:if limestone{Some(Failure{output:968,xp_tenths:15,low:137,high:434,message:"failure"})}else{None}};
 if mode==19{r.ticks=1;}let handler=match mode{20=>Handler::ToolCrafting(r),21=>Handler::ToolCraftingChoices{recipes:Arc::new(vec![r]),cycle_ticks:2},_=>Handler::ToolCraftingImmediate(r)};w.held_use_handlers.register(Scope::Types,tool,raw,handler).unwrap();w.drain_events();
 let q=if reverse{Request{selected_slot:17,selected_id:raw,target_slot:0,target_id:tool}}else{Request{selected_slot:0,selected_id:tool,target_slot:17,target_id:raw}};(w,a,q)
}
#[test]
fn eighty_eight_refusals_leave_whole_inventory_stats_and_no_pending_production(){
 let mut count=0;for limestone in[false,true]{for reverse in[false,true]{for mode in 0..22{
 let(mut w,a,q)=fixture(limestone,reverse,mode);assert!(w.prepare_held_use(a,q).unwrap().is_some());let(tool,raw,out)=if limestone{(1755,3211,3420)}else{(946,771,772)};
 match mode{0=>w.set_membership(a,false).unwrap(),1=>w.data_mut(a).unwrap().stats.set_current(12,if limestone{11}else{30}),2=>w.set_modal_open(a,true).unwrap(),3=>w.request_logout(a).unwrap(),4=>w.delay_player(a,3).unwrap(),5=>{w.data_mut(a).unwrap().inventory.restore_slot(17,None).unwrap();},6=>{w.data_mut(a).unwrap().inventory.restore_slot(0,None).unwrap();},7=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:raw,count:1,vars:1})).unwrap();},8=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:tool,count:1,vars:1})).unwrap();},9=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:raw,count:2,vars:0})).unwrap();},10=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:tool,count:2,vars:0})).unwrap();},11=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:1931,count:1,vars:0})).unwrap();},12=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1931,count:1,vars:0})).unwrap();},13=>w.data_mut(a).unwrap().inventory.stack=StackType::Always,14=>w.data_mut(a).unwrap().inventory.placeholders=true,15=>{w.object_types.dummy.insert(out);},16=>{w.object_types.dummy.insert(if limestone{968}else{tool});},17=>{w.data_mut(a).unwrap().xp_rate_hundreds=-1;if !limestone{w.set_membership(a,false).unwrap();}},_=>{}}
 let before=w.data(a).unwrap().clone();let clock=w.clock.0;let who=if mode==18{ActorId{generation:a.generation+1,..a}}else{a};let result=w.finish_held_use(who,q);
 if matches!(mode,2|3|4|18|19|20|21){assert!(result.is_err(),"mode{mode}");}
 assert_eq!(w.clock.0,clock);assert!(w.take_tool_crafting_request(a).unwrap().is_none());assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::SkillAdvanced{..}|WorldEvent::PlayerCombatAnimation{..})),"mode{mode}");for _ in 0..8{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before.inventory,"limestone{limestone} mode{mode}");assert_eq!(w.data(a).unwrap().stats,before.stats,"limestone{limestone} mode{mode}");count+=1;
 }}}assert_eq!(count,88);
}
