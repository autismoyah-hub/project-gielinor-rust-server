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

fn chest_setup(inside: bool) -> (World, ActorId, Location) {
    use gielinor_world::{
        tower_chest::{Config, HandlerConfig},
        tower_diary::Progress,
    };
    let (mut w, actor, loc) = setup(inside);
    w.location_handlers.remove(&(loc.id, 1));
    w.tower_chest = Some(HandlerConfig {
        location: loc.id,
        origin: CoordGrid::new(10368, 64, 0).unwrap(),
        skills: [12, 13, 11],
        bones: 526,
        inventory: Config {
            ids: std::array::from_fn(|i| 100 + i as i32),
            merge_aliases: [[102].into(), [109].into(), [110].into()],
        },
    });
    w.object_types.stackable.extend([102, 109, 110]);
    w.data_mut(actor).unwrap().tower_diary = Some(Progress {
        counts: [5, 0, 0, 0, 0, 0],
        bounty: 1,
        schema11: true,
        ..Default::default()
    });
    (w, actor, loc)
}
fn chest_search(w: &mut World, actor: ActorId, loc: Location) -> Vec<WorldEvent> {
    search(w, actor, loc);
    let mut events = Vec::new();
    for _ in 0..18 {
        w.tick().unwrap();
        events.extend(w.drain_events());
    }
    events
}
#[test]
fn chest_route_grants_bounty_once_and_keeps_payment_in_save() {
    let (mut w, actor, loc) = chest_setup(true);
    assert!(w.has_location_handler(loc.id, 1));
    assert!(!w.has_location_handler(loc.id, 2));
    let events = chest_search(&mut w, actor, loc);
    let observed: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            WorldEvent::GameMessage { text, .. } => Some(text.as_str()),
            WorldEvent::TowerDiary { .. } => Some("DIARY"),
            _ => None,
        })
        .collect();
    assert_eq!(observed, ["The chest yields Bronze axe.", "DIARY"]);
    assert!(w.data(actor).unwrap().tower_diary.unwrap().done[0]);
    assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 100);
    let events = chest_search(&mut w, actor, loc);
    assert!(events
        .iter()
        .any(|e| matches!(e,WorldEvent::GameMessage{text,..}if text=="The chest is empty.")));
    assert_eq!(
        w.data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .count(),
        1
    );
    w.request_logout(actor).unwrap();
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e,WorldEvent::Save{data,..}if data.tower_diary.unwrap().done[0])));
}
#[test]
fn chest_outside_tower_or_unknown_marks_never_grants_items() {
    for known in [true, false] {
        let (mut w, actor, loc) = chest_setup(!known);
        if !known {
            w.data_mut(actor).unwrap().tower_diary = None;
        }
        let events = chest_search(&mut w, actor, loc);
        assert_eq!(
            w.data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .count(),
            0
        );
        assert!(!events
            .iter()
            .any(|e| matches!(e, WorldEvent::TowerDiary { .. })));
        if !known {
            assert!(events.iter().any(|e| matches!(
                e,
                WorldEvent::ActionRejected {
                    error: gielinor_world::simulation::ActionError::UnknownTowerDiary,
                    ..
                }
            )));
        }
    }
}
#[test]
fn chest_partial_grant_keeps_original_bounty_and_emits_no_diary_refresh() {
    use gielinor_world::inventory::{Inventory, Item, StackType};
    let (mut w, actor, loc) = chest_setup(true);
    let data = w.data_mut(actor).unwrap();
    data.inventory = Inventory::new(1, StackType::Normal, false);
    data.bank
        .restore_slot(
            0,
            Some(Item {
                id: 110,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    let diary = data.tower_diary.as_mut().unwrap();
    diary.done[..10].fill(true);
    diary.pending[1] = true;
    diary.bounty = 11;
    let before = *diary;
    let events = chest_search(&mut w, actor, loc);
    assert_eq!(w.data(actor).unwrap().tower_diary, Some(before));
    assert_eq!(
        w.data(actor).unwrap().inventory.get(0),
        Some(Item {
            id: 110,
            count: 50,
            vars: 0
        })
    );
    let messages: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let WorldEvent::GameMessage { text, .. } = e {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        messages,
        [
            "The chest yields 50 Coins.",
            "Your coins are waiting. Drop something."
        ]
    );
    assert!(!events
        .iter()
        .any(|e| matches!(e, WorldEvent::TowerDiary { .. })));
}

fn staff_setup(inside: bool) -> (World, ActorId, Location) {
    let (mut w, actor, loc) = setup(inside);
    w.location_handlers.remove(&(loc.id, 1));
    w.tower_staff = Some(gielinor_world::tower_staff::HandlerConfig {
        location: loc.id,
        origin: CoordGrid::new(10368, 64, 0).unwrap(),
        inventory: gielinor_world::tower_staff::Config {
            staff: 1381,
            talisman: 1448,
            staff_names: [1381].into(),
            staff_owned: [1381].into(),
            talisman_owned: [1448].into(),
        },
    });
    (w, actor, loc)
}
#[test]
fn staff_crate_route_requires_known_offering_and_persists_actual_kit_once() {
    let (mut w, a, loc) = staff_setup(true);
    assert!(w.has_location_handler(loc.id, 1));
    let rejected = chest_search(&mut w, a, loc);
    assert!(rejected.iter().any(|e| matches!(
        e,
        WorldEvent::ActionRejected {
            error: gielinor_world::simulation::ActionError::UnknownTowerBones,
            ..
        }
    )));
    assert_eq!(w.data(a).unwrap().inventory.free_space(), 28);
    w.data_mut(a).unwrap().tower_arrival.bones_offered = Some(false);
    let sealed = chest_search(&mut w, a, loc);
    assert!(sealed.iter().any(|e|matches!(e,WorldEvent::GameMessage{text,..} if text=="Offer bones at the altar first. The crate is sealed.")));
    w.data_mut(a).unwrap().tower_arrival.bones_offered = Some(true);
    chest_search(&mut w, a, loc);
    assert!(w.data(a).unwrap().tower_arrival.staff_taken);
    assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 1381);
    assert_eq!(w.data(a).unwrap().inventory.get(1).unwrap().id, 1448);
    let repeated = chest_search(&mut w, a, loc);
    assert!(repeated
        .iter()
        .any(|e| matches!(e,WorldEvent::GameMessage{text,..} if text=="The crate is empty.")));
    assert_eq!(w.data(a).unwrap().inventory.free_space(), 26);
    w.request_logout(a).unwrap();
    assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Save{data,..} if data.tower_arrival.staff_taken && data.tower_arrival.bones_offered==Some(true))));
}
#[test]
fn staff_crate_outside_floor_and_stale_location_cannot_grant() {
    for inside in [false, true] {
        let (mut w, a, loc) = staff_setup(inside);
        w.data_mut(a).unwrap().tower_arrival.bones_offered = Some(true);
        search(&mut w, a, loc);
        if inside {
            w.locations.delete(loc);
        }
        for _ in 0..18 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(a).unwrap().inventory.free_space(), 28);
        assert!(!w.data(a).unwrap().tower_arrival.staff_taken);
    }
}

fn offering_setup() -> (World, ActorId, Location) {
    use gielinor_world::{inventory::Insert, prayer, tower_diary::Progress, tower_offering};
    let (mut w, a, old) = staff_setup(true);
    let origin = CoordGrid::new(10368, 64, 0).unwrap();
    let ty = w.location_types[&old.id];
    w.locations.delete(old);
    loc_collision::toggle(
        &mut w.collision,
        old.coords,
        old.shape,
        old.angle,
        ty,
        false,
    )
    .unwrap();
    let altar = Location::new(origin.translate(60, 52, 0).unwrap(), 77, 10, 0).unwrap();
    w.locations.load(altar);
    loc_collision::toggle(&mut w.collision, altar.coords, 10, 0, ty, true).unwrap();
    let staff = Location::new(origin.translate(58, 43, 0).unwrap(), 78, 10, 0).unwrap();
    w.locations.load(staff);
    loc_collision::toggle(&mut w.collision, staff.coords, 10, 0, ty, true).unwrap();
    w.tower_staff.as_mut().unwrap().location = 78;
    w.prayer = Some(prayer::Config {
        skill: 5,
        animation: 827,
        bones: [(526, 4.5), (532, 15.0)].into(),
        necklace: None,
        restore: Default::default(),
        worn_xp_percent: Default::default(),
        chaos: None,
    });
    w.tower_offering = Some(tower_offering::HandlerConfig {
        location: 77,
        origin,
        animation: 3705,
        accepted: [526, 532],
        bones: tower_offering::Bones {
            named: [526, 527, 532].into(),
        },
        xp: [(526, Some(4.5)), (527, None), (532, Some(15.0))].into(),
        skills: [12, 13, 11],
        drop_bones: 526,
    });
    w.held_use_types.insert(
        526,
        gielinor_world::held_use::Type {
            id: 526,
            content: -1,
        },
    );
    w.held_use_types.insert(
        532,
        gielinor_world::held_use::Type {
            id: 532,
            content: -1,
        },
    );
    let mut progress = Progress {
        schema11: true,
        ..Default::default()
    };
    progress.done[0] = true;
    progress.counts[0] = 5;
    let p = w.data_mut(a).unwrap();
    p.tower_diary = Some(progress);
    p.tower_arrival.bones_offered = Some(false);
    p.inventory
        .insert(&Default::default(), Insert::new(526, 5))
        .unwrap();
    (w, a, altar)
}
fn offer(w: &mut World, a: ActorId, loc: Location, slot: i32) {
    w.submit(
        a,
        Action::UseLocation {
            coords: loc.coords,
            id: loc.id,
            slot,
            object: 526,
            control: false,
        },
    )
    .unwrap();
}
#[test]
fn altar_use_consumes_five_then_publishes_diary_and_walks_to_staff_crate() {
    use gielinor_world::prayer::Event as Prayer;
    let (mut w, a, altar) = offering_setup();
    offer(&mut w, a, altar, 0);
    let mut events = Vec::new();
    for _ in 0..160 {
        w.tick().unwrap();
        events.extend(w.drain_events());
    }
    let p = w.data(a).unwrap();
    assert_eq!(p.tower_diary.unwrap().counts[1], 5);
    assert_eq!(p.tower_diary.unwrap().bounty, 2);
    assert_eq!(p.stats.fine_xp(5), 450);
    assert_eq!(p.tower_arrival.bones_offered, Some(true));
    assert!(p.tower_arrival.staff_taken);
    let items = p
        .inventory
        .slots()
        .iter()
        .flatten()
        .map(|i| i.id)
        .collect::<Vec<_>>();
    assert_eq!(items, [1381, 1448]);
    let text = events
        .iter()
        .filter_map(|e| match e {
            WorldEvent::Prayer {
                event: Prayer::Message(s),
                ..
            } => Some((*s).to_owned()),
            WorldEvent::GameMessage { text, .. } => Some(text.clone()),
            WorldEvent::TowerDiary { .. } => Some("DIARY".into()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        &text[..5],
        &vec!["The gods are pleased with your offering.".to_string(); 5]
    );
    assert_eq!(
        &text[5..9],
        &[
            "DIARY",
            "The chest holds a needle and thread.",
            "Bones offered 5/5. Hides tanned 5/5.",
            "The altar accepts the offering. Search the staff crate."
        ]
    );
    assert!(text[9].starts_with("magicStand 56,40 now "));
    assert!(text[10].starts_with("crateStand 57,43 now "));
    assert_eq!(
        text[11],
        "You take a staff of air and a mind talisman. Craft mind runes at the hub altar."
    );
    assert_eq!(text.len(), 12);
}
#[test]
fn interrupted_altar_loop_keeps_xp_but_does_not_run_diary_continuation() {
    let (mut w, a, altar) = offering_setup();
    offer(&mut w, a, altar, 0);
    for _ in 0..30 {
        w.tick().unwrap();
        if w.data(a).unwrap().stats.fine_xp(5) > 0 {
            break;
        }
    }
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 90);
    w.submit(a, Action::BurySlot { slot: 0 }).unwrap();
    for _ in 0..20 {
        w.tick().unwrap();
    }
    let p = w.data(a).unwrap();
    assert_eq!(p.stats.fine_xp(5), 90);
    assert_eq!(p.tower_diary.unwrap().counts[1], 0);
    assert_eq!(p.tower_arrival.bones_offered, Some(false));
    assert_eq!(p.inventory.slots().iter().flatten().count(), 4);
}
#[test]
fn unknown_diary_rejects_altar_before_any_consumption() {
    let (mut w, a, altar) = offering_setup();
    w.data_mut(a).unwrap().tower_diary = None;
    offer(&mut w, a, altar, 0);
    for _ in 0..30 {
        w.tick().unwrap();
    }
    assert_eq!(
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .count(),
        5
    );
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::ActionRejected {
            error: gielinor_world::simulation::ActionError::UnknownTowerDiary,
            ..
        }
    )));
}
#[test]
fn altar_chooses_first_named_bones_and_missing_xp_errors_after_consumption() {
    use gielinor_world::inventory::Item;
    let (mut w, a, altar) = offering_setup();
    w.data_mut(a)
        .unwrap()
        .inventory
        .restore_slot(
            0,
            Some(Item {
                id: 527,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    offer(&mut w, a, altar, 1);
    for _ in 0..30 {
        w.tick().unwrap();
    }
    let p = w.data(a).unwrap();
    assert!(p.inventory.get(0).is_none());
    assert_eq!(p.inventory.slots().iter().flatten().count(), 4);
    assert_eq!(p.stats.fine_xp(5), 0);
    assert_eq!(p.tower_diary.unwrap().counts[1], 0);
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::ActionRejected {
            error: gielinor_world::simulation::ActionError::InvalidStat,
            ..
        }
    )));
}

#[test]
fn cancelling_crate_walk_retains_completed_offering_but_never_grants_kit() {
    let (mut w, a, altar) = offering_setup();
    offer(&mut w, a, altar, 0);
    for _ in 0..40 {
        w.tick().unwrap();
        if w.data(a).unwrap().tower_arrival.bones_offered == Some(true) {
            break;
        }
    }
    let p = w.data(a).unwrap();
    assert_eq!(p.tower_diary.unwrap().counts[1], 5);
    assert_eq!(p.stats.fine_xp(5), 450);
    assert!(!p.tower_arrival.staff_taken);
    w.cancel_location_target_action(a).unwrap();
    for _ in 0..160 {
        w.tick().unwrap();
    }
    let p = w.data(a).unwrap();
    assert_eq!(p.tower_arrival.bones_offered, Some(true));
    assert_eq!(p.tower_diary.unwrap().counts[1], 5);
    assert!(!p.tower_arrival.staff_taken);
    assert!(p.inventory.slots().iter().all(Option::is_none));
}

#[test]
fn altar_rechecks_selected_slot_after_routing_before_selecting_other_bones() {
    use gielinor_world::inventory::Item;
    let (mut w, a, altar) = offering_setup();
    offer(&mut w, a, altar, 0);
    w.tick().unwrap();
    // The player is still approaching. Four other valid bones do not permit
    // a stale item-on-location packet to start the content callback.
    w.data_mut(a)
        .unwrap()
        .inventory
        .restore_slot(
            0,
            Some(Item {
                id: 999,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    for _ in 0..40 {
        w.tick().unwrap();
    }
    let p = w.data(a).unwrap();
    assert_eq!(p.stats.fine_xp(5), 0);
    assert_eq!(p.tower_diary.unwrap().counts[1], 0);
    assert_eq!(p.tower_arrival.bones_offered, Some(false));
    assert_eq!(p.inventory.slots().iter().flatten().count(), 5);
}
