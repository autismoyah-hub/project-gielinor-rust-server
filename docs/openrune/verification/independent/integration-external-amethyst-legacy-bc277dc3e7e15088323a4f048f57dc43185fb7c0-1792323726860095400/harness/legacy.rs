//! New integration regression at exact71; no historical63 acceptance inherited.
use gielinor_world::{grinding,tool_crafting::{Recipe,Request},held_use::{Handler,Scope,Type,Request as Held},inventory::Item,simulation::{World,PlayerData,ActorId,Action},stats::Stats,CoordGrid};
use std::sync::Arc;
const GEMS:[(i32,i32,u8,u16,bool);7]=[(1623,1607,20,500,false),(1621,1605,27,675,false),(1619,1603,34,850,false),(1617,1601,43,1075,false),(1631,1615,55,1375,true),(6571,6573,67,1675,true),(19496,19493,89,500,true)];
fn profile(i:usize)->(i32,i32,u8,u16,bool,usize,usize,u16){if i<7{let(r,o,l,x,m)=GEMS[i];(r,o,l,x,m,1,2,8833)}else{(3345+2*(i-7) as i32,3327+2*(i-7) as i32,15,325,true,2,3,1280)}}
fn fixture(index:usize,reverse:bool)->(World,ActorId,Request){
    let(raw,output,level,xp_tenths,members,first,cycle,anim)=profile(index);let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
    let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();let mut data=PlayerData::new(1,"Independent production upgrade".into(),pos,stats);
    for s in 0..28{data.inventory.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}for s in [0,1]{data.inventory.restore_slot(s,Some(Item{id:1755,count:1,vars:0})).unwrap();}for s in [3,9,17,26]{data.inventory.restore_slot(s,Some(Item{id:raw,count:1,vars:0})).unwrap();}
    w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();
    let recipes=Arc::new(vec![Recipe{exchange:grinding::Recipe{tool:1755,input:raw,output,members,animation:Some(anim)},level,xp_tenths,ticks:first as u8,name:"independent product",action:if index<7{"cut"}else{"carve"},output_count:1,failure:None}]);
    for id in [1755,raw]{w.held_use_types.insert(id,Type{id,content:-1});}w.held_use_handlers.register(Scope::Types,1755,raw,Handler::ToolCraftingChoices{recipes,cycle_ticks:cycle as u8}).unwrap();
    let held=if reverse{Held{selected_slot:17,selected_id:raw,target_slot:0,target_id:1755}}else{Held{selected_slot:0,selected_id:1755,target_slot:17,target_id:raw}};assert!(w.prepare_held_use(a,held).unwrap().is_some());w.finish_held_use(a,held).unwrap();let req=w.take_tool_crafting_request(a).unwrap().unwrap();assert_eq!(req.available(&w.data(a).unwrap().inventory),4);assert_eq!((req.tool_slot,req.input_slot),(0,17));(w,a,req)
}
fn assert_outputs(w:&World,a:ActorId,initial:&PlayerData,index:usize,done:usize,rate:i32){
    let(_,output,_,xp,_,_,_,_)=profile(index);let mut inv=initial.inventory.clone();for slot in [17,3,9,26].into_iter().take(done){inv.restore_slot(slot,Some(Item{id:output,count:1,vars:0})).unwrap();}assert_eq!(w.data(a).unwrap().inventory,inv);
    let mut stats=initial.stats.clone();stats.set_fine_xp(12,130344311+done as i32*(xp as i32*rate/100)).unwrap();assert_eq!(w.data(a).unwrap().stats,stats);
}
#[test]
fn all_sixteen_both_directions_three_rates_quantities_selected_order_and_exact_cadence(){
    let mut cases=0;for index in 0..16{for reverse in [false,true]{for rate in [100,125,500]{for count in [1,2,4,5,10,28]{
        let(mut w,a,req)=fixture(index,reverse);w.data_mut(a).unwrap().xp_rate_hundreds=rate;let initial=w.data(a).unwrap().clone();w.start_tool_crafting_choice(a,req,0,count).unwrap();let(_,_,_,_,_,first,cycle,_)=profile(index);
        for step in 1..=16{w.tick().unwrap();let done=if step<first+1{0}else{(1+(step-first-1)/cycle).min((count as usize).min(4))};assert_outputs(&w,a,&initial,index,done,rate);}cases+=1;
    }}}}assert_eq!(cases,576);
}
#[test]
fn f2p_gems_remain_available_to_nonmembers_in_quantity_route(){
    for index in 0..4{for reverse in [false,true]{for count in [1,5,28]{let(mut w,a,req)=fixture(index,reverse);w.set_membership(a,false).unwrap();w.data_mut(a).unwrap().xp_rate_hundreds=125;let initial=w.data(a).unwrap().clone();w.start_tool_crafting_choice(a,req,0,count).unwrap();for _ in 0..16{w.tick().unwrap();}assert_outputs(&w,a,&initial,index,(count as usize).min(4),125);}}}
}
#[test]
fn invalid_admission_and_late_failure_cancel_captured_work_without_fallback(){
    let mut initial_cases=0;let mut late_cases=0;
    for index in 0..16{let(raw,output,level,xp,members,first,_,_)=profile(index);
        for mode in 0..9{if mode==3 && !members{continue;}let(mut w,a,req)=fixture(index,mode%2==1);
            match mode{3=>w.set_membership(a,false).unwrap(),4=>w.data_mut(a).unwrap().stats.set_current(12,level as i8-1),5=>w.data_mut(a).unwrap().xp_rate_hundreds=0,6=>{w.data_mut(a).unwrap().inventory.restore_slot(17,Some(Item{id:raw,count:1,vars:1})).unwrap();},7=>{w.data_mut(a).unwrap().inventory.restore_slot(0,None).unwrap();},8=>{w.object_types.dummy.insert(output);},_=>{}}
            let before=w.data(a).unwrap().clone();let result=w.start_tool_crafting_choice(a,req,if mode==0{1}else{0},if mode==1{0}else if mode==2{29}else{1});if mode<=2{assert!(result.is_err());}else{result.unwrap();}for _ in 0..8{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);initial_cases+=1;
        }
        for mode in 0..12{let(mut w,a,req)=fixture(index,mode%2==0);w.data_mut(a).unwrap().xp_rate_hundreds=125;let initial=w.data(a).unwrap().clone();w.start_tool_crafting_choice(a,req,0,28).unwrap();for _ in 0..first+1{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory.get(17).unwrap().id,output);assert_eq!(w.data(a).unwrap().stats.fine_xp(12),130344311+xp as i32*125/100);
            match mode{0=>{w.data_mut(a).unwrap().inventory.restore_slot(3,Some(Item{id:1931,count:1,vars:0})).unwrap();w.data_mut(a).unwrap().inventory.restore_slot(25,Some(Item{id:raw,count:1,vars:0})).unwrap();},1=>{w.data_mut(a).unwrap().inventory.restore_slot(3,Some(Item{id:raw,count:1,vars:1})).unwrap();},2=>{w.data_mut(a).unwrap().inventory.restore_slot(0,None).unwrap();},3=>w.set_membership(a,false).unwrap(),4=>w.data_mut(a).unwrap().stats.set_current(12,level as i8-1),5=>w.data_mut(a).unwrap().xp_rate_hundreds=0,6=>{w.object_types.dummy.insert(output);},7=>{w.set_modal_open(a,true).unwrap();w.set_modal_open(a,false).unwrap();},8=>w.submit(a,Action::Walk{destination:CoordGrid::new(3201,3200,0).unwrap(),run:false}).unwrap(),9=>w.request_logout(a).unwrap(),10=>{w.close_dialogs(a).unwrap();},_=>w.submit(a,Action::EatItemSlot{slot:2,item:1931}).unwrap()}
            let before=w.data(a).unwrap().clone();for _ in 0..14{w.tick().unwrap();}if mode==3 && !members{assert_outputs(&w,a,&initial,index,4,125);}else{assert_eq!(w.data(a).unwrap().inventory,before.inventory,"index{index} mode{mode}");assert_eq!(w.data(a).unwrap().stats,before.stats);}late_cases+=1;
        }
    }assert_eq!((initial_cases,late_cases),(140,192));
}
