//! Independent zero-reward and immediate/fallible transaction obligations for65.
use gielinor_world::{grinding,tool_crafting::{Job,Recipe,Failure},inventory::{Inventory,Catalog,Item,StackType},stats::Stats,npc::RandomSource,simulation::{World,PlayerData,WorldEvent},held_use::{Handler,Scope,Type,Request},CoordGrid};
struct Draw{value:u32,calls:usize}impl RandomSource for Draw{fn below(&mut self,n:u32)->u32{assert_eq!(n,256);self.calls+=1;self.value}}
fn fixture(limestone:bool,slot:usize)->(Job,Inventory,Catalog,Stats){
    let(tool,raw,output,level,xp,anim)=if limestone{(1755,3211,3420,12,60,1309)}else{(946,771,772,31,0,10937)};let mut inv=Inventory::new(28,StackType::Normal,false);
    for s in 0..28{inv.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}for s in [0,1]{inv.restore_slot(s,Some(Item{id:tool,count:1,vars:0})).unwrap();}for s in [slot,if slot==4{9}else{4}]{inv.restore_slot(s,Some(Item{id:raw,count:1,vars:0})).unwrap();}
    let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();
    (Job{recipe:Recipe{exchange:grinding::Recipe{tool,input:raw,output,members:true,animation:Some(anim)},level,xp_tenths:xp,ticks:0,name:"independent immediate",action:"craft",failure:if limestone{Some(Failure{output:968,xp_tenths:15,low:137,high:434,message:"Independent failure"})}else{None}},tool_slot:0,input_slot:slot},inv,Catalog::default(),stats)
}
fn threshold(level:i32)->u32{((137*98+297*(level.clamp(1,99)-1)+49)/98+1).clamp(0,256) as u32}
#[test]
fn all_byte_rolls_levels_membership_rates_match_integer_oracle_and_optional_advancement(){
    assert_eq!(threshold(12),171);assert!(threshold(39)<256);assert_eq!(threshold(40),256);let mut cases=0;
    for limestone in [false,true]{for level in (1..=99).chain([100,120]){for member in [false,true]{for rate in [0,100,125,500]{for roll in 0..(if limestone{256}else{1}){
        let(job,inv,cat,mut stats)=fixture(limestone,17);stats.set_current(12,level as i8);let before=(inv.clone(),stats.clone());let mut draw=Draw{value:roll,calls:0};let result=job.resolve(&inv,&cat,&stats,rate,member,&mut draw);
        if !member || level<job.recipe.level as i32 || (limestone && rate==0){assert!(result.is_err());assert_eq!(draw.calls,0);}else{
            let(p,success)=result.unwrap();let good=!limestone || roll<threshold(level);assert_eq!(success,good);assert_eq!(draw.calls,usize::from(limestone));
            let out=if good{job.recipe.exchange.output}else{968};let xp=if !limestone{0}else if good{60}else{15};let mut expected=inv.clone();expected.restore_slot(17,Some(Item{id:out,count:1,vars:0})).unwrap();assert_eq!(p.inventory,expected);let mut expected_stats=stats.clone();expected_stats.set_fine_xp(12,130344311+xp*rate/100).unwrap();assert_eq!(p.stats,expected_stats);assert_eq!(p.advancement.is_some(),xp>0);
        }assert_eq!((inv,stats),before);cases+=1;
    }}}}}assert_eq!(cases,207656);
}
#[test]
fn every_invalid_possible_credit_input_tool_and_policy_refuses_before_rng(){
    for limestone in [false,true]{for mode in 0..15{
        let(mut job,mut inv,mut cat,mut stats)=fixture(limestone,17);let mut rate=125;let mut member=true;
        match mode{0=>{cat.dummy.insert(job.recipe.exchange.output);},1=>{if limestone{cat.dummy.insert(968);}else{job.recipe.exchange.output=771;}},2=>{inv.restore_slot(17,Some(Item{id:job.recipe.exchange.input,count:1,vars:1})).unwrap();},3=>{inv.restore_slot(0,Some(Item{id:job.recipe.exchange.tool,count:1,vars:1})).unwrap();},4=>{inv.restore_slot(17,None).unwrap();},5=>{inv.restore_slot(0,None).unwrap();},6=>job.input_slot=0,7=>member=false,8=>stats.set_current(12,job.recipe.level as i8-1),9=>{if limestone{rate=0;}else{job.recipe.level=0;}},10=>{if limestone{rate=-1;}else{job.recipe.level=100;}},11=>inv.stack=StackType::Always,12=>inv.placeholders=true,13=>{inv.restore_slot(0,Some(Item{id:job.recipe.exchange.tool,count:2,vars:0})).unwrap();},_=>{inv.restore_slot(17,Some(Item{id:job.recipe.exchange.input,count:2,vars:0})).unwrap();}}
        let before=(inv.clone(),stats.clone());for roll in [0,255]{let mut draw=Draw{value:roll,calls:0};assert!(job.resolve(&inv,&cat,&stats,rate,member,&mut draw).is_err(),"limestone{limestone} mode{mode}");assert_eq!(draw.calls,0);assert_eq!((inv.clone(),stats.clone()),before);}
    }}
    let(mut job,inv,cat,mut stats)=fixture(true,17);stats.set_current(12,12);job.recipe.failure.as_mut().unwrap().xp_tenths=0;let mut draw=Draw{value:255,calls:0};let(p,success)=job.resolve(&inv,&cat,&stats,125,true,&mut draw).unwrap();assert!(!success);assert_eq!(draw.calls,1);assert_eq!(p.inventory.get(17).unwrap().id,968);assert_eq!(p.stats,stats);assert!(p.advancement.is_none());let mut draw=Draw{value:255,calls:0};assert!(job.resolve(&inv,&cat,&stats,0,true,&mut draw).is_err());assert_eq!(draw.calls,0);
}
#[test]
fn actual_held_immediate_routes_commit_before_tick_with_no_menu_or_delayed_replay(){
    let mut cases=0;for limestone in [false,true]{for slot in [2,4,17,26]{for reverse in [false,true]{for rate in if limestone{[100,125,500]}else{[0,125,500]}{
        let(job,inv,_,stats)=fixture(limestone,slot);let mut w=World::default();w.collision.allocate(3200,3200,0);let mut data=PlayerData::new(1,"Independent immediate".into(),CoordGrid::new(3200,3200,0).unwrap(),stats);data.inventory=inv;data.xp_rate_hundreds=rate;w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();
        let r=job.recipe;for id in [r.exchange.tool,r.exchange.input]{w.held_use_types.insert(id,Type{id,content:-1});}w.held_use_handlers.register(Scope::Types,r.exchange.tool,r.exchange.input,Handler::ToolCraftingImmediate(r)).unwrap();
        let req=if reverse{Request{selected_slot:slot as i32,selected_id:r.exchange.input,target_slot:0,target_id:r.exchange.tool}}else{Request{selected_slot:0,selected_id:r.exchange.tool,target_slot:slot as i32,target_id:r.exchange.input}};let before=w.data(a).unwrap().clone();let clock=w.clock.0;assert!(w.prepare_held_use(a,req).unwrap().is_some());w.finish_held_use(a,req).unwrap();assert_eq!(w.clock.0,clock);
        let mut expected=before.inventory.clone();expected.restore_slot(slot,Some(Item{id:r.exchange.output,count:1,vars:0})).unwrap();assert_eq!(w.data(a).unwrap().inventory,expected);let mut stats=before.stats.clone();stats.set_fine_xp(12,130344311+if limestone{60*rate/100}else{0}).unwrap();assert_eq!(w.data(a).unwrap().stats,stats);assert!(w.take_tool_crafting_request(a).unwrap().is_none());assert_eq!(w.drain_events().iter().filter(|e|matches!(e,WorldEvent::SkillAdvanced{..})).count(),usize::from(limestone));
        assert!(w.finish_held_use(a,req).is_err());for _ in 0..6{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,expected);assert_eq!(w.data(a).unwrap().stats,stats);cases+=1;
    }}}}assert_eq!(cases,48);
}
