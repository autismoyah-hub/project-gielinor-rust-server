//! Distinct Draynor Manor cabbage consumption reuses base-level food boosts.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatEffects, StatFormula};
use std::collections::BTreeMap;
pub fn magic_cabbage(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeMap<i32, StatEffects>, String> {
    if stats.get("defence") != Some(&1)
        || defs.stats.get(&1).is_none_or(|d| {
            d.name != "Defence" || d.unreleased || d.min_level != 1 || d.max_level != 99
        })
    {
        return Err("Unattested cabbage Defence stat".into());
    }
    for (symbol, id) in [("magic_cabbage", 1967), ("cabbage", 1965)] {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != "Cabbage"
                    || d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref() != Some("Eat")
            })
        {
            return Err(format!("Unattested distinct cabbage {symbol}"));
        }
    }
    if foods.contains_key(&1967) {
        return Err("Draynor cabbage already registered".into());
    }
    foods.insert(
        1967,
        Edible {
            id: 1967,
            heal: 1,
            delay: 3,
            replacement: None,
            message: "You eat the cabbage.".into(),
        },
    );
    Ok(BTreeMap::from([(
        1967,
        StatEffects {
            boosts: vec![StatFormula {
                skill: 1,
                percent: 2,
                constant: 1,
            }],
            subtractions: Vec::new(),
        },
    )]))
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot_with_boosts,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let objects = Symbols::from([("magic_cabbage".into(), 1967), ("cabbage".into(), 1965)]);
        let stats = Symbols::from([("defence".into(), 1)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for id in [1965, 1967] {
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = "Cabbage".into();
            d.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id, d);
        }
        let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
        d.name = "Defence".into();
        defs.stats.insert(1, d);
        (objects, stats, defs)
    }
    #[test]
    fn magic_cabbage_distinct_f2p1967_heals1_boosts_defence_at50boundary_and_keeps_ordinary1965() {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::from([(
            1965,
            Edible {
                id: 1965,
                heal: 1,
                delay: 3,
                replacement: None,
                message: "Ordinary".into(),
            },
        )]);
        let effects = magic_cabbage(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!(effects.len(), 1);
        assert!(!effects.contains_key(&1965));
        assert_eq!(foods[&1965].message, "Ordinary");
        assert_eq!((foods[&1967].heal, foods[&1967].delay), (1, 3));
        for (base, delta) in [(1, 1), (49, 1), (50, 2), (99, 2)] {
            for current in [1, base, 115] {
                for hp in [50, 99, 115] {
                    let mut inv = Inventory::new(2, StackType::Normal, false);
                    for slot in 0..2 {
                        inv.restore_slot(
                            slot,
                            Some(Item {
                                id: 1967,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    let mut state = Stats::default();
                    state.set_base(1, base);
                    state.set_current(1, current);
                    state.set_base(3, 99);
                    state.set_current(3, hp);
                    let (b, s) = effects[&1967].resolve(&state, 3).unwrap();
                    assert!(s.is_empty());
                    assert_eq!(b[0].amount, delta);
                    assert!(eat_slot_with_boosts(
                        &foods[&1967],
                        1,
                        &mut inv,
                        &mut state,
                        &Default::default(),
                        3,
                        829,
                        &b,
                        |_| {}
                    )
                    .unwrap());
                    assert!(inv.get(1).is_none());
                    assert_eq!(inv.get(0).unwrap().id, 1967);
                    assert_eq!(
                        state.current(1),
                        if current > base + delta as i8 {
                            current
                        } else {
                            (current + delta as i8).min(base + delta as i8)
                        }
                    );
                    assert_eq!(state.current(3), if hp >= 99 { hp } else { hp + 1 });
                    assert_eq!(state.current(0), 1);
                    for skill in 0..24 {
                        assert_eq!(state.fine_xp(skill), 0);
                    }
                }
            }
        }
    }
    #[test]
    fn magic_cabbage_aliasing_same_name_sibling_or_invalid_f2p_eat_definition_is_refused_atomically(
    ) {
        for mode in 0..6 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => {
                    objects.insert("magic_cabbage".into(), 1965);
                }
                1 => defs.items.get_mut(&1967).unwrap().members = true,
                2 => defs.items.get_mut(&1965).unwrap().inventory_ops[0] = Some("Drink".into()),
                3 => defs.stats.get_mut(&1).unwrap().unreleased = true,
                4 => defs.items.get_mut(&1967).unwrap().stackable = true,
                _ => defs.items.get_mut(&1967).unwrap().transform.template = 65534,
            };
            let mut foods = BTreeMap::new();
            assert!(magic_cabbage(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        magic_cabbage(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(magic_cabbage(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 1);
    }
}
