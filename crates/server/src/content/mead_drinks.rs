//! Ordinary glass and multi-dose mead profiles reuse selected food replacement.
use super::{Definitions, Edible, Symbols};
use std::collections::{BTreeMap, BTreeSet};
pub fn mead_drinks(
    objects: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeSet<i32>, String> {
    let mut profiles = vec![
        (
            "moonlight_mead".to_string(),
            2955,
            "Moonlight mead".to_string(),
            4,
            Some(1919),
        ),
        (
            "mature_moonlight_mead".into(),
            5749,
            "Moonlight mead(m)".into(),
            6,
            Some(1919),
        ),
    ];
    for mature in [false, true] {
        for doses in 1..=4 {
            let id = if mature { 5891 } else { 5811 } + 2 * (doses - 1);
            profiles.push((
                format!(
                    "keg_{}moonlight_mead_{doses}",
                    if mature { "mature_" } else { "" }
                ),
                id,
                format!("Moonlight mead({}{doses})", if mature { "m" } else { "" }),
                if mature { 6 } else { 4 },
                Some(if doses == 1 { 5769 } else { id - 2 }),
            ));
        }
    }
    let mut definitions: Vec<_> = profiles
        .iter()
        .map(|(s, id, n, _, _)| (s.clone(), *id, n.clone(), true, Some("Drink")))
        .collect();
    definitions.extend([
        ("beer_glass".into(), 1919, "Beer glass".into(), false, None),
        (
            "calquat_fruit_keg_empty".into(),
            5769,
            "Calquat keg".into(),
            true,
            None,
        ),
    ]);
    for (symbol, id, name, member, op) in definitions {
        if objects.get(&symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || d.members != member
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref() != op
            })
        {
            return Err(format!("Unattested ordinary mead stage {symbol}"));
        }
    }
    if profiles
        .iter()
        .any(|(_, id, _, _, _)| foods.contains_key(id))
    {
        return Err("Mead drink already registered".into());
    }
    let mut selected = BTreeSet::new();
    for (_, id, _, heal, replacement) in profiles {
        selected.insert(id);
        foods.insert(
            id,
            Edible {
                id,
                heal,
                delay: 3,
                replacement,
                message: "You drink the mead.".into(),
            },
        );
    }
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
        let mut entries = vec![
            (
                "moonlight_mead".to_string(),
                2955,
                "Moonlight mead".to_string(),
                true,
                true,
            ),
            (
                "mature_moonlight_mead".into(),
                5749,
                "Moonlight mead(m)".into(),
                true,
                true,
            ),
            ("beer_glass".into(), 1919, "Beer glass".into(), false, false),
            (
                "calquat_fruit_keg_empty".into(),
                5769,
                "Calquat keg".into(),
                true,
                false,
            ),
        ];
        for mature in [false, true] {
            for n in 1..=4 {
                entries.push((
                    format!(
                        "keg_{}moonlight_mead_{n}",
                        if mature { "mature_" } else { "" }
                    ),
                    if mature { 5891 } else { 5811 } + 2 * (n - 1),
                    format!("Moonlight mead({}{n})", if mature { "m" } else { "" }),
                    true,
                    true,
                ));
            }
        }
        for (symbol, id, name, member, drink) in entries {
            objects.insert(symbol, id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name;
            d.members = member;
            if drink {
                d.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id as u32, d);
        }
        (objects, defs)
    }
    #[test]
    fn mead_drinks_ten_profiles_retain_glass_and_decrement_all_keg_doses_in_selected_full_slot() {
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        let selected = mead_drinks(&objects, &defs, &mut foods).unwrap();
        assert_eq!(selected.len(), 10);
        assert!(!foods.contains_key(&1919));
        assert!(!foods.contains_key(&5769));
        for (id, heal) in [(2955, 4), (5749, 6), (5817, 4), (5897, 6)] {
            for hp in [1, 98, 115] {
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
                stats.set_fine_xp(3, 12345).unwrap();
                let doses = if id < 5800 { 1 } else { 4 };
                for n in 0..doses {
                    let current = inv.get(1).unwrap().id;
                    let f = &foods[&current];
                    assert_eq!((f.heal, f.delay), (heal, 3));
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
                    let expected = if doses == 1 {
                        1919
                    } else if n == 3 {
                        5769
                    } else {
                        current - 2
                    };
                    assert_eq!(
                        inv.get(1),
                        Some(Item {
                            id: expected,
                            count: 1,
                            vars: 0
                        })
                    );
                    assert_eq!(
                        stats.current(3),
                        if hp > 99 {
                            hp
                        } else {
                            (i32::from(hp) + heal * (n + 1)).min(99) as i8
                        }
                    );
                    assert_eq!(stats.fine_xp(3), 12345);
                    assert_eq!(stats.current(0), 1);
                }
            }
        }
    }
    #[test]
    fn mead_drinks_binding_is_atomic_for_late_stage_container_membership_and_duplicate_failures() {
        for mode in 0..6 {
            let (mut objects, mut defs) = fixture();
            match mode {
                0 => defs.items.get_mut(&5897).unwrap().inventory_ops[0] = Some("Eat".into()),
                1 => defs.items.get_mut(&5769).unwrap().stackable = true,
                2 => defs.items.get_mut(&1919).unwrap().members = true,
                3 => defs.items.get_mut(&5813).unwrap().transform.template = 65534,
                4 => {
                    objects.insert("keg_mature_moonlight_mead_1".into(), 5893);
                }
                _ => defs.items.get_mut(&5749).unwrap().members = false,
            };
            let mut foods = BTreeMap::new();
            assert!(mead_drinks(&objects, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, defs) = fixture();
        let mut foods = BTreeMap::new();
        mead_drinks(&objects, &defs, &mut foods).unwrap();
        assert!(mead_drinks(&objects, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 10);
    }
}
