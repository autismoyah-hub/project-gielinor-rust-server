//! Ordinary manual consumption; acquisition/quest/desert-heat scripts are separate.
use super::{apply_ordinary_food_profiles, Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};

pub fn meal_foods(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    let profiles = [
        ("cooked_mystery_meat", 24785, "Cooked mystery meat", true, 5),
        ("baguette", 6961, "Baguette", false, 6),
        ("giant_frog_legs", 4517, "Giant frog legs", true, 6),
        ("giant_carp", 337, "Giant carp", true, 6),
        ("hundred_pirate_fishcake", 7530, "Cooked fishcake", true, 11),
        ("cooked_chompy", 2878, "Cooked chompy", true, 10),
        ("elid_choc_ice", 6794, "Choc-ice", true, 7),
        ("pumpkin", 1959, "Pumpkin", false, 14),
        ("easter_egg", 1961, "Easter egg", false, 14),
        ("cooked_oomlie", 2343, "Cooked oomlie wrap", true, 14),
        ("100_jubbly_meat_cooked", 7568, "Cooked jubbly", true, 15),
        ("potato_baked", 6701, "Baked potato", true, 4),
        ("potato_butter", 6703, "Potato with butter", true, 14),
        ("chocolate_bar", 1973, "Chocolate bar", false, 3),
    ];
    let mut selected = BTreeSet::new();
    for &(symbol, id, _, members, _) in &profiles {
        if objects.get(symbol) != Some(&id)
            || defs
                .items
                .get(&(id as u32))
                .is_none_or(|d| d.members != members)
            || !selected.insert(id)
        {
            return Err(format!("Unattested meal profile {symbol}"));
        }
    }
    let mut next = foods.clone();
    let ordinary: Vec<_> = profiles
        .iter()
        .map(|&(symbol, _, name, _, heal)| (symbol, name, heal))
        .collect();
    apply_ordinary_food_profiles(objects, defs, &mut next, &ordinary)?;
    next.get_mut(&4517).unwrap().message =
        "You eat the giant frog legs. They're a bit chewy.".into();
    next.get_mut(&2343).unwrap().message = "You eat the oomlie meat; it tastes very gamy.".into();
    *foods = next;
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    #[test]
    fn meal_foods_register_exact_ordinary_profiles_and_consume_only_selected_slot() {
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
        for (symbol, id, name, members) in [
            ("cooked_mystery_meat", 24785, "Cooked mystery meat", true),
            ("baguette", 6961, "Baguette", false),
            ("giant_frog_legs", 4517, "Giant frog legs", true),
            ("giant_carp", 337, "Giant carp", true),
            ("hundred_pirate_fishcake", 7530, "Cooked fishcake", true),
            ("cooked_chompy", 2878, "Cooked chompy", true),
            ("elid_choc_ice", 6794, "Choc-ice", true),
            ("pumpkin", 1959, "Pumpkin", false),
            ("easter_egg", 1961, "Easter egg", false),
            ("cooked_oomlie", 2343, "Cooked oomlie wrap", true),
            ("100_jubbly_meat_cooked", 7568, "Cooked jubbly", true),
            ("potato_baked", 6701, "Baked potato", true),
            ("potato_butter", 6703, "Potato with butter", true),
            ("chocolate_bar", 1973, "Chocolate bar", false),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            d.members = members;
            d.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id as u32, d);
        }
        let mut foods = BTreeMap::new();
        assert_eq!(meal_foods(&objects, &defs, &mut foods).unwrap().len(), 14);
        for (id, heal) in [
            (24785, 5),
            (6961, 6),
            (4517, 6),
            (337, 6),
            (7530, 11),
            (2878, 10),
            (6794, 7),
            (1959, 14),
            (1961, 14),
            (2343, 14),
            (7568, 15),
            (6701, 4),
            (6703, 14),
            (1973, 3),
        ] {
            let food = &foods[&id];
            assert_eq!((food.heal, food.delay, food.replacement), (heal, 3, None));
            for (current, expected) in [(1, 1 + heal as i8), (98, 99), (115, 115)] {
                let mut inv = Inventory::new(2, StackType::Normal, false);
                for slot in 0..2 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                let mut stats = Stats::default();
                stats.set_base(3, 99);
                stats.set_current(3, current);
                assert_eq!(
                    eat_slot(
                        food,
                        1,
                        &mut inv,
                        &mut stats,
                        &Default::default(),
                        3,
                        829,
                        |_| {}
                    ),
                    Ok(true)
                );
                assert_eq!(stats.current(3), expected);
                assert_eq!(stats.fine_xp(3), 0);
                assert_eq!(inv.get(0).unwrap().id, id);
                assert!(inv.get(1).is_none());
            }
        }
        foods.clear();
        defs.items.get_mut(&1973).unwrap().transform.template = 1;
        assert!(meal_foods(&objects, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
        defs.items.get_mut(&1973).unwrap().transform.template = 0;
        defs.items.get_mut(&6961).unwrap().members = true;
        assert!(meal_foods(&objects, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
    }
}
