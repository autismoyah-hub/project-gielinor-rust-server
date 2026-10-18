//! Dorgesh-Kaan manual consumption, with pinned Wiki corrections to donor rows.
use super::{apply_ordinary_food_profiles, BoostFoodBindings, Definitions, Edible, Symbols};
use gielinor_world::food::StatBoost;
use std::collections::{BTreeMap, BTreeSet};

pub fn dorgesh_foods(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BoostFoodBindings, String> {
    let cooking = *stats.get("cooking").ok_or("Missing Dorgesh Cooking stat")?;
    if cooking != 7
        || defs.stats.get(&7).is_none_or(|d| {
            d.name != "Cooking" || d.unreleased || d.min_level != 1 || d.max_level != 99
        })
    {
        return Err("Unattested Dorgesh Cooking stat".into());
    }
    let profiles = [
        ("dorgesh_bat_shish", 10964, "Bat shish", 2),
        ("dorgesh_green_gloop_soup", 10960, "Green gloop soup", 2),
        ("dorgesh_frog_spawn_gumbo", 10961, "Frogspawn gumbo", 2),
        ("dorgesh_sauteed_mushrooms", 10968, "Mushrooms", 2),
        ("dorgesh_cave_eel_sushi", 10971, "Eel sushi", 10),
        ("dorgesh_frog_burger", 10962, "Frogburger", 2),
    ];
    let mut selected = BTreeSet::new();
    for &(symbol, id, _, _) in &profiles {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| !d.members)
            || !selected.insert(id)
        {
            return Err(format!("Unattested ordinary Dorgesh food {symbol}"));
        }
    }
    let mut next = foods.clone();
    let ordinary: Vec<_> = profiles
        .iter()
        .map(|&(symbol, _, name, heal)| (symbol, name, heal))
        .collect();
    apply_ordinary_food_profiles(objects, defs, &mut next, &ordinary)?;
    next.get_mut(&10964).unwrap().message = "You eat the kebab.".into();
    let boosts = [10968, 10971]
        .into_iter()
        .map(|id| {
            (
                id,
                vec![StatBoost {
                    skill: cooking as u8,
                    amount: 1,
                }],
            )
        })
        .collect();
    *foods = next;
    Ok(BoostFoodBindings {
        selected,
        boosts,
        energy: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot_with_boosts,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    #[test]
    fn dorgesh_foods_use_wiki_heals_and_cooking_boosts_without_quest_gates_or_partial_catalog() {
        let mut objects = Symbols::new();
        let stats = Symbols::from([("cooking".into(), 7)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut cooking = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
        cooking.name = "Cooking".into();
        defs.stats.insert(7, cooking);
        for (symbol, id, name) in [
            ("dorgesh_bat_shish", 10964, "Bat shish"),
            ("dorgesh_green_gloop_soup", 10960, "Green gloop soup"),
            ("dorgesh_frog_spawn_gumbo", 10961, "Frogspawn gumbo"),
            ("dorgesh_sauteed_mushrooms", 10968, "Mushrooms"),
            ("dorgesh_cave_eel_sushi", 10971, "Eel sushi"),
            ("dorgesh_frog_burger", 10962, "Frogburger"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            d.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id as u32, d);
        }
        let mut foods = BTreeMap::new();
        let bindings = dorgesh_foods(&objects, &stats, &defs, &mut foods).unwrap();
        assert_eq!(bindings.selected.len(), 6);
        assert_eq!(bindings.boosts.len(), 2);
        assert!(bindings.energy.is_empty());
        for (id, heal, boost) in [
            (10964, 2, false),
            (10960, 2, false),
            (10961, 2, false),
            (10968, 2, true),
            (10971, 10, true),
            (10962, 2, false),
        ] {
            let food = &foods[&id];
            assert_eq!((food.heal, food.delay, food.replacement), (heal, 3, None));
            assert_eq!(bindings.boosts.contains_key(&id), boost);
            for (hp, cooking) in [(1, 99), (98, 50), (115, 105)] {
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
                let mut state = Stats::default();
                state.set_base(3, 99);
                state.set_current(3, hp);
                state.set_base(7, 99);
                state.set_current(7, cooking);
                state.set_fine_xp(7, 12345).unwrap();
                assert_eq!(
                    eat_slot_with_boosts(
                        food,
                        1,
                        &mut inv,
                        &mut state,
                        &Default::default(),
                        3,
                        829,
                        bindings.boosts.get(&id).map(Vec::as_slice).unwrap_or(&[]),
                        |_| {}
                    ),
                    Ok(true)
                );
                assert_eq!(
                    state.current(3),
                    if hp > 99 {
                        hp
                    } else {
                        (i32::from(hp) + heal).min(99) as i8
                    }
                );
                assert_eq!(
                    state.current(7),
                    if boost && cooking <= 99 {
                        cooking + 1
                    } else {
                        cooking
                    }
                );
                assert_eq!(state.fine_xp(3), 0);
                assert_eq!(state.fine_xp(7), 12345);
                assert_eq!(inv.get(0).unwrap().id, id);
                assert!(inv.get(1).is_none());
            }
        }
        foods.clear();
        defs.items.get_mut(&10962).unwrap().transform.template = 1;
        assert!(dorgesh_foods(&objects, &stats, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
        defs.items.get_mut(&10962).unwrap().transform.template = 0;
        defs.stats.get_mut(&7).unwrap().name = "Fishing".into();
        assert!(dorgesh_foods(&objects, &stats, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
    }
}
