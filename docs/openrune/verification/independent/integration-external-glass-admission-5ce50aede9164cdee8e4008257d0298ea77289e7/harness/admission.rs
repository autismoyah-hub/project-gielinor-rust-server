//! Independent eight-product choices, captured slots, exact fine XP and cadence.
use gielinor_world::{grinding,tool_crafting::{Recipe,Request},held_use::{Handler,Scope,Type,Request as Held},inventory::Item,simulation::{World,PlayerData,ActorId,Action},stats::Stats,CoordGrid};
use std::sync::Arc;
const PROFILES:[(i32,u8,u16);8]=[(1919,1,175),(4527,4,190),(4525,12,250),(229,33,350),(6667,42,425),(567,46,525),(4542,49,550),(10980,87,700)];
fn fixture(reverse:bool)->(World,ActorId,Request){
    let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
    let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();
    let mut data=PlayerData::new(1,"Independent glass".into(),pos,stats);
    for slot in 0..28{data.inventory.restore_slot(slot,Some(Item{id:1931,count:1,vars:0})).unwrap();}
    for slot in [0,1]{data.inventory.restore_slot(slot,Some(Item{id:1785,count:1,vars:0})).unwrap();}
    for slot in [3,9,17,26]{data.inventory.restore_slot(slot,Some(Item{id:1775,count:1,vars:0})).unwrap();}
    w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();
    let recipes=Arc::new(PROFILES.into_iter().map(|(output,level,xp_tenths)|Recipe{exchange:grinding::Recipe{tool:1785,input:1775,output,members:true,animation:Some(884)},level,xp_tenths,ticks:2,name:"independent product",action:"blow"}).collect());
    for id in [1785,1775]{w.held_use_types.insert(id,Type{id,content:-1});}
    w.held_use_handlers.register(Scope::Types,1785,1775,Handler::ToolCraftingChoices{recipes,cycle_ticks:3}).unwrap();
    let held=if reverse{Held{selected_slot:17,selected_id:1775,target_slot:0,target_id:1785}}else{Held{selected_slot:0,selected_id:1785,target_slot:17,target_id:1775}};
    assert!(w.prepare_held_use(a,held).unwrap().is_some());w.finish_held_use(a,held).unwrap();let req=w.take_tool_crafting_request(a).unwrap().unwrap();assert_eq!(req.available(&w.data(a).unwrap().inventory),4);assert_eq!(req.input_slot,17);assert_eq!(req.tool_slot,0);(w,a,req)
}
#[test]
fn sixteen_additional_captured_admission_failures_for_each_glass_profile(){
    let mut cases=0;
    for index in 0..8{for mode in 0..16{
        let (mut w,a,mut req)=fixture(mode%2==1);
        match mode{
            0=>{w.data_mut(a).unwrap().inventory.restore_slot(17,None).unwrap();},
            1=>{w.data_mut(a).unwrap().inventory.restore_slot(0,None).unwrap();},
            2=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:1775,count:2,vars:0})).unwrap();},
            3=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1785,count:2,vars:0})).unwrap();},
            4=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:1776,count:1,vars:0})).unwrap();},
            5=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1786,count:1,vars:0})).unwrap();},
            6=>{w.object_types.notes.insert(PROFILES[index].0,gielinor_world::inventory::Template{link:1,template:799});},
            7=>{w.object_types.placeholders.insert(1775,gielinor_world::inventory::Template{link:1,template:14401});},
            8=>w.data_mut(a).unwrap().inventory.stack=gielinor_world::inventory::StackType::Always,
            9=>w.data_mut(a).unwrap().inventory.placeholders=true,
            10=>w.set_modal_open(a,true).unwrap(),11=>w.request_logout(a).unwrap(),
            12=>req.tool_slot=req.input_slot,13=>req.origin=CoordGrid::new(3201,3200,0).unwrap(),14=>req.cycle_ticks=0,_=>{}
        }
        let before=w.data(a).unwrap().clone();let result=w.start_tool_crafting_choice(a,req,if mode==15{8}else{index},1);
        if matches!(mode,10|11|13|14|15){assert!(result.is_err());}else{result.unwrap();}
        for _ in 0..8{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before.inventory,"index{index} mode{mode}");assert_eq!(w.data(a).unwrap().stats,before.stats);cases+=1;
    }}
    assert_eq!(cases,128);
}
