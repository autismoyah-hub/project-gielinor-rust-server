//! Ordinary cooked karambwan uses the existing selected food and native combo clocks.
use super::{Definitions, Edible, Symbols};
use std::collections::BTreeMap;
pub fn cooked_karambwan(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<i32, String> {
    const ID: i32 = 3144;
    if objects.get("tbwt_cooked_karambwan") != Some(&ID)
        || foods.contains_key(&ID)
        || defs.items.get(&(ID as u32)).is_none_or(|d| {
            d.name != "Cooked karambwan"
                || !d.members
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
                || d.inventory_ops[0].as_deref() != Some("Eat")
        })
    {
        return Err("Unattested or duplicate ordinary cooked karambwan".into());
    }
    foods.insert(
        ID,
        Edible {
            id: ID,
            heal: 18,
            delay: 3,
            replacement: None,
            message: "You eat the cooked karambwan.".into(),
        },
    );
    Ok(ID)
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
        objects.insert("tbwt_cooked_karambwan".into(), 3144);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
        d.name = "Cooked karambwan".into();
        d.inventory_ops[0] = Some("Eat".into());
        defs.items.insert(3144, d);
        (objects, defs)
    }
    #[test]
    fn cooked_karambwan18_healing_removes_selected_item_and_preserves_duplicate_cap_and_overheal() {
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        assert_eq!(cooked_karambwan(&objects, &defs, &mut foods).unwrap(), 3144);
        assert_eq!(foods.len(), 1);
        assert!(!foods.contains_key(&24595));
        let f = &foods[&3144];
        assert_eq!((f.heal, f.delay, f.replacement), (18, 3, None));
        for hp in [50, 98, 99, 115] {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            let item = Item {
                id: 3144,
                count: 1,
                vars: 0,
            };
            for slot in 0..2 {
                inv.restore_slot(slot, Some(item)).unwrap();
            }
            let mut stats = Stats::default();
            stats.set_base(3, 99);
            stats.set_current(3, hp);
            assert!(eat_slot_with_boosts(
                f,
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
            assert!(inv.get(1).is_none());
            assert_eq!(inv.get(0), Some(item));
            assert_eq!(
                stats.current(3),
                if hp >= 99 { hp } else { (hp + 18).min(99) }
            );
            for skill in 0..24 {
                assert_eq!(stats.fine_xp(skill), 0);
            }
            let before = (inv.clone(), stats.clone());
            assert!(!eat_slot_with_boosts(
                f,
                1,
                &mut inv,
                &mut stats,
                &Default::default(),
                3,
                829,
                &[],
                |_| panic!("stale effect")
            )
            .unwrap());
            assert_eq!((inv, stats), before);
        }
    }
    #[test]
    fn cooked_karambwan_wrong_alias_member_operation_template_or_duplicate_is_refused() {
        for mode in 0..7 {
            let (mut objects, mut defs) = fixture();
            let mut foods = BTreeMap::new();
            let d = defs.items.get_mut(&3144).unwrap();
            match mode {
                0 => {
                    objects.insert("tbwt_cooked_karambwan".into(), 24595);
                }
                1 => d.members = false,
                2 => d.stackable = true,
                3 => d.name = "Blighted karambwan".into(),
                4 => d.inventory_ops[0] = None,
                5 => d.transform.template = 65534,
                _ => {
                    foods.insert(
                        3144,
                        Edible {
                            id: 3144,
                            heal: 99,
                            delay: 3,
                            replacement: None,
                            message: "Existing".into(),
                        },
                    );
                }
            }
            let before = format!("{foods:?}");
            assert!(cooked_karambwan(&objects, &defs, &mut foods).is_err());
            assert_eq!(format!("{foods:?}"), before);
        }
    }
    #[test]
    fn cooked_karambwan_actual_native_food_potion_combo_and_combo_first_refusal() {
        for combo_first in [false, true] {
            let (objects, mut defs) = fixture();
            let mut w = World::default();
            let id = cooked_karambwan(&objects, &defs, &mut w.foods).unwrap();
            w.member_foods.insert(id);
            w.slot_preserving_foods.insert(id);
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
                    next: 9,
                    vial: 9,
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
                "Karambwan".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            for slot in 0..28 {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: match slot {
                                0 => 1,
                                1 => 2,
                                26 | 27 => 3144,
                                _ => 995,
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
            w.submit(
                a,
                Action::EatItemSlot {
                    slot: 27,
                    item: 3144,
                },
            )
            .unwrap();
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, 3144);
            assert_eq!(w.data(a).unwrap().stats.current(3), 50);
            w.set_membership(a, true).unwrap();
            w.set_next_attack_delay(a, 20).unwrap();
            w.drain_events();
            let order = if combo_first { [2, 0, 1] } else { [0, 1, 2] };
            for which in order {
                w.submit(
                    a,
                    match which {
                        0 => Action::EatItemSlot { slot: 0, item: 1 },
                        1 => Action::DrinkSlot {
                            slot: 1,
                            item: 2,
                            empty: false,
                        },
                        _ => Action::EatItemSlot {
                            slot: 27,
                            item: 3144,
                        },
                    },
                )
                .unwrap();
            }
            w.tick().unwrap();
            let p = w.data(a).unwrap();
            assert!(p.inventory.get(27).is_none());
            assert_eq!(p.inventory.get(26).unwrap().id, 3144);
            assert_eq!(p.stats.current(3), if combo_first { 68 } else { 69 });
            assert_eq!(p.inventory.get(0).is_some(), combo_first);
            assert_eq!(
                p.inventory.get(1).unwrap().id,
                if combo_first { 2 } else { 9 }
            );
            for skill in 0..24 {
                assert_eq!(p.stats.fine_xp(skill), 0);
            }
            assert!(w.movement_input_allowed(a).unwrap());
            let trace = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::FoodTransition { trace, .. } = e {
                        Some(trace)
                    } else {
                        None
                    }
                })
                .find(|t| t.before.id == 3144)
                .unwrap();
            assert_eq!(trace.attack_delay, Some(if combo_first { 22 } else { 25 }));
        }
    }
}
