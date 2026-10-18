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
