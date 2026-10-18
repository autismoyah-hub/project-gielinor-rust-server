//! Four alcoholic cocktails use explicit Wiki equations over shared food metadata.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatEffects, StatFormula};
use std::collections::BTreeMap;
const PROFILES: [(&str, i32, &str, u8); 4] = [
    ("sgg", 2080, "Short green guy", 5),
    ("premade_sgg", 2038, "Premade sgg", 5),
    ("wizard_blizzard", 2054, "Wizard blizzard", 6),
    ("premade_wizard_blizzard", 2040, "Premade wiz blz'd", 6),
];
pub fn sgg_blizzard(
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
            return Err(format!("Unattested cocktail stat {symbol}"));
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
            return Err(format!("Unattested ordinary SGG/Blizzard {symbol}"));
        }
    }
    if PROFILES.iter().any(|(_, id, _, _)| foods.contains_key(id)) {
        return Err("SGG/Blizzard already registered".into());
    }
    let mut effects = BTreeMap::new();
    for (_, id, _, percent) in PROFILES {
        effects.insert(
            id,
            StatEffects {
                boosts: vec![StatFormula {
                    skill: 2,
                    percent,
                    constant: 1,
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
                heal: 5,
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
            panic!("fixed cocktail RNG")
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
    fn sgg_blizzard_four_profiles_use_positive_wiki_drain_equation_and_selected_glass_with_capped_boost(
    ) {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::new();
        let effects = sgg_blizzard(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!((foods.len(), effects.len()), (4, 4));
        for (_, id, _, percent) in PROFILES {
            for current in [50, 99, 110] {
                let mut stats = Stats::default();
                for skill in [0, 2, 3] {
                    stats.set_base(skill, 99);
                    stats.set_current(skill, 99);
                }
                stats.set_current(2, current);
                stats.set_current(3, 50);
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
                    let (b, s) = effects[&id].resolve(&stats, 3).unwrap();
                    assert_eq!(b[0].amount, if percent == 5 { 5 } else { 6 });
                    assert_eq!(s[0].amount, 4);
                    assert!(eat_slot_with_secondary_effects(
                        &foods[&id],
                        slot,
                        &mut inv,
                        &mut stats,
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
                    assert_eq!(inv.get(slot).unwrap().id, 2026);
                }
                let amount = if percent == 5 { 5 } else { 6 };
                assert_eq!(
                    stats.current(2),
                    if current == 50 {
                        50 + 2 * amount
                    } else if current == 99 {
                        99 + amount
                    } else {
                        110
                    }
                );
                assert_eq!(stats.current(0), 91);
                assert_eq!(stats.current(3), 60);
                assert_eq!(stats.current(6), 1);
                for skill in 0..24 {
                    assert_eq!(stats.fine_xp(skill), 0);
                }
            }
            for (base, sgg, wizard, drain) in
                [(1, 1, 1, 3), (49, 3, 3, 3), (50, 3, 4, 4), (99, 5, 6, 4)]
            {
                let mut stats = Stats::default();
                for skill in [0, 2] {
                    stats.set_base(skill, base);
                    stats.set_current(skill, base);
                }
                let (b, s) = effects[&id].resolve(&stats, 3).unwrap();
                assert_eq!(b[0].amount, if percent == 5 { sgg } else { wizard });
                assert_eq!(s[0].amount, drain);
            }
        }
    }
    #[test]
    fn sgg_blizzard_rejects_aliasing_premade_member_stats_and_glass_before_registration() {
        for mode in 0..6 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => {
                    objects.insert("premade_sgg".into(), 2080);
                }
                1 => defs.items.get_mut(&2040).unwrap().members = false,
                2 => defs.items.get_mut(&2026).unwrap().stackable = true,
                3 => defs.stats.get_mut(&2).unwrap().unreleased = true,
                4 => defs.items.get_mut(&2054).unwrap().inventory_ops[0] = Some("Eat".into()),
                _ => defs.items.get_mut(&2026).unwrap().transform.template = 65534,
            };
            let mut foods = BTreeMap::new();
            assert!(sgg_blizzard(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        sgg_blizzard(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(sgg_blizzard(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 4);
    }
}
