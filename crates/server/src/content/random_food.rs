//! Variable-healing ordinary foods, independently pinned to Wiki/source rules.
use super::{Definitions, Symbols};
use gielinor_world::food::{Edible, RandomHealing};
use std::collections::{BTreeMap, BTreeSet};

pub struct RandomFoodBindings {
    pub selected: BTreeSet<i32>,
    pub healing: BTreeMap<i32, RandomHealing>,
}

pub fn random_foods(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<RandomFoodBindings, String> {
    let mut next = foods.clone();
    let mut healing = BTreeMap::new();
    let stick = *objects
        .get("tbw_skewer_stick")
        .ok_or("Missing skewer stick")?;
    let d = defs
        .items
        .get(&(stick as u32))
        .ok_or("Missing skewer definition")?;
    if stick != 6305
        || d.name != "Skewer stick"
        || !d.members
        || !d.stackable
        || d.dummy == 1
        || d.cert.template != 0
        || d.placeholder.template != 0
        || d.transform.template != 0
    {
        return Err("Unattested ordinary skewer stick".into());
    }
    for (symbol, id, name, minimum, maximum) in [
        ("snail_corpse_cooked1", 3369, "Thin snail meat", 5, 7),
        ("snail_corpse_cooked2", 3371, "Lean snail meat", 5, 8),
        ("snail_corpse_cooked3", 3373, "Fat snail meat", 7, 9),
        ("tbw_spider_on_stick_cooked", 6297, "Spider on stick", 7, 10),
        ("tbw_spider_on_shaft_cooked", 6299, "Spider on shaft", 7, 10),
        ("giant_frogspawn", 5004, "Frog spawn", 3, 6),
        ("mort_slimey_eel_cooked", 3381, "Cooked slimy eel", 6, 10),
        ("cave_eel", 5003, "Cave eel", 8, 12),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing or aliased random food {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing random food definition")?;
        if d.name != name
            || !d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || d.inventory_ops[0].as_deref() != Some("Eat")
            || next.contains_key(&id)
        {
            return Err(format!("Invalid ordinary random food {symbol}"));
        }
        let message = match id {
            6297 => "You eat the spider. Mmmm, crunchy!".into(),
            5004 => "You eat the frogspawn. Yuck.".into(),
            _ => format!("You eat the {}.", name.to_lowercase()),
        };
        next.insert(
            id,
            Edible {
                id,
                heal: minimum.into(),
                delay: 3,
                replacement: (id == 6297).then_some(stick),
                message,
            },
        );
        healing.insert(id, RandomHealing { minimum, maximum });
    }
    let selected = healing.keys().copied().collect();
    *foods = next;
    Ok(RandomFoodBindings { selected, healing })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Catalog, Inventory, Item, StackType},
        npc::RandomSource,
        stats::Stats,
    };
    struct Endpoint(bool);
    impl RandomSource for Endpoint {
        fn below(&mut self, n: u32) -> u32 {
            if self.0 {
                n - 1
            } else {
                0
            }
        }
    }
    #[test]
    fn random_food_profiles_heal_inclusive_endpoints_and_return_only_the_skewer() {
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
            ("snail_corpse_cooked1", 3369, "Thin snail meat"),
            ("snail_corpse_cooked2", 3371, "Lean snail meat"),
            ("snail_corpse_cooked3", 3373, "Fat snail meat"),
            ("tbw_spider_on_stick_cooked", 6297, "Spider on stick"),
            ("tbw_spider_on_shaft_cooked", 6299, "Spider on shaft"),
            ("giant_frogspawn", 5004, "Frog spawn"),
            ("mort_slimey_eel_cooked", 3381, "Cooked slimy eel"),
            ("cave_eel", 5003, "Cave eel"),
            ("tbw_skewer_stick", 6305, "Skewer stick"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            if id == 6305 {
                d.stackable = true;
            } else {
                d.inventory_ops[0] = Some("Eat".into());
            }
            defs.items.insert(id as u32, d);
        }
        let mut foods = BTreeMap::new();
        let bindings = random_foods(&objects, &defs, &mut foods).unwrap();
        assert_eq!(bindings.selected.len(), 8);
        let mut catalog = Catalog::default();
        catalog.stackable.insert(6305);
        for (id, minimum, maximum) in [
            (3369, 5, 7),
            (3371, 5, 8),
            (3373, 7, 9),
            (6297, 7, 10),
            (6299, 7, 10),
            (5004, 3, 6),
            (3381, 6, 10),
            (5003, 8, 12),
        ] {
            assert_eq!(bindings.healing[&id], RandomHealing { minimum, maximum });
            for high in [false, true] {
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
                stats.set_current(3, 1);
                gielinor_world::food::eat_slot_with_random_healing(
                    &foods[&id],
                    1,
                    &mut inv,
                    &mut stats,
                    &catalog,
                    3,
                    829,
                    &[],
                    bindings.healing[&id],
                    &mut Endpoint(high),
                    |_| {},
                )
                .unwrap();
                assert_eq!(
                    stats.current(3) as u8,
                    1 + if high { maximum } else { minimum }
                );
                assert_eq!(stats.fine_xp(3), 0);
                assert_eq!(inv.get(0).unwrap().id, id);
                assert_eq!(
                    inv.get(1).map(|i| i.id),
                    if id == 6297 { Some(6305) } else { None }
                );
            }
        }
        foods.clear();
        defs.items.get_mut(&5003).unwrap().transform.template = 1;
        assert!(random_foods(&objects, &defs, &mut foods).is_err());
        assert!(foods.is_empty());
    }
}
