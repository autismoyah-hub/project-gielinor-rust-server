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
fn eight_choices_both_directions_every_count_three_rates_exact_captured_order_and_2_then_3_clocks(){
    let mut cases=0;
    for index in 0..8{for reverse in [false,true]{for rate in [100,125,500]{for count in 1..=28{
        let (mut w,a,req)=fixture(reverse);w.data_mut(a).unwrap().xp_rate_hundreds=rate;let initial=w.data(a).unwrap().clone();w.start_tool_crafting_choice(a,req,index,count).unwrap();
        let cap=(count as usize).min(4);let award=PROFILES[index].2 as i32*rate/100;
        for step in 1..=16{
            w.tick().unwrap();let done=if step<3{0}else{(1+(step-3)/3).min(cap)};let mut expected=initial.inventory.clone();
            for slot in [17,3,9,26].into_iter().take(done){expected.restore_slot(slot,Some(Item{id:PROFILES[index].0,count:1,vars:0})).unwrap();}
            assert_eq!(w.data(a).unwrap().inventory,expected,"index{index} reverse{reverse} rate{rate} count{count} step{step}");
            let mut stats=initial.stats.clone();stats.set_fine_xp(12,130344311+done as i32*award).unwrap();assert_eq!(w.data(a).unwrap().stats,stats);
        }
        cases+=1;
    }}}}
    assert_eq!(cases,1344);
}
#[test]
fn invalid_selection_count_and_live_admission_never_award(){
    for index in 0..8{for mode in 0..9{
        let (mut w,a,req)=fixture(mode%2==1);
        match mode{
            3=>w.set_membership(a,false).unwrap(),4=>w.data_mut(a).unwrap().stats.set_current(12,PROFILES[index].1 as i8-1),
            5=>w.data_mut(a).unwrap().xp_rate_hundreds=0,
            6=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:1775,count:1,vars:1})).unwrap();},
            7=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1785,count:1,vars:1})).unwrap();},
            8=>{w.object_types.dummy.insert(PROFILES[index].0);},_=>{}
        }
        let before=w.data(a).unwrap().clone();let result=w.start_tool_crafting_choice(a,req,if mode==0{8}else{index},if mode==1{0}else if mode==2{29}else{1});
        if mode<=2{assert!(result.is_err());}else{result.unwrap();}
        for _ in 0..8{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before.inventory,"index{index} mode{mode}");assert_eq!(w.data(a).unwrap().stats,before.stats);
    }}
}
#[test]
fn late_captured_slot_and_lifecycle_failures_keep_completed_outputs_no_fallback_or_future_loss(){
    for index in 0..8{for mode in 0..12{
        let (mut w,a,req)=fixture(mode%2==0);w.data_mut(a).unwrap().xp_rate_hundreds=125;w.start_tool_crafting_choice(a,req,index,28).unwrap();
        for _ in 0..3{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory.get(17).unwrap().id,PROFILES[index].0);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12),130344311+PROFILES[index].2 as i32*125/100);
        match mode{
            0=>{w.data_mut(a).unwrap().inventory.restore_slot(3,Some(Item{id:1931,count:1,vars:0})).unwrap();w.data_mut(a).unwrap().inventory.restore_slot(25,Some(Item{id:1775,count:1,vars:0})).unwrap();},
            1=>{w.data_mut(a).unwrap().inventory.restore_slot(3,Some(Item{id:1775,count:1,vars:1})).unwrap();},
            2=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1785,count:1,vars:1})).unwrap();},
            3=>w.set_membership(a,false).unwrap(),4=>w.data_mut(a).unwrap().stats.set_current(12,PROFILES[index].1 as i8-1),
            5=>w.data_mut(a).unwrap().xp_rate_hundreds=0,6=>{w.object_types.dummy.insert(PROFILES[index].0);},
            7=>{w.set_modal_open(a,true).unwrap();w.set_modal_open(a,false).unwrap();},
            8=>w.submit(a,Action::Walk{destination:CoordGrid::new(3201,3200,0).unwrap(),run:false}).unwrap(),
            9=>w.request_logout(a).unwrap(),10=>{w.close_dialogs(a).unwrap();},
            _=>w.submit(a,Action::EatItemSlot{slot:2,item:1931}).unwrap(),
        }
        let before=w.data(a).unwrap().clone();for _ in 0..12{w.tick().unwrap();}
        assert_eq!(w.data(a).unwrap().inventory,before.inventory,"index{index} mode{mode}");assert_eq!(w.data(a).unwrap().stats,before.stats);
    }}
}
