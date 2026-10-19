//! Exact145 authoritative dispatch; controlled ordinary food fixtures, no native claim.
use gielinor_world::{
    food::{Edible, StatBoost, StatHealing, StatSubtraction},
    inventory::{Item, StackType},
    npc::RandomSource,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
struct NoRandom;
impl RandomSource for NoRandom {
    fn below(&mut self, _: u32) -> u32 {
        panic!("fixed ordinary food drew random")
    }
}
fn setup(slot: usize, hp: i8, prayer: i8) -> (World, ActorId) {
    let mut w = World::default();
    w.collision.allocate(3224, 3216, 0);
    w.foods.insert(
        247,
        Edible {
            id: 247,
            heal: 2,
            delay: 3,
            replacement: None,
            message: "You eat the berries.".into(),
        },
    );
    w.member_foods.insert(247);
    w.slot_preserving_foods.insert(247);
    w.food_boosts.insert(
        247,
        vec![
            StatBoost {
                skill: 0,
                amount: 2,
            },
            StatBoost {
                skill: 2,
                amount: 1,
            },
        ],
    );
    w.food_subtractions.insert(
        247,
        vec![StatSubtraction {
            skill: 1,
            amount: 1,
        }],
    );
    w.food_stat_healing.insert(
        247,
        vec![StatHealing {
            skill: 5,
            amount: 1,
        }],
    );
    w.eating_animation = 829;
    let mut s = Stats::default();
    for (id, current) in [(0, 99), (1, 1), (2, 99), (3, hp), (5, prayer)] {
        s.set_base(id, 99);
        s.set_current(id, current);
        s.set_fine_xp(id, 130344311).unwrap();
    }
    let mut p = PlayerData::new(
        1,
        "Independent ordinary berry World".into(),
        CoordGrid::new(3224, 3218, 0).unwrap(),
        s,
    );
    for i in 0..28 {
        p.inventory
            .restore_slot(
                i,
                Some(Item {
                    id: if i == slot || i == (slot + 1) % 28 {
                        247
                    } else {
                        1931
                    },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    w.queue_login(1, p).unwrap();
    w.tick_with_random(&mut NoRandom).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, true).unwrap();
    w.drain_events();
    (w, a)
}
#[test]
fn actual_selected_dispatch_caps_prayer_without_full_restore_or_overcap_loss() {
    let mut n = 0;
    for slot in [0, 13, 27] {
        for hp in [50i8, 99, 105] {
            for prayer in [0i8, 98, 99, 105] {
                let (mut w, a) = setup(slot, hp, prayer);
                let before = w.data(a).unwrap().clone();
                w.submit(a, Action::EatItemSlot { slot, item: 247 })
                    .unwrap();
                w.tick_with_random(&mut NoRandom).unwrap();
                let d = w.data(a).unwrap();
                let mut inv = before.inventory;
                inv.restore_slot(slot, None).unwrap();
                let mut stats = before.stats;
                stats.set_current(3, if hp < 99 { (hp + 2).min(99) } else { hp });
                stats.set_current(5, if prayer < 99 { prayer + 1 } else { prayer });
                stats.set_current(0, 101);
                stats.set_current(2, 100);
                stats.set_current(1, 0);
                assert_eq!(d.inventory.slots(), inv.slots());
                assert_eq!(d.stats, stats);
                assert_eq!(d.equipment, before.equipment);
                assert!(w.protection(a).unwrap().delayed());
                let events = w.drain_events();
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| matches!(e, WorldEvent::FoodTransition { .. }))
                        .count(),
                    1
                );
                n += 1;
            }
        }
    }
    assert_eq!(n, 36);
    println!("PG_JANGER_WORLD36 selected fullbag duplicate HP/Prayercapovercap currentboost/drain exactStatsfineXPgear noRNG protection3 PASS");
}
#[test]
fn current_member_and_late_malformed_policy_never_publish_partial_effects() {
    let mut n = 0;
    for slot in [0, 13, 27] {
        for mode in 0..12 {
            let (mut w, a) = setup(slot, 50, 0);
            w.submit(a, Action::EatItemSlot { slot, item: 247 })
                .unwrap();
            match mode {
                0 => w.set_membership(a, false).unwrap(),
                1 => w.food_stat_healing.get_mut(&247).unwrap()[0].skill = 3,
                2 => w.food_stat_healing.get_mut(&247).unwrap()[0].skill = 24,
                3 => w.food_stat_healing.get_mut(&247).unwrap()[0].amount = -1,
                4 => w.food_stat_healing.get_mut(&247).unwrap()[0].amount = 256,
                5 => w
                    .food_stat_healing
                    .get_mut(&247)
                    .unwrap()
                    .push(StatHealing {
                        skill: 5,
                        amount: -1,
                    }),
                6 => w.food_subtractions.get_mut(&247).unwrap()[0].amount = -1,
                7 => w.food_boosts.get_mut(&247).unwrap()[0].amount = -1,
                8 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1931,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                9 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 247,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap(),
                10 => {
                    w.object_types.dummy.insert(247);
                }
                _ => w.data_mut(a).unwrap().inventory.stack = StackType::Always,
            };
            let before = w.data(a).unwrap().clone();
            w.tick_with_random(&mut NoRandom).unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(
                d.inventory.slots(),
                before.inventory.slots(),
                "slot{slot} mode{mode}"
            );
            assert_eq!(d.stats, before.stats);
            assert_eq!(d.equipment, before.equipment);
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::Food { .. } | WorldEvent::FoodTransition { .. }
            )));
            n += 1;
        }
    }
    assert_eq!(n, 36);
    println!("PG_JANGER_WORLD_GUARD36 currentmember selected/vars/catalog/shape and lateinvalidrestoration/boost/drain refusebeforewholepublication PASS");
}
#[test]
fn observe_unresolved_death_at_the_new_healing_target_independently() {
    use gielinor_world::hits::{Hit, Style, Target};
    let mut cases = 0;
    let mut violations = 0;
    for slot in [0, 13, 27] {
        for phase in 0..3 {
            let (mut w, a) = setup(slot, 1, 0);
            if phase == 0 {
                w.data_mut(a).unwrap().stats.set_current(3, 0);
            } else {
                w.queue_hit(
                    Target::Player(a),
                    Hit {
                        display: None,
                        source: None,
                        style: Style::Typeless,
                        damage: 20,
                    },
                    1,
                )
                .unwrap();
                w.tick_with_random(&mut NoRandom).unwrap();
                assert!(w
                    .drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
                assert_eq!(w.data(a).unwrap().stats.current(3), 0);
                if phase == 2 {
                    w.data_mut(a).unwrap().stats.set_current(3, 10);
                }
            }
            let before = w.data(a).unwrap().clone();
            let submitted = w.submit(a, Action::EatItemSlot { slot, item: 247 }).is_ok();
            if submitted {
                w.tick_with_random(&mut NoRandom).unwrap();
            }
            let d = w.data(a).unwrap();
            let bad = d.inventory.slots() != before.inventory.slots() || d.stats != before.stats;
            violations += usize::from(bad);
            cases += 1;
            println!("PG_JANGER_LIFE slot={slot} phase={phase} hp={} prayer={} attack={} defence={} strength={} retained={} violation={bad}",d.stats.current(3),d.stats.current(5),d.stats.current(0),d.stats.current(1),d.stats.current(2),d.inventory.get(slot).is_some());
        }
    }
    assert_eq!(cases, 9);
    println!("PG_JANGER_LIFE_SUMMARY cases={cases} violations={violations};diagnostic completion is not acceptancePASS if violations>0");
}
