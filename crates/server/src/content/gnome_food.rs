//! Finished gnome foods keep individual normal/premade identities and Wiki healing.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
// Native239 binding facts; healing is taken from each individual pinned Wiki page.
const PROFILES: &[(&str, i32, &str, i32)] = &[
    ("toad_crunchies", 2217, "Toad crunchies", 8),
    ("premade_toad_crunchies", 2243, "Premade t'd crunch", 8),
    ("spicy_crunchies", 2213, "Spicy crunchies", 7),
    ("premade_spicy_crunchies", 2241, "Premade s'y crunch", 7),
    ("worm_crunchies", 2205, "Worm crunchies", 8),
    ("premade_worm_crunchies", 2237, "Premade w'm crun'", 8),
    ("chocchip_crunchies", 2209, "Chocchip crunchies", 7),
    ("premade_chocchip_crunchies", 2239, "Premade ch' crunch", 8),
    ("fruit_batta", 2277, "Fruit batta", 11),
    ("premade_fruit_batta", 2225, "Premade fr't batta", 11),
    ("toad_batta", 2255, "Toad batta", 12),
    ("premade_toad_batta", 2221, "Premade t'd batta", 11),
    ("worm_batta", 2253, "Worm batta", 12),
    ("premade_worm_batta", 2219, "Premade w'm batta", 11),
    ("vegetable_batta", 2281, "Vegetable batta", 11),
    ("premade_vegetable_batta", 2227, "Premade veg batta", 11),
    ("cheese+tom_batta", 2259, "Cheese+tom batta", 11),
    ("premade_cheese+tom_batta", 2223, "Premade c+t batta", 11),
    ("worm_hole", 2191, "Worm hole", 12),
    ("premade_worm_hole", 2233, "Premade worm hole", 12),
    ("veg_ball", 2195, "Veg ball", 12),
    ("premade_veg_ball", 2235, "Premade veg ball", 12),
    ("chocolate_bomb", 2185, "Chocolate bomb", 15),
    ("premade_chocolate_bomb", 2229, "Premade choc bomb", 15),
    ("tangled_toads_legs", 2187, "Tangled toad's legs", 15),
    ("premade_tangled_toads_legs", 2231, "Premade ttl", 15),
];
pub fn gnome_food(
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
            return Err(format!("Unattested or duplicate gnome food {symbol}"));
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
        food::eat_slot_with_boosts,
        inventory::{Inventory, Item, StackType},
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
    fn gnome_food_individual_heals_do_not_alias_normal_and_premade_and_selected_transaction_preserves_overheal(
    ) {
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        assert_eq!(gnome_food(&objects, &defs, &mut foods).unwrap().len(), 26);
        for (id, heal) in [
            (2213, 7),
            (2241, 7),
            (2209, 7),
            (2239, 8),
            (2255, 12),
            (2221, 11),
            (2253, 12),
            (2219, 11),
        ] {
            assert_eq!(foods[&id].heal, heal);
        }
        for &(_, id, _, heal) in PROFILES {
            for hp in [50, 98, 99, 115] {
                let mut inv = Inventory::new(2, StackType::Normal, false);
                let original = Item {
                    id,
                    count: 1,
                    vars: 0,
                };
                for slot in 0..2 {
                    inv.restore_slot(slot, Some(original)).unwrap();
                }
                let mut stats = Stats::default();
                stats.set_base(3, 99);
                stats.set_current(3, hp);
                let food = &foods[&id];
                assert_eq!((food.delay, food.replacement), (3, None));
                assert!(eat_slot_with_boosts(
                    food,
                    1,
                    &mut inv,
                    &mut stats,
                    &Default::default(),
                    3,
                    829,
                    &[],
                    |_| {}
                )
                .unwrap());
                assert_eq!(inv.get(0), Some(original));
                assert!(inv.get(1).is_none());
                assert_eq!(
                    stats.current(3),
                    if hp >= 99 {
                        hp
                    } else {
                        (hp as i32 + heal).min(99) as i8
                    }
                );
                for skill in 0..24 {
                    assert_eq!(stats.fine_xp(skill), 0);
                }
                let before = (inv.clone(), stats.clone());
                assert!(!eat_slot_with_boosts(
                    food,
                    1,
                    &mut inv,
                    &mut stats,
                    &Default::default(),
                    3,
                    829,
                    &[],
                    |_| panic!("stale food effect")
                )
                .unwrap());
                assert_eq!((inv, stats), before);
            }
        }
    }
    #[test]
    fn gnome_food_last_profile_binding_or_duplicate_failure_has_no_partial_registration() {
        for mode in 0..8 {
            let (mut objects, mut defs) = fixture();
            let mut foods = BTreeMap::new();
            let d = defs.items.get_mut(&2231).unwrap();
            match mode {
                0 => {
                    objects.insert("premade_tangled_toads_legs".into(), 2187);
                }
                1 => d.members = false,
                2 => d.stackable = true,
                3 => d.name = "Tangled toad's legs".into(),
                4 => d.inventory_ops[0] = None,
                5 => d.placeholder.template = 14401,
                6 => d.transform.template = 65534,
                _ => {
                    foods.insert(
                        2231,
                        Edible {
                            id: 2231,
                            heal: 99,
                            delay: 3,
                            replacement: None,
                            message: "Existing".into(),
                        },
                    );
                }
            }
            let before = format!("{foods:?}");
            assert!(gnome_food(&objects, &defs, &mut foods).is_err());
            assert_eq!(format!("{foods:?}"), before);
        }
    }
    #[test]
    fn gnome_food_all26_actual_native_member_combo_actions_block_later_food_and_potion_with_active_attack_trace(
    ) {
        for &(_, id, _, heal) in PROFILES {
            let (objects, mut defs) = fixture();
            let mut w = World::default();
            let ids = gnome_food(&objects, &defs, &mut w.foods).unwrap();
            w.member_foods = ids.clone();
            w.slot_preserving_foods = ids;
            for id in [1, 2] {
                let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
                d.inventory_ops[0] = Some(if id == 1 { "Eat" } else { "Drink" }.into());
                defs.items.insert(id, d);
            }
            w.foods.insert(
                1,
                Edible {
                    id: 1,
                    heal: 1,
                    delay: 3,
                    replacement: None,
                    message: "Food".into(),
                },
            );
            w.potions.insert(
                2,
                Potion {
                    id: 2,
                    next: 3,
                    vial: 3,
                    sip: "Sip".into(),
                    remaining: "Left".into(),
                    effects: vec![],
                },
            );
            w.consumable_rules = Some(
                crate::content::native_consumables(&objects, &defs, &w.foods, &w.potions).unwrap(),
            );
            let mut stats = Stats::default();
            stats.set_base(3, 99);
            stats.set_current(3, 50);
            let mut data = PlayerData::new(
                1,
                "Gnome".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            for (slot, item) in [id, 1, 2].into_iter().enumerate() {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: item,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.submit(a, Action::EatItemSlot { slot: 0, item: id })
                .unwrap();
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, id);
            assert_eq!(w.data(a).unwrap().stats.current(3), 50);
            w.set_membership(a, true).unwrap();
            w.set_next_attack_delay(a, 20).unwrap();
            w.drain_events();
            for action in [
                Action::EatItemSlot { slot: 0, item: id },
                Action::EatItemSlot { slot: 1, item: 1 },
                Action::DrinkSlot {
                    slot: 2,
                    item: 2,
                    empty: false,
                },
            ] {
                w.submit(a, action).unwrap();
            }
            w.tick().unwrap();
            let p = w.data(a).unwrap();
            assert!(p.inventory.get(0).is_none());
            assert_eq!(p.inventory.get(1).unwrap().id, 1);
            assert_eq!(p.inventory.get(2).unwrap().id, 2);
            assert_eq!(p.stats.current(3), 50 + heal as i8);
            for skill in 0..24 {
                assert_eq!(p.stats.fine_xp(skill), 0);
            }
            assert!(w.movement_input_allowed(a).unwrap());
            let traces: Vec<_> = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::FoodTransition { trace, .. } = e {
                        Some(trace)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(traces.len(), 1);
            assert_eq!(traces[0].attack_delay, Some(22));
        }
    }
}
