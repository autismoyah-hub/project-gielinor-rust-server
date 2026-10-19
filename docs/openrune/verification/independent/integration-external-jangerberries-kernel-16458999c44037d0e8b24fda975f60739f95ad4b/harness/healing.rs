//! Independent exact145 selected-item/stat transaction checks. Ordinary food only.
use gielinor_world::{
    food::{self, Edible, RandomHealing, StatBoost, StatHealing, StatSubtraction},
    inventory::{Catalog, Inventory, Item, StackType},
    npc::RandomSource,
    stats::Stats,
};
struct NoRandom;
impl RandomSource for NoRandom {
    fn below(&mut self, _: u32) -> u32 {
        panic!("fixed food or refused policy drew random")
    }
}
fn food() -> Edible {
    Edible {
        id: 247,
        heal: 2,
        delay: 3,
        replacement: None,
        message: "You eat the berries.".into(),
    }
}
fn inventory(slot: usize) -> Inventory {
    let mut v = Inventory::new(28, StackType::Normal, false);
    for s in 0..28 {
        v.restore_slot(
            s,
            Some(Item {
                id: if s == slot || s == (slot + 1) % 28 {
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
    v
}
fn stats(hp: i8, prayer: i8, attack: i8, defence: i8, strength: i8) -> Stats {
    let mut s = Stats::default();
    for (id, current) in [
        (3, hp),
        (5, prayer),
        (0, attack),
        (1, defence),
        (2, strength),
    ] {
        s.set_base(id, 99);
        s.set_current(id, current);
        s.set_fine_xp(id, 130344311).unwrap();
    }
    s
}
const BOOSTS: [StatBoost; 2] = [
    StatBoost {
        skill: 0,
        amount: 2,
    },
    StatBoost {
        skill: 2,
        amount: 1,
    },
];
const SUB: [StatSubtraction; 1] = [StatSubtraction {
    skill: 1,
    amount: 1,
}];
const HEAL: [StatHealing; 1] = [StatHealing {
    skill: 5,
    amount: 1,
}];
#[test]
fn selected_full_bag_capped_secondary_healing_and_all_stat_conservation() {
    let mut cases = 0;
    for slot in 0..28 {
        for hp in [1i8, 50, 98, 99, 105] {
            for prayer in [0i8, 1, 98, 99, 105] {
                for attack in [1i8, 98, 99, 105] {
                    for defence in [0i8, 1, 99, 105] {
                        for strength in [1i8, 99, 105] {
                            let mut inv = inventory(slot);
                            let mut s = stats(hp, prayer, attack, defence, strength);
                            let mut expected_inv = inv.clone();
                            expected_inv.restore_slot(slot, None).unwrap();
                            let mut expected = s.clone();
                            expected.set_current(3, if hp < 99 { (hp + 2).min(99) } else { hp });
                            expected.set_current(5, if prayer < 99 { prayer + 1 } else { prayer });
                            expected.set_current(
                                0,
                                if attack < 101 {
                                    (attack + 2).min(101)
                                } else {
                                    attack
                                },
                            );
                            expected.set_current(1, (defence - 1).max(0));
                            expected.set_current(
                                2,
                                if strength < 100 {
                                    (strength + 1).min(100)
                                } else {
                                    strength
                                },
                            );
                            let mut events = Vec::new();
                            assert!(food::eat_slot_with_stat_healing(
                                &food(),
                                slot,
                                &mut inv,
                                &mut s,
                                &Catalog::default(),
                                3,
                                829,
                                &BOOSTS,
                                &SUB,
                                &HEAL,
                                None,
                                &mut NoRandom,
                                |e| events.push(e)
                            )
                            .unwrap());
                            assert_eq!(inv, expected_inv);
                            assert_eq!(s, expected);
                            assert!(!events.is_empty());
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 134400);
    println!("PG_JANGER_KERNEL134400 all28slots fullbag duplicate HPcap/overheal Prayerexact+1cap/overcap AttackStrengthboostcaps Defencefloor wholeStatsfineXP selectedconservation zeroRNG PASS");
}
#[test]
fn invalid_late_restoration_policy_refuses_before_random_or_any_publication() {
    let mut cases = 0;
    for slot in [0, 13, 27] {
        for mode in 0..6 {
            for prefix in 0..3 {
                let mut v = inventory(slot);
                let mut s = stats(50, 1, 1, 1, 1);
                let before_v = v.clone();
                let before_s = s.clone();
                let bad = match mode {
                    0 => StatHealing {
                        skill: 5,
                        amount: -1,
                    },
                    1 => StatHealing {
                        skill: 5,
                        amount: 256,
                    },
                    2 => StatHealing {
                        skill: 24,
                        amount: 1,
                    },
                    3 => StatHealing {
                        skill: 255,
                        amount: 1,
                    },
                    4 => StatHealing {
                        skill: 3,
                        amount: 1,
                    },
                    _ => StatHealing {
                        skill: 5,
                        amount: i32::MAX,
                    },
                };
                let mut restores = vec![
                    StatHealing {
                        skill: 5,
                        amount: 1
                    };
                    prefix
                ];
                restores.push(bad);
                let mut events = Vec::new();
                let result = food::eat_slot_with_stat_healing(
                    &food(),
                    slot,
                    &mut v,
                    &mut s,
                    &Catalog::default(),
                    3,
                    829,
                    &BOOSTS,
                    &SUB,
                    &restores,
                    Some(RandomHealing {
                        minimum: 1,
                        maximum: 9,
                    }),
                    &mut NoRandom,
                    |e| events.push(e),
                );
                assert!(result.is_err());
                assert_eq!(v, before_v);
                assert_eq!(s, before_s);
                assert!(events.is_empty());
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 54);
    println!("PG_JANGER_POLICY54 invalidlate/nonHP/negative/overflowpolicy beforeRNG inventoryHPboostrestoreSubtractandevents PASS");
}
#[test]
fn current_selected_shape_and_catalog_refusals_keep_every_stat() {
    let mut cases = 0;
    for slot in [0, 13, 27] {
        for mode in 0..10 {
            let mut v = inventory(slot);
            let mut s = stats(50, 1, 1, 1, 1);
            let mut c = Catalog::default();
            match mode {
                0 => v.stack = StackType::Always,
                1 => v.placeholders = true,
                2 => v.restore_slot(slot, None).unwrap(),
                3 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1931,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                4 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 247,
                            count: 0,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                5 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 247,
                            count: 2,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                6 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 247,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap(),
                7 => {
                    c.dummy.insert(247);
                }
                8 => {
                    c.stackable.insert(247);
                }
                _ => {
                    c.notes.insert(
                        247,
                        gielinor_world::inventory::Template {
                            link: 1931,
                            template: 799,
                        },
                    );
                }
            }
            let before_v = v.clone();
            let before_s = s.clone();
            let mut events = Vec::new();
            let result = food::eat_slot_with_stat_healing(
                &food(),
                slot,
                &mut v,
                &mut s,
                &c,
                3,
                829,
                &BOOSTS,
                &SUB,
                &HEAL,
                None,
                &mut NoRandom,
                |e| events.push(e),
            );
            assert!(!result.unwrap_or(false), "slot{slot} mode{mode}");
            assert_eq!(v, before_v);
            assert_eq!(s, before_s);
            assert!(events.is_empty());
            cases += 1;
        }
    }
    assert_eq!(cases, 30);
    println!("PG_JANGER_SELECTED30 currentselected/countvars/catalog/shape atomicrefusals PASS");
}
