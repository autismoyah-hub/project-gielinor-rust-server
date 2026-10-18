//! Manual bowl-food consumption; preparation, cooking and bait use are separate.
use super::{apply_multipart_foods, apply_ordinary_food_profiles, Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};

pub fn bowl_foods(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    let bowl = *objects.get("bowl_empty").ok_or("Missing empty bowl")?;
    let d = defs
        .items
        .get(&(bowl as u32))
        .ok_or("Missing empty bowl definition")?;
    if bowl != 1923
        || d.name != "Bowl"
        || d.members
        || d.stackable
        || d.dummy == 1
        || d.cert.template != 0
        || d.placeholder.template != 0
        || d.transform.template != 0
    {
        return Err("Invalid ordinary empty bowl".into());
    }
    let profiles = [
        ("bowl_onion", 1871, "Chopped onion", 1),
        ("bowl_tomato", 1869, "Chopped tomato", 2),
        ("bowl_chilli", 7072, "Spicy sauce", 2),
        ("bowl_carne", 7070, "Minced meat", 2),
        ("bowl_spicytomato", 9994, "Spicy tomato", 2),
        ("bowl_oniontomato", 1875, "Onion & tomato", 3),
        ("bowl_spicymeat", 9996, "Spicy minced meat", 3),
        ("bowl_egg_scrambled", 7078, "Scrambled egg", 5),
        ("bowl_onion_fried", 7084, "Fried onions", 5),
        ("bowl_chilli+carne", 7062, "Chilli con carne", 5),
        ("bowl_egg+tomato", 7064, "Egg and tomato", 8),
        ("bowl_mushroom+onion", 7066, "Mushroom & onion", 11),
        ("bowl_tuna+sweetcorn", 7068, "Tuna and corn", 13),
        ("potato_egg+tomato", 7056, "Egg potato", 16),
    ];
    for &(symbol, id, _, _) in &profiles {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| !d.members)
        {
            return Err(format!("Unattested member bowl food {symbol}"));
        }
    }
    let mut next = foods.clone();
    let ordinary: Vec<_> = profiles
        .iter()
        .map(|&(symbol, _, name, heal)| (symbol, name, heal))
        .collect();
    apply_ordinary_food_profiles(objects, defs, &mut next, &ordinary)?;
    let remainders: Vec<_> = profiles
        .iter()
        .map(|&(symbol, id, _, heal)| (symbol, (id != 7056).then_some("bowl_empty"), heal))
        .collect();
    let selected = apply_multipart_foods(objects, defs, &mut next, &remainders)?;
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
    fn bowl_foods_return_containers_at_selected_slot_and_keep_egg_potato_containerless() {
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
        for (symbol, id, name) in [
            ("bowl_onion", 1871, "Chopped onion"),
            ("bowl_tomato", 1869, "Chopped tomato"),
            ("bowl_chilli", 7072, "Spicy sauce"),
            ("bowl_carne", 7070, "Minced meat"),
            ("bowl_spicytomato", 9994, "Spicy tomato"),
            ("bowl_oniontomato", 1875, "Onion & tomato"),
            ("bowl_spicymeat", 9996, "Spicy minced meat"),
            ("bowl_egg_scrambled", 7078, "Scrambled egg"),
            ("bowl_onion_fried", 7084, "Fried onions"),
            ("bowl_chilli+carne", 7062, "Chilli con carne"),
            ("bowl_egg+tomato", 7064, "Egg and tomato"),
            ("bowl_mushroom+onion", 7066, "Mushroom & onion"),
            ("bowl_tuna+sweetcorn", 7068, "Tuna and corn"),
            ("potato_egg+tomato", 7056, "Egg potato"),
            ("bowl_empty", 1923, "Bowl"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if id != 1923 {
                d.members = true;
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let mut foods = BTreeMap::new();
        assert_eq!(bowl_foods(&objects, &defs, &mut foods).unwrap().len(), 14);
        assert!(!foods.contains_key(&1923));
        for (id, heal) in [
            (1871, 1),
            (1869, 2),
            (7072, 2),
            (7070, 2),
            (9994, 2),
            (1875, 3),
            (9996, 3),
            (7078, 5),
            (7084, 5),
            (7062, 5),
            (7064, 8),
            (7066, 11),
            (7068, 13),
            (7056, 16),
        ] {
            let food = &foods[&id];
            assert_eq!((food.heal, food.delay), (heal, 3));
            assert_eq!(food.replacement, if id == 7056 { None } else { Some(1923) });
            for current in [1, 98, 115] {
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
                assert_eq!(
                    stats.current(3),
                    if current > 99 {
                        current
                    } else {
                        (i32::from(current) + heal).min(99) as i8
                    }
                );
                assert_eq!(stats.fine_xp(3), 0);
                assert_eq!(inv.get(0).unwrap().id, id);
                assert_eq!(inv.get(1).map(|i| i.id), food.replacement);
            }
        }
        foods.clear();
        defs.items.get_mut(&7056).unwrap().inventory_ops[0] = None;
        assert!(bowl_foods(&objects, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
        defs.items.get_mut(&7056).unwrap().inventory_ops[0] = Some("Eat".into());
        defs.items.get_mut(&1923).unwrap().stackable = true;
        assert!(bowl_foods(&objects, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
    }
}
