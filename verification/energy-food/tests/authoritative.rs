use gielinor_world::{CoordGrid,food::{Edible,Event,StatBoost},inventory::Item,simulation::{World,PlayerData,ActorId,Action,WorldEvent},stats::Stats};
fn fixture(energy:i32)->(World,ActorId) {
    let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
    let mut s=Stats::default();s.set_base(3,99);s.set_current(3,50);s.set_base(16,50);s.set_current(16,50);
    w.queue_login(1,PlayerData::new(1,"PG disposable energy".into(),pos,s)).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();
    let d=w.data_mut(a).unwrap();d.energy=energy;
    for slot in 0..28 {d.inventory.restore_slot(slot,Some(Item{id:if slot==0||slot==27{7218}else{1931},count:1,vars:0})).unwrap();}
    w.foods.insert(7218,Edible{id:7218,heal:11,delay:1,replacement:Some(7220),message:"Summer pie".into()});
    w.food_boosts.insert(7218,vec![StatBoost{skill:16,amount:5}]);w.food_energy.insert(7218,10);w.drain_events();(w,a)
}
#[test]
fn selected_food_energy_hp_boost_and_remainder_commit_once_through_real_action_queue() {
    for energy in [0,500,9000,9500,9999,10000,11000] {
        let (mut w,a)=fixture(energy);let before=w.data(a).unwrap().clone();
        for _ in 0..2{w.submit(a,Action::EatItemSlot{slot:27,item:7218}).unwrap();}w.tick().unwrap();
        let after=w.data(a).unwrap();assert_eq!(after.inventory.get(27),Some(Item{id:7220,count:1,vars:0}));
        for slot in 0..27 {assert_eq!(after.inventory.get(slot),before.inventory.get(slot));}
        assert_eq!((after.stats.current(3),after.stats.current(16)),(61,55));
        for skill in 0..=255 {assert_eq!(after.stats.fine_xp(skill),before.stats.fine_xp(skill));}
        let expected=if energy>10000{energy}else{(energy+1000).min(10000)};
        let events=w.drain_events();let updates:Vec<_>=events.iter().filter_map(|e|if let WorldEvent::Food{event:Event::Energy(n),..}=e{Some(*n)}else{None}).collect();
        assert_eq!(updates,if expected!=energy{vec![expected]}else{vec![]});
        assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::Food{event:Event::Animation(_),..})).count(),1);
    }
}
#[test]
fn failed_food_transaction_has_exactly_the_same_energy_as_a_no_action_control() {
    for mode in 0..9 {
        let (mut w,a)=fixture(500);let (mut control,b)=fixture(500);
        match mode {
            0=>{w.food_energy.insert(7218,101);},1=>{w.food_energy.insert(7218,-1);},
            2=>w.foods.get_mut(&7218).unwrap().heal=-1,
            3=>{w.food_boosts.insert(7218,vec![StatBoost{skill:16,amount:-1}]);},
            4=>{w.object_types.dummy.insert(7220);},5=>{w.object_types.stackable.insert(7218);},
            6=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:7218,count:2,vars:0})).unwrap();},
            7=>{w.data_mut(a).unwrap().inventory.restore_slot(27,None).unwrap();},
            _=>{w.foods.get_mut(&7218).unwrap().replacement=Some(7218);},
        }
        let before=w.data(a).unwrap().clone();w.submit(a,Action::EatItemSlot{slot:27,item:7218}).unwrap();w.tick().unwrap();control.tick().unwrap();
        let after=w.data(a).unwrap();assert_eq!(after.inventory.slots(),before.inventory.slots(),"mode {mode}");assert_eq!(after.stats,before.stats,"mode {mode}");
        assert_eq!(after.energy,control.data(b).unwrap().energy,"mode {mode}");
        assert!(!w.protection(a).unwrap().delayed());assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{..})));
    }
}
