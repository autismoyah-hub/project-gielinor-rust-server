//! Four ordinary non-alcoholic cocktails use Wiki retained-glass consumption.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
pub fn fruit_cocktails(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    let profiles = [
        ("fruit_blast", 2084, "Fruit blast"),
        ("premade_fruit_blast", 2034, "Premade fr' blast"),
        ("pineapple_punch", 2048, "Pineapple punch"),
        ("premade_pineapple_punch", 2036, "Premade p' punch"),
    ];
    for (symbol, id, name) in
        profiles
            .into_iter()
            .chain([("cocktail_glass_empty", 2026, "Cocktail glass")])
    {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || !d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref()
                        != if id == 2026 { None } else { Some("Drink") }
            })
        {
            return Err(format!("Unattested ordinary fruit cocktail {symbol}"));
        }
    }
    if profiles.iter().any(|(_, id, _)| foods.contains_key(id)) {
        return Err("Fruit cocktail already registered".into());
    }
    for (_, id, _) in profiles {
        foods.insert(
            id,
            Edible {
                id,
                heal: 9,
                delay: 3,
                replacement: Some(2026),
                message: "You drink the cocktail. It tastes great.".into(),
            },
        );
    }
    Ok(profiles.into_iter().map(|(_, id, _)| id).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
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
        for (symbol, id, name) in [
            ("fruit_blast", 2084, "Fruit blast"),
            ("premade_fruit_blast", 2034, "Premade fr' blast"),
            ("pineapple_punch", 2048, "Pineapple punch"),
            ("premade_pineapple_punch", 2036, "Premade p' punch"),
            ("cocktail_glass_empty", 2026, "Cocktail glass"),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            if id != 2026 {
                d.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id as u32, d);
        }
        (objects, defs)
    }
    #[test]
    fn fruit_cocktails_four_profiles_retain_selected_glass_heal9_and_never_drain_or_award_xp() {
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        let selected = fruit_cocktails(&objects, &defs, &mut foods).unwrap();
        assert_eq!(selected, BTreeSet::from([2084, 2034, 2048, 2036]));
        assert!(!foods.contains_key(&2026));
        for id in selected {
            let f = &foods[&id];
            assert_eq!((f.heal, f.delay, f.replacement), (9, 3, Some(2026)));
            for hp in [1, 95, 99, 115] {
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
                stats.set_current(3, hp);
                stats.set_current(0, 5);
                stats.set_fine_xp(3, 12345).unwrap();
                let mut events = Vec::new();
                assert!(eat_slot(
                    f,
                    1,
                    &mut inv,
                    &mut stats,
                    &Default::default(),
                    3,
                    829,
                    |e| events.push(e)
                )
                .unwrap());
                assert_eq!(
                    inv.get(0),
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0
                    })
                );
                assert_eq!(
                    inv.get(1),
                    Some(Item {
                        id: 2026,
                        count: 1,
                        vars: 0
                    })
                );
                assert_eq!(
                    stats.current(3),
                    if hp > 99 { hp } else { (hp + 9).min(99) }
                );
                assert_eq!(stats.current(0), 5);
                assert_eq!(stats.fine_xp(3), 12345);
                assert_eq!(stats.fine_xp(7), 0);
                assert!(!events
                    .iter()
                    .any(|e| matches!(e,gielinor_world::food::Event::Stat(u)if u.skill!=3)));
                let before = (inv.clone(), stats.clone());
                assert!(!eat_slot(
                    f,
                    1,
                    &mut inv,
                    &mut stats,
                    &Default::default(),
                    3,
                    829,
                    |_| panic!("stale cocktail event")
                )
                .unwrap());
                assert_eq!((inv, stats), before);
            }
        }
    }
    #[test]
    fn fruit_cocktails_atomic_binding_rejects_bad_terminal_or_late_premade_without_aliasing_regular(
    ) {
        for mode in 0..7 {
            let (mut objects, mut defs) = fixture();
            match mode {
                0 => defs.items.get_mut(&2036).unwrap().inventory_ops[0] = Some("Eat".into()),
                1 => defs.items.get_mut(&2026).unwrap().stackable = true,
                2 => defs.items.get_mut(&2084).unwrap().members = false,
                3 => defs.items.get_mut(&2026).unwrap().transform.template = 65534,
                4 => {
                    objects.insert("premade_fruit_blast".into(), 2084);
                }
                5 => defs.items.get_mut(&2034).unwrap().name = "Fruit blast".into(),
                _ => defs.items.get_mut(&2026).unwrap().members = false,
            };
            let mut foods = BTreeMap::new();
            assert!(fruit_cocktails(&objects, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        fruit_cocktails(&objects, &defs, &mut foods).unwrap();
        assert!(fruit_cocktails(&objects, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 4);
    }
}
