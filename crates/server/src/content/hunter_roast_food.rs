//! Ordinary hunter roast foods reuse selected food transactions and native FOOD clocks.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
const PROFILES: &[(&str, i32, &str, i32)] = &[
    ("spit_roasted_bird_meat", 9980, "Roast bird meat", 6),
    ("spit_roasted_rabbit_meat", 7223, "Roast rabbit", 7),
    ("spit_roasted_beast_meat", 9988, "Roast beast meat", 8),
];
pub fn hunter_roast_food(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    for &(symbol, id, name, _) in PROFILES {
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
                    || d.inventory_ops[0].as_deref() != Some("Eat")
            })
        {
            return Err(format!(
                "Unattested or duplicate hunter roast food {symbol}"
            ));
        }
    }
    for &(_, id, name, heal) in PROFILES {
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
    Ok(PROFILES.iter().map(|&(_, id, _, _)| id).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        consumable_clocks::Kind,
        inventory::Item,
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
        for &(symbol, id, name, _) in PROFILES {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            d.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id as u32, d);
        }
        (objects, defs)
    }
    #[test]
    fn hunter_roast_food_batch_refuses_late_alias_member_operation_or_duplicate_and_does_not_alias_cooked_rabbit(
    ) {
        for mode in 0..7 {
            let (mut objects, mut defs) = fixture();
            let mut foods = BTreeMap::from([(
                3228,
                Edible {
                    id: 3228,
                    heal: 5,
                    delay: 3,
                    replacement: None,
                    message: "Cooked rabbit".into(),
                },
            )]);
            let d = defs.items.get_mut(&9988).unwrap();
            match mode {
                0 => {
                    objects.insert("spit_roasted_beast_meat".into(), 3228);
                }
                1 => d.members = false,
                2 => d.stackable = true,
                3 => d.name = "Raw beast meat".into(),
                4 => d.inventory_ops[0] = None,
                5 => d.transform.template = 65534,
                _ => {
                    foods.insert(
                        9988,
                        Edible {
                            id: 9988,
                            heal: 99,
                            delay: 3,
                            replacement: None,
                            message: "Existing".into(),
                        },
                    );
                }
            }
            let before = format!("{foods:?}");
            assert!(hunter_roast_food(&objects, &defs, &mut foods).is_err());
            assert_eq!(format!("{foods:?}"), before);
        }
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::from([(
            3228,
            Edible {
                id: 3228,
                heal: 5,
                delay: 3,
                replacement: None,
                message: "Cooked rabbit".into(),
            },
        )]);
        assert_eq!(
            hunter_roast_food(&objects, &defs, &mut foods)
                .unwrap()
                .len(),
            3
        );
        assert_eq!(foods[&3228].heal, 5);
        assert_eq!(foods[&7223].heal, 7);
    }
    #[test]
    fn hunter_roast_food_three_native_member_selected_fullpack_actions_heal6_7_8_and_preserve_overheal_duplicate_and_xp(
    ) {
        for &(_, id, _, heal) in PROFILES {
            for hp in [50, 98, 99, 115] {
                let (objects, defs) = fixture();
                let mut w = World::default();
                let ids = hunter_roast_food(&objects, &defs, &mut w.foods).unwrap();
                w.member_foods = ids.clone();
                w.slot_preserving_foods = ids;
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
                    "Hunterroast".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    stats,
                );
                for slot in 0..28 {
                    data.inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: if slot >= 26 { id } else { 995 },
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                w.queue_login(1, data).unwrap();
                w.tick().unwrap();
                let a = w.actors().next().unwrap();
                w.submit(a, Action::EatItemSlot { slot: 27, item: id })
                    .unwrap();
                w.tick().unwrap();
                assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, id);
                assert_eq!(w.data(a).unwrap().stats.current(3), hp);
                w.set_membership(a, true).unwrap();
                w.set_next_attack_delay(a, 20).unwrap();
                w.drain_events();
                w.submit(a, Action::EatItemSlot { slot: 27, item: id })
                    .unwrap();
                w.tick().unwrap();
                let d = w.data(a).unwrap();
                assert!(d.inventory.get(27).is_none());
                assert_eq!(d.inventory.get(26).unwrap().id, id);
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
                w.submit(a, Action::EatItemSlot { slot: 27, item: id })
                    .unwrap();
                w.tick().unwrap();
                assert_eq!(w.data(a).unwrap().inventory.slots(), before.0.as_slice());
                assert_eq!(w.data(a).unwrap().stats, before.1);
            }
        }
    }
}
