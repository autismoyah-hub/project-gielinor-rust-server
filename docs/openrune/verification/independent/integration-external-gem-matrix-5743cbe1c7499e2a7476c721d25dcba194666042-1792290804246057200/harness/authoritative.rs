//! Independently specified gem profiles; no Herblore scenarios are executed.
use gielinor_world::{grinding,tool_crafting::{Job,Recipe},inventory::{Inventory,Catalog,Item,StackType},stats::Stats};
const PROFILES:[(i32,i32,u8,u16,bool,&str);7]=[(1623,1607,20,500,false,"sapphire"),(1621,1605,27,675,false,"emerald"),(1619,1603,34,850,false,"ruby"),(1617,1601,43,1075,false,"diamond"),(1631,1615,55,1375,true,"dragonstone"),(6571,6573,67,1675,true,"onyx"),(19496,19493,89,500,true,"zenyte")];
fn fixture(index:usize,slot:usize)->(Job,Inventory,Catalog,Stats){
    let (raw,cut,level,xp,members,name)=PROFILES[index];let tool=(slot+7)%28;
    let mut inv=Inventory::new(28,StackType::Normal,false);
    for s in 0..28{inv.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}
    for s in [slot,(slot+21)%28]{inv.restore_slot(s,Some(Item{id:raw,count:1,vars:0})).unwrap();}
    for s in [tool,(slot+14)%28]{inv.restore_slot(s,Some(Item{id:1755,count:1,vars:0})).unwrap();}
    let mut stats=Stats::default();stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();
    (Job{recipe:Recipe{exchange:grinding::Recipe{tool:1755,input:raw,output:cut,members,animation:Some(8833)},level,xp_tenths:xp,ticks:1,name},tool_slot:tool,input_slot:slot},inv,Catalog::default(),stats)
}
#[test]
fn all_seven_gems_all_selected_slots_live_level_membership_and_three_rates(){
    let mut cases=0;
    for index in 0..7{for slot in 0..28{for rate in [100,125,500]{for member in [false,true]{for level in [PROFILES[index].2-1,PROFILES[index].2,99]{
        let (job,inv,cat,mut stats)=fixture(index,slot);stats.set_current(12,level as i8);let before=(inv.clone(),stats.clone());
        let outcome=job.prepare(&inv,&cat,&stats,rate,member);let good=level>=job.recipe.level && (!job.recipe.exchange.members || member);
        if good{let p=outcome.unwrap();let mut expected=inv.clone();expected.restore_slot(slot,Some(Item{id:PROFILES[index].1,count:1,vars:0})).unwrap();
            let mut expected_stats=stats.clone();expected_stats.set_fine_xp(12,130344311+(job.recipe.xp_tenths as i32*rate/100)).unwrap();assert_eq!(p.inventory,expected);assert_eq!(p.stats,expected_stats);
        }else{assert!(outcome.is_err(),"profile{index} slot{slot} member{member} level{level}");}
        assert_eq!((inv,stats),before);cases+=1;
    }}}}}
    assert_eq!(cases,3528);
}
#[test]
fn invalid_gem_job_never_returns_a_committable_reward_or_replaces_duplicate_inputs(){
    for index in 0..7{for mode in 0..18{
        let (mut job,mut inv,mut cat,mut stats)=fixture(index,27);let mut rate=125;
        match mode{
            0=>{inv.restore_slot(job.input_slot,None).unwrap();},1=>{inv.restore_slot(job.tool_slot,None).unwrap();},
            2=>{inv.restore_slot(job.input_slot,Some(Item{id:job.recipe.exchange.input,count:1,vars:1})).unwrap();},
            3=>{inv.restore_slot(job.tool_slot,Some(Item{id:1755,count:1,vars:1})).unwrap();},
            4=>{inv.restore_slot(job.input_slot,Some(Item{id:job.recipe.exchange.input,count:2,vars:0})).unwrap();},
            5=>{inv.restore_slot(job.tool_slot,Some(Item{id:1755,count:2,vars:0})).unwrap();},
            6=>job.tool_slot=job.input_slot,7=>rate=0,8=>rate=-1,9=>stats.set_current(12,job.recipe.level as i8-1),
            10=>job.recipe.level=0,11=>job.recipe.ticks=0,12=>job.recipe.xp_tenths=0,
            13=>{cat.dummy.insert(job.recipe.exchange.output);},14=>inv.stack=StackType::Always,15=>inv.placeholders=true,
            16=>job.recipe.exchange.output=job.recipe.exchange.input,_=>job.recipe.exchange.output=1755,
        }
        let before=(inv.clone(),stats.clone());assert!(job.prepare(&inv,&cat,&stats,rate,true).is_err(),"profile{index} mode{mode}");assert_eq!((inv,stats),before);
    }}
}
#[test]
fn all_seven_actual_world_held_routes_both_directions_recheck_late_state_and_cancel(){
    use gielinor_world::{held_use::{Handler,Request,Scope,Type},simulation::{World,PlayerData,Action},CoordGrid};
    let mut cases=0;
    for index in 0..7{for reverse in [false,true]{for mode in 0..11{
        let (job,inv,_,mut stats)=fixture(index,27);stats.set_base(3,10);stats.set_current(3,10);
        let mut data=PlayerData::new(1,"Independent gem route".into(),CoordGrid::new(3200,3200,0).unwrap(),stats);data.inventory=inv;
        let mut w=World::default();w.collision.allocate(3200,3200,0);w.queue_login(1,data).unwrap();w.tick().unwrap();let actor=w.actors().next().unwrap();
        w.set_membership(actor,true).unwrap();w.data_mut(actor).unwrap().xp_rate_hundreds=125;
        for id in [1755,job.recipe.exchange.input]{w.held_use_types.insert(id,Type{id,content:-1});}
        w.held_use_handlers.register(Scope::Types,1755,job.recipe.exchange.input,Handler::ToolCrafting(job.recipe)).unwrap();
        let request=if reverse{Request{selected_slot:27,selected_id:job.recipe.exchange.input,target_slot:job.tool_slot as i32,target_id:1755}}else{Request{selected_slot:job.tool_slot as i32,selected_id:1755,target_slot:27,target_id:job.recipe.exchange.input}};
        assert!(w.prepare_held_use(actor,request).unwrap().is_some());w.finish_held_use(actor,request).unwrap();
        w.tick().unwrap();assert_eq!(w.data(actor).unwrap().inventory.get(27).unwrap().id,job.recipe.exchange.input);assert_eq!(w.data(actor).unwrap().stats.fine_xp(12),130344311);
        match mode{
            0=>{},1=>{w.data_mut(actor).unwrap().inventory.restore_slot(27,None).unwrap();},
            2=>{w.data_mut(actor).unwrap().inventory.restore_slot(job.tool_slot,None).unwrap();},
            3=>{w.data_mut(actor).unwrap().inventory.restore_slot(27,Some(Item{id:job.recipe.exchange.input,count:1,vars:1})).unwrap();},
            4=>{w.data_mut(actor).unwrap().inventory.restore_slot(job.tool_slot,Some(Item{id:1755,count:1,vars:1})).unwrap();},
            5=>w.data_mut(actor).unwrap().stats.set_current(12,job.recipe.level as i8-1),
            6=>w.set_membership(actor,false).unwrap(),
            7=>{w.set_modal_open(actor,true).unwrap();w.set_modal_open(actor,false).unwrap();},
            8=>{w.object_types.dummy.insert(job.recipe.exchange.output);},
            9=>w.data_mut(actor).unwrap().xp_rate_hundreds=0,
            _=>w.submit(actor,Action::EatItemSlot{slot:2,item:1931}).unwrap(),
        }
        let before=w.data(actor).unwrap().clone();w.tick().unwrap();let good=mode==0 || (mode==6 && !job.recipe.exchange.members);
        let expected_fine=130344311+if good{job.recipe.xp_tenths as i32*125/100}else{0};
        let mut expected=before.inventory.clone();if good{expected.restore_slot(27,Some(Item{id:job.recipe.exchange.output,count:1,vars:0})).unwrap();}
        assert_eq!(w.data(actor).unwrap().inventory.slots(),expected.slots(),"profile{index} reverse{reverse} mode{mode}");assert_eq!(w.data(actor).unwrap().stats.fine_xp(12),expected_fine,"profile{index} mode{mode}");
        assert_eq!(w.data(actor).unwrap().inventory.get(20).unwrap().id,job.recipe.exchange.input);assert_eq!(w.data(actor).unwrap().inventory.get(13).unwrap().id,1755);
        for _ in 0..3{w.tick().unwrap();}assert_eq!(w.data(actor).unwrap().inventory.slots(),expected.slots());assert_eq!(w.data(actor).unwrap().stats.fine_xp(12),expected_fine);
        cases+=1;
    }}}
    assert_eq!(cases,154);
}
