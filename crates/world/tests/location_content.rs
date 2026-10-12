use gielinor_world::{
    loc_collision::{self, LocCollision},
    location_content::Handler,
    locations::Location,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn setup(inside: bool) -> (World, ActorId, Location) {
    let origin = CoordGrid::new(10368, 64, 0).unwrap();
    let base = if inside {
        origin
    } else {
        CoordGrid::new(3200, 3200, 0).unwrap()
    };
    let mut world = World::default();
    for x in 32..80 {
        for z in 32..80 {
            world.collision.set(base.x() + x, base.z() + z, 0, 0);
        }
    }
    if inside {
        world.locations.register_constructed(origin, 128).unwrap();
    }
    let ty = LocCollision {
        width: 1,
        length: 1,
        block_walk: 2,
        block_range: true,
        break_route_finding: false,
    };
    world.location_types.insert(77, ty);
    world.location_types.insert(78, ty);
    let loc = Location::new(base.translate(53, 58, 0).unwrap(), 77, 10, 0).unwrap();
    world.locations.load(loc);
    loc_collision::toggle(&mut world.collision, loc.coords, 10, 0, ty, true).unwrap();
    world
        .location_handlers
        .insert((77, 1), Handler::VialCrate { origin, item: 227 });
    world
        .queue_login(
            1,
            PlayerData::new(
                1,
                "Location tester".into(),
                base.translate(56, 56, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    (world, actor, loc)
}
fn search(world: &mut World, actor: ActorId, loc: Location) {
    world
        .submit(
            actor,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
}
fn held(world: &World, actor: ActorId) -> i32 {
    world
        .data(actor)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .filter(|item| item.id == 227)
        .map(|item| item.count)
        .sum()
}
#[test]
fn crate_waits_for_reach_and_repeated_search_fills_only_free_slots_without_xp() {
    let (mut world, actor, loc) = setup(true);
    let xp = (0..25)
        .map(|s| world.data(actor).unwrap().stats.fine_xp(s))
        .collect::<Vec<_>>();
    search(&mut world, actor, loc);
    world.tick().unwrap();
    assert_eq!(held(&world, actor), 0);
    for _ in 0..15 {
        world.tick().unwrap();
    }
    assert_eq!(held(&world, actor), 5);
    let messages = world
        .drain_events()
        .into_iter()
        .filter(|e| matches!(e, WorldEvent::LocationMessage { .. }))
        .count();
    assert_eq!(messages, 1);
    for expected in [10, 15, 20, 25, 28, 28] {
        search(&mut world, actor, loc);
        world.tick().unwrap();
        assert_eq!(held(&world, actor), expected);
    }
    assert_eq!(world.locations.find_type(loc.coords, loc.id), Some(loc));
    assert_eq!(
        (0..25)
            .map(|s| world.data(actor).unwrap().stats.fine_xp(s))
            .collect::<Vec<_>>(),
        xp
    );
    assert!(world.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::LocationMessage {
            message: "You don't have enough inventory space.",
            ..
        }
    )));
}
#[test]
fn identical_campus_crate_does_not_grant_floor_items() {
    let (mut world, actor, loc) = setup(false);
    search(&mut world, actor, loc);
    for _ in 0..15 {
        world.tick().unwrap();
    }
    assert_eq!(held(&world, actor), 0);
    assert!(!world
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::LocationMessage { .. })));
}
#[test]
fn delayed_and_replaced_targets_cannot_complete_a_search() {
    let (mut world, actor, loc) = setup(true);
    world.delay_player(actor, 3).unwrap();
    search(&mut world, actor, loc);
    for _ in 0..6 {
        world.tick().unwrap();
    }
    assert_eq!(held(&world, actor), 0);
    search(&mut world, actor, loc);
    world.tick().unwrap();
    world
        .change_location(Location { id: 78, ..loc }, true)
        .unwrap();
    for _ in 0..15 {
        world.tick().unwrap();
    }
    assert_eq!(held(&world, actor), 0);
}

#[test]
fn altar_restores_after_arrival_delay_and_repeated_prayer_does_not_award_xp() {
    let (mut world, actor, loc) = setup(true);
    world.location_handlers.insert(
        (77, 1),
        Handler::PrayerAltar {
            origin: CoordGrid::new(10368, 64, 0).unwrap(),
            prayer: 5,
        },
    );
    let data = world.data_mut(actor).unwrap();
    data.stats.set_base(5, 43);
    data.stats.set_current(5, 0);
    data.stats.set_xp(5, 50339).unwrap();
    let before = data.stats.clone();
    search(&mut world, actor, loc);
    let mut last_move = 0;
    let mut previous = world.observe(actor).unwrap().coords;
    let mut restored = false;
    for _ in 0..15 {
        world.tick().unwrap();
        let observation = world.observe(actor).unwrap();
        if observation.coords != previous {
            last_move = observation.tick;
            previous = observation.coords;
        }
        if world.data(actor).unwrap().stats.current(5) == 43 {
            assert_eq!(observation.tick, last_move + 2);
            restored = true;
            break;
        }
    }
    assert!(restored);
    // A boosted level is reset too; a stationary repeat needs no arrival delay.
    world.data_mut(actor).unwrap().stats.set_current(5, 60);
    search(&mut world, actor, loc);
    world.tick().unwrap();
    assert_eq!(world.data(actor).unwrap().stats.current(5), 43);
    let mut expected = before;
    expected.set_current(5, 43);
    assert_eq!(world.data(actor).unwrap().stats, expected);
    assert_eq!(
        world
            .drain_events()
            .iter()
            .filter(|e| matches!(
                e,
                WorldEvent::LocationMessage {
                    message: "You recharge your Prayer points.",
                    ..
                }
            ))
            .count(),
        2
    );
}

#[test]
fn altar_is_inert_outside_floor_and_on_unopened_planes() {
    let origin = CoordGrid::new(10368, 64, 0).unwrap();
    for coords in [
        CoordGrid::new(3200, 3200, 0).unwrap(),
        origin.translate(50, 50, 2).unwrap(),
        origin.translate(128, 50, 0).unwrap(),
    ] {
        let mut player = PlayerData::new(1, "Altar".into(), coords, Stats::default());
        player.stats.set_current(5, 0);
        assert_eq!(
            Handler::PrayerAltar { origin, prayer: 5 }
                .apply(&mut player, &Default::default())
                .unwrap(),
            None
        );
        assert_eq!(player.stats.current(5), 0);
    }
}

#[test]
fn altar_levels_match_original_kotlin_stat_restore_vectors() {
    let origin = CoordGrid::new(10368, 64, 0).unwrap();
    let fixture = include_str!("fixtures/stat-ops.expected");
    let mut compared = 0;
    for line in fixture.lines() {
        let f = line.split_whitespace().collect::<Vec<_>>();
        if f[2] != "5" || f[5] != "false" {
            continue;
        }
        let mut player = PlayerData::new(1, "Prayer".into(), origin, Stats::default());
        player.stats.set_base(5, f[0].parse::<u8>().unwrap() as i8);
        player
            .stats
            .set_current(5, f[1].parse::<u8>().unwrap() as i8);
        let result = Handler::PrayerAltar { origin, prayer: 5 }
            .apply(&mut player, &Default::default())
            .unwrap();
        assert_eq!(result, Some("You recharge your Prayer points."));
        assert_eq!(
            player.stats.current(5) as u8,
            f[6].parse::<u8>().unwrap(),
            "{line}"
        );
        compared += 1;
    }
    assert!(compared > 300);
}
