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
fn furnace_reach_and_tutorial_gate_request_real_selection_without_consuming_ore() {
    use gielinor_world::{
        smelting::Furnace,
        tutorial_progression::Config,
        variables::{Transmit, Varp},
    };
    for (step, done, allowed) in [(24, 0, false), (25, 0, true), (50, 0, true), (0, 1, true)] {
        let (mut world, actor, loc) = setup(10, pos(3204, 3200));
        world.doors.clear();
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Different,
            bit_protect: false,
            client_code: -1,
        };
        let progression = Config {
            step: varp(7000),
            completed: varp(7001),
        };
        world
            .data_mut(actor)
            .unwrap()
            .variables
            .set(progression.step, step, false);
        world
            .data_mut(actor)
            .unwrap()
            .variables
            .set(progression.completed, done, false);
        world.furnaces.insert(
            loc.id,
            Furnace {
                operations: 1 << 2,
                item_use: false,
                ore_ids: None,
                tutorial: Some(progression),
            },
        );
        assert!(!world.has_location_handler(loc.id, 1));
        assert!(world.has_location_handler(loc.id, 2));
        let inventory = world.data(actor).unwrap().inventory.clone();
        world
            .submit(
                actor,
                Action::OperateLocation {
                    coords: loc.coords,
                    id: loc.id,
                    operation: 2,
                    control: false,
                },
            )
            .unwrap();
        for _ in 0..10 {
            world.tick().unwrap();
        }
        let events = world.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::SmeltingRequested { .. }))
                .count(),
            usize::from(allowed)
        );
        assert_eq!(events.iter().filter(|e| matches!(e, WorldEvent::LocationMessage { message: "This is a furnace for smelting metal. You'll learn how to use it soon.", .. })).count(), usize::from(!allowed));
        assert_eq!(world.data(actor).unwrap().inventory, inventory);
        assert_eq!(world.data(actor).unwrap().variables.get(7000), step);
        assert!(world.locations.find_type(loc.coords, loc.id).is_some());
    }
}
#[test]
fn tutorial_tree_ops_route_through_reach_and_oak_never_awards_logs() {
    use gielinor_world::{
        tutorial_progression, tutorial_woodcutting,
        variables::{Transmit, Varp},
    };
    let (mut w, actor, loc) = setup(10, pos(3197, 3200));
    w.doors.clear();
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    w.tutorial_woodcutting = Some(tutorial_woodcutting::Config {
        progression: tutorial_progression::Config {
            step: varp(5003),
            completed: varp(5004),
        },
        trees: [1, 2],
        oak: 3,
        logs: 10,
        animation: 879,
        skill: 8,
        max_level: 99,
        global_xp_rate: 1.0,
    });
    assert!(w.has_location_handler(1, 1));
    assert!(w.has_location_handler(3, 1));
    assert!(!w.has_location_handler(1, 2));
    w.data_mut(actor)
        .unwrap()
        .variables
        .set(varp(5003), 10, true);
    for cuts in 1..=2 {
        click(&mut w, actor, loc);
        let mut events = vec![];
        for _ in 0..10 {
            w.tick().unwrap();
            events.extend(w.drain_events());
        }
        assert_eq!(w.data(actor).unwrap().stats.xp(8), 25 * cuts);
        assert_eq!(w.data(actor).unwrap().variables.get(5003), 11);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 8, .. }))
                .count(),
            1
        );
    }
    w.data_mut(actor)
        .unwrap()
        .variables
        .set(varp(5003), 12, true);
    click(&mut w, actor, loc);
    let mut events = vec![];
    for _ in 0..4 {
        w.tick().unwrap();
        events.extend(w.drain_events());
    }
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::LocationMessage {
            message: "Perhaps you've done enough woodcutting now.",
            ..
        }
    )));
    assert_eq!(w.data(actor).unwrap().stats.xp(8), 50);
    let oak = Location::new(loc.coords, 3, 10, 0).unwrap();
    w.change_location(oak, true).unwrap();
    click(&mut w, actor, oak);
    let mut events = vec![];
    for _ in 0..4 {
        w.tick().unwrap();
        events.extend(w.drain_events());
    }
    assert!(events.iter().any(|e|matches!(e,WorldEvent::LocationMessage {message,..} if *message==tutorial_woodcutting::OAK)));
    assert_eq!(w.data(actor).unwrap().stats.xp(8), 50);
    assert_eq!(
        w.data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 10)
            .count(),
        2
    );
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

#[test]
fn tutorial_furnace_item_use_is_bound_only_to_tin_and_copper_and_rechecks_slot() {
    use gielinor_world::{
        held_use,
        inventory::Item,
        smelting::Furnace,
        tutorial_progression::Config,
        variables::{Transmit, Varp},
    };
    for step in [24, 25] {
        for object in [436, 438, 440] {
            for stale in [false, true] {
                let (mut world, actor, loc) = setup(10, pos(3204, 3200));
                world.doors.clear();
                let varp = |id| Varp {
                    id,
                    permanent: true,
                    transmit: Transmit::Different,
                    bit_protect: false,
                    client_code: -1,
                };
                let progression = Config {
                    step: varp(7000),
                    completed: varp(7001),
                };
                world
                    .data_mut(actor)
                    .unwrap()
                    .variables
                    .set(progression.step, step, false);
                world.furnaces.insert(
                    loc.id,
                    Furnace {
                        operations: 1 << 1,
                        item_use: true,
                        ore_ids: Some([438, 436]),
                        tutorial: Some(progression),
                    },
                );
                // Even a generic smelting ore must not acquire a tutorial-specific binding.
                world.smelting_ores.extend([436, 438, 440]);
                world.held_use_types.insert(
                    object,
                    held_use::Type {
                        id: object,
                        content: 51,
                    },
                );
                world
                    .data_mut(actor)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: if stale { 999 } else { object },
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                let before = world.data(actor).unwrap().inventory.slots().to_vec();
                world
                    .submit(
                        actor,
                        Action::UseLocation {
                            coords: loc.coords,
                            id: loc.id,
                            slot: 0,
                            object,
                            control: false,
                        },
                    )
                    .unwrap();
                for _ in 0..10 {
                    world.tick().unwrap();
                }
                let events = world.drain_events();
                let bound = object != 440 && !stale;
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| matches!(e, WorldEvent::SmeltingRequested { .. }))
                        .count(),
                    usize::from(bound && step == 25),
                    "step={step} object={object} stale={stale}"
                );
                assert_eq!(events.iter().filter(|e| matches!(e, WorldEvent::LocationMessage { message: "This is a furnace for smelting metal. You'll learn how to use it soon.", .. })).count(), usize::from(bound && step == 24));
                assert_eq!(world.data(actor).unwrap().inventory.slots(), before);
            }
        }
    }
}

#[test]
fn anvil_reach_opens_grid_and_only_bar_content_supplies_preferred_tier() {
    use gielinor_world::{held_use, inventory::Item};
    for object in [None, Some(2349), Some(2357)] {
        let (mut world, actor, loc) = setup(10, pos(3204, 3200));
        world.doors.clear();
        world.anvils.insert(loc.id);
        world.smithing_bars.insert(2349);
        assert!(world.has_location_handler(loc.id, 1));
        assert!(!world.has_location_handler(loc.id, 2));
        let action = if let Some(object) = object {
            world.held_use_types.insert(
                object,
                held_use::Type {
                    id: object,
                    content: if object == 2349 { 50 } else { -1 },
                },
            );
            world
                .data_mut(actor)
                .unwrap()
                .inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: object,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            Action::UseLocation {
                coords: loc.coords,
                id: loc.id,
                slot: 0,
                object,
                control: false,
            }
        } else {
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            }
        };
        let before = world.data(actor).unwrap().inventory.clone();
        world.submit(actor, action).unwrap();
        for _ in 0..10 {
            world.tick().unwrap();
        }
        let events = world.drain_events();
        let selections: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                WorldEvent::AnvilRequested { preferred, .. } => Some(*preferred),
                _ => None,
            })
            .collect();
        assert_eq!(
            selections,
            if object == Some(2357) {
                vec![]
            } else {
                vec![object]
            }
        );
        assert_eq!(world.data(actor).unwrap().inventory, before);
    }
}

#[test]
fn tutorial_booths_gate_at_reach_and_publish_one_request_without_repeating_gifts() {
    use gielinor_world::{
        inventory::Item,
        tutorial_bank::{Booth, Config},
        tutorial_progression,
        tutorial_steps::Step,
        variables::{Transmit, Variables, Varp},
    };
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    for (booth, raw, done, allowed, next) in [
        (Booth::Bank, 40, 0, false, None),
        (Booth::Bank, 41, 0, true, Some(Step::PollView)),
        (Booth::Bank, 43, 0, true, None),
        (Booth::Bank, 0, 1, true, Some(Step::PollView)),
        (Booth::Poll, 41, 0, false, None),
        (Booth::Poll, 42, 0, true, Some(Step::AccountTalk)),
        (Booth::Poll, 44, 0, true, None),
    ] {
        for coins in [0, 1, 25] {
            let (mut w, actor, loc) = setup(10, pos(3204, 3200));
            w.doors.clear();
            w.tutorial_bank = Some(Config {
                progression: tutorial_progression::Config {
                    step: varp(7000),
                    completed: varp(7001),
                },
                bank: if booth == Booth::Bank { loc.id } else { 999 },
                poll: if booth == Booth::Poll { loc.id } else { 999 },
                coins: 995,
            });
            w.object_types.stackable.insert(995);
            let data = w.data_mut(actor).unwrap();
            data.variables = Variables::restore([(7000, raw), (7001, done)].into());
            if coins > 0 {
                data.bank
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 995,
                            count: coins,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            let before = data.inventory.clone();
            click(&mut w, actor, loc);
            assert_eq!(
                w.data(actor).unwrap().bank.get(0).map_or(0, |i| i.count),
                coins
            );
            for _ in 0..12 {
                w.tick().unwrap();
            }
            let events = w.drain_events();
            let requests: Vec<_> = events
                .iter()
                .filter_map(|e| match e {
                    WorldEvent::TutorialBank { booth, next, .. } => Some((*booth, *next)),
                    _ => None,
                })
                .collect();
            assert_eq!(requests, if allowed { vec![(booth, next)] } else { vec![] });
            assert_eq!(
                w.data(actor).unwrap().bank.get(0).map_or(0, |i| i.count),
                if allowed && booth == Booth::Bank && coins == 0 {
                    25
                } else {
                    coins
                }
            );
            assert_eq!(w.data(actor).unwrap().inventory, before);
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::LocationMessage { .. }))
                    .count(),
                usize::from(!allowed)
            );
        }
    }
}
