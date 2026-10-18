//! Ordinary owned food item only; no potion or chemistry scenario.
use gielinor_world::{
    food::{self, Edible, Event, RandomHealing, StatBoost, StatSubtraction},
    inventory::{Catalog, Inventory, Item, StackType, Template},
    npc::RandomSource,
    stats::Stats,
};
use gielinor_world::{
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    CoordGrid,
};
fn fixture(hp: i8, attack: i8, member: bool, slot: usize) -> (World, ActorId) {
    let mut w = World::default();
    w.collision.allocate(3224, 3216, 0);
    w.foods.insert(1993, food());
    w.slot_preserving_foods.insert(1993);
    w.food_subtractions.insert(
        1993,
        vec![StatSubtraction {
            skill: 0,
            amount: 2,
        }],
    );
    w.eating_animation = 829;
    let mut p = PlayerData::new(
        1,
        "Independent ordinary wine World".into(),
        CoordGrid::new(3224, 3218, 0).unwrap(),
        stats(hp, attack),
    );
    p.inventory = inventory(slot, true);
    w.queue_login(1, p).unwrap();
    w.tick_with_random(&mut NoRandom).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, member).unwrap();
    w.drain_events();
    (w, a)
}
#[test]
fn actual_member_and_f2p_selected_dispatch_delay_and_stale_controls() {
    let mut cases = 0;
    for member in [false, true] {
        for slot in 0..28 {
            for attack in [0i8, 1, 2, 5, 99, 105] {
                for hp in [1i8, 10, 15] {
                    let (mut w, a) = fixture(hp, attack, member, slot);
                    let before = w.data(a).unwrap().clone();
                    for _ in 0..2 {
                        w.submit(a, Action::EatItemSlot { slot, item: 1993 })
                            .unwrap();
                    }
                    w.tick_with_random(&mut NoRandom).unwrap();
                    let d = w.data(a).unwrap();
                    let mut inv = before.inventory;
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: 1935,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                    assert_eq!(d.inventory.slots(), inv.slots());
                    let mut expected = before.stats;
                    expected.set_current(3, if hp < 10 { 10 } else { hp });
                    expected.set_current(0, (attack - 2).max(0));
                    assert_eq!(d.stats, expected);
                    assert_eq!(d.equipment, before.equipment);
                    assert!(w.protection(a).unwrap().delayed());
                    let events = w.drain_events();
                    let traces = events
                        .iter()
                        .filter_map(|e| {
                            if let WorldEvent::FoodTransition { trace, .. } = e {
                                Some(trace)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(traces.len(), 1);
                    assert_eq!(traces[0].slot, slot);
                    assert_eq!(traces[0].configured_delay, 3);
                    assert!(traces[0].json(a).contains("\"attack_delay\":null"));
                    assert_eq!(
                        events
                            .iter()
                            .filter(|e| matches!(
                                e,
                                WorldEvent::Food {
                                    event: Event::Animation(829),
                                    ..
                                }
                            ))
                            .count(),
                        1
                    );
                    for _ in 0..4 {
                        w.tick_with_random(&mut NoRandom).unwrap();
                    }
                    w.drain_events();
                    let stable = w.data(a).unwrap().clone();
                    w.submit(a, Action::EatItemSlot { slot, item: 1993 })
                        .unwrap();
                    w.tick_with_random(&mut NoRandom).unwrap();
                    assert_eq!(
                        w.data(a).unwrap().inventory.slots(),
                        stable.inventory.slots()
                    );
                    assert_eq!(w.data(a).unwrap().stats, stable.stats);
                    assert!(!w.drain_events().iter().any(|e| matches!(
                        e,
                        WorldEvent::FoodTransition { .. } | WorldEvent::Food { .. }
                    )));
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 1008);
    println!("PG_WINE_WORLD1008 member/F2P all28slots sixAttack threeHPstates actualEatItemSlot duplicate sameclock delayed thenstaleclaim preservesotherduplicate varjug/fullbag/base/fineXP allStats;exactheal11andcurrent-2;attackdelaynotclaimed PASS");
}
#[test]
fn invalid_current_selected_or_secondary_state_emits_no_food_effects() {
    let mut cases = 0;
    for slot in [0, 13, 27] {
        for mode in 0..16 {
            let (mut w, a) = fixture(1, 5, true, slot);
            match mode {
                0 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(slot, None)
                    .unwrap(),
                1 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1993,
                            count: 2,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                2 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1993,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap(),
                3 => {
                    w.object_types.dummy.insert(1993);
                }
                4 => {
                    w.object_types.stackable.insert(1993);
                }
                5 => {
                    w.object_types.dummy.insert(1935);
                }
                6 => {
                    w.object_types.stackable.insert(1935);
                }
                7 => {
                    w.object_types.transforms.insert(
                        1935,
                        Template {
                            link: 1931,
                            template: 799,
                        },
                    );
                }
                8 => w.food_subtractions.get_mut(&1993).unwrap()[0].amount = -1,
                9 => w.food_subtractions.get_mut(&1993).unwrap()[0].amount = 256,
                10 => w.food_subtractions.get_mut(&1993).unwrap()[0].skill = 24,
                11 => w.food_subtractions.get_mut(&1993).unwrap()[0].skill = 3,
                12 => w.foods.get_mut(&1993).unwrap().heal = -1,
                13 => w.foods.get_mut(&1993).unwrap().replacement = Some(1993),
                14 => w.data_mut(a).unwrap().inventory.stack = StackType::Always,
                _ => w.data_mut(a).unwrap().inventory.placeholders = true,
            }
            let before = w.data(a).unwrap().clone();
            w.submit(a, Action::EatItemSlot { slot, item: 1993 })
                .unwrap();
            w.tick_with_random(&mut NoRandom).unwrap();
            assert_eq!(
                w.data(a).unwrap().inventory.slots(),
                before.inventory.slots(),
                "mode{mode}slot{slot}"
            );
            assert_eq!(w.data(a).unwrap().stats, before.stats);
            assert_eq!(w.data(a).unwrap().equipment, before.equipment);
            assert!(!w.protection(a).unwrap().delayed());
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::FoodTransition { .. } | WorldEvent::Food { .. }
            )));
            cases += 1;
        }
    }
    assert_eq!(cases, 48);
    println!("PG_WINE_WORLD_REFUSAL48 publicdispatch currentselected/count/vars/catalog/raw-container/secondary/HP-target/heal/shape invalidity no inventoryStats/food-event/RNG publication PASS");
}
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
