//! Independent integer probability oracle and both-stage/one-draw invariants.
use gielinor_world::{grinding,tool_crafting::{Job,Recipe,Failure},inventory::{Inventory,Catalog,Item,StackType},stats::Stats,npc::RandomSource};
const PROFILES:[(i32,i32,u8,u16,u16,i32,i32,u16);3]=[(1625,1609,1,150,38,128,250,890),(1627,1611,13,200,50,100,245,891),(1629,1613,16,250,63,90,240,892)];
struct Draw{value:u32,calls:usize}
impl RandomSource for Draw{fn below(&mut self,n:u32)->u32{assert_eq!(n,256);self.calls+=1;self.value}}
fn fixture(i:usize)->(Job,Inventory,Catalog,Stats){
    let (raw,cut,level,xp,failed,low,high,animation)=PROFILES[i];let mut inv=Inventory::new(28,StackType::Normal,false);
    for slot in 0..28{inv.restore_slot(slot,Some(Item{id:1931,count:1,vars:0})).unwrap();}
    for slot in [0,1]{inv.restore_slot(slot,Some(Item{id:1755,count:1,vars:0})).unwrap();}for slot in [4,17]{inv.restore_slot(slot,Some(Item{id:raw,count:1,vars:0})).unwrap();}
    let mut stats=Stats::default();stats.set_base(12,99);stats.set_current(12,99);stats.set_fine_xp(12,130344311).unwrap();
    let job=Job{recipe:Recipe{exchange:grinding::Recipe{tool:1755,input:raw,output:cut,members:true,animation:Some(animation)},level,xp_tenths:xp,ticks:1,name:"independent semiprecious",action:"cut",failure:Some(Failure{output:1633,xp_tenths:failed,low,high,message:"crushed"})},tool_slot:0,input_slot:17};(job,inv,Catalog::default(),stats)
}
fn threshold(i:usize,level:i32)->u32{let (_,_,_,_,_,low,high,_)=PROFILES[i];let l=level.clamp(1,99);((low*98+(high-low)*(l-1)+49)/98+1).clamp(0,256) as u32}
#[test]
fn every_byte_roll_99_levels_two_boosts_membership_three_rates_matches_integer_oracle(){
    assert_eq!([threshold(0,1),threshold(1,13),threshold(2,16)],[129,119,114]);assert_eq!([threshold(0,99),threshold(1,99),threshold(2,99)],[251,246,241]);
    let mut cases=0;
    for i in 0..3{for level in (1..=99).chain([100,120]){for member in [false,true]{for rate in [100,125,500]{for roll in 0..256{
        let (job,inv,cat,mut stats)=fixture(i);stats.set_current(12,level as i8);let before=(inv.clone(),stats.clone());let mut random=Draw{value:roll,calls:0};let p=job.resolve(&inv,&cat,&stats,rate,member,&mut random);
        if !member || level<PROFILES[i].2 as i32{assert!(p.is_err());assert_eq!(random.calls,0);}else{
            let (p,made)=p.unwrap();let success=roll<threshold(i,level);assert_eq!(made,success,"profile{i} level{level} roll{roll}");assert_eq!(random.calls,1);
            let mut expected=inv.clone();expected.restore_slot(17,Some(Item{id:if success{PROFILES[i].1}else{1633},count:1,vars:0})).unwrap();assert_eq!(p.inventory,expected);
            let mut expected_stats=stats.clone();let xp=if success{PROFILES[i].3}else{PROFILES[i].4};expected_stats.set_fine_xp(12,130344311+xp as i32*rate/100).unwrap();assert_eq!(p.stats,expected_stats);
        }
        assert_eq!((inv,stats),before);cases+=1;
    }}}}}
    assert_eq!(cases,465408);
}
#[test]
fn invalid_either_outcome_and_live_policy_never_draw_or_publish(){
    for i in 0..3{for mode in 0..16{
        let (mut job,mut inv,mut cat,mut stats)=fixture(i);let mut rate=125;let mut member=true;
        match mode{
            0=>{cat.dummy.insert(job.recipe.exchange.output);},1=>{cat.dummy.insert(1633);},
            2=>{inv.restore_slot(17,Some(Item{id:job.recipe.exchange.input,count:1,vars:1})).unwrap();},
            3=>{inv.restore_slot(0,Some(Item{id:1755,count:1,vars:1})).unwrap();},
            4=>{inv.restore_slot(17,Some(Item{id:job.recipe.exchange.input,count:2,vars:0})).unwrap();},
            5=>{inv.restore_slot(0,Some(Item{id:1755,count:2,vars:0})).unwrap();},
            6=>member=false,7=>stats.set_current(12,job.recipe.level as i8-1),8=>rate=0,9=>rate=-1,
            10=>job.recipe.failure.as_mut().unwrap().low=246,11=>job.recipe.failure.as_mut().unwrap().high=256,
            12=>job.recipe.failure.as_mut().unwrap().low=-1,13=>job.recipe.failure.as_mut().unwrap().xp_tenths=0,
            14=>job.recipe.failure.as_mut().unwrap().output=job.recipe.exchange.input,_=>job.recipe.exchange.output=1755,
        }
        // Invalid reversed endpoints must exceed this profile's actual high.
        if mode==10{job.recipe.failure.as_mut().unwrap().low=job.recipe.failure.unwrap().high+1;}
        let before=(inv.clone(),stats.clone());let mut random=Draw{value:0,calls:0};assert!(job.resolve(&inv,&cat,&stats,rate,member,&mut random).is_err(),"profile{i} mode{mode}");assert_eq!(random.calls,0,"profile{i} mode{mode}");assert_eq!((inv,stats),before);
    }}
}
