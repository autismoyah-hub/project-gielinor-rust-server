use gielinor_world::{CoordGrid,food::Edible,inventory::{Item,Template},simulation::{World,PlayerData,ActorId,Action,WorldEvent},stats::Stats};
fn fixture(hp:u8)->(World,ActorId) {
    let mut w=World::default();let coord=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
    let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,hp as i8);stats.set_fine_xp(3,10007).unwrap();
    w.queue_login(1,PlayerData::new(1,"PG disposable".into(),coord,stats)).unwrap();w.tick().unwrap();
    let actor=w.actors().next().unwrap();w.drain_events();
    w.foods.insert(385,Edible{id:385,heal:20,delay:3,replacement:None,message:"Shark".into()});(w,actor)
}
fn fill(w:&mut World,a:ActorId,selected:usize) {
    let data=w.data_mut(a).unwrap();
    for slot in 0..28 {data.inventory.restore_slot(slot,Some(Item{id:if slot==selected||slot==0{385}else{995},count:1,vars:0})).unwrap();}
    data.inventory.clear_modified();
}
#[test]
fn actual_world_queue_preserves_every_other_slot_hp_cap_and_xp_under_duplicate_input() {
    for hp in [1u8,50,98,99,100,115,127,128,255] {for selected in 0..28 {
        let (mut w,a)=fixture(hp);fill(&mut w,a,selected);
        let before=w.data(a).unwrap().clone();
        for _ in 0..2 {w.submit(a,Action::EatItemSlot{slot:selected,item:385}).unwrap();}
        w.tick().unwrap();let after=w.data(a).unwrap();
        assert_eq!(after.stats.current(3) as u8,if hp<99{hp.saturating_add(20).min(99)}else{hp});
        for i in 0..28 {assert_eq!(after.inventory.get(i),if i==selected{None}else{before.inventory.get(i)});}
        for skill in 0..=255 {assert_eq!(after.stats.fine_xp(skill),before.stats.fine_xp(skill));}
        let events=w.drain_events();assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::FoodTransition{..})).count(),1);
        assert!(w.protection(a).unwrap().delayed());
        for _ in 0..5{w.tick().unwrap();}w.drain_events();let stable=w.data(a).unwrap().clone();
        w.submit(a,Action::EatItemSlot{slot:selected,item:385}).unwrap();w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory,stable.inventory);assert_eq!(w.data(a).unwrap().stats,stable.stats);
        assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::FoodTransition{..})));
    }}
}
#[test]
fn stale_claim_cannot_consume_a_different_configured_food_after_submission() {
    for slot in 0..28 {
        let (mut w,a)=fixture(50);fill(&mut w,a,slot);
        w.foods.insert(999,Edible{id:999,heal:49,delay:3,replacement:None,message:"Other".into()});
        w.submit(a,Action::EatItemSlot{slot,item:385}).unwrap();
        w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:999,count:1,vars:0})).unwrap();
        let before=w.data(a).unwrap().clone();w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);
        assert!(!w.protection(a).unwrap().delayed());
        assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::FoodTransition{..})));
    }
}
#[test]
fn full_bag_remainder_stays_selected_and_does_not_merge_with_existing_identity() {
    for selected in 1..28 {
        let (mut w,a)=fixture(50);fill(&mut w,a,selected);w.foods.get_mut(&385).unwrap().replacement=Some(386);
        w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:386,count:1,vars:0})).unwrap();
        let before=w.data(a).unwrap().inventory.clone();w.submit(a,Action::EatItemSlot{slot:selected,item:385}).unwrap();w.tick().unwrap();
        let after=w.data(a).unwrap();assert_eq!(after.inventory.get(selected),Some(Item{id:386,count:1,vars:0}));
        for slot in 0..28 {if slot!=selected{assert_eq!(after.inventory.get(slot),before.get(slot));}}
        assert_eq!(after.stats.current(3),70);
    }
}
#[test]
fn invalid_prepared_exchange_or_heal_cannot_publish_a_partial_world_action() {
    for mode in 0..12 {
        let (mut w,a)=fixture(115);fill(&mut w,a,27);w.foods.get_mut(&385).unwrap().replacement=Some(386);
        match mode {
            0..=2=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:385,count:[0,-1,2][mode],vars:0})).unwrap();},
            3=>{w.object_types.stackable.insert(385);},4=>{w.object_types.dummy.insert(385);},
            5=>{w.object_types.notes.insert(385,Template{link:1,template:1});},
            6=>{w.object_types.placeholders.insert(385,Template{link:1,template:1});},
            7=>{w.object_types.transforms.insert(386,Template{link:1,template:1});},
            8=>{w.object_types.stackable.insert(386);},9=>{w.object_types.dummy.insert(386);},
            10=>w.foods.get_mut(&385).unwrap().heal=-1,
            _=>w.foods.get_mut(&385).unwrap().replacement=Some(385),
        }
        let before=w.data(a).unwrap().clone();w.submit(a,Action::EatItemSlot{slot:27,item:385}).unwrap();w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory,before.inventory,"mode {mode}");assert_eq!(w.data(a).unwrap().stats,before.stats,"mode {mode}");
        assert!(!w.protection(a).unwrap().delayed());assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::FoodTransition{..})));
    }
}
