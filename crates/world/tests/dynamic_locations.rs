use gielinor_world::{
    ground::{Object, Scope},
    loc_collision::{self, LocCollision},
    locations::{Changed, Event, Location},
    simulation::{World, WorldEvent},
    CoordGrid,
};
fn pos(x: i32, z: i32) -> CoordGrid {
    CoordGrid::new(x, z, 0).unwrap()
}
fn ty() -> LocCollision {
    LocCollision {
        width: 1,
        length: 1,
        block_walk: 2,
        block_range: true,
        break_route_finding: false,
    }
}
#[test]
fn world_location_respawn_restores_collision_and_preserves_mutation_order() {
    let mut world = World::default();
    let loc = Location::new(pos(3200, 3200), 1, 0, 0).unwrap();
    world.location_types.insert(1, ty());
    world.locations.load(loc);
    loc_collision::toggle(
        &mut world.collision,
        loc.coords,
        loc.shape,
        loc.angle,
        ty(),
        true,
    )
    .unwrap();
    assert_ne!(world.collision.get(3200, 3200, 0) & 128, 0);
    assert_ne!(world.collision.get(3199, 3200, 0) & 8, 0);
    world
        .objects
        .add(
            Object {
                coords: loc.coords,
                item: 1,
                count: 1,
                scope: Scope::Temporary,
                created: 0,
                receiver: None,
                owner: None,
            },
            &world.object_types,
            world.clock,
            i32::MAX,
            100,
        )
        .unwrap();
    assert_eq!(
        world.change_location_for(loc, false, 2).unwrap(),
        Changed::Map
    );
    let events = world.drain_events();
    assert!(
        matches!(events.as_slice(),[WorldEvent::Ground(_),WorldEvent::Location(Event::Delete(v))] if *v==loc)
    );
    assert_eq!(world.collision.get(3200, 3200, 0) & 128, 0);
    assert_eq!(world.collision.get(3199, 3200, 0) & 8, 0);
    world.tick().unwrap();
    assert!(world.locations.find_type(loc.coords, 1).is_none());
    world.drain_events();
    world.tick().unwrap();
    assert_eq!(world.locations.find_type(loc.coords, 1), Some(loc));
    assert_ne!(world.collision.get(3200, 3200, 0) & 128, 0);
    assert_ne!(world.collision.get(3199, 3200, 0) & 8, 0);
    assert!(world
        .drain_events()
        .into_iter()
        .any(|e| matches!(e,WorldEvent::Location(Event::Add(v))if v==loc)));
}
#[test]
fn invalid_dynamic_footprint_rolls_back_registry_and_keeps_live_collision() {
    let mut world = World::default();
    let loc = Location::new(pos(0, 3200), 1, 0, 0).unwrap();
    world.location_types.insert(1, ty());
    world.collision.set(0, 3200, 0, 0x200000);
    assert!(world.change_location(loc, true).is_err());
    assert!(world.locations.spawned(loc.coords).next().is_none());
    assert_eq!(world.collision.get(0, 3200, 0), 0x200000);
    assert!(world.drain_events().is_empty());
}
