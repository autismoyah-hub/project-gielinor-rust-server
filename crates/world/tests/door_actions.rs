use gielinor_world::{
    doors::{Door, Kind},
    loc_collision::{self, LocCollision},
    locations::Location,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn pos(x: i32, z: i32) -> CoordGrid {
    CoordGrid::new(x, z, 0).unwrap()
}
fn setup(shape: u8, player: CoordGrid) -> (World, ActorId, Location) {
    let mut w = World::default();
    for x in 3184..3224 {
        for z in 3184..3224 {
            w.collision.set(x, z, 0, 0);
        }
    }
    let ty = LocCollision {
        width: 1,
        length: 1,
        block_walk: 2,
        block_range: true,
        break_route_finding: false,
    };
    for id in 1..=4 {
        w.location_types.insert(id, ty);
    }
    w.doors.insert(
        1,
        Door {
            kind: Kind::ClosedSingle,
            next: 2,
            sound: 62,
        },
    );
    w.doors.insert(
        2,
        Door {
            kind: Kind::OpenedSingle,
            next: 1,
            sound: 60,
        },
    );
    let loc = Location::new(pos(3200, 3200), 1, shape, 0).unwrap();
    w.locations.load(loc);
    loc_collision::toggle(&mut w.collision, loc.coords, shape, 0, ty, true).unwrap();
    w.queue_login(
        1,
        PlayerData::new(1, "Door tester".into(), player, Stats::default()),
    )
    .unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    w.drain_events();
    (w, actor, loc)
}
fn click(w: &mut World, actor: ActorId, loc: Location) {
    w.submit(
        actor,
        Action::OperateDoor {
            coords: loc.coords,
            id: loc.id,
            control: false,
        },
    )
    .unwrap();
}
#[test]
fn arrival_waits_one_cycle_and_closed_map_door_respawns() {
    let (mut w, a, loc) = setup(0, pos(3197, 3200));
    click(&mut w, a, loc);
    for _ in 0..2 {
        w.tick().unwrap();
        assert_eq!(w.locations.find_type(loc.coords, 1), Some(loc));
    }
    assert_eq!(w.data(a).unwrap().coords, pos(3199, 3200));
    w.drain_events();
    w.tick().unwrap();
    let opened = Location::new(pos(3199, 3200), 2, 0, 1).unwrap();
    assert_eq!(w.locations.find_type(opened.coords, 2), Some(opened));
    let events = w.drain_events();
    assert!(events
        .iter()
        .any(|e| matches!(e,WorldEvent::DoorSound{actor,sound:62}if *actor==a)));
    assert!(events
        .iter()
        .any(|e| matches!(e,WorldEvent::ClearMapFlag(actor)if *actor==a)));
    for _ in 0..500 {
        w.tick().unwrap();
        w.drain_events();
    }
    assert_eq!(w.locations.find_type(loc.coords, 1), Some(loc));
    assert!(w.locations.find_type(opened.coords, 2).is_none());
}
#[test]
fn diagonal_step_away_defers_mutation_for_two_cycles() {
    let (mut w, a, loc) = setup(9, pos(3200, 3201));
    click(&mut w, a, loc);
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().coords, pos(3199, 3201));
    assert_eq!(w.locations.find_type(loc.coords, 1), Some(loc));
    assert!(w.protection(a).unwrap().suspended);
    w.tick().unwrap();
    assert_eq!(w.locations.find_type(loc.coords, 1), Some(loc));
    w.tick().unwrap();
    assert!(w.locations.find_type(loc.coords, 1).is_none());
    assert!(!w.protection(a).unwrap().suspended);
    assert!(w.locations.find_type(pos(3200, 3201), 2).is_some());
}
#[test]
fn held_operation_cancels_suspended_door_even_when_delayed() {
    let (mut w, a, loc) = setup(9, pos(3200, 3201));
    click(&mut w, a, loc);
    w.tick().unwrap();
    w.submit(a, Action::EatSlot { slot: 0 }).unwrap();
    for _ in 0..3 {
        w.tick().unwrap();
    }
    assert_eq!(w.locations.find_type(loc.coords, 1), Some(loc));
    assert!(w.locations.find_type(pos(3200, 3201), 2).is_none());
}
#[test]
fn moving_actor_cancels_when_another_actor_replaces_target() {
    let (mut w, a, loc) = setup(0, pos(3196, 3200));
    click(&mut w, a, loc);
    w.tick().unwrap();
    w.change_location(loc, false).unwrap();
    w.drain_events();
    w.tick().unwrap();
    let stopped = w.data(a).unwrap().coords;
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().coords, stopped);
    assert!(!w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::DoorSound { .. })));
}
#[test]
fn double_door_changes_both_halves_and_restores_together() {
    let (mut w, a, loc) = setup(0, pos(3199, 3200));
    w.doors.insert(
        1,
        Door {
            kind: Kind::ClosedLeft,
            next: 2,
            sound: 62,
        },
    );
    w.doors.insert(
        3,
        Door {
            kind: Kind::ClosedRight,
            next: 4,
            sound: 62,
        },
    );
    let partner = Location::new(pos(3200, 3201), 3, 0, 0).unwrap();
    w.locations.load(partner);
    loc_collision::toggle(
        &mut w.collision,
        partner.coords,
        0,
        0,
        w.location_types[&3],
        true,
    )
    .unwrap();
    click(&mut w, a, loc);
    w.tick().unwrap();
    assert!(w.locations.find_type(loc.coords, 1).is_none());
    assert!(w.locations.find_type(partner.coords, 3).is_none());
    assert!(w.locations.find_type(pos(3199, 3200), 2).is_some());
    assert!(w.locations.find_type(pos(3199, 3201), 4).is_some());
    for _ in 0..500 {
        w.tick().unwrap();
        w.drain_events();
    }
    assert_eq!(w.locations.find_type(loc.coords, 1), Some(loc));
    assert_eq!(w.locations.find_type(partner.coords, 3), Some(partner));
}
