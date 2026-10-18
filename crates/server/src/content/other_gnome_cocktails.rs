//! Remaining finished/premade gnome cocktails reuse current-drain food metadata.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatEffects, StatFormula};
use std::collections::BTreeMap;
const PROFILES: [(&str, i32, &str, i32); 6] = [
    ("blurberry_special", 2064, "Blurberry special", 7),
    ("premade_blurberry_special", 2028, "Premade blurb' sp.", 7),
    ("drunk_dragon", 2092, "Drunk dragon", 5),
    ("premade_drunk_dragon", 2032, "Premade dr' dragon", 5),
    ("chocolate_saturday", 2074, "Choc saturday", 5),
    ("premade_choc_saturday", 2030, "Premade choc s'dy", 5),
];
pub fn other_gnome_cocktails(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeMap<i32, StatEffects>, String> {
    for (symbol, id, name) in [("attack", 0, "Attack"), ("strength", 2, "Strength")] {
        if stats.get(symbol) != Some(&id)
            || defs.stats.get(&(id as u32)).is_none_or(|d| {
                d.name != name || d.unreleased || d.min_level != 1 || d.max_level != 99
            })
        {
            return Err(format!("Unattested gnome cocktail stat {symbol}"));
        }
    }
    for (symbol, id, name) in PROFILES
        .map(|(s, id, n, _)| (s, id, n))
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
            return Err(format!("Unattested ordinary gnome cocktail {symbol}"));
        }
    }
    if PROFILES.iter().any(|(_, id, _, _)| foods.contains_key(id)) {
        return Err("Gnome cocktail already registered".into());
    }
    let mut effects = BTreeMap::new();
    for (_, id, _, heal) in PROFILES {
        effects.insert(
            id,
            StatEffects {
                boosts: vec![StatFormula {
                    skill: 2,
                    percent: 5,
                    constant: 2,
                }],
                subtractions: vec![StatFormula {
                    skill: 0,
                    percent: 2,
                    constant: 3,
                }],
            },
        );
        foods.insert(
            id,
            Edible {
                id,
                heal,
                delay: 3,
                replacement: Some(2026),
                message: "You drink the cocktail.".into(),
            },
        );
    }
    Ok(effects)
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot_with_secondary_effects,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    struct NoDraw;
    impl gielinor_world::npc::RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("gnome cocktail RNG")
        }
    }
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let mut objects = Symbols::new();
        let stats = Symbols::from([("attack".into(), 0), ("strength".into(), 2)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name) in PROFILES
            .map(|(s, id, n, _)| (s, id, n))
            .into_iter()
            .chain([("cocktail_glass_empty", 2026, "Cocktail glass")])
        {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            if id != 2026 {
                d.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id as u32, d);
        }
        for (id, name) in [(0, "Attack"), (2, "Strength")] {
            let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
            d.name = name.into();
            defs.stats.insert(id, d);
        }
        (objects, stats, defs)
    }
    #[test]
    fn other_gnome_cocktails_six_profiles_capture_current_threshold_drains_and_base_boost_with_selected_glass(
    ) {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::new();
        let effects = other_gnome_cocktails(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!((foods.len(), effects.len()), (6, 6));
        for (_, id, _, heal) in PROFILES {
            for current in [50, 99, 110] {
                for hp in [1, 98, 115] {
                    let mut state = Stats::default();
                    for skill in [0, 2, 3] {
                        state.set_base(skill, 99);
                    }
                    state.set_current(0, 50);
                    state.set_current(2, current);
                    state.set_current(3, hp);
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
                    for slot in [1, 0] {
                        let (b, s) = effects[&id].resolve(&state, 3).unwrap();
                        assert_eq!(
                            (b[0].amount, s[0].amount),
                            (6, if slot == 1 { 4 } else { 3 })
                        );
                        assert!(eat_slot_with_secondary_effects(
                            &foods[&id],
                            slot,
                            &mut inv,
                            &mut state,
                            &Default::default(),
                            3,
                            829,
                            &b,
                            &s,
                            None,
                            &mut NoDraw,
                            |_| {}
                        )
                        .unwrap());
                        assert_eq!(
                            inv.get(slot),
                            Some(Item {
                                id: 2026,
                                count: 1,
                                vars: 0
                            })
                        );
                        if slot == 1 {
                            assert_eq!(inv.get(0).unwrap().id, id);
                        }
                    }
                    assert_eq!(state.current(0), 43);
                    assert_eq!(
                        state.current(2),
                        if current == 50 {
                            62
                        } else if current == 99 {
                            105
                        } else {
                            110
                        }
                    );
                    assert_eq!(
                        state.current(3),
                        if hp > 99 {
                            hp
                        } else {
                            (i32::from(hp) + heal * 2).min(99) as i8
                        }
                    );
                    assert_eq!(state.current(6), 1);
                    for skill in 0..24 {
                        assert_eq!(state.fine_xp(skill), 0);
                    }
                }
            }
            for (current, want) in [(1, 3), (49, 3), (50, 4), (99, 4), (100, 5)] {
                let mut s = Stats::default();
                s.set_base(0, 99);
                s.set_current(0, current);
                assert_eq!(effects[&id].resolve(&s, 3).unwrap().1[0].amount, want);
            }
        }
    }
    #[test]
    fn other_gnome_cocktails_atomic_target_binding_keeps_finished_premade_and_refuses_invalid_late_dependencies(
    ) {
        for mode in 0..7 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => {
                    objects.insert("premade_blurberry_special".into(), 2064);
                }
                1 => defs.items.get_mut(&2030).unwrap().members = false,
                2 => defs.items.get_mut(&2026).unwrap().stackable = true,
                3 => defs.stats.get_mut(&0).unwrap().unreleased = true,
                4 => defs.items.get_mut(&2092).unwrap().inventory_ops[0] = Some("Eat".into()),
                5 => defs.items.get_mut(&2026).unwrap().placeholder.template = 14401,
                _ => defs.items.get_mut(&2074).unwrap().dummy = 1,
            };
            let mut foods = BTreeMap::new();
            assert!(other_gnome_cocktails(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        other_gnome_cocktails(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(other_gnome_cocktails(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 6);
    }
}
