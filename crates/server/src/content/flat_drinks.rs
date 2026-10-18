//! Fixed-effect alcoholic drinks reuse the ordinary selected food/stat transaction.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatBoost, StatSubtraction};
use std::collections::{BTreeMap, BTreeSet};
pub struct FlatDrinks {
    pub selected: BTreeSet<i32>,
    pub boosts: BTreeMap<i32, Vec<StatBoost>>,
    pub subtractions: BTreeMap<i32, Vec<StatSubtraction>>,
}
pub fn flat_drinks(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<FlatDrinks, String> {
    for (symbol, id, name) in [
        ("attack", 0, "Attack"),
        ("ranged", 4, "Ranged"),
        ("magic", 6, "Magic"),
        ("smithing", 13, "Smithing"),
        ("herblore", 15, "Herblore"),
        ("farming", 19, "Farming"),
    ] {
        if stats.get(symbol) != Some(&id)
            || defs.stats.get(&(id as u32)).is_none_or(|d| {
                d.name != name || d.unreleased || d.min_level != 1 || d.max_level != 99
            })
        {
            return Err(format!("Unattested fixed-drink stat {symbol}"));
        }
    }
    let profiles = [
        (
            "kovacs_grog",
            27014,
            "Kovac's grog",
            1,
            "You drink Kovac's grog. It tastes like giants' feet.",
        ),
        (
            "ixcoztic_white",
            29952,
            "Ixcoztic white",
            16,
            "You drink the Ixcoztic white. The citrusy profile perfectly energises your palate.",
        ),
    ];
    for (symbol, id, name, _, _) in profiles {
        if objects.get(symbol) != Some(&id)
            || foods.contains_key(&id)
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
            return Err(format!("Unattested or duplicate fixed drink {symbol}"));
        }
    }
    let boosts = BTreeMap::from([
        (
            27014,
            vec![StatBoost {
                skill: 13,
                amount: 4,
            }],
        ),
        (
            29952,
            vec![StatBoost {
                skill: 19,
                amount: 1,
            }],
        ),
    ]);
    let subtractions = BTreeMap::from([
        (
            27014,
            vec![
                StatSubtraction {
                    skill: 0,
                    amount: 2,
                },
                StatSubtraction {
                    skill: 4,
                    amount: 2,
                },
                StatSubtraction {
                    skill: 6,
                    amount: 2,
                },
            ],
        ),
        (
            29952,
            vec![
                StatSubtraction {
                    skill: 0,
                    amount: 5,
                },
                StatSubtraction {
                    skill: 15,
                    amount: 1,
                },
            ],
        ),
    ]);
    for (_, id, _, heal, message) in profiles {
        foods.insert(
            id,
            Edible {
                id,
                heal,
                delay: 3,
                replacement: None,
                message: message.into(),
            },
        );
    }
    Ok(FlatDrinks {
        selected: BTreeSet::from([27014, 29952]),
        boosts,
        subtractions,
    })
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
            panic!("fixed drink draws RNG")
        }
    }
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let objects = Symbols::from([
            ("kovacs_grog".into(), 27014),
            ("ixcoztic_white".into(), 29952),
        ]);
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
        for (symbol, id, name) in [
            ("attack", 0, "Attack"),
            ("ranged", 4, "Ranged"),
            ("magic", 6, "Magic"),
            ("smithing", 13, "Smithing"),
            ("herblore", 15, "Herblore"),
            ("farming", 19, "Farming"),
        ] {
            stats.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
            d.name = name.into();
            defs.stats.insert(id as u32, d);
        }
        for (id, name) in [(27014, "Kovac's grog"), (29952, "Ixcoztic white")] {
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            d.inventory_ops[0] = Some("Drink".into());
            defs.items.insert(id, d);
        }
        (objects, stats, defs)
    }
    #[test]
    fn fixed_drinks_use_wiki_healing_flat_boosts_and_stacking_drains_on_one_transaction() {
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        let b = flat_drinks(&objects, &stats, &defs, &mut foods).unwrap();
        assert_eq!(b.selected, BTreeSet::from([27014, 29952]));
        for (id, heal, skill, boost) in [(27014, 1, 13, 4), (29952, 16, 19, 1)] {
            let f = &foods[&id];
            assert_eq!((f.heal, f.delay, f.replacement), (heal, 3, None));
            for current in [50, 99, 110] {
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
                let mut s = Stats::default();
                s.set_base(3, 99);
                s.set_current(3, 50);
                s.set_base(skill, 99);
                s.set_current(skill, current);
                for e in &b.subtractions[&id] {
                    s.set_base(e.skill, 99);
                    s.set_current(e.skill, 7);
                }
                let xp = s.fine_xp(skill);
                for slot in [1, 0] {
                    assert!(eat_slot_with_secondary_effects(
                        f,
                        slot,
                        &mut inv,
                        &mut s,
                        &Default::default(),
                        3,
                        829,
                        &b.boosts[&id],
                        &b.subtractions[&id],
                        None,
                        &mut NoDraw,
                        |_| {}
                    )
                    .unwrap());
                    assert!(inv.get(slot).is_none());
                }
                assert_eq!(s.current(3), 50 + heal as i8 * 2);
                assert_eq!(
                    s.current(skill),
                    if current == 50 {
                        current + boost * 2
                    } else if current == 99 {
                        99 + boost
                    } else {
                        current
                    }
                );
                assert_eq!(s.fine_xp(skill), xp);
                for e in &b.subtractions[&id] {
                    assert_eq!(s.current(e.skill), (7 - e.amount as i8 * 2).max(0));
                }
            }
        }
    }
    #[test]
    fn fixed_drink_catalog_rejects_partial_member_operation_stat_and_duplicate_bindings() {
        for mode in 0..6 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => defs.items.get_mut(&29952).unwrap().members = false,
                1 => defs.items.get_mut(&29952).unwrap().inventory_ops[0] = Some("Eat".into()),
                2 => defs.items.get_mut(&27014).unwrap().stackable = true,
                3 => defs.stats.get_mut(&13).unwrap().unreleased = true,
                4 => {
                    objects.insert("kovacs_grog".into(), 29952);
                }
                _ => defs.items.get_mut(&29952).unwrap().transform.template = 65534,
            };
            let mut foods = BTreeMap::new();
            assert!(flat_drinks(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        flat_drinks(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(flat_drinks(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 2);
    }
}
