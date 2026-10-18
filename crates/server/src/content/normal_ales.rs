//! Ten normal ales: pinned Wiki effects on the existing food/stat transaction.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatEffects, StatFormula};
use std::collections::BTreeMap;
const PROFILES: [(&str, i32, &str, bool); 10] = [
    ("beer", 1917, "Beer", false),
    ("asgarnian_ale", 1905, "Asgarnian ale", false),
    ("wizards_mind_bomb", 1907, "Wizard's mind bomb", false),
    ("greenmans_ale", 1909, "Greenman's ale", true),
    ("dragon_bitter", 1911, "Dragon bitter", true),
    ("dwarven_stout", 1913, "Dwarven stout", false),
    ("cider", 5763, "Cider", true),
    ("axemans_folly", 5751, "Axeman's folly", true),
    ("chefs_delight", 5755, "Chef's delight", true),
    ("slayers_respite", 5759, "Slayer's respite", true),
];
const SKILLS: [(&str, i32, &str); 11] = [
    ("attack", 0, "Attack"),
    ("defence", 1, "Defence"),
    ("strength", 2, "Strength"),
    ("magic", 6, "Magic"),
    ("cooking", 7, "Cooking"),
    ("woodcutting", 8, "Woodcutting"),
    ("smithing", 13, "Smithing"),
    ("mining", 14, "Mining"),
    ("herblore", 15, "Herblore"),
    ("slayer", 18, "Slayer"),
    ("farming", 19, "Farming"),
];
pub fn normal_ales(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeMap<i32, StatEffects>, String> {
    for (symbol, id, name) in SKILLS {
        if stats.get(symbol) != Some(&id)
            || defs.stats.get(&(id as u32)).is_none_or(|d| {
                d.name != name || d.unreleased || d.min_level != 1 || d.max_level != 99
            })
        {
            return Err(format!("Unattested ale stat {symbol}"));
        }
    }
    for (symbol, id, name, member) in
        PROFILES
            .into_iter()
            .chain([("beer_glass", 1919, "Beer glass", false)])
    {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || d.members != member
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref()
                        != if id == 1919 { None } else { Some("Drink") }
            })
        {
            return Err(format!("Unattested ordinary ale {symbol}"));
        }
    }
    if PROFILES.iter().any(|(_, id, _, _)| foods.contains_key(id)) {
        return Err("Normal ale already registered".into());
    }
    let f = |skill, percent, constant| StatFormula {
        skill,
        percent,
        constant,
    };
    let mut effects = BTreeMap::new();
    for (_, id, _, _) in PROFILES {
        let (boosts, subtractions) = match id {
            1917 => (vec![f(2, 2, 1)], vec![f(0, 6, 1)]),
            1905 | 1911 => (vec![f(2, 0, 2)], vec![f(0, 5, 2)]),
            1907 => (vec![f(6, 2, 2)], vec![f(0, 5, 1), f(1, 5, 1), f(2, 5, 1)]),
            1909 => (vec![f(15, 0, 1)], vec![f(0, 0, 3), f(1, 0, 3), f(2, 0, 3)]),
            1913 => (
                vec![f(14, 0, 1), f(13, 0, 1)],
                vec![f(0, 4, 2), f(1, 4, 2), f(2, 4, 2)],
            ),
            5763 => (vec![f(19, 0, 1)], vec![f(0, 0, 2), f(2, 0, 2)]),
            5751 => (vec![f(8, 0, 1)], vec![f(0, 0, 3), f(2, 0, 3)]),
            5755 => (vec![f(7, 5, 1)], vec![f(0, 5, 2), f(2, 5, 2)]),
            5759 => (vec![f(18, 0, 2)], vec![f(0, 2, 2), f(2, 2, 2)]),
            _ => unreachable!(),
        };
        effects.insert(
            id,
            StatEffects {
                boosts,
                subtractions,
            },
        );
        foods.insert(
            id,
            Edible {
                id,
                heal: 1,
                delay: 3,
                replacement: Some(1919),
                message: "You drink the ale.".into(),
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
            panic!("ale RNG")
        }
    }
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let mut objects = Symbols::new();
        let mut stats = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, name, member) in
            PROFILES
                .into_iter()
                .chain([("beer_glass", 1919, "Beer glass", false)])
        {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            d.members = member;
            if id != 1919 {
                d.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id as u32, d);
        }
        for (symbol, id, name) in SKILLS {
            stats.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
            d.name = name.into();
            defs.stats.insert(id as u32, d);
        }
        (objects, stats, defs)
    }
    #[test]
    fn normal_ales_ten_wiki_profiles_resolve_live_base_and_use_shared_transaction_with_retained_glass(
    ) {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::new();
        let rules = normal_ales(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!((foods.len(), rules.len()), (10, 10));
        let expected: [(i32, &[(u8, i32)], &[(u8, i32)]); 10] = [
            (1917, &[(2, 2)], &[(0, 6)]),
            (1905, &[(2, 2)], &[(0, 6)]),
            (1911, &[(2, 2)], &[(0, 6)]),
            (1907, &[(6, 3)], &[(0, 5), (1, 5), (2, 5)]),
            (1909, &[(15, 1)], &[(0, 3), (1, 3), (2, 3)]),
            (1913, &[(14, 1), (13, 1)], &[(0, 5), (1, 5), (2, 5)]),
            (5763, &[(19, 1)], &[(0, 2), (2, 2)]),
            (5751, &[(8, 1)], &[(0, 3), (2, 3)]),
            (5755, &[(7, 5)], &[(0, 6), (2, 6)]),
            (5759, &[(18, 2)], &[(0, 3), (2, 3)]),
        ];
        for (id, want_boost, want_sub) in expected {
            let mut state = Stats::default();
            for skill in 0..24 {
                state.set_base(skill, 99);
                state.set_current(skill, 99);
            }
            let (b, s) = rules[&id].resolve(&state, 3).unwrap();
            assert_eq!(
                b.iter().map(|e| (e.skill, e.amount)).collect::<Vec<_>>(),
                want_boost
            );
            assert_eq!(
                s.iter().map(|e| (e.skill, e.amount)).collect::<Vec<_>>(),
                want_sub
            );
            state.set_current(3, 50);
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
                let (b, s) = rules[&id].resolve(&state, 3).unwrap();
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
                assert_eq!(inv.get(slot).unwrap().id, 1919);
            }
            assert_eq!(state.current(3), 52);
            for &(skill, amount) in want_boost {
                assert_eq!(state.current(skill), 99 + amount as i8);
            }
            for &(skill, amount) in want_sub {
                assert_eq!(state.current(skill), 99 - amount as i8 * 2);
            }
            for skill in 0..24 {
                assert_eq!(state.fine_xp(skill), 0);
            }
            if id == 5759 {
                assert_eq!(state.current(1), 99);
            }
        }
        for (level, boost, drain) in [
            (1, 2, 1),
            (19, 2, 1),
            (20, 2, 2),
            (49, 2, 3),
            (50, 3, 3),
            (99, 3, 5),
        ] {
            let mut state = Stats::default();
            for skill in [0, 1, 2, 6] {
                state.set_base(skill, level);
                state.set_current(skill, 100);
            }
            let (b, s) = rules[&1907].resolve(&state, 3).unwrap();
            assert_eq!(b[0].amount, boost);
            assert!(s.iter().all(|e| e.amount == drain));
        }
    }
    #[test]
    fn normal_ale_catalog_requires_all_stats_items_member_flags_glass_and_no_duplicate_before_any_insert(
    ) {
        for mode in 0..7 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => defs.items.get_mut(&5759).unwrap().inventory_ops[0] = Some("Eat".into()),
                1 => defs.items.get_mut(&1919).unwrap().stackable = true,
                2 => defs.items.get_mut(&1917).unwrap().members = true,
                3 => defs.items.get_mut(&1909).unwrap().members = false,
                4 => defs.stats.get_mut(&18).unwrap().unreleased = true,
                5 => {
                    objects.insert("beer".into(), 1905);
                }
                _ => defs.items.get_mut(&5763).unwrap().transform.template = 65534,
            };
            let mut foods = BTreeMap::new();
            assert!(normal_ales(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        normal_ales(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(normal_ales(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 10);
    }
}
