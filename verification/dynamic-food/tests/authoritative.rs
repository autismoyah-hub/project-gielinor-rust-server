use gielinor_world::{CoordGrid,food::{Edible,Healing,Event},inventory::Item,simulation::{World,PlayerData,ActorId,Action,WorldEvent},stats::Stats};

fn fixture(id:i32,base:u8,current:u8,rule:Healing,bowl:bool)->(World,ActorId) {
    let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
    let mut stats=Stats::default();stats.set_base(3,base.try_into().unwrap());stats.set_current(3,current.try_into().unwrap());
    w.queue_login(1,PlayerData::new(1,"PG disposable dynamic food".into(),pos,stats)).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();
    let d=w.data_mut(a).unwrap();
    for slot in 0..28 {d.inventory.restore_slot(slot,Some(Item{id:if slot==0||slot==27{id}else{1931},count:1,vars:0})).unwrap();}
    w.foods.insert(id,Edible{id,heal:0,delay:3,replacement:if bowl{Some(1923)}else{None},message:"Dynamic food".into()});
    w.food_healing.insert(id,rule);w.drain_events();(w,a)
}

#[test]
fn independent_rounding_tables_all_ordinary_base_levels_and_hp_states_through_action_queue() {
    // Expected values come from explicit healing-table change points, not the production percent formula.
    let profiles=[
        (5504,Healing{percent:6,constant:0,round_up:true},false,vec![1,17,34,51,67,84]),
        (5984,Healing{percent:5,constant:1,round_up:false},false,vec![0,20,40,60,80]),
        (5988,Healing{percent:10,constant:1,round_up:false},false,vec![0,10,20,30,40,50,60,70,80,90]),
        (7088,Healing{percent:10,constant:1,round_up:false},true,vec![0,10,20,30,40,50,60,70,80,90]),
    ];
    for (id,rule,bowl,thresholds) in profiles {
        for base in 1u8..=99 {
            let heal=thresholds.iter().filter(|&&n|i32::from(base)>=n).count() as u8;
            for current in [0,base.saturating_sub(1),base,base+5] {
                let (mut w,a)=fixture(id,base,current,rule,bowl);let before=w.data(a).unwrap().clone();
                for _ in 0..2 {w.submit(a,Action::EatItemSlot{slot:27,item:id}).unwrap();}w.tick().unwrap();
                let after=w.data(a).unwrap();let expected=if current>base{current}else{current.saturating_add(heal).min(base)};
                assert_eq!(after.stats.current(3),i8::try_from(expected).unwrap(),"id={id} base={base} current={current}");
                assert_eq!(after.inventory.get(27),if bowl{Some(Item{id:1923,count:1,vars:0})}else{None});
                for slot in 0..27 {assert_eq!(after.inventory.get(slot),before.inventory.get(slot));}
                for skill in 0..=255 {assert_eq!(after.stats.fine_xp(skill),before.stats.fine_xp(skill));if skill!=3{assert_eq!(after.stats.current(skill),before.stats.current(skill));}}
                let events=w.drain_events();assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::Food{event:Event::Animation(_),..})).count(),1);
                assert!(w.protection(a).unwrap().delayed());
            }
        }
    }
}

#[test]
fn live_base_level_is_read_at_execution_and_replayed_claim_does_not_consume_duplicate() {
    let (mut w,a)=fixture(5984,99,1,Healing{percent:5,constant:1,round_up:false},false);
    w.submit(a,Action::EatItemSlot{slot:27,item:5984}).unwrap();w.data_mut(a).unwrap().stats.set_base(3,20);w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().stats.current(3),3);assert!(w.data(a).unwrap().inventory.get(27).is_none());
    for _ in 0..4 {w.tick().unwrap();}let before=w.data(a).unwrap().clone();w.drain_events();
    w.submit(a,Action::EatItemSlot{slot:27,item:5984}).unwrap();w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().inventory.slots(),before.inventory.slots());assert_eq!(w.data(a).unwrap().stats,before.stats);
    assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{..})));
}

#[test]
fn invalid_rules_and_failed_bowl_transactions_publish_no_healing_or_item_changes() {
    for mode in 0..7 {
        let (mut w,a)=fixture(7088,50,1,Healing{percent:10,constant:1,round_up:false},true);
        match mode {
            0=>w.food_healing.get_mut(&7088).unwrap().percent=101,
            1=>w.food_healing.get_mut(&7088).unwrap().constant=-1,
            2=>{w.object_types.dummy.insert(1923);},
            3=>{w.data_mut(a).unwrap().inventory.restore_slot(27,None).unwrap();},
            4=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:7088,count:2,vars:0})).unwrap();},
            5=>{w.foods.get_mut(&7088).unwrap().replacement=Some(7088);},
            _=>{w.object_types.stackable.insert(7088);},
        }
        let before=w.data(a).unwrap().clone();w.submit(a,Action::EatItemSlot{slot:27,item:7088}).unwrap();w.tick().unwrap();
        let after=w.data(a).unwrap();assert_eq!(after.inventory.slots(),before.inventory.slots(),"mode {mode}");assert_eq!(after.stats,before.stats,"mode {mode}");
        assert!(!w.protection(a).unwrap().delayed());assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{..})));
    }
}
