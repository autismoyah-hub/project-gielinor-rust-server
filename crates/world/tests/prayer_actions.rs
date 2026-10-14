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
