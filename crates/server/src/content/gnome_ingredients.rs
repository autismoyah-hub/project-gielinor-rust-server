//! Gnome ingredients are consumed through their actual native Eat operations.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
const PROFILES: [(&str, i32, &str, u8, i32); 2] = [
    ("equa_leaves", 2128, "Equa leaves", 2, 1),
    ("toads_legs", 2152, "Toad's legs", 6, 3),
];
pub fn gnome_ingredients(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    for &(symbol, id, name, operation, _) in &PROFILES {
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
                    || crate::food_operations::button_operation(d) != Some(operation)
            })
        {
            return Err(format!("Unattested gnome ingredient food {symbol}"));
        }
    }
    for &(_, id, name, _, heal) in &PROFILES {
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
        for &(symbol, id, name, op, _) in &PROFILES {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            d.inventory_ops[if op == 6 { 3 } else { 0 }] = Some("Eat".into());
            defs.items.insert(id as u32, d);
        }
        (objects, defs)
    }
    #[test]
    fn gnome_ingredient_food_binding_refuses_wrong_cache_operation_member_alias_or_duplicate_atomically(
    ) {
        for mode in 0..7 {
            let (mut objects, mut defs) = fixture();
            let mut foods = BTreeMap::new();
            let d = defs.items.get_mut(&2152).unwrap();
            match mode {
                0 => {
                    objects.insert("toads_legs".into(), 2128);
                }
                1 => d.members = false,
                2 => d.stackable = true,
                3 => d.name = "Swamp toad".into(),
                4 => {
                    d.inventory_ops[3] = None;
                    d.inventory_ops[0] = Some("Eat".into());
                }
                5 => d.transform.template = 65534,
                _ => {
                    foods.insert(
                        2152,
                        Edible {
                            id: 2152,
                            heal: 99,
                            delay: 3,
                            replacement: None,
                            message: "Existing".into(),
                        },
                    );
                }
            }
            let before = format!("{foods:?}");
            assert!(gnome_ingredients(&objects, &defs, &mut foods).is_err());
            assert_eq!(format!("{foods:?}"), before);
        }
    }
    #[test]
    fn gnome_ingredient_food_decoded_native_option4_and_real_equaleaves_potion_toad_combo_dispatch()
    {
        let (objects, mut defs) = fixture();
        let mut w = World::default();
        let ids = gnome_ingredients(&objects, &defs, &mut w.foods).unwrap();
        w.member_foods = ids.clone();
        w.slot_preserving_foods = ids;
        assert_eq!(w.foods[&2128].heal, 1);
        assert_eq!(w.foods[&2152].heal, 3);
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
            crate::content::native_consumables(&objects, &defs, &w.foods, &w.potions).unwrap(),
        );
        assert_eq!(
            w.consumable_rules.as_ref().unwrap().foods[&2152].kind,
            gielinor_world::consumable_clocks::Kind::ComboFood
        );
        assert_eq!(
            w.consumable_rules.as_ref().unwrap().foods[&2128].kind,
            gielinor_world::consumable_clocks::Kind::Food
        );
        let mut stats = Stats::default();
        stats.set_base(3, 99);
        stats.set_current(3, 50);
        let mut data = PlayerData::new(
            1,
            "Ingredients".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        for (slot, id) in [(0, 2128), (1, 2), (26, 2152), (27, 2152)] {
            data.inventory
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
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        let make = |object: i32, op: u8| {
            let mut p = (149u32 << 16).to_be_bytes().to_vec();
            p.extend(27u16.to_be_bytes());
            p.extend((object as u16).to_be_bytes());
            p.push(op);
            gielinor_protocol::buttons::button(&p).unwrap()
        };
        assert!(crate::food_operations::held_action(
            &make(2152, 2),
            27,
            2152,
            &w.foods,
            &defs,
            true
        )
        .is_none());
        let toad =
            crate::food_operations::held_action(&make(2152, 6), 27, 2152, &w.foods, &defs, true)
                .unwrap();
        w.submit(a, toad.clone()).unwrap();
        w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, 2152);
        assert_eq!(w.data(a).unwrap().stats.current(3), 50);
        w.set_membership(a, true).unwrap();
        w.set_next_attack_delay(a, 20).unwrap();
        w.drain_events();
        let stale =
            crate::food_operations::held_action(&make(2128, 2), 27, 2152, &w.foods, &defs, true)
                .unwrap();
        w.submit(a, stale).unwrap();
        for action in [
            Action::EatItemSlot {
                slot: 0,
                item: 2128,
            },
            Action::DrinkSlot {
                slot: 1,
                item: 2,
                empty: false,
            },
            toad,
        ] {
            w.submit(a, action).unwrap();
        }
        w.tick().unwrap();
        let d = w.data(a).unwrap();
        assert!(d.inventory.get(0).is_none());
        assert!(d.inventory.get(27).is_none());
        assert_eq!(d.inventory.get(26).unwrap().id, 2152);
        assert_eq!(d.inventory.get(1).unwrap().id, 9);
        assert_eq!(d.stats.current(3), 54);
        for skill in 0..24 {
            assert_eq!(d.stats.fine_xp(skill), 0);
        }
        assert!(w.movement_input_allowed(a).unwrap());
        let t = w
            .drain_events()
            .into_iter()
            .filter_map(|e| {
                if let WorldEvent::FoodTransition { trace, .. } = e {
                    Some(trace)
                } else {
                    None
                }
            })
            .find(|t| t.before.id == 2152)
            .unwrap();
        assert_eq!(t.attack_delay, Some(25));
    }
}
