use gielinor_world::{CoordGrid,food::{Edible,Event},inventory::Item,simulation::{World,PlayerData,Action,WorldEvent},stats::Stats,player_regen};
#[test]
fn food_commit_and_subsequent_passive_regeneration_have_distinct_valid_hp_observations() {
    let mut w=World::default();let pos=CoordGrid::new(3200,3200,0).unwrap();w.collision.allocate(3200,3200,0);
    w.regeneration=Some(player_regen::Config{skills:vec![3],hitpoints:3,prayer:5,bracelet:11133,drained_timer:11,boosted_timer:12,health_timer:13,rapid_timer:14});
    let mut s=Stats::default();s.set_base(3,10);s.set_current(3,5);
    w.queue_login(1,PlayerData::new(1,"PG passive-regen diagnosis".into(),pos,s)).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();
    w.foods.insert(1891,Edible{id:1891,heal:4,delay:2,replacement:Some(1893),message:"Cake".into()});w.slot_preserving_foods.insert(1891);
    w.data_mut(a).unwrap().inventory.restore_slot(3,Some(Item{id:1891,count:1,vars:0})).unwrap();
    while w.clock.0<98{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().stats.current(3),5);w.drain_events();
    w.submit(a,Action::EatItemSlot{slot:3,item:1891}).unwrap();w.tick().unwrap();assert_eq!(w.clock.0,99);
    let events=w.drain_events();
    assert!(events.iter().any(|e|matches!(e,WorldEvent::Food{event:Event::Stat(u),..} if u.old==5 && u.level==9)));
    assert_eq!(w.data(a).unwrap().stats.current(3),9);
    let mut timeline=vec![(w.clock.0,w.data(a).unwrap().stats.current(3))];
    for _ in 0..4 {w.tick().unwrap();timeline.push((w.clock.0,w.data(a).unwrap().stats.current(3)));}
    assert_eq!(w.data(a).unwrap().stats.current(3),10);
    assert_eq!(w.data(a).unwrap().inventory.get(3),Some(Item{id:1893,count:1,vars:0}));
    assert_eq!(w.data(a).unwrap().stats.fine_xp(3),0);
    println!("Food transition5->9, followed by passive regeneration without more food: {timeline:?}. An observer that waits for scripts/packet batches must not require the intermediate9 indefinitely.");
}
