use super::*;
use crate::{
    consumable_clocks::{Kind, Policy, Rules},
    inventory::Item,
    potions::{Potion, ResolvedEffect},
};
fn setup() -> (World, ActorId) {
    let mut w = World::default();
    w.collision.allocate(3200, 3200, 0);
    w.queue_login(
        1,
        PlayerData::new(
            1,
            "Clocks".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        ),
    )
    .unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    for id in [1, 2] {
        w.foods.insert(
            id,
            crate::food::Edible {
                id,
                heal: 1,
                delay: 3,
                replacement: None,
                message: "Eat".into(),
            },
        );
    }
    w.potions.insert(
        3,
        Potion {
            id: 3,
            next: 9,
            vial: 9,
            sip: "Sip".into(),
            remaining: "Left".into(),
            effects: vec![ResolvedEffect::Boost {
                skill: 0,
                constant: 1,
                percent: 0,
            }],
        },
    );
    w.consumable_rules = Some(Rules {
        foods: BTreeMap::from([(1, policy(Kind::Food)), (2, policy(Kind::ComboFood))]),
        potions: BTreeMap::from([(3, policy(Kind::Potion))]),
    });
    let p = w.players.get_mut(actor.slot as usize).unwrap();
    p.data.stats.set_base(3, 99);
    p.data.stats.set_current(3, 50);
    for (slot, id) in [1, 2, 3, 1, 2, 3].into_iter().enumerate() {
        p.data
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
    w.drain_events();
    (w, actor)
}
fn policy(kind: Kind) -> Policy {
    Policy {
        kind,
        consume_delay: 3,
        combat_delay: match kind {
            Kind::Food => 3,
            Kind::Potion => 0,
            Kind::ComboFood => 2,
        },
        members: false,
    }
}
fn action(kind: Kind) -> Action {
    match kind {
        Kind::Food => Action::EatItemSlot { slot: 0, item: 1 },
        Kind::ComboFood => Action::EatItemSlot { slot: 1, item: 2 },
        Kind::Potion => Action::DrinkSlot {
            slot: 2,
            item: 3,
            empty: false,
        },
    }
}
#[test]
fn native_consumable_real_action_queue_preserves_all_six_category_orders() {
    use Kind::*;
    for (order, want) in [
        ([Food, Potion, ComboFood], [true, true, true]),
        ([Potion, Food, ComboFood], [true, true, true]),
        ([Food, ComboFood, Potion], [true, true, false]),
        ([Potion, ComboFood, Food], [true, true, false]),
        ([ComboFood, Food, Potion], [true, false, false]),
        ([ComboFood, Potion, Food], [true, false, false]),
    ] {
        let (mut w, a) = setup();
        w.set_next_attack_delay(a, 20).unwrap();
        let initial = w.players.get(a.slot as usize).unwrap().action_delay;
        for kind in order {
            w.submit(a, action(kind)).unwrap();
        }
        w.tick().unwrap();
        let p = w.players.get(a.slot as usize).unwrap();
        let mut attack = initial;
        for (i, kind) in order.into_iter().enumerate() {
            let slot = match kind {
                Food => 0,
                ComboFood => 1,
                Potion => 2,
            };
            let consumed = if kind == Potion {
                p.data.inventory.get(slot).unwrap().id == 9
            } else {
                p.data.inventory.get(slot).is_none()
            };
            assert_eq!(consumed, want[i], "{order:?}/{kind:?}");
            if want[i] {
                attack += policy(kind).combat_delay;
            }
        }
        assert_eq!(p.action_delay, attack);
        assert!(!p.protection.delayed());
        assert!(w.movement_input_allowed(a).unwrap());
        assert!(w.is_attack_delayed(a).unwrap());
        let events = w.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(
                    e,
                    WorldEvent::ActionRejected {
                        error: ActionError::Delayed,
                        ..
                    }
                ))
                .count(),
            want.iter().filter(|&&v| !v).count()
        );
        for e in events {
            if let WorldEvent::FoodTransition { trace, .. } = e {
                assert!(trace.attack_delay.is_some());
                assert!(trace.json(a).contains("\"attack_delay\":"));
            }
        }
    }
}
#[test]
fn native_consumable_exact_deadline_and_movement_do_not_bypass_committed_combat_lock() {
    let (mut w, a) = setup();
    let now = w.clock.0;
    w.set_next_attack_delay(a, 10).unwrap();
    w.submit(a, action(Kind::Food)).unwrap();
    w.tick().unwrap();
    let committed = now + 13;
    assert_eq!(
        w.players.get(a.slot as usize).unwrap().action_delay,
        committed
    );
    assert!(w
        .prepare_movement_input(a, CoordGrid::new(3201, 3200, 0).unwrap(), false)
        .unwrap()
        .is_some());
    w.set_next_attack_delay(a, 0).unwrap();
    assert_eq!(
        w.players.get(a.slot as usize).unwrap().action_delay,
        committed
    );
    w.players.get_mut(a.slot as usize).unwrap().action_delay = 0;
    assert!(w.is_attack_delayed(a).unwrap());
    for _ in 0..2 {
        w.submit(a, Action::EatItemSlot { slot: 3, item: 1 })
            .unwrap();
        w.tick().unwrap();
        assert!(w.data(a).unwrap().inventory.get(3).is_some());
    }
    assert_eq!(w.clock.0, now + 3);
    w.submit(a, Action::EatItemSlot { slot: 3, item: 1 })
        .unwrap();
    w.tick().unwrap();
    assert!(w.data(a).unwrap().inventory.get(3).is_none());
}
#[test]
fn native_consumable_rejections_do_not_commit_clocks_and_membership_is_checked_before_effects() {
    for mode in 0..4 {
        let (mut w, a) = setup();
        match mode {
            0 => {
                w.food_subtractions.insert(
                    1,
                    vec![crate::food::StatSubtraction {
                        skill: 24,
                        amount: 1,
                    }],
                );
            }
            1 => {
                w.players
                    .get_mut(a.slot as usize)
                    .unwrap()
                    .data
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 1,
                            count: 2,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            2 => {
                w.consumable_rules
                    .as_mut()
                    .unwrap()
                    .foods
                    .get_mut(&1)
                    .unwrap()
                    .members = true
            }
            _ => {
                w.consumable_rules
                    .as_mut()
                    .unwrap()
                    .foods
                    .get_mut(&1)
                    .unwrap()
                    .consume_delay = 2
            }
        }
        let before = w.data(a).unwrap().stats.clone();
        w.submit(a, action(Kind::Food)).unwrap();
        w.tick().unwrap();
        assert_eq!(
            w.players
                .get(a.slot as usize)
                .unwrap()
                .consumables
                .deadlines(),
            [-1; 4]
        );
        assert_eq!(w.data(a).unwrap().stats, before);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 1);
        w.food_subtractions.clear();
        *w.consumable_rules
            .as_mut()
            .unwrap()
            .foods
            .get_mut(&1)
            .unwrap() = policy(Kind::Food);
        w.submit(a, Action::EatItemSlot { slot: 3, item: 1 })
            .unwrap();
        w.tick().unwrap();
        assert!(w.data(a).unwrap().inventory.get(3).is_none());
    }
    let (mut w, a) = setup();
    w.consumable_rules
        .as_mut()
        .unwrap()
        .potions
        .get_mut(&3)
        .unwrap()
        .members = true;
    w.submit(a, action(Kind::Potion)).unwrap();
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().inventory.get(2).unwrap().id, 3);
    assert_eq!(
        w.players
            .get(a.slot as usize)
            .unwrap()
            .consumables
            .deadlines(),
        [-1; 4]
    );
    w.players.get_mut(a.slot as usize).unwrap().member = true;
    w.submit(a, action(Kind::Potion)).unwrap();
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().inventory.get(2).unwrap().id, 9);
}
#[test]
fn native_consumable_empty_and_ready_weapon_add_no_attack_or_category_lock_but_stuns_still_block() {
    let (mut w, a) = setup();
    w.submit(
        a,
        Action::DrinkSlot {
            slot: 2,
            item: 3,
            empty: true,
        },
    )
    .unwrap();
    w.submit(a, action(Kind::Food)).unwrap();
    w.submit(
        a,
        Action::DrinkSlot {
            slot: 5,
            item: 3,
            empty: false,
        },
    )
    .unwrap();
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().inventory.get(2).unwrap().id, 9);
    assert_eq!(w.data(a).unwrap().inventory.get(5).unwrap().id, 9);
    assert!(w.data(a).unwrap().inventory.get(0).is_none());
    assert!(!w.is_attack_delayed(a).unwrap());
    assert_eq!(
        w.players
            .get(a.slot as usize)
            .unwrap()
            .consumables
            .deadlines()[3],
        -1
    );
    let (mut w, a) = setup();
    w.players
        .get_mut(a.slot as usize)
        .unwrap()
        .protection
        .delay(10);
    w.submit(a, action(Kind::Food)).unwrap();
    w.tick().unwrap();
    assert!(w.data(a).unwrap().inventory.get(0).is_some());
    assert_eq!(
        w.players
            .get(a.slot as usize)
            .unwrap()
            .consumables
            .deadlines(),
        [-1; 4]
    );
}
#[test]
fn native_consumable_actual_lethal_hit_clears_temporary_category_state() {
    let (mut w, a) = setup();
    w.submit(a, action(Kind::ComboFood)).unwrap();
    w.tick().unwrap();
    assert!(
        w.players
            .get(a.slot as usize)
            .unwrap()
            .consumables
            .deadlines()[2]
            >= 0
    );
    w.queue_hit(
        crate::hits::Target::Player(a),
        crate::hits::Hit {
            display: None,
            source: None,
            style: crate::hits::Style::Typeless,
            damage: 99,
        },
        0,
    )
    .unwrap();
    w.tick().unwrap();
    let p = w.players.get(a.slot as usize).unwrap();
    assert!(p.death_queued);
    assert_eq!(p.consumables.deadlines(), [-1; 4]);
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e,WorldEvent::DeathRequested(crate::hits::Target::Player(id))if *id==a)));
}
