use gielinor_world::{
    ground::{Handle, Object, Scope},
    ground_transfer::{Effect, PickupEffects, NO_SPACE, TAKEN},
    inventory::Item,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn pos(x: i32, z: i32) -> CoordGrid {
    CoordGrid::new(x, z, 0).unwrap()
}
fn setup(coords: CoordGrid) -> (World, ActorId) {
    let mut w = World::default();
    for x in 3192..3216 {
        for z in 3192..3216 {
            w.collision.set(x, z, 0, 0);
        }
    }
    w.pickup_effects = Some(PickupEffects {
        sound: 2582,
        animation: 832,
    });
    w.object_respawns.insert(1, 3);
    w.queue_login(
        1,
        PlayerData::new(77, "Pickup tester".into(), coords, Stats::default()),
    )
    .unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    w.set_observer_id(actor, 900).unwrap();
    w.drain_events();
    (w, actor)
}
fn spawn(w: &mut World, scope: Scope, receiver: Option<i64>) -> Handle {
    w.objects
        .add(
            Object {
                coords: pos(3200, 3200),
                item: 1,
                count: 1,
                scope,
                created: w.clock.0,
                receiver,
                owner: None,
            },
            &w.object_types,
            w.clock,
            i32::MAX,
            100,
        )
        .unwrap()
        .remove(0)
}
fn take(w: &mut World, a: ActorId) {
    w.submit(
        a,
        Action::TakeObject {
            coords: pos(3200, 3200),
            id: 1,
            control: false,
        },
    )
    .unwrap();
}
fn message(events: &[WorldEvent], expected: &str) -> bool {
    events
        .iter()
        .any(|e| matches!(e,WorldEvent::GroundAction{effect:Effect::Message(s),..}if *s==expected))
}
#[test]
fn close_pickup_updates_inventory_and_permanent_item_respawns() {
    let (mut w, a) = setup(pos(3200, 3200));
    spawn(&mut w, Scope::Permanent, None);
    take(&mut w, a);
    w.tick().unwrap();
    assert_eq!(
        w.data(a).unwrap().inventory.get(0),
        Some(Item {
            id: 1,
            count: 1,
            vars: 0
        })
    );
    assert_eq!(w.objects.count(), 0);
    let events = w.drain_events();
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::GroundAction {
            effect: Effect::Sound(2582),
            ..
        }
    )));
    for _ in 0..2 {
        w.tick().unwrap();
    }
    assert_eq!(w.objects.count(), 1);
}
#[test]
fn blocked_tile_pickup_waits_a_cycle_and_preserves_race_messages() {
    let (mut w, a) = setup(pos(3199, 3200));
    let object = spawn(&mut w, Scope::Temporary, None);
    w.collision.set(3200, 3200, 0, 0x100);
    take(&mut w, a);
    w.tick().unwrap();
    assert_eq!(w.objects.count(), 1);
    assert!(w.protection(a).unwrap().suspended);
    w.objects.delete(&object, w.clock, i32::MAX);
    w.drain_events();
    w.tick().unwrap();
    assert!(message(&w.drain_events(), TAKEN));
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
}
#[test]
fn private_pickup_uses_character_identity_and_rejects_wrong_receiver() {
    let (mut w, a) = setup(pos(3200, 3200));
    spawn(&mut w, Scope::Private, Some(77));
    take(&mut w, a);
    w.tick().unwrap();
    assert_eq!(w.objects.count(), 1);
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
    spawn(&mut w, Scope::Private, Some(900));
    take(&mut w, a);
    w.tick().unwrap();
    assert_eq!(w.objects.count(), 1);
    assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 1);
}
#[test]
fn full_inventory_keeps_object_but_capacity_change_during_delay_matches_kotlin_loss() {
    for during_delay in [false, true] {
        let (mut w, a) = setup(if during_delay {
            pos(3199, 3200)
        } else {
            pos(3200, 3200)
        });
        spawn(&mut w, Scope::Temporary, None);
        if during_delay {
            w.collision.set(3200, 3200, 0, 0x100);
            take(&mut w, a);
            w.tick().unwrap();
        }
        let inv = &mut w.data_mut(a).unwrap().inventory;
        for slot in 0..inv.slots().len() {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 2,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        if !during_delay {
            take(&mut w, a);
        }
        w.drain_events();
        w.tick().unwrap();
        assert!(message(&w.drain_events(), NO_SPACE));
        assert_eq!(w.objects.count(), usize::from(!during_delay));
    }
}
#[test]
fn walking_and_lenient_held_actions_cancel_their_respective_pickup_states() {
    let (mut w, a) = setup(pos(3196, 3200));
    spawn(&mut w, Scope::Temporary, None);
    take(&mut w, a);
    w.tick().unwrap();
    w.submit(
        a,
        Action::Walk {
            destination: pos(3196, 3199),
            run: false,
        },
    )
    .unwrap();
    for _ in 0..5 {
        w.tick().unwrap();
    }
    assert_eq!(w.objects.count(), 1);
    let (mut w, a) = setup(pos(3199, 3200));
    spawn(&mut w, Scope::Temporary, None);
    w.collision.set(3200, 3200, 0, 0x100);
    take(&mut w, a);
    w.tick().unwrap();
    w.submit(a, Action::EatSlot { slot: 0 }).unwrap();
    for _ in 0..3 {
        w.tick().unwrap();
    }
    assert_eq!(w.objects.count(), 1);
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
}

#[test]
fn normal_held_action_clears_pending_pickup_interaction_before_item_validation() {
    let (mut world, actor) = setup(pos(3196, 3200));
    spawn(&mut world, Scope::Temporary, None);
    take(&mut world, actor);
    world.tick().unwrap();
    world.submit(actor, Action::EatSlot { slot: 0 }).unwrap();
    for _ in 0..8 { world.tick().unwrap(); }
    assert_eq!(world.objects.count(), 1);
    assert!(world.data(actor).unwrap().inventory.get(0).is_none());
}
