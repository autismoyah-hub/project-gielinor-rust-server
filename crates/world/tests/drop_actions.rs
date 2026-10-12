use gielinor_world::{
    bank_actions::Binding,
    drop_actions::{Config, Event, Kind, Pending, Rule, Stage},
    ground::Scope,
    inventory::Item,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    variables::{Transmit, Varbit, Varp},
    CoordGrid,
};
fn rule(kind: Kind) -> Rule {
    Rule {
        kind,
        name: "Test Item".into(),
        market_price: 100,
        destroy_title: "Destroy?".into(),
        destroy_description: "Cannot undo".into(),
        release_title: None,
        release_message: Some("Released.".into()),
    }
}
fn binding(id: u16) -> Binding {
    Binding {
        base: Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: id,
            lsb: 0,
            msb: 30,
        },
    }
}
fn item(id: i32, count: i32) -> Item {
    Item { id, count, vars: 0 }
}
#[test]
fn close_dialogs_invalidates_pause_without_destroying_the_item() {
    let (mut world, actor) = setup(Kind::Destroy, 1, false);
    world.submit(actor, Action::DropSlot { slot: 0 }).unwrap();
    world.tick().unwrap();
    let token = world.drop_dialog_token(actor).unwrap().unwrap();
    assert!(world.close_dialogs(actor).unwrap());
    assert!(!world.close_dialogs(actor).unwrap());
    world
        .submit(actor, Action::ResumeDrop { token, choice: 1 })
        .unwrap();
    world.tick().unwrap();
    assert_eq!(
        world.data(actor).unwrap().inventory.get(0),
        Some(item(1, 1))
    );
    assert!(!world.protection(actor).unwrap().suspended);
}
fn setup(kind: Kind, count: i32, warning: bool) -> (World, ActorId) {
    let mut world = World::default();
    world.object_types.stackable.extend([1, 2]);
    let config = Config {
        rules: [(1, rule(kind))].into(),
        warning_enabled: binding(0),
        warning_threshold: binding(1),
        sound: 2739,
    };
    let mut data = PlayerData::new(
        77,
        "Drop tester".into(),
        CoordGrid::new(3200, 3200, 0).unwrap(),
        Stats::default(),
    );
    data.inventory
        .restore_slot(0, Some(item(1, count)))
        .unwrap();
    config
        .warning_enabled
        .set(&mut data.variables, i32::from(warning))
        .unwrap();
    config
        .warning_threshold
        .set(&mut data.variables, 100)
        .unwrap();
    world.dropping = Some(config);
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.set_observer_id(actor, 900).unwrap();
    world.drain_events();
    (world, actor)
}
fn act(world: &mut World, actor: ActorId, action: Action) {
    world.submit(actor, action).unwrap();
    world.tick().unwrap();
}
fn answer(world: &mut World, actor: ActorId, choice: i32) {
    let token = world.drop_dialog_token(actor).unwrap().unwrap();
    act(world, actor, Action::ResumeDrop { token, choice });
}
#[test]
fn warning_price_uses_jvm_wrapping_and_release_only_confirms_stacks() {
    let mut r = rule(Kind::Drop);
    r.market_price = i32::MAX;
    assert_eq!(
        Pending::begin(0, item(1, 2), &r, true, 100).stage,
        Stage::Ready
    );
    assert_eq!(
        Pending::begin(0, item(1, 1), &r, true, 100).stage,
        Stage::WarningBox
    );
    r.kind = Kind::Release;
    assert_eq!(
        Pending::begin(0, item(1, 1), &r, false, 0).stage,
        Stage::Ready
    );
    assert_eq!(
        Pending::begin(0, item(1, 2), &r, false, 0).stage,
        Stage::ReleaseChoice
    );
}
#[test]
fn private_drop_reveals_and_expires_on_source_schedule() {
    let (mut w, a) = setup(Kind::Drop, 7, false);
    act(&mut w, a, Action::DropSlot { slot: 0 });
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
    let pos = w.data(a).unwrap().coords;
    let obj = w.objects.at(pos).next().unwrap().snapshot();
    assert_eq!(
        (obj.item, obj.count, obj.scope, obj.receiver),
        (1, 7, Scope::Private, Some(900))
    );
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Drop {
            event: Event::Sound(2739),
            ..
        }
    )));
    for _ in 0..100 {
        w.tick().unwrap();
    }
    assert_ne!(
        w.objects.at(pos).next().unwrap().snapshot().scope,
        Scope::Private
    );
    for _ in 0..100 {
        w.tick().unwrap();
    }
    assert_eq!(w.objects.count(), 0);
}
#[test]
fn warning_pages_reject_replayed_tokens_and_drop_current_slot() {
    let (mut w, a) = setup(Kind::Drop, 7, true);
    act(&mut w, a, Action::DropSlot { slot: 0 });
    let token = w.drop_dialog_token(a).unwrap().unwrap();
    w.submit(a, Action::ResumeDrop { token, choice: 1 })
        .unwrap();
    w.submit(a, Action::ResumeDrop { token, choice: 1 })
        .unwrap();
    w.tick().unwrap();
    assert!(w.drop_dialog_token(a).unwrap().is_some());
    assert_eq!(w.objects.count(), 0);
    w.data_mut(a)
        .unwrap()
        .inventory
        .restore_slot(0, Some(item(2, 3)))
        .unwrap();
    answer(&mut w, a, 1);
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
    assert_eq!(
        w.objects
            .at(w.data(a).unwrap().coords)
            .next()
            .unwrap()
            .snapshot()
            .item,
        2
    );
}
#[test]
fn destroy_and_release_reject_changed_slot_then_commit_unchanged_items() {
    for kind in [Kind::Destroy, Kind::Release] {
        let (mut w, a) = setup(kind, 7, false);
        act(&mut w, a, Action::DropSlot { slot: 0 });
        let inv = &mut w.data_mut(a).unwrap().inventory;
        inv.restore_slot(0, Some(item(2, 3))).unwrap();
        inv.restore_slot(4, Some(item(1, 9))).unwrap();
        answer(&mut w, a, 1);
        assert_eq!(w.data(a).unwrap().inventory.get(0), Some(item(2, 3)));
        assert_eq!(w.data(a).unwrap().inventory.get(4), Some(item(1, 9)));
        assert_eq!(w.objects.count(), 0);
        assert!(!w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::Drop {
                event: Event::Committed { .. } | Event::Message(_),
                ..
            }
        )));
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(0, Some(item(1, 7)))
            .unwrap();
        act(&mut w, a, Action::DropSlot { slot: 0 });
        answer(&mut w, a, 1);
        assert!(w.data(a).unwrap().inventory.get(0).is_none());
        let messages = w
            .drain_events()
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    WorldEvent::Drop {
                        event: Event::Message(_),
                        ..
                    }
                )
            })
            .count();
        assert_eq!(messages, usize::from(kind == Kind::Release));
    }
}
#[test]
fn cancelled_dialogs_never_consume_items() {
    for (kind, choice) in [(Kind::Destroy, 0), (Kind::Release, 2), (Kind::Drop, 2)] {
        let (mut w, a) = setup(kind, 7, true);
        act(&mut w, a, Action::DropSlot { slot: 0 });
        if kind == Kind::Drop {
            answer(&mut w, a, 0);
        }
        answer(&mut w, a, choice);
        assert_eq!(w.data(a).unwrap().inventory.get(0), Some(item(1, 7)));
        assert!(w.drop_dialog_token(a).unwrap().is_none());
        assert!(!w.protection(a).unwrap().suspended);
    }
    let (mut w, a) = setup(Kind::Destroy, 1, false);
    act(&mut w, a, Action::DropSlot { slot: 0 });
    let token = w.drop_dialog_token(a).unwrap().unwrap();
    assert!(w.cancel_drop_dialog(a).unwrap());
    act(&mut w, a, Action::ResumeDrop { token, choice: 1 });
    assert_eq!(w.data(a).unwrap().inventory.get(0), Some(item(1, 1)));
}
#[test]
fn movement_cancels_drop_coroutine_and_closes_its_dialog() {
    let (mut w, a) = setup(Kind::Destroy, 1, false);
    act(&mut w, a, Action::DropSlot { slot: 0 });
    let token = w.drop_dialog_token(a).unwrap().unwrap();
    w.drain_events();
    act(
        &mut w,
        a,
        Action::MoveClick {
            destination: CoordGrid::new(3201, 3200, 0).unwrap(),
            control: false,
        },
    );
    assert!(w.drop_dialog_token(a).unwrap().is_none());
    assert!(!w.protection(a).unwrap().suspended);
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Drop {
            event: Event::Closed,
            ..
        }
    )));
    act(&mut w, a, Action::ResumeDrop { token, choice: 1 });
    assert_eq!(w.data(a).unwrap().inventory.get(0), Some(item(1, 1)));
}
#[test]
fn content_can_override_drop_lifetime_without_changing_reveal_delay() {
    let (mut w, a) = setup(Kind::Drop, 1, false);
    w.set_loot_drop_duration(a, Some(3)).unwrap();
    act(&mut w, a, Action::DropSlot { slot: 0 });
    assert_eq!(w.objects.count(), 1);
    for _ in 0..3 {
        w.tick().unwrap();
    }
    assert_eq!(w.objects.count(), 0);
}
