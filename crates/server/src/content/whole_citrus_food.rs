//! Ordinary whole citrus uses native first-option Eat and shared food transactions.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
const PROFILES: &[(&str, i32, &str, u8, i32)] = &[
    ("lemon", 2102, "Lemon", 2, 2),
    ("orange", 2108, "Orange", 2, 2),
    ("lime", 2120, "Lime", 2, 2),
];
pub fn whole_citrus_food(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    for &(symbol, id, name, op, _) in PROFILES {
        if objects.get(symbol) != Some(&id)
            || foods.contains_key(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || !d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || crate::food_operations::button_operation(d) != Some(op)
                    || d.inventory_ops[(op - 2) as usize].as_deref() != Some("Eat")
            })
        {
            return Err(format!(
                "Unattested or duplicate whole citrus food {symbol}"
            ));
        }
    }
    for &(_, id, name, _, heal) in PROFILES {
        foods.insert(
            id,
            Edible {
                id,
                heal,
                delay: 3,
                replacement: None,
                message: format!("You eat the {}.", name.to_lowercase()),
            },
        );
    }
    Ok(PROFILES.iter().map(|&(_, id, _, _, _)| id).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        consumable_clocks::Kind,
        inventory::Item,
        potions::Potion,
        simulation::{Action, PlayerData, World, WorldEvent},
        stats::Stats,
        CoordGrid,
    };
    fn fixture() -> (Symbols, Definitions) {
        let mut objects = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for &(symbol, id, name, op, _) in PROFILES {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            d.inventory_ops[(op - 2) as usize] = Some("Eat".into());
            defs.items.insert(id as u32, d);
        }
        (objects, defs)
    }
    #[test]
    fn whole_citrus_food_batch_checks_exact_native_eat_position_and_late_binding_before_insertion()
    {
        for mode in 0..8 {
            let (mut objects, mut defs) = fixture();
            let mut foods = BTreeMap::new();
            let d = defs.items.get_mut(&2120).unwrap();
            match mode {
                0 => {
                    objects.insert("lime".into(), 2102);
                }
                1 => d.members = false,
                2 => d.stackable = true,
                3 => d.name = "Pot".into(),
                4 => d.inventory_ops[0] = Some("Drink".into()),
                5 => d.transform.template = 65534,
                6 => {
                    foods.insert(
                        2120,
                        Edible {
                            id: 2120,
                            heal: 99,
                            delay: 3,
                            replacement: None,
                            message: "Existing".into(),
                        },
                    );
                }
                _ => {
                    let d = defs.items.get_mut(&2102).unwrap();
                    d.inventory_ops[0] = None;
                    d.inventory_ops[3] = Some("Eat".into());
                }
            }
            let before = format!("{foods:?}");
            assert!(whole_citrus_food(&objects, &defs, &mut foods).is_err());
            assert_eq!(format!("{foods:?}"), before);
        }
    }
    #[test]
    fn whole_citrus_food_native_op2_citrus_are_food_not_combo_and_potion_can_follow_with_selected_healing(
    ) {
        for &(_, id, _, op, heal) in PROFILES {
            for hp in [50, 98, 99, 115] {
                let (objects, mut defs) = fixture();
                let mut w = World::default();
                let ids = whole_citrus_food(&objects, &defs, &mut w.foods).unwrap();
                w.member_foods = ids.clone();
                w.slot_preserving_foods = ids;
                let d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
                defs.items.insert(2, d);
                w.potions.insert(
                    2,
                    Potion {
                        id: 2,
                        next: 9,
                        vial: 9,
                        sip: "Sip".into(),
                        remaining: "Left".into(),
                        effects: vec![],
                    },
                );
                w.consumable_rules = Some(
                    crate::content::native_consumables(&objects, &defs, &w.foods, &w.potions)
                        .unwrap(),
                );
                let policy = w.consumable_rules.as_ref().unwrap().foods[&id];
                assert_eq!(
                    (policy.kind, policy.consume_delay, policy.combat_delay),
                    (Kind::Food, 3, 3)
                );
                let mut stats = Stats::default();
                stats.set_base(3, 99);
                stats.set_current(3, hp);
                let mut data = PlayerData::new(
                    1,
                    "Citrus".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    stats,
                );
                for slot in 0..28 {
                    data.inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: if slot >= 26 {
                                    id
                                } else if slot == 1 {
                                    2
                                } else {
                                    995
                                },
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                w.queue_login(1, data).unwrap();
                w.tick().unwrap();
                let a = w.actors().next().unwrap();
                let button = |op: u8| {
                    let mut p = (149u32 << 16).to_be_bytes().to_vec();
                    p.extend(27u16.to_be_bytes());
                    p.extend((id as u16).to_be_bytes());
                    p.push(op);
                    gielinor_protocol::buttons::button(&p).unwrap()
                };
                assert!(crate::food_operations::held_action(
                    &button(if op == 2 { 5 } else { 2 }),
                    27,
                    id,
                    &w.foods,
                    &defs,
                    true
                )
                .is_none());
                let action =
                    crate::food_operations::held_action(&button(op), 27, id, &w.foods, &defs, true)
                        .unwrap();
                w.submit(a, action.clone()).unwrap();
                w.tick().unwrap();
                assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, id);
                assert_eq!(w.data(a).unwrap().stats.current(3), hp);
                w.set_membership(a, true).unwrap();
                w.set_next_attack_delay(a, 20).unwrap();
                w.drain_events();
                w.submit(a, action.clone()).unwrap();
                w.submit(
                    a,
                    Action::DrinkSlot {
                        slot: 1,
                        item: 2,
                        empty: false,
                    },
                )
                .unwrap();
                w.tick().unwrap();
                let d = w.data(a).unwrap();
                assert!(d.inventory.get(27).is_none());
                assert_eq!(d.inventory.get(26).unwrap().id, id);
                assert_eq!(d.inventory.get(1).unwrap().id, 9);
                assert_eq!(
                    d.stats.current(3),
                    if hp >= 99 {
                        hp
                    } else {
                        (hp as i32 + heal).min(99) as i8
                    }
                );
                for skill in 0..24 {
                    assert_eq!(d.stats.fine_xp(skill), 0);
                }
                assert!(w.movement_input_allowed(a).unwrap());
                let t = w
                    .drain_events()
                    .into_iter()
                    .find_map(|e| {
                        if let WorldEvent::FoodTransition { trace, .. } = e {
                            Some(trace)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                assert_eq!(t.attack_delay, Some(23));
                let before = (
                    w.data(a).unwrap().inventory.slots().to_vec(),
                    w.data(a).unwrap().stats.clone(),
                );
                w.submit(a, action).unwrap();
                w.tick().unwrap();
                assert_eq!(w.data(a).unwrap().inventory.slots(), before.0.as_slice());
                assert_eq!(w.data(a).unwrap().stats, before.1);
            }
        }
    }
}
