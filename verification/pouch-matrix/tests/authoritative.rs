//! Independent selected-stack transaction oracle; no acquisition/Thieving claims.
use gielinor_world::{inventory::{Inventory,Catalog,Item,StackType},item_packs::{Rule,PouchRule,open_count},simulation::{World,PlayerData,Action},stats::Stats,CoordGrid};
const POUCH:i32=22521;
fn fixture(slot:usize,n:i32,coins:Option<i32>,full:bool)->(World,gielinor_world::simulation::ActorId){
    let mut w=World::default();w.coin_pouches.insert(POUCH,PouchRule{rule:Rule{output:995,count:3,noted:false},members:true});w.object_types.stackable.extend([POUCH,995]);
    let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);
    let mut data=PlayerData::new(1,"Independent pouch".into(),pos,stats);
    for i in 0..28{data.inventory.restore_slot(i,Some(Item{id:1931,count:1,vars:0})).unwrap();}
    data.inventory.restore_slot(slot,Some(Item{id:POUCH,count:n,vars:0})).unwrap();data.inventory.restore_slot((slot+7)%28,Some(Item{id:POUCH,count:2,vars:0})).unwrap();
    if let Some(count)=coins{data.inventory.restore_slot((slot+14)%28,Some(Item{id:995,count,vars:0})).unwrap();}
    if !full{data.inventory.restore_slot((slot+21)%28,None).unwrap();}
    w.queue_login(1,data).unwrap();w.tick().unwrap();let actor=w.actors().next().unwrap();w.set_membership(actor,true).unwrap();(w,actor)
}
#[test]
fn selected_stack_full_bag_one_all_overflow_and_membership_4480_cases(){
    let mut cases=0;
    for slot in 0..28{for n in [1,2,28,i32::MAX/3,i32::MAX/3+1]{for coins in [None,Some(10),Some(i32::MAX-3),Some(i32::MAX-84)]{for all in [false,true]{for member in [false,true]{for full in [false,true]{
        let (mut w,a)=fixture(slot,n,coins,full);w.set_membership(a,member).unwrap();let before=w.data(a).unwrap().clone();let quantity=if all{n}else{1};
        let amount=quantity.checked_mul(3);let total=amount.and_then(|v|coins.unwrap_or(0).checked_add(v));let allowed=member && total.is_some() && (coins.is_some() || all || n==1 || !full);
        let mut expected=before.inventory.clone();
        if allowed{
            expected.restore_slot(slot,if n==quantity{None}else{Some(Item{id:POUCH,count:n-quantity,vars:0})}).unwrap();
            let target=if coins.is_some(){(slot+14)%28}else{expected.slots().iter().position(Option::is_none).unwrap()};
            expected.restore_slot(target,Some(Item{id:995,count:total.unwrap(),vars:0})).unwrap();
        }
        w.submit(a,Action::OpenCoinPouch{slot,item:POUCH,all}).unwrap();w.tick().unwrap();assert_eq!(w.data(a).unwrap().inventory,expected,"slot{slot} n{n} coins{coins:?} all{all} member{member} full{full}");assert_eq!(w.data(a).unwrap().stats,before.stats);
        for _ in 0..4{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,expected);assert_eq!(w.data(a).unwrap().stats,before.stats);cases+=1;
    }}}}}}
    assert_eq!(cases,4480);
}
#[test]
fn checked_count_refusals_leave_whole_selected_inventory_unchanged(){
    let mut cat=Catalog::default();cat.stackable.extend([POUCH,995]);let mut inv=Inventory::new(28,StackType::Normal,false);inv.restore_slot(5,Some(Item{id:POUCH,count:28,vars:0})).unwrap();inv.restore_slot(7,Some(Item{id:POUCH,count:28,vars:0})).unwrap();
    for quantity in [i32::MIN,-1,0,29,i32::MAX]{let before=inv.clone();assert!(open_count(&mut inv,&cat,5,POUCH,Rule{output:995,count:3,noted:false},quantity).is_err());assert_eq!(inv,before);}
}
#[test]
fn actual_world_invalid_metadata_modal_claims_duplicate_and_zero_xp_controls(){
    use gielinor_world::inventory::Template;
    for mode in 0..18{
        let (mut w,a)=fixture(27,3,Some(10),true);
        match mode{
            0=>w.set_membership(a,false).unwrap(),
            1=>{w.data_mut(a).unwrap().inventory.restore_slot(27,None).unwrap();},
            2=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:POUCH,count:3,vars:1})).unwrap();},
            3=>{w.data_mut(a).unwrap().inventory.restore_slot(13,Some(Item{id:995,count:10,vars:1})).unwrap();},
            4=>{w.object_types.dummy.insert(POUCH);},5=>{w.object_types.dummy.insert(995);},
            6=>{w.object_types.notes.insert(POUCH,Template{link:995,template:799});},
            7=>{w.object_types.placeholders.insert(995,Template{link:995,template:14401});},
            8=>{w.object_types.transforms.insert(POUCH,Template{link:995,template:1});},
            9=>{w.object_types.stackable.remove(&POUCH);},10=>{w.object_types.stackable.remove(&995);},
            11=>w.data_mut(a).unwrap().inventory.stack=StackType::Always,
            12=>w.data_mut(a).unwrap().inventory.placeholders=true,
            13=>w.coin_pouches.get_mut(&POUCH).unwrap().rule.noted=true,
            14=>w.set_modal_open(a,true).unwrap(),
            15=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:1931,count:1,vars:0})).unwrap();},
            _=>{},
        }
        let before=w.data(a).unwrap().clone();w.submit(a,Action::OpenCoinPouch{slot:if mode==16{28}else{27},item:if mode==17{POUCH+1}else{POUCH},all:true}).unwrap();w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory,before.inventory,"mode{mode}");assert_eq!(w.data(a).unwrap().stats,before.stats);
    }
    let (mut w,a)=fixture(27,3,Some(10),true);w.data_mut(a).unwrap().xp_rate_hundreds=0;w.data_mut(a).unwrap().stats.set_current(13,0);let stats=w.data(a).unwrap().stats.clone();
    for _ in 0..2{w.submit(a,Action::OpenCoinPouch{slot:27,item:POUCH,all:true}).unwrap();}w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().inventory.get(13).unwrap().count,19);assert!(w.data(a).unwrap().inventory.get(27).is_none());assert_eq!(w.data(a).unwrap().inventory.get(6).unwrap().count,2);assert_eq!(w.data(a).unwrap().stats,stats);
}
