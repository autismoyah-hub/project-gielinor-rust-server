//! Independent counted-credit and per-raw XP oracle at the amethyst contract SHA.
use gielinor_world::{grinding,tool_crafting::{Job,Recipe},inventory::{Inventory,Catalog,Item,StackType},stats::Stats};
const PROFILES:[(i32,u16,u8);4]=[(21338,15,83),(21350,15,85),(21352,5,87),(25853,8,89)];
fn fixture(raw:usize,output:i32,count:u16,level:u8)->(Job,Inventory,Catalog,Stats){
    let mut inv=Inventory::new(28,StackType::Normal,false);
    for s in 0..28{inv.restore_slot(s,Some(Item{id:1931,count:1,vars:0})).unwrap();}
    inv.restore_slot(0,Some(Item{id:1755,count:1,vars:0})).unwrap();
    for s in [raw,raw%27+1]{inv.restore_slot(s,Some(Item{id:21347,count:1,vars:0})).unwrap();}
    let mut catalog=Catalog::default();catalog.stackable.insert(output);
    let mut stats=Stats::default();stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();
    (Job{recipe:Recipe{exchange:grinding::Recipe{tool:1755,input:21347,output,members:true,animation:Some(6295)},level,xp_tenths:600,output_count:count,ticks:1,name:"amethyst",action:"cut",failure:None},tool_slot:0,input_slot:raw},inv,catalog,stats)
}
#[test]
fn full_bag_every_selected_slot_checked_stack_credit_and_once_per_raw_xp(){
    let mut cases=0;
    for (output,count,required) in PROFILES{for raw in 1..28{for rate in [100,125,500]{for level in [required-1,required,99]{for member in [false,true]{for mode in 0..4{
        let (job,mut inv,cat,mut stats)=fixture(raw,output,count,required);stats.set_current(12,level as i8);
        let other=(1..28).find(|&s|s!=raw && s!=raw%27+1).unwrap();
        let existing=match mode{0=>None,1=>Some(10),2=>Some(i32::MAX-count as i32),_=>Some(i32::MAX-count as i32+1)};
        if let Some(n)=existing{inv.restore_slot(other,Some(Item{id:output,count:n,vars:0})).unwrap();}
        let before=(inv.clone(),stats.clone());let p=job.prepare(&inv,&cat,&stats,rate,member);
        if !member || level<required || mode==3{assert!(p.is_err());}else{
            let p=p.unwrap();let mut expected=inv.clone();expected.restore_slot(raw,None).unwrap();
            expected.restore_slot(if existing.is_some(){other}else{raw},Some(Item{id:output,count:existing.unwrap_or(0)+count as i32,vars:0})).unwrap();
            assert_eq!(p.inventory,expected);let mut expected_stats=stats.clone();expected_stats.set_fine_xp(12,130344311+600*rate/100).unwrap();assert_eq!(p.stats,expected_stats);
            assert_eq!(p.inventory.get(0),inv.get(0));assert_eq!(p.inventory.get(raw%27+1),inv.get(raw%27+1));
        }
        assert_eq!((inv,stats),before);cases+=1;
    }}}}}}assert_eq!(cases,7776);println!("PG_AMETHYST_COUNTED {cases} full-bag/selected-slot/member/level/rate/existing-stack-boundary cases; XP per raw, no publication on error");
}
#[test]
fn invalid_count_credit_tool_and_selected_state_refuse_atomically(){
    let mut cases=0;
    for (output,count,required) in PROFILES{for mode in 0..9{
        let (mut job,mut inv,mut cat,stats)=fixture(17,output,count,required);
        match mode{0=>job.recipe.output_count=0,1=>{cat.stackable.remove(&output);},2=>{cat.dummy.insert(output);},3=>{inv.restore_slot(17,Some(Item{id:21347,count:1,vars:1})).unwrap();},4=>{inv.restore_slot(0,None).unwrap();},5=>{inv.restore_slot(17,None).unwrap();},6=>job.input_slot=0,7=>{inv.restore_slot(0,Some(Item{id:1755,count:2,vars:0})).unwrap();},_=>{job.recipe.exchange.output=21347;}}
        let before=(inv.clone(),stats.clone());assert!(job.prepare(&inv,&cat,&stats,125,true).is_err());assert_eq!((inv,stats),before);cases+=1;
    }}assert_eq!(cases,36);println!("PG_AMETHYST_REFUSALS {cases} invalid counted-credit/captured-slot/tool states; authoritative resolver only, native and registered batch separate");
}
