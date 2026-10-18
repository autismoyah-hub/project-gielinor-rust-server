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
