//! Independently specified selected-pair/manual finish boundaries at OR70.
use gielinor_world::{assembly,grinding,tool_crafting,inventory::{Item,Inventory,StackType},stats::Stats,npc::RandomSource,simulation::{World,PlayerData,ActorId,WorldEvent},held_use::{Handler,Scope,Type,Request},hits::{Hit,Target,Style},CoordGrid};
struct NoDraw;impl RandomSource for NoDraw{fn below(&mut self,_:u32)->u32{panic!("manual finish drew RNG")}}
fn item(id:i32)->Option<Item>{Some(Item{id,count:1,vars:0})}
fn setup(helm:bool,slot:usize,reverse:bool,member:bool,current:i8,rate:i32)->(World,ActorId,Request){
 let(first,raw) = if helm{(1755,12927)}else{(10980,10981)};
 let mut inv=Inventory::new(28,StackType::Normal,false);for s in 0..28{inv.restore_slot(s,item(1931)).unwrap();}
 for s in [0,1]{inv.restore_slot(s,item(first)).unwrap();}for s in [slot,if slot==4{9}else{4}]{inv.restore_slot(s,item(raw)).unwrap();}inv.restore_slot(27,Some(Item{id:995,count:1000,vars:0})).unwrap();
 let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,1);stats.set_current(12,current);stats.set_fine_xp(12,0).unwrap();stats.set_current(1,0);stats.set_current(11,0);
 let mut w=World::default();w.collision.allocate(3200,3200,0);let mut d=PlayerData::new(1,"Independent manual finishing".into(),CoordGrid::new(3200,3200,0).unwrap(),stats);d.inventory=inv;d.xp_rate_hundreds=rate;w.queue_login(1,d).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,member).unwrap();
 for id in[first,raw]{w.held_use_types.insert(id,Type{id,content:-1});}
 let handler=if helm{Handler::ToolCrafting(tool_crafting::Recipe{exchange:grinding::Recipe{tool:1755,input:12927,output:12929,members:true,animation:None},level:52,xp_tenths:1200,ticks:1,name:"serpentine helm",action:"carve",failure:None})}else{Handler::AssemblyImmediate(assembly::Recipe{first:10980,second:10981,output:10973,skill:12,level:87,base_xp:104,members:true})};w.held_use_handlers.register(Scope::Types,first,raw,handler).unwrap();w.drain_events();
 let h=if reverse{Request{selected_slot:slot as i32,selected_id:raw,target_slot:0,target_id:first}}else{Request{selected_slot:0,selected_id:first,target_slot:slot as i32,target_id:raw}};(w,a,h)
}
#[test]fn selected_current_member_rate_and_timing_matrix(){let mut n=0;for helm in[false,true]{let need=if helm{52}else{87};for slot in[2,4,17,26]{for reverse in[false,true]{for member in[false,true]{for current in[need-1,need,99]{for rate in[0,100,125,500]{let(mut w,a,h)=setup(helm,slot,reverse,member,current,rate);let before=w.data(a).unwrap().clone();let good=member&&current>=need&&rate>0;let mut expected=before.inventory.clone();let mut xp=before.stats.clone();if good{if helm{expected.restore_slot(slot,item(12929)).unwrap();}else{expected.restore_slot(0,item(10973)).unwrap();expected.restore_slot(slot,None).unwrap();}xp.set_fine_xp(12,if helm{1200*rate/100}else{1040*rate/100}).unwrap();}
 w.prepare_held_use(a,h).unwrap();w.finish_held_use(a,h).unwrap();if helm{assert_eq!(w.data(a).unwrap().inventory,before.inventory);w.tick_with_random(&mut NoDraw).unwrap();assert_eq!(w.data(a).unwrap().inventory,before.inventory);w.tick_with_random(&mut NoDraw).unwrap();}assert_eq!(w.data(a).unwrap().inventory,expected,"helm{helm} current{current} member{member} rate{rate}");assert_eq!(w.data(a).unwrap().stats,xp);for _ in 0..8{w.tick_with_random(&mut NoDraw).unwrap();}assert_eq!(w.data(a).unwrap().inventory,expected);assert_eq!(w.data(a).unwrap().stats,xp);assert_eq!(w.drain_events().iter().filter(|e|matches!(e,WorldEvent::SkillAdvanced{..})).count(),usize::from(good));n+=1;
 }}}}}}assert_eq!(n,384);}
fn life(helm:bool,death:bool,reverse:bool){let(mut w,a,h)=setup(helm,17,reverse,true,99,125);if death{w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:20},1).unwrap();w.tick_with_random(&mut NoDraw).unwrap();assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(_))));assert_eq!(w.data(a).unwrap().stats.current(3),0);w.data_mut(a).unwrap().stats.set_current(3,10);}else{w.data_mut(a).unwrap().stats.set_current(3,0);}let before=w.data(a).unwrap().clone();if w.prepare_held_use(a,h).is_ok(){let _=w.finish_held_use(a,h);}for _ in 0..8{w.tick_with_random(&mut NoDraw).unwrap();}let d=w.data(a).unwrap();println!("PG_FINISH_LIFE helm={helm} death={death} reverse={reverse} inventory_unchanged={} fine_delta={}",d.inventory==before.inventory,d.stats.fine_xp(12)-before.stats.fine_xp(12));assert_eq!(d.inventory,before.inventory);assert_eq!(d.stats,before.stats);}
#[test]fn wire_zero_hp_forward(){life(false,false,false)}
#[test]fn wire_zero_hp_reverse(){life(false,false,true)}
#[test]fn wire_queued_death_forward(){life(false,true,false)}
#[test]fn wire_queued_death_reverse(){life(false,true,true)}
#[test]fn helm_zero_hp_forward(){life(true,false,false)}
#[test]fn helm_zero_hp_reverse(){life(true,false,true)}
#[test]fn helm_queued_death_forward(){life(true,true,false)}
#[test]fn helm_queued_death_reverse(){life(true,true,true)}
