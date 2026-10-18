//! Jangerberries share selected food HP, boosts, capped Prayer healing and drain staging.
use super::{Definitions, Edible, Symbols};
use gielinor_world::food::{StatBoost, StatHealing, StatSubtraction};
use std::collections::BTreeMap;
pub struct Jangerberries {
    pub item: i32,
    pub boosts: Vec<StatBoost>,
    pub subtractions: Vec<StatSubtraction>,
    pub healing: Vec<StatHealing>,
}
pub fn jangerberries(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<Jangerberries, String> {
    for (symbol, id, name) in [
        ("attack", 0, "Attack"),
        ("defence", 1, "Defence"),
        ("strength", 2, "Strength"),
        ("prayer", 5, "Prayer"),
    ] {
        if stats.get(symbol) != Some(&id)
            || defs.stats.get(&(id as u32)).is_none_or(|d| {
                d.name != name || d.unreleased || d.min_level != 1 || d.max_level != 99
            })
        {
            return Err(format!("Unattested Jangerberries stat {symbol}"));
        }
    }
    if objects.get("jangerberries") != Some(&247)
        || foods.contains_key(&247)
        || defs.items.get(&247).is_none_or(|d| {
            d.name != "Jangerberries"
                || !d.members
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
                || d.inventory_ops[0].as_deref() != Some("Eat")
        })
    {
        return Err("Unattested or duplicate ordinary Jangerberries".into());
    }
    foods.insert(
        247,
        Edible {
            id: 247,
            heal: 2,
            delay: 3,
            replacement: None,
            message: "You eat the jangerberries.".into(),
        },
    );
    Ok(Jangerberries {
        item: 247,
        boosts: vec![
            StatBoost {
                skill: 0,
                amount: 2,
            },
            StatBoost {
                skill: 2,
                amount: 1,
            },
        ],
        subtractions: vec![StatSubtraction {
            skill: 1,
            amount: 1,
        }],
        healing: vec![StatHealing {
            skill: 5,
            amount: 1,
        }],
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        food::eat_slot_with_stat_healing,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    struct NoDraw;
    impl gielinor_world::npc::RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Jangerberries RNG")
        }
    }
    fn fixture() -> (Symbols, Symbols, Definitions) {
        let objects = Symbols::from([("jangerberries".into(), 247)]);
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
        let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
        d.name = "Jangerberries".into();
        d.inventory_ops[0] = Some("Eat".into());
        defs.items.insert(247, d);
        for (symbol, id, name) in [
            ("attack", 0, "Attack"),
            ("defence", 1, "Defence"),
            ("strength", 2, "Strength"),
            ("prayer", 5, "Prayer"),
        ] {
            stats.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
            d.name = name.into();
            defs.stats.insert(id as u32, d);
        }
        (objects, stats, defs)
    }
    #[test]
    fn jangerberries_heal2_prayer1_and_stage_both_boosts_and_defence_loss_without_lowering_overboost(
    ) {
        let (objects, symbols, defs) = fixture();
        let mut foods = BTreeMap::new();
        let b = jangerberries(&objects, &symbols, &defs, &mut foods).unwrap();
        assert_eq!((b.item, foods[&247].heal, foods[&247].delay), (247, 2, 3));
        for hp in [50, 99, 115] {
            for prayer in [0, 98, 99, 105] {
                let mut inv = Inventory::new(2, StackType::Normal, false);
                for slot in 0..2 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: 247,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                let mut s = Stats::default();
                for skill in [0, 1, 2, 3, 5] {
                    s.set_base(skill, 99);
                    s.set_current(skill, 99);
                }
                s.set_current(1, 1);
                s.set_current(3, hp);
                s.set_current(5, prayer);
                let mut events = Vec::new();
                assert!(eat_slot_with_stat_healing(
                    &foods[&247],
                    1,
                    &mut inv,
                    &mut s,
                    &Default::default(),
                    3,
                    829,
                    &b.boosts,
                    &b.subtractions,
                    &b.healing,
                    None,
                    &mut NoDraw,
                    |e| events.push(e)
                )
                .unwrap());
                assert!(inv.get(1).is_none());
                assert_eq!(inv.get(0).unwrap().id, 247);
                assert_eq!((s.current(0), s.current(1), s.current(2)), (101, 0, 100));
                assert_eq!(s.current(3), if hp > 99 { hp } else { (hp + 2).min(99) });
                assert_eq!(s.current(5), if prayer < 99 { prayer + 1 } else { prayer });
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| matches!(e,gielinor_world::food::Event::Stat(u)if u.skill==5))
                        .count(),
                    usize::from(prayer < 99)
                );
                for skill in 0..24 {
                    assert_eq!(s.fine_xp(skill), 0);
                }
            }
        }
    }
    #[test]
    fn jangerberries_binding_requires_exact_member_eat_item_and_all_released_effect_stats_atomically(
    ) {
        for mode in 0..6 {
            let (mut objects, stats, mut defs) = fixture();
            match mode {
                0 => {
                    objects.insert("jangerberries".into(), 248);
                }
                1 => defs.items.get_mut(&247).unwrap().members = false,
                2 => defs.items.get_mut(&247).unwrap().stackable = true,
                3 => defs.stats.get_mut(&5).unwrap().unreleased = true,
                4 => defs.items.get_mut(&247).unwrap().inventory_ops[0] = Some("Drink".into()),
                _ => defs.items.get_mut(&247).unwrap().transform.template = 65534,
            };
            let mut foods = BTreeMap::new();
            assert!(jangerberries(&objects, &stats, &defs, &mut foods).is_err());
            assert!(foods.is_empty());
        }
        let (objects, stats, defs) = fixture();
        let mut foods = BTreeMap::new();
        jangerberries(&objects, &stats, &defs, &mut foods).unwrap();
        assert!(jangerberries(&objects, &stats, &defs, &mut foods).is_err());
        assert_eq!(foods.len(), 1);
    }
}
