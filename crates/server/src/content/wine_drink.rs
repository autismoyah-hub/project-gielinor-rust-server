//! Normal wine uses the ordinary food transaction with Wiki container retention.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::StatSubtraction;
use std::collections::BTreeMap;

pub fn wine_drink(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<(i32, StatSubtraction), String> {
    for (symbol, id, name, min) in [
        ("attack", 0, "Attack", 1),
        ("hitpoints", 3, "Hitpoints", 10),
    ] {
        if stats.get(symbol) != Some(&id)
            || defs.stats.get(&(id as u32)).is_none_or(|d| {
                d.name != name || d.unreleased || d.min_level != min || d.max_level != 99
            })
        {
            return Err(format!("Unattested wine stat {symbol}"));
        }
    }
    for (symbol, id, name, op) in [
        ("jug_wine", 1993, "Jug of wine", Some("Drink")),
        ("jug_empty", 1935, "Jug", None),
    ] {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref() != op
            })
        {
            return Err(format!("Unattested ordinary wine item {symbol}"));
        }
    }
    if foods.contains_key(&1993) {
        return Err("Wine drink already registered".into());
    }
    foods.insert(
        1993,
        Edible {
            id: 1993,
            heal: 11,
            delay: 3,
            replacement: Some(1935),
            message: "You drink the wine.".into(),
        },
    );
    Ok((
        1993,
        StatSubtraction {
            skill: 0,
            amount: 2,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let objects = Symbols::from([("jug_wine".into(), 1993), ("jug_empty".into(), 1935)]);
        let stats = Symbols::from([("attack".into(), 0), ("hitpoints".into(), 3)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (id, name, drink) in [(1993, "Jug of wine", true), (1935, "Jug", false)] {
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            if drink {
                d.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id, d);
        }
        for (id, name, min) in [(0, "Attack", 1), (3, "Hitpoints", 10)] {
            let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
            d.name = name.into();
            d.min_level = min;
            defs.stats.insert(id, d);
        }
        (objects, stats, defs)
    }
    #[test]
    fn wine_drink_requires_exact_f2p_drink_container_and_stats_before_registration() {
        let (objects, stats, mut defs) = fixture();
        let mut foods = BTreeMap::new();
        let (id, effect) = wine_drink(&objects, &stats, &defs, &mut foods).unwrap();
        assert_eq!((id, effect.skill, effect.amount), (1993, 0, 2));
        let food = &foods[&id];
        assert_eq!(
            (food.heal, food.delay, food.replacement),
            (11, 3, Some(1935))
        );
        assert!(wine_drink(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 1);
        for mode in 0..7 {
            let (objects, stats, mut d) = fixture();
            match mode {
                0 => d.items.get_mut(&1993).unwrap().inventory_ops[0] = Some("Eat".into()),
                1 => d.items.get_mut(&1935).unwrap().stackable = true,
                2 => d.items.get_mut(&1993).unwrap().members = true,
                3 => d.items.get_mut(&1935).unwrap().transform.template = 65534,
                4 => d.stats.get_mut(&0).unwrap().unreleased = true,
                5 => d.items.get_mut(&1935).unwrap().name = "Wine".into(),
                _ => {
                    d.stats.remove(&3);
                }
            }
            let mut f = BTreeMap::new();
            assert!(wine_drink(&objects, &stats, &d, &mut f).is_err());
            assert!(f.is_empty());
        }
        defs.items.get_mut(&1935).unwrap().placeholder.template = 14401;
        let mut f = BTreeMap::new();
        assert!(wine_drink(&objects, &stats, &defs, &mut f).is_err());
        assert!(f.is_empty());
    }
}
