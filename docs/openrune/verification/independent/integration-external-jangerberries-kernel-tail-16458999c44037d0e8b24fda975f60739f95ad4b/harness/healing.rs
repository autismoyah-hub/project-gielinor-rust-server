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
    assert_eq!(cases, 33600);
    println!("PG_JANGER_KERNEL33600 all28slots fullbag duplicate HPcap/overheal Prayerexact+1cap/overcap AttackStrengthboostcaps Defencefloor wholeStatsfineXP selectedconservation zeroRNG PASS");
}
