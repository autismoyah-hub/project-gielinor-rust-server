//! Bottled spirits and the large beer keg reuse the selected food/stat transaction.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatEffects, StatFormula};
use std::collections::BTreeMap;
const PROFILES: [(&str, i32, &str, i32, u8, u8, u8, u8); 4] = [
    ("vodka", 2015, "Vodka", 5, 5, 1, 2, 3),
    ("whisky", 2017, "Whisky", 5, 5, 1, 2, 3),
    ("gin", 2019, "Gin", 5, 5, 1, 2, 3),
    ("keg_of_beer", 3801, "Keg of beer", 15, 10, 2, 50, 5),
];
pub fn spirits_keg(
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
            return Err(format!("Unattested spirit stat {symbol}"));
        }
    }
    for (symbol, id, name, _, _, _, _, _) in PROFILES {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || !d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref() != Some("Drink")
            })
        {
            return Err(format!("Unattested ordinary spirit {symbol}"));
        }
    }
    if PROFILES
        .iter()
        .any(|(_, id, _, _, _, _, _, _)| foods.contains_key(id))
    {
        return Err("Spirit/keg already registered".into());
    }
    let mut effects = BTreeMap::new();
    for (_, id, _, heal, boost_percent, boost_constant, drain_percent, drain_constant) in PROFILES {
        effects.insert(
            id,
            StatEffects {
                boosts: vec![StatFormula {
                    skill: 2,
                    percent: boost_percent,
                    constant: boost_constant,
                }],
                subtractions: vec![StatFormula {
                    skill: 0,
                    percent: drain_percent,
                    constant: drain_constant,
                }],
            },
        );
        foods.insert(
            id,
            Edible {
                id,
                heal,
                delay: 3,
                replacement: None,
                message: "You drink the alcoholic drink.".into(),
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
            panic!("spirits RNG")
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
        for (symbol, id, name, _, _, _, _, _) in PROFILES {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            d.inventory_ops[0] = Some("Drink".into());
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
    fn spirits_keg_four_profiles_use_current_drains_base_boosts_and_single_stage_selected_consumption(
    ) {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::new();
        let effects = spirits_keg(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!((foods.len(), effects.len()), (4, 4));
        for (_, id, _, heal, _, _, _, _) in PROFILES {
            for current in [50, 99, 115] {
                let mut s = Stats::default();
                for skill in [0, 2, 3] {
                    s.set_base(skill, 99);
                }
                s.set_current(0, 99);
                s.set_current(2, current);
                s.set_current(3, 50);
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
                    let (b, d) = effects[&id].resolve(&s, 3).unwrap();
                    assert!(eat_slot_with_secondary_effects(
                        &foods[&id],
                        slot,
                        &mut inv,
                        &mut s,
                        &Default::default(),
                        3,
                        829,
                        &b,
                        &d,
                        None,
                        &mut NoDraw,
                        |_| {}
                    )
                    .unwrap());
                    assert!(inv.get(slot).is_none());
                    if slot == 1 {
                        assert_eq!(inv.get(0).unwrap().id, id);
                        assert_eq!(s.current(0), if id == 3801 { 45 } else { 95 });
                    }
                }
                let delta = if id == 3801 { 11 } else { 5 };
                assert_eq!(
                    s.current(2),
                    if current == 50 {
                        50 + 2 * delta
                    } else if current == 99 {
                        99 + delta
                    } else {
                        current
                    }
                );
                assert_eq!(s.current(0), if id == 3801 { 18 } else { 91 });
                assert_eq!(s.current(3), 50 + heal as i8 * 2);
                for skill in 0..24 {
                    assert_eq!(s.fine_xp(skill), 0);
                }
            }
        }
        for current in [0, 1, 10] {
            let mut s = Stats::default();
            s.set_base(0, 99);
            s.set_current(0, current);
            let (b, d) = effects[&3801].resolve(&s, 3).unwrap();
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: 3801,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            assert!(eat_slot_with_secondary_effects(
                &foods[&3801],
                0,
                &mut inv,
                &mut s,
                &Default::default(),
                3,
                829,
                &b,
                &d,
                None,
                &mut NoDraw,
                |_| {}
            )
            .unwrap());
            assert_eq!(s.current(0), 0);
        }
    }
    #[test]
    fn spirits_keg_requires_distinct_ordinary_member_drink_ids_and_released_stats_before_any_registration(
    ) {
        for mode in 0..6 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => {
                    objects.insert("vodka".into(), 2019);
                }
                1 => defs.items.get_mut(&3801).unwrap().members = false,
                2 => defs.items.get_mut(&2017).unwrap().stackable = true,
                3 => defs.stats.get_mut(&0).unwrap().unreleased = true,
                4 => defs.items.get_mut(&2019).unwrap().inventory_ops[0] = Some("Eat".into()),
                _ => defs.items.get_mut(&3801).unwrap().transform.template = 65534,
            };
            let mut foods = BTreeMap::new();
            assert!(spirits_keg(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        spirits_keg(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(spirits_keg(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 4);
    }
}
