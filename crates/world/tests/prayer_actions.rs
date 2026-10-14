use gielinor_world::{
    inventory::{Insert, Item},
    prayer::{Config, Event},
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn setup(chaos: bool) -> (World, ActorId) {
    let mut world = World::default();
    world.prayer = Some(Config {
        skill: 5,
        animation: 827,
        bones: [(526, 4.5)].into(),
        necklace: Some(22111),
        restore: [(526, 1)].into(),
        worn_xp_percent: [(900, 3)].into(),
        chaos: Some([2947, 3817, 2958, 3825]),
    });
    let coords = if chaos {
        CoordGrid::new(2950, 3820, 0).unwrap()
    } else {
        CoordGrid::new(3200, 3200, 0).unwrap()
    };
    for x in coords.x() - 8..coords.x() + 8 {
        for z in coords.z() - 8..coords.z() + 8 {
            world.collision.set(x, z, 0, 0);
        }
    }
    let mut data = PlayerData::new(1, "Burial".into(), coords, Stats::default());
    data.xp_rate_hundreds = 500;
    data.inventory
        .insert(&world.object_types, Insert::new(526, 2))
        .unwrap();
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    (world, actor)
}
fn count(world: &World, actor: ActorId) -> usize {
    world
        .data(actor)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .filter(|i| i.id == 526)
        .count()
}
fn bury(world: &mut World, actor: ActorId) {
    world.submit(actor, Action::BurySlot { slot: 1 }).unwrap();
    world.tick().unwrap();
}
fn token(world: &mut World) -> u64 {
    world
        .drain_events()
        .into_iter()
        .find_map(|e| match e {
            WorldEvent::Prayer {
                event: Event::Confirm { token },
                ..
            } => Some(token),
            _ => None,
        })
        .unwrap()
}
#[test]
fn animation_precedes_one_cycle_consumption_and_first_matching_slot_is_deleted() {
    let (mut w, a) = setup(false);
    bury(&mut w, a);
    assert_eq!(count(&w, a), 2);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Prayer {
            event: Event::Animation(827),
            ..
        }
    )));
    w.tick().unwrap();
    assert_eq!(count(&w, a), 1);
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
    assert!(w.data(a).unwrap().inventory.get(1).is_some());
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 225);
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Prayer {
            event: Event::Message("You bury the bones."),
            ..
        }
    )));
    for _ in 0..4 {
        w.tick().unwrap();
    }
    assert_eq!(count(&w, a), 1);
}
#[test]
fn tutorial_prayer_cap_runs_after_burial_resumes_in_the_same_main_pass() {
    let (mut world, actor) = setup(false);
    world.tutorial_rules_completed_varp = Some(900);
    let stats = &mut world.data_mut(actor).unwrap().stats;
    stats.set_base(5, 3);
    stats.set_current(5, 3);
    stats.set_xp(5, 385).unwrap();
    bury(&mut world, actor);
    assert_eq!(world.data(actor).unwrap().stats.xp(5), 385);
    world.drain_events();
    world.tick().unwrap();
    let stats = &world.data(actor).unwrap().stats;
    assert_eq!((stats.base(5), stats.current(5), stats.xp(5)), (3, 3, 174));
    assert_eq!(count(&world, actor), 1);
    let events = world.drain_events();
    let buried = events
        .iter()
        .position(|e| {
            matches!(
                e,
                WorldEvent::Prayer {
                    event: Event::Completed { .. },
                    ..
                }
            )
        })
        .unwrap();
    let capped=events.iter().position(|e| matches!(e,WorldEvent::LocationMessage {message,..} if *message==gielinor_world::tutorial_rules::CAP_MESSAGE)).unwrap();
    assert!(buried < capped);
}
#[test]
fn removed_bone_during_delay_grants_no_xp_or_message() {
    let (mut w, a) = setup(false);
    bury(&mut w, a);
    w.drain_events();
    for slot in 0..2 {
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(slot, None)
            .unwrap();
    }
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
    assert!(!w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Prayer {
            event: Event::Completed { .. },
            ..
        }
    )));
}
#[test]
fn repeated_held_input_cancels_old_coroutine_before_delayed_rejection() {
    let (mut w, a) = setup(false);
    bury(&mut w, a);
    bury(&mut w, a);
    for _ in 0..3 {
        w.tick().unwrap();
    }
    assert_eq!(count(&w, a), 2);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
}
#[test]
fn chaos_choice_cancellation_stale_token_and_duplicate_confirmation_are_safe() {
    let (mut w, a) = setup(true);
    bury(&mut w, a);
    let first = token(&mut w);
    w.submit(
        a,
        Action::ResumeBury {
            token: first,
            choice: 2,
        },
    )
    .unwrap();
    w.tick().unwrap();
    assert_eq!(count(&w, a), 2);
    bury(&mut w, a);
    let second = token(&mut w);
    assert_ne!(first, second);
    w.submit(
        a,
        Action::ResumeBury {
            token: first,
            choice: 1,
        },
    )
    .unwrap();
    w.tick().unwrap();
    assert_eq!(count(&w, a), 2);
    for _ in 0..2 {
        w.submit(
            a,
            Action::ResumeBury {
                token: second,
                choice: 1,
            },
        )
        .unwrap();
    }
    w.tick().unwrap();
    assert_eq!(count(&w, a), 2);
    w.tick().unwrap();
    assert_eq!(count(&w, a), 1);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 225);
}
#[test]
fn moving_away_cancels_chaos_confirmation() {
    let (mut w, a) = setup(true);
    bury(&mut w, a);
    let pending = token(&mut w);
    let destination = w.observe(a).unwrap().coords.translate(1, 0, 0).unwrap();
    w.submit(
        a,
        Action::Walk {
            destination,
            run: false,
        },
    )
    .unwrap();
    w.tick().unwrap();
    w.submit(
        a,
        Action::ResumeBury {
            token: pending,
            choice: 1,
        },
    )
    .unwrap();
    for _ in 0..3 {
        w.tick().unwrap();
    }
    assert_eq!(count(&w, a), 2);
}
#[test]
fn necklace_is_captured_before_delay_and_worn_xp_is_evaluated_at_completion() {
    let (mut w, a) = setup(false);
    let p = w.data_mut(a).unwrap();
    p.stats.set_base(5, 10);
    p.stats.set_current(5, 2);
    p.equipment
        .restore_slot(
            2,
            Some(Item {
                id: 22111,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    bury(&mut w, a);
    let p = w.data_mut(a).unwrap();
    p.equipment.restore_slot(2, None).unwrap();
    p.equipment
        .restore_slot(
            0,
            Some(Item {
                id: 900,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().stats.current(5), 3);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 231);
}

fn offering_results(events: &[WorldEvent]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|e| match e {
            WorldEvent::Prayer {
                event: Event::OfferingFinished { offered },
                ..
            } => Some(*offered),
            _ => None,
        })
        .collect()
}
#[test]
fn chapel_offers_selected_type_every_three_cycles_and_returns_once() {
    let (mut w, a) = setup(true); // Chaos confirmation belongs only to burial.
    let catalog = gielinor_world::inventory::Catalog::default();
    w.data_mut(a)
        .unwrap()
        .inventory
        .insert(&catalog, Insert::new(532, 1))
        .unwrap();
    w.begin_altar_offering(a, 526, 3705).unwrap();
    let start = w.drain_events();
    assert!(start.iter().any(|e| matches!(
        e,
        WorldEvent::Prayer {
            event: Event::Animation(3705),
            ..
        }
    )));
    assert!(!start.iter().any(|e| matches!(
        e,
        WorldEvent::Prayer {
            event: Event::Confirm { .. },
            ..
        }
    )));
    // This trusted call is made between ticks, before the start clock's main
    // pass. Process that clock plus the next two; consumption occurs at start+3.
    let started = w.clock.0;
    for _ in 0..3 {
        w.tick().unwrap();
        assert_eq!(count(&w, a), 2);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
    }
    w.tick().unwrap();
    assert_eq!(w.clock.0 - 1, started + 3);
    assert_eq!(count(&w, a), 1);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 450);
    let first = w.drain_events();
    assert!(offering_results(&first).is_empty());
    let message = first
        .iter()
        .position(|e| {
            matches!(
                e,
                WorldEvent::Prayer {
                    event: Event::Message("The gods are pleased with your offering."),
                    ..
                }
            )
        })
        .unwrap();
    let next = first
        .iter()
        .position(|e| {
            matches!(
                e,
                WorldEvent::Prayer {
                    event: Event::Animation(3705),
                    ..
                }
            )
        })
        .unwrap();
    assert!(message < next);
    for _ in 0..2 {
        w.tick().unwrap();
        assert_eq!(count(&w, a), 1);
    }
    w.tick().unwrap();
    assert_eq!(count(&w, a), 0);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 900);
    assert!(w
        .data(a)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .any(|i| i.id == 532));
    assert_eq!(offering_results(&w.drain_events()), vec![true]);
    for _ in 0..4 {
        w.tick().unwrap();
    }
    assert!(offering_results(&w.drain_events()).is_empty());
}
#[test]
fn chapel_does_not_restore_necklace_points_and_reads_worn_modifier_after_delay() {
    let (mut w, a) = setup(false);
    let p = w.data_mut(a).unwrap();
    p.stats.set_base(5, 10);
    p.stats.set_current(5, 2);
    p.equipment
        .restore_slot(
            2,
            Some(Item {
                id: 22111,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    w.begin_altar_offering(a, 526, 3705).unwrap();
    w.tick().unwrap();
    w.data_mut(a)
        .unwrap()
        .equipment
        .restore_slot(
            0,
            Some(Item {
                id: 900,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    w.tick().unwrap();
    w.tick().unwrap();
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().stats.current(5), 2);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 463);
}
#[test]
fn chapel_rechecks_missing_bones_after_delay_and_reports_actual_return_value() {
    for completed_first in [false, true] {
        let (mut w, a) = setup(false);
        w.begin_altar_offering(a, 526, 3705).unwrap();
        if completed_first {
            for _ in 0..4 {
                w.tick().unwrap();
            }
        }
        w.drain_events();
        for slot in 0..2 {
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(slot, None)
                .unwrap();
        }
        for _ in 0..4 {
            w.tick().unwrap();
        }
        let events = w.drain_events();
        assert_eq!(offering_results(&events), vec![completed_first]);
        assert!(!events.iter().any(|e| matches!(
            e,
            WorldEvent::Prayer {
                event: Event::Completed { .. },
                ..
            }
        )));
        assert_eq!(
            w.data(a).unwrap().stats.fine_xp(5),
            if completed_first { 450 } else { 0 }
        );
    }
}
#[test]
fn held_action_cancellation_keeps_consumed_offerings_without_returning_success() {
    let (mut w, a) = setup(false);
    w.begin_altar_offering(a, 526, 3705).unwrap();
    for _ in 0..4 {
        w.tick().unwrap();
    }
    assert_eq!(count(&w, a), 1);
    w.drain_events();
    // launchLenient cancels the suspended coroutine before its delay rejection.
    w.submit(a, Action::BurySlot { slot: 1 }).unwrap();
    for _ in 0..6 {
        w.tick().unwrap();
    }
    assert_eq!(count(&w, a), 1);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 450);
    assert!(offering_results(&w.drain_events()).is_empty());
}
