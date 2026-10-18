//! Standard tea consumption reuses base-level food boosts and retained containers.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatEffects, StatFormula};
use std::collections::BTreeMap;
pub fn standard_tea(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<BTreeMap<i32, StatEffects>, String> {
    if stats.get("attack") != Some(&0)
        || defs.stats.get(&0).is_none_or(|d| {
            d.name != "Attack" || d.unreleased || d.min_level != 1 || d.max_level != 99
        })
    {
        return Err("Unattested tea Attack stat".into());
    }
    for (symbol, id, name, op) in [
        ("cup_of_tea", 1978, "Cup of tea", Some("Drink")),
        ("cup_empty", 1980, "Empty cup", None),
    ] {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || !d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[0].as_deref() != op
                    || d.inventory_ops
                        .iter()
                        .any(|op| op.as_deref() == Some("Empty"))
            })
        {
            return Err(format!("Unattested standard tea/container {symbol}"));
        }
    }
    if foods.contains_key(&1978) {
        return Err("Standard tea already registered".into());
    }
    foods.insert(
        1978,
        Edible {
            id: 1978,
            heal: 3,
            delay: 3,
            replacement: Some(1980),
            message: "You drink the tea.".into(),
        },
    );
    Ok(BTreeMap::from([(
        1978,
        StatEffects {
            boosts: vec![StatFormula {
                skill: 0,
                percent: 2,
                constant: 2,
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
        let objects = Symbols::from([("cup_of_tea".into(), 1978), ("cup_empty".into(), 1980)]);
        let stats = Symbols::from([("attack".into(), 0)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (id, name, drink) in [(1978, "Cup of tea", true), (1980, "Empty cup", false)] {
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            if drink {
                d.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id, d);
        }
        let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
        d.name = "Attack".into();
        defs.stats.insert(0, d);
        (objects, stats, defs)
    }
    #[test]
    fn standard_tea_retains_selected_cup_heals3_and_caps_attack_with_level50_boundary() {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::new();
        let effects = standard_tea(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!(effects.len(), 1);
        assert!(!foods.contains_key(&1980));
        assert_eq!(
            (
                foods[&1978].heal,
                foods[&1978].delay,
                foods[&1978].replacement
            ),
            (3, 3, Some(1980))
        );
        for (base, delta) in [(1, 2), (49, 2), (50, 3), (99, 3)] {
            for current in [1, base, 115] {
                for hp in [50, 99, 115] {
                    let mut inv = Inventory::new(2, StackType::Normal, false);
                    for slot in 0..2 {
                        inv.restore_slot(
                            slot,
                            Some(Item {
                                id: 1978,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    let mut state = Stats::default();
                    state.set_base(0, base);
                    state.set_current(0, current);
                    state.set_base(3, 99);
                    state.set_current(3, hp);
                    let (b, s) = effects[&1978].resolve(&state, 3).unwrap();
                    assert!(s.is_empty());
                    assert_eq!(b[0].amount, delta);
                    assert!(eat_slot_with_boosts(
                        &foods[&1978],
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
                    assert_eq!(
                        inv.get(1),
                        Some(Item {
                            id: 1980,
                            count: 1,
                            vars: 0
                        })
                    );
                    assert_eq!(inv.get(0).unwrap().id, 1978);
                    assert_eq!(
                        state.current(0),
                        if current > base + delta as i8 {
                            current
                        } else {
                            (current + delta as i8).min(base + delta as i8)
                        }
                    );
                    assert_eq!(state.current(3), if hp >= 99 { hp } else { hp + 3 });
                    assert_eq!(state.current(1), 1);
                    assert_eq!(state.current(5), 1);
                    for skill in 0..24 {
                        assert_eq!(state.fine_xp(skill), 0);
                    }
                    let before = (inv.clone(), state.clone());
                    assert!(!eat_slot_with_boosts(
                        &foods[&1978],
                        1,
                        &mut inv,
                        &mut state,
                        &Default::default(),
                        3,
                        829,
                        &b,
                        |_| panic!("stale tea event")
                    )
                    .unwrap());
                    assert_eq!((inv, state), before);
                }
            }
        }
    }
    #[test]
    fn standard_tea_requires_member_drink_and_ordinary_member_empty_cup_without_empty_option() {
        for mode in 0..7 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => {
                    objects.insert("cup_of_tea".into(), 1980);
                }
                1 => defs.items.get_mut(&1978).unwrap().members = false,
                2 => defs.items.get_mut(&1980).unwrap().stackable = true,
                3 => defs.stats.get_mut(&0).unwrap().unreleased = true,
                4 => defs.items.get_mut(&1978).unwrap().inventory_ops[1] = Some("Empty".into()),
                5 => defs.items.get_mut(&1980).unwrap().transform.template = 65534,
                _ => defs.items.get_mut(&1980).unwrap().members = false,
            };
            let mut foods = BTreeMap::new();
            assert!(standard_tea(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        standard_tea(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(standard_tea(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 1);
    }
}
