//! New life boundaries only; do not repeat completed immutable campaigns.
use gielinor_world::{grinding,tool_crafting::{Recipe,Failure},held_use::{Handler,Scope,Type,Request},inventory::Item,simulation::{World,PlayerData,WorldEvent},stats::Stats,hits::{Hit,Target,Style},CoordGrid};
fn scenario(limestone:bool,reverse:bool,death:u8){
 let(tool,raw,out,level,xp)=if limestone{(1755,3211,3420,12,60)}else{(946,771,772,31,0)};
 let mut w=World::default();w.collision.allocate(3200,3200,0);let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();let mut d=PlayerData::new(1,"Independent immediate life boundary".into(),CoordGrid::new(3200,3200,0).unwrap(),stats);d.xp_rate_hundreds=500;
 for s in 0..28{d.inventory.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}for(s,id)in[(0,tool),(1,tool),(4,raw),(17,raw),(27,995)]{d.inventory.restore_slot(s,Some(Item{id,count:1,vars:0})).unwrap();}
 w.queue_login(1,d).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();for id in[tool,raw]{w.held_use_types.insert(id,Type{id,content:-1});}
 let r=Recipe{exchange:grinding::Recipe{tool,input:raw,output:out,members:true,animation:Some(if limestone{1309}else{10937})},level,xp_tenths:xp,ticks:0,name:"independent",action:"craft",failure:if limestone{Some(Failure{output:968,xp_tenths:15,low:137,high:434,message:"failure"})}else{None}};
 w.held_use_handlers.register(Scope::Types,tool,raw,Handler::ToolCraftingImmediate(r)).unwrap();w.drain_events();
 if death==1{w.data_mut(a).unwrap().stats.set_current(3,0);}else if death==2{w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:20},1).unwrap();w.tick().unwrap();assert_eq!(w.data(a).unwrap().stats.current(3),0);assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(_))));w.data_mut(a).unwrap().stats.set_current(3,10);}
 let q=if reverse{Request{selected_slot:17,selected_id:raw,target_slot:0,target_id:tool}}else{Request{selected_slot:0,selected_id:tool,target_slot:17,target_id:raw}};let before=w.data(a).unwrap().clone();let clock=w.clock.0;
 let prepared=w.prepare_held_use(a,q);if death==0{assert!(prepared.unwrap().is_some());w.finish_held_use(a,q).unwrap();}else if prepared.is_ok(){let _=w.finish_held_use(a,q);}
 assert_eq!(w.clock.0,clock);let immediate=w.data(a).unwrap().clone();println!("PG_IMMEDIATE_LIFE limestone={limestone} reverse={reverse} death={death} output={:?} fine_delta={} inventory_unchanged={}",immediate.inventory.slots()[17],immediate.stats.fine_xp(12)-before.stats.fine_xp(12),immediate.inventory==before.inventory);
 for _ in 0..8{w.tick().unwrap();}let after=w.data(a).unwrap();assert_eq!(after.inventory,immediate.inventory,"delayed inventory replay");assert_eq!(after.stats,immediate.stats,"delayed stats replay");
 if death>0{assert_eq!(immediate.inventory,before.inventory,"life refusal must conserve inventory");assert_eq!(immediate.stats,before.stats,"life refusal must conserve Stats");}else{let item=immediate.inventory.slots()[17].unwrap();assert!(item.id==out||(limestone&&item.id==968));let mut expected=before.inventory.clone();expected.restore_slot(17,Some(item)).unwrap();assert_eq!(immediate.inventory,expected);let mut stats=before.stats.clone();let award=if !limestone{0}else if item.id==968{75}else{300};stats.set_fine_xp(12,before.stats.fine_xp(12)+award).unwrap();assert_eq!(immediate.stats,stats);}
}
#[test]fn alive_controls(){for limestone in[false,true]{for reverse in[false,true]{scenario(limestone,reverse,0);}}}
macro_rules! refusal{($name:ident,$l:expr,$r:expr,$d:expr)=>{#[test]fn $name(){scenario($l,$r,$d);}};}
refusal!(staff_zero_hp_forward,false,false,1);
refusal!(staff_zero_hp_reverse,false,true,1);
refusal!(staff_queued_death_forward,false,false,2);
refusal!(staff_queued_death_reverse,false,true,2);
refusal!(limestone_zero_hp_forward,true,false,1);
refusal!(limestone_zero_hp_reverse,true,true,1);
refusal!(limestone_queued_death_forward,true,false,2);
refusal!(limestone_queued_death_reverse,true,true,2);
