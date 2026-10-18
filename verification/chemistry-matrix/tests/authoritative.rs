use std::collections::VecDeque;
use gielinor_world::{chemistry,chemistry_modifier::Binding,finished_potions::{Job,Recipe,Outcome},inventory::{Inventory,Item,Catalog,StackType},npc::RandomSource,stats::Stats};
struct Draw(VecDeque<(u32,u32)>);
impl RandomSource for Draw{fn below(&mut self,upper:u32)->u32{let (expected,n)=self.0.pop_front().expect("unexpected RNG draw before validation");assert_eq!(upper,expected);n}}
fn fixture(reverse:bool,goggles:bool)->(Job,Inventory,Inventory,Stats){
    let mut inv=Inventory::new(28,StackType::Normal,false);let (a,b)=if reverse{(27,4)}else{(4,27)};
    for slot in 0..28{inv.restore_slot(slot,Some(Item{id:if slot==a{93}else if slot==b{235}else if slot==0{2446}else{1931},count:1,vars:0})).unwrap();}
    let mut equip=Inventory::new(14,StackType::Normal,false);equip.restore_slot(2,Some(Item{id:21163,count:1,vars:0})).unwrap();if goggles{equip.restore_slot(0,Some(Item{id:29974,count:1,vars:0})).unwrap();}
    let mut s=Stats::default();s.set_base(15,5);s.set_current(15,5);s.set_fine_xp(15,3881).unwrap();
    (Job{recipe:Recipe{unfinished:93,secondary:235,output:175,level:5,boostable:true,base_fine_xp:375},unfinished_slot:a,secondary_slot:b},inv,equip,s)
}
#[test]
fn exhaustive_ordinary_chemistry_probability_goggles_last_charge_and_conservation(){
    let mut cases=0;
    for reverse in [false,true]{for remaining in 1..=5{for stop in [false,true]{for goggles in [false,true]{for gd in 0..if goggles{10}else{1}{for cd in 0..100{
        let (job,mut inv,mut equip,mut stats)=fixture(reverse,goggles);let before=(inv.clone(),equip.clone(),stats.clone());let mut state=chemistry::State::restore(remaining,stop as i32).unwrap();
        let mut values=VecDeque::new();if goggles{values.push_back((10,gd));}values.push_back((100,cd));let mut rng=Draw(values);
        let r=job.complete_with_modifiers(&mut inv,&Catalog::default(),&mut stats,&mut equip,&mut state,true,true,500,goggles,Some(Binding{amulet:21163,extra_output:2446}),&mut rng);
        let extra=cd<5;let saved=goggles&&gd==0;let crumble=extra&&remaining==1;
        assert!(matches!(r.outcome,Outcome::Made(_)));assert_eq!(r.secondary_saved,saved);assert_eq!(r.chemistry.extra_dose,extra);assert_eq!(r.chemistry.crumbled,crumble);assert_eq!(r.chemistry.stop,crumble&&stop);
        assert_eq!(state.remaining(),if crumble{5}else if extra{remaining as u8-1}else{remaining as u8});assert_eq!(state.stop_on_crumble(),stop);
        assert_eq!(equip.get(2).is_none(),crumble);assert_eq!(equip.get(0),before.1.get(0));assert!(rng.0.is_empty());
        assert_eq!(stats.fine_xp(15)-before.2.fine_xp(15),1875);assert_eq!((stats.base(15),stats.current(15)),(6,6));
        for skill in 0..25{if skill!=15{assert_eq!((stats.fine_xp(skill),stats.current(skill)),(before.2.fine_xp(skill),before.2.current(skill)));}}
        for slot in 0..28{if slot!=job.unfinished_slot&&slot!=job.secondary_slot{assert_eq!(inv.get(slot),before.0.get(slot));}}
        let mut actual:Vec<_>=[job.unfinished_slot,job.secondary_slot].into_iter().filter_map(|slot|inv.get(slot)).map(|item|{assert_eq!((item.count,item.vars),(1,0));item.id}).collect();actual.sort();
        let mut expected=vec![if extra{2446}else{175}];if saved{expected.push(235);assert_eq!(inv.get(job.secondary_slot),Some(Item{id:235,count:1,vars:0}));}expected.sort();assert_eq!(actual,expected);cases+=1;
    }}}}}}
    assert_eq!(cases,22000);
}
#[test]
fn all_validation_failures_preserve_inventory_equipment_stats_charges_and_rng(){
    for mode in 0..12{let (mut job,mut inv,mut equip,mut stats)=fixture(false,true);let mut state=chemistry::State::restore(1,1).unwrap();let mut catalog=Catalog::default();let mut binding=Binding{amulet:21163,extra_output:2446};
        match mode{0=>stats.set_current(15,4),1=>{inv.restore_slot(job.unfinished_slot,None).unwrap();},2=>{catalog.dummy.insert(2446);},3=>{catalog.stackable.insert(2446);},4=>binding.extra_output=93,5=>binding.extra_output=175,6=>binding.extra_output=21163,7=>job.secondary_slot=job.unfinished_slot,8=>{inv.restore_slot(job.secondary_slot,Some(Item{id:235,count:1,vars:1})).unwrap();},_=>{}}
        let before=(inv.clone(),equip.clone(),stats.clone(),state);let mut rng=Draw(VecDeque::new());
        let r=job.complete_with_modifiers(&mut inv,&catalog,&mut stats,&mut equip,&mut state,mode!=9,mode!=10,if mode==11{0}else{500},true,Some(binding),&mut rng);
        assert!(!matches!(r.outcome,Outcome::Made(_)),"mode {mode}");assert_eq!((inv,equip,stats,state),before);assert!(rng.0.is_empty());
    }
}
