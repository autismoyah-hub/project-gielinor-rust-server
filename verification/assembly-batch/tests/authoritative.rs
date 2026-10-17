use std::sync::Arc;
use gielinor_world::{assembly::{BatchRecipe,Outcome},held_use::Request,inventory::{Inventory,Item,Catalog,StackType},stats::Stats};
#[test]
fn independent_full_bag_selected_material_and_late_return_atomicity_matrix() {
    let ids=[1887,1944,1927,1933];
    for start in 0..28usize {for other in 1..4usize {for reverse in [false,true] {for mode in 0..9 {
        let slots:Vec<_>=(0..4).map(|n|(start+n)%28).collect();
        let mut inv=Inventory::new(28,StackType::Normal,false);
        for slot in 0..28{inv.restore_slot(slot,Some(Item{id:1931,count:1,vars:0})).unwrap();}
        for n in 0..4 {for slot in [slots[n],(start+8+n)%28] {inv.restore_slot(slot,Some(Item{id:ids[n],count:1,vars:0})).unwrap();}}
        let recipe=Arc::new(BatchRecipe{inputs:ids.to_vec(),outputs:vec![1889,1925,1931],skill:7,level:40,base_xp:if mode==6{30}else{0},members:mode==8,ticks:2});
        let (first,second)=if reverse{(other,0)}else{(0,other)};
        let request=Request{selected_slot:slots[first] as i32,selected_id:ids[first],target_slot:slots[second] as i32,target_id:ids[second]};
        let mut job=recipe.bind(request,&inv).unwrap();let captured=job.slots.clone();
        assert_eq!(captured[first],slots[first]);assert_eq!(captured[second],slots[second]);
        for n in 0..4{assert_eq!(inv.get(captured[n]).unwrap().id,ids[n]);}
        let mut stats=Stats::default();stats.set_base(7,39);stats.set_current(7,40);let mut catalog=Catalog::default();
        match mode {
            1=>{inv.restore_slot(captured[2],None).unwrap();}, // An untouched matching duplicate is still present.
            2=>stats.set_current(7,39),
            3..=5=>{catalog.dummy.insert([1889,1925,1931][mode-3]);},
            7=>job.slots[2]=job.slots[1], // Public job metadata must still reject repeated captured slots.
            _=>{},
        }
        let before=(inv.clone(),stats.clone());let (outcome,reward)=job.complete_awarding(&mut inv,&catalog,&mut stats,if mode==6{0}else{125},false);
        if mode==0 {
            assert_eq!(outcome,Outcome::Made);assert!(reward.is_none());assert_eq!(stats,before.1);
            assert_eq!(inv.slots().iter().flatten().count(),27);
            for slot in 0..28{if !captured.contains(&slot){assert_eq!(inv.get(slot),before.0.get(slot));}}
            for id in ids{assert_eq!(inv.slots().iter().flatten().filter(|v|v.id==id).count(),1);}
            for id in [1889,1925]{assert_eq!(inv.slots().iter().flatten().filter(|v|v.id==id).count(),1);}
            let stable=(inv.clone(),stats.clone());assert_ne!(job.complete_awarding(&mut inv,&catalog,&mut stats,125,false).0,Outcome::Made);assert_eq!((inv,stats),stable);
        }else{
            assert_ne!(outcome,Outcome::Made,"start={start}, other={other}, reverse={reverse}, mode={mode}");assert!(reward.is_none());assert_eq!((inv,stats),before);
        }
    }}}}
    println!("1512 independently specified full-bag/slot-direction/late-failure combinations; direct authoritative transaction API, not native/runtime registration acceptance.");
}
