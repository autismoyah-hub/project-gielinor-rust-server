//! Five selected-slot bites reuse the existing food transaction and native FOOD clocks.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
const STAGES: [(&str, i32); 5] = [
    ("hundred_pirate_giant_crab_meat_5", 7521),
    ("hundred_pirate_giant_crab_meat_4", 7523),
    ("hundred_pirate_giant_crab_meat_3", 7524),
    ("hundred_pirate_giant_crab_meat_2", 7525),
    ("hundred_pirate_giant_crab_meat_1", 7526),
];
pub fn giant_crab_food(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    for &(symbol, id) in &STAGES {
        if objects.get(symbol) != Some(&id)
            || foods.contains_key(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != "Cooked giant crab meat"
                    || !d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref() != Some("Eat")
            })
        {
            return Err(format!("Unattested or duplicate giant crab stage {symbol}"));
        }
    }
    for (index, &(_, id)) in STAGES.iter().enumerate() {
        foods.insert(
            id,
            Edible {
                id,
                heal: 2,
                delay: 2,
                replacement: STAGES.get(index + 1).map(|&(_, id)| id),
                message: "You eat some of the giant crab meat.".into(),
            },
        );
    }
    Ok(STAGES.iter().map(|&(_, id)| id).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot_with_boosts,
        inventory::{Inventory, Item, StackType},
        simulation::{Action, PlayerData, World},
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
        for &(symbol, id) in &STAGES {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = "Cooked giant crab meat".into();
            d.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id as u32, d);
        }
        (objects, defs)
    }
    #[test]
    fn giant_crab_food_five_bites_keep_selected_full_slot_and_cap_without_lowering_overheal() {
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        let ids = giant_crab_food(&objects, &defs, &mut foods).unwrap();
        assert_eq!(ids.len(), 5);
        for hp in [50, 98, 99, 115] {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            let original = Item {
                id: 7521,
                count: 1,
                vars: 0,
            };
            for slot in 0..2 {
                inv.restore_slot(slot, Some(original)).unwrap();
            }
            let mut stats = Stats::default();
            stats.set_base(3, 99);
            stats.set_current(3, hp);
            for (index, &(_, id)) in STAGES.iter().enumerate() {
                let food = &foods[&id];
                assert_eq!((food.heal, food.delay), (2, 2));
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
                assert_eq!(
                    inv.get(1).map(|i| i.id),
                    STAGES.get(index + 1).map(|&(_, id)| id)
                );
                assert_eq!(
                    stats.current(3),
                    if hp >= 99 {
                        hp
                    } else {
                        (hp + 2 * (index as i8 + 1)).min(99)
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
                    |_| panic!("stale stage effect")
                )
                .unwrap());
                assert_eq!((inv.clone(), stats.clone()), before);
            }
        }
    }
    #[test]
    fn giant_crab_food_late_stage_binding_or_duplicate_refuses_before_any_registration() {
        for mode in 0..8 {
            let (mut objects, mut defs) = fixture();
            let mut foods = BTreeMap::new();
            let d = defs.items.get_mut(&7526).unwrap();
            match mode {
                0 => {
                    objects.insert("hundred_pirate_giant_crab_meat_1".into(), 7521);
                }
                1 => d.members = false,
                2 => d.stackable = true,
                3 => d.name = "Giant crab meat".into(),
                4 => d.inventory_ops[0] = None,
                5 => d.cert.template = 799,
                6 => d.transform.template = 65534,
                _ => {
                    foods.insert(
                        7526,
                        Edible {
                            id: 7526,
                            heal: 99,
                            delay: 3,
                            replacement: None,
                            message: "Existing".into(),
                        },
                    );
                }
            }
            let before = format!("{foods:?}");
            assert!(giant_crab_food(&objects, &defs, &mut foods).is_err());
            assert_eq!(format!("{foods:?}"), before);
        }
    }
    #[test]
    fn giant_crab_food_actual_native_member_and_two_tick_five_stage_dispatch() {
        let (objects, defs) = fixture();
        let mut w = World::default();
        let ids = giant_crab_food(&objects, &defs, &mut w.foods).unwrap();
        w.member_foods = ids.clone();
        w.slot_preserving_foods = ids;
        w.consumable_rules = Some(
            crate::content::native_consumables(&objects, &defs, &w.foods, &w.potions).unwrap(),
        );
        let mut stats = Stats::default();
        stats.set_base(3, 99);
        stats.set_current(3, 50);
        let mut data = PlayerData::new(
            1,
            "Crab".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if slot == 0 || slot == 27 { 7521 } else { 995 },
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
                item: 7521,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, 7521);
        assert_eq!(w.data(a).unwrap().stats.current(3), 50);
        w.set_membership(a, true).unwrap();
        w.set_next_attack_delay(a, 20).unwrap();
        for (index, &(_, id)) in STAGES.iter().enumerate() {
            w.submit(a, Action::EatItemSlot { slot: 27, item: id })
                .unwrap();
            w.tick().unwrap();
            assert_eq!(
                w.data(a).unwrap().inventory.get(27).map(|i| i.id),
                STAGES.get(index + 1).map(|&(_, id)| id)
            );
            assert_eq!(w.data(a).unwrap().stats.current(3), 52 + 2 * index as i8);
            assert!(w.movement_input_allowed(a).unwrap());
            assert!(w.is_attack_delayed(a).unwrap());
            if let Some(&(_, next)) = STAGES.get(index + 1) {
                w.submit(
                    a,
                    Action::EatItemSlot {
                        slot: 27,
                        item: next,
                    },
                )
                .unwrap();
                w.tick().unwrap();
                assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id, next);
                assert_eq!(w.data(a).unwrap().stats.current(3), 52 + 2 * index as i8);
            }
        }
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 7521);
        assert_eq!(w.data(a).unwrap().stats.current(3), 60);
        for skill in 0..24 {
            assert_eq!(w.data(a).unwrap().stats.fine_xp(skill), 0);
        }
    }
}
