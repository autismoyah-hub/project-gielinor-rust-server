//! Bounded actual-action checks; original prayer_actions reference cases stay intact.
use gielinor_world::{
    inventory::Item,
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
        necklace: None,
        restore: Default::default(),
        worn_xp_percent: Default::default(),
        chaos: chaos.then_some([2947, 3817, 2958, 3825]),
    });
    let coords = CoordGrid::new(
        if chaos { 2950 } else { 3200 },
        if chaos { 3820 } else { 3200 },
        0,
    )
    .unwrap();
    for x in coords.x() - 8..coords.x() + 8 {
        for z in coords.z() - 8..coords.z() + 8 {
            world.collision.set(x, z, 0, 0);
        }
    }
    let mut data = PlayerData::new(1, "OpenRune burial".into(), coords, Stats::default());
    data.xp_rate_hundreds = 500;
    for slot in 0..data.inventory.slots().len() {
        data.inventory
            .restore_slot(
                slot,
                Some(Item {
                    id: 526,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    (world, actor)
}
fn completed(events: &[WorldEvent]) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            WorldEvent::Prayer {
                event: Event::Completed { .. },
                ..
            }
        )
    })
}
#[test]
fn actual_selected_action_consumes_clicked_full_bag_slot_and_rejects_changed_slot() {
    for stale in [false, true] {
        let (mut w, a) = setup(false);
        w.submit(
            a,
            Action::BuryItemSlot {
                slot: 27,
                item: 526,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert!(w.data(a).unwrap().inventory.get(27).is_some());
        w.drain_events();
        if stale {
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(27, None)
                .unwrap();
        }
        w.tick().unwrap();
        let p = w.data(a).unwrap();
        assert_eq!(p.inventory.get(0).unwrap().id, 526);
        assert!(p.inventory.get(27).is_none());
        assert_eq!(p.stats.fine_xp(5), if stale { 0 } else { 225 });
        assert_eq!(completed(&w.drain_events()), !stale);
    }
}
#[test]
fn chaos_confirmation_retains_selected_slot_and_revalidates_after_choice() {
    let (mut w, a) = setup(true);
    w.submit(
        a,
        Action::BuryItemSlot {
            slot: 27,
            item: 526,
        },
    )
    .unwrap();
    w.tick().unwrap();
    let token = w
        .drain_events()
        .into_iter()
        .find_map(|e| {
            if let WorldEvent::Prayer {
                event: Event::Confirm { token },
                ..
            } = e
            {
                Some(token)
            } else {
                None
            }
        })
        .unwrap();
    w.data_mut(a)
        .unwrap()
        .inventory
        .restore_slot(27, None)
        .unwrap();
    w.submit(a, Action::ResumeBury { token, choice: 1 })
        .unwrap();
    for _ in 0..3 {
        w.tick().unwrap();
    }
    assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 526);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
    assert!(!completed(&w.drain_events()));
}
#[test]
fn claimed_identity_membership_revocation_and_opened_then_closed_modal_cannot_consume() {
    for bad in 0..4 {
        let (mut w, a) = setup(false);
        if bad == 1 || bad == 2 {
            w.prayer_member_bones.insert(526);
            w.set_membership(a, bad == 2).unwrap();
        }
        w.submit(
            a,
            Action::BuryItemSlot {
                slot: 27,
                item: if bad == 0 { 999 } else { 526 },
            },
        )
        .unwrap();
        w.tick().unwrap();
        w.drain_events();
        if bad == 2 {
            w.set_membership(a, false).unwrap();
        }
        if bad == 3 {
            w.set_modal_open(a, true).unwrap();
            w.set_modal_open(a, false).unwrap();
        }
        for _ in 0..3 {
            w.tick().unwrap();
        }
        assert!(w.data(a).unwrap().inventory.get(27).is_some());
        assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
        assert!(!completed(&w.drain_events()));
    }
}

#[test]
fn registered_ashes_scatter_in_chaos_without_bone_restore_or_altar_offering() {
    for (id, xp) in [
        (25766, 10),
        (25769, 25),
        (25772, 65),
        (25775, 85),
        (25778, 110),
    ] {
        let (mut w, a) = setup(true);
        let config = w.prayer.as_mut().unwrap();
        config.bones.insert(id, f64::from(xp));
        config.restore.insert(id, 4);
        config.necklace = Some(22111);
        w.prayer_ashes.insert(id, 2295);
        if id >= 25772 {
            w.prayer_member_bones.insert(id);
            w.set_membership(a, true).unwrap();
        }
        let p = w.data_mut(a).unwrap();
        p.xp_rate_hundreds = 100;
        p.stats.set_base(5, 50);
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
        for slot in [0, 27] {
            p.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        assert!(w.begin_altar_offering(a, id, 3705).is_err());
        assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, id);
        w.submit(a, Action::BuryItemSlot { slot: 27, item: id })
            .unwrap();
        w.tick().unwrap();
        let start = w.drain_events();
        assert!(start.iter().any(|e| matches!(
            e,
            WorldEvent::Prayer {
                event: Event::Animation(2295),
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
        w.tick().unwrap();
        let p = w.data(a).unwrap();
        assert_eq!(p.inventory.get(0).unwrap().id, id);
        assert!(p.inventory.get(27).is_none());
        assert_eq!(p.stats.fine_xp(5), xp * 10);
        assert_eq!(p.stats.current(5), 2);
        let events = w.drain_events();
        assert!(events.iter().any(|e| matches!(
            e,
            WorldEvent::Prayer {
                event: Event::Message("You scatter the ashes."),
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(
            e,
            WorldEvent::Prayer {
                event: Event::Message("You bury the bones."),
                ..
            }
        )));
    }
}
#[test]
fn member_ashes_refuse_nonmembers_and_stale_selected_ash_never_substitutes_duplicate() {
    for stale in [false, true] {
        let (mut w, a) = setup(false);
        let id = 25778;
        w.prayer.as_mut().unwrap().bones.insert(id, 110.0);
        w.prayer_ashes.insert(id, 2295);
        w.prayer_member_bones.insert(id);
        w.set_membership(a, stale).unwrap();
        for slot in [0, 27] {
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.submit(a, Action::BuryItemSlot { slot: 27, item: id })
            .unwrap();
        w.tick().unwrap();
        w.drain_events();
        if stale {
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(27, None)
                .unwrap();
        }
        w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, id);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(5), 0);
        assert!(!completed(&w.drain_events()));
    }
}
