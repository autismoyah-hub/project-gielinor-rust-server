//! Ordinary owned food item only; no potion or chemistry scenario.
use gielinor_world::{
    food::{self, Edible, Event, RandomHealing, StatBoost, StatSubtraction},
    inventory::{Catalog, Inventory, Item, StackType, Template},
    npc::RandomSource,
    stats::Stats,
};
struct NoRandom;
impl RandomSource for NoRandom {
    fn below(&mut self, _: u32) -> u32 {
        panic!("ordinary wine or refused policy drew random")
    }
}
fn food() -> Edible {
    Edible {
        id: 1993,
        heal: 11,
        delay: 3,
        replacement: Some(1935),
        message: "You drink the wine.".into(),
    }
}
fn stats(hp: i8, attack: i8) -> Stats {
    let mut s = Stats::default();
    s.set_base(3, 10);
    s.set_current(3, hp);
    s.set_fine_xp(3, 11540).unwrap();
    s.set_base(0, 99);
    s.set_current(0, attack);
    s.set_fine_xp(0, 130344311).unwrap();
    s.set_base(12, 30);
    s.set_current(12, 30);
    s.set_fine_xp(12, 133630).unwrap();
    s
}
fn inventory(slot: usize, full: bool) -> Inventory {
    let mut v = Inventory::new(28, StackType::Normal, false);
    if full {
        for s in 0..28 {
            v.restore_slot(
                s,
                Some(Item {
                    id: 1931,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
    }
    v.restore_slot(
        slot,
        Some(Item {
            id: 1993,
            count: 1,
            vars: 0,
        }),
    )
    .unwrap();
    v.restore_slot(
        (slot + 1) % 28,
        Some(Item {
            id: 1993,
            count: 1,
            vars: 0,
        }),
    )
    .unwrap();
    v.restore_slot(
        (slot + 2) % 28,
        Some(Item {
            id: 1935,
            count: 1,
            vars: 9,
        }),
    )
    .unwrap();
    v
}
fn consume(v: &mut Inventory, s: &mut Stats, slot: usize) -> Vec<Event> {
    let mut e = Vec::new();
    assert!(food::eat_slot_with_secondary_effects(
        &food(),
        slot,
        v,
        s,
        &Catalog::default(),
        3,
        829,
        &[],
        &[StatSubtraction {
            skill: 0,
            amount: 2
        }],
        None,
        &mut NoRandom,
        |x| e.push(x)
    )
    .unwrap());
    e
}
#[test]
fn all_selected_slots_full_sparse_hp_and_current_attack_boundaries() {
    let mut cases = 0;
    for slot in 0..28 {
        for full in [false, true] {
            for hp in [1i8, 2, 9, 10, 15, 99, 105] {
                for attack in [0i8, 1, 2, 5, 99, 105] {
                    let mut inv = inventory(slot, full);
                    let mut s = stats(hp, attack);
                    let original = inv.clone();
                    let before = s.clone();
                    let events = consume(&mut inv, &mut s, slot);
                    let mut expected = original;
                    expected
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 1935,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    assert_eq!(inv, expected);
                    let mut expected = before;
                    expected.set_current(3, if hp < 10 { 10 } else { hp });
                    expected.set_current(0, (attack - 2).max(0));
                    assert_eq!(s, expected);
                    assert!(matches!(events.first(), Some(Event::Animation(829))));
                    assert!(events
                        .iter()
                        .any(|e| matches!(e,Event::Message(t)if t=="You drink the wine.")));
                    assert!(events.iter().any(|e| matches!(e, Event::Delay(3))));
                    assert!(
                        matches!(events.last(),Some(Event::Stat(u))if u.skill==0&&u.old==attack as u8&&u.level==(attack-2).max(0)as u8)
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 2352);
    println!("PG_WINE_KERNEL2352 all28selectedslots full/sparse sevencurrentHP sixcurrentAttack heal11cap/overheal exact-2floor0 same-slotjug duplicate/varjug/wholeStatsXP retained zeroRNG PASS");
}
#[test]
fn repeated_subtraction_stacks_below_base_at_full_hp_and_floors_zero() {
    let mut cases = 0;
    for attack in [0i8, 1, 2, 5, 99, 105] {
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for s in 0..28 {
            inv.restore_slot(
                s,
                Some(Item {
                    id: if [0, 4, 7, 13, 20, 27, 1].contains(&s) {
                        1993
                    } else {
                        1931
                    },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut s = stats(1, attack);
        let mut current = attack;
        let mut trajectory = Vec::new();
        for slot in [0, 4, 7, 13, 20, 27] {
            let old = inv.clone();
            let before = s.clone();
            consume(&mut inv, &mut s, slot);
            current = (current - 2).max(0);
            trajectory.push(current);
            let mut expected = old;
            expected
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 1935,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            assert_eq!(inv, expected);
            let mut expected = before;
            expected.set_current(3, 10);
            expected.set_current(0, current);
            assert_eq!(s, expected);
            assert_eq!(inv.get(1).unwrap().id, 1993);
            cases += 1;
        }
        if attack == 5 {
            assert_eq!(&trajectory[..3], &[3, 1, 0]);
        }
        if attack == 99 {
            assert_eq!(&trajectory[..3], &[97, 95, 93]);
        }
    }
    assert_eq!(cases, 36);
    println!("PG_WINE_REPEAT36 sixinitialAttack levels sixdrinks HPcap doesnotstopcurrentstacking including5to3to1to0 and99to97to95to93 noXP/RNG PASS");
}
#[test]
fn invalid_secondary_or_selected_exchange_refuses_before_rng_events_or_publication() {
    let mut cases = 0;
    for slot in [0, 13, 27] {
        for mode in 0..22 {
            let mut f = food();
            let mut v = inventory(slot, true);
            let mut s = stats(2, 5);
            let mut items = Catalog::default();
            let mut sub = StatSubtraction {
                skill: 0,
                amount: 2,
            };
            let mut boosts = Vec::new();
            let mut healing = None;
            match mode {
                0 => sub.amount = -1,
                1 => sub.amount = 256,
                2 => sub.skill = 24,
                3 => sub.skill = 3,
                4 => v.stack = StackType::Always,
                5 => v.placeholders = true,
                6 => v.restore_slot(slot, None).unwrap(),
                7 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1993,
                            count: 0,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                8 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1993,
                            count: 2,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                9 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1993,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap(),
                10 => {
                    items.dummy.insert(1993);
                }
                11 => {
                    items.stackable.insert(1993);
                }
                12 => {
                    items.notes.insert(
                        1993,
                        Template {
                            link: 1931,
                            template: 799,
                        },
                    );
                }
                13 => {
                    items.dummy.insert(1935);
                }
                14 => {
                    items.stackable.insert(1935);
                }
                15 => {
                    items.transforms.insert(
                        1935,
                        Template {
                            link: 1931,
                            template: 799,
                        },
                    );
                }
                16 => f.replacement = Some(1993),
                17 => f.heal = -1,
                18 => boosts.push(StatBoost {
                    skill: 12,
                    amount: -1,
                }),
                19 => v
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1935,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                20 => {
                    items.placeholders.insert(
                        1935,
                        Template {
                            link: 1931,
                            template: 14401,
                        },
                    );
                }
                _ => {
                    sub.amount = -1;
                    healing = Some(RandomHealing {
                        minimum: 1,
                        maximum: 5,
                    });
                }
            }
            let before = (v.clone(), s.clone());
            let mut events = Vec::new();
            let out = food::eat_slot_with_secondary_effects(
                &f,
                slot,
                &mut v,
                &mut s,
                &items,
                3,
                829,
                &boosts,
                &[sub],
                healing,
                &mut NoRandom,
                |e| events.push(e),
            );
            assert!(!matches!(out, Ok(true)), "mode{mode}slot{slot}");
            assert_eq!((v, s), before);
            assert!(events.is_empty());
            cases += 1;
        }
    }
    assert_eq!(cases, 66);
    println!("PG_WINE_REFUSAL66 invalidsecondary/shape/selectedcount-vars/catalog/bothsides/heal/boost/RNGpreflight refuses before events or any inventoryStats mutation PASS");
}
