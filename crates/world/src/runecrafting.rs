//! Ordinary altar batches stage existing inventory and XP transactions together.
use crate::{
    assembly,
    inventory::{Catalog, Delete, Insert, Inventory, StackType},
    stats::Stats,
    CoordGrid,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
pub(crate) const QUEUE: i32 = -2391050;
pub const RUNE_ESSENCE: i32 = 1436;
pub const PURE_ESSENCE: i32 = 7936;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub output: i32,
    pub level: u8,
    pub xp_tenths: u16,
    pub interval: u8,
    pub name: &'static str,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub altars: BTreeMap<u32, Recipe>,
    pub blocked_held: BTreeSet<i32>,
    pub blocked_worn: BTreeSet<i32>,
    pub animation: u16,
    pub spot: u16,
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub location: crate::locations::Location,
    pub revision: u64,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub recipe: Recipe,
    pub materials: Arc<Vec<(i32, usize)>>,
}
pub struct Prepared {
    pub inventory: Inventory,
    pub stats: Stats,
    pub advancement: Option<crate::skills::Advancement>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Policy,
    Level,
    Empty,
    Unsupported,
    Stale,
    Capacity,
    Reward,
}
impl Config {
    pub fn capture(&self, inventory: &Inventory) -> Result<Arc<Vec<(i32, usize)>>, Error> {
        let parts = inventory
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| {
                item.filter(|i| [RUNE_ESSENCE, PURE_ESSENCE].contains(&i.id))
                    .map(|i| (i.id, slot))
            })
            .collect::<Vec<_>>();
        if parts.is_empty() {
            return Err(Error::Empty);
        }
        if parts.len() > 28 {
            return Err(Error::Policy);
        }
        Ok(Arc::new(parts))
    }
    pub fn prepare(
        &self,
        recipe: Recipe,
        parts: &[(i32, usize)],
        inventory: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        rate: i32,
    ) -> Result<Prepared, Error> {
        if !self.altars.values().any(|&v| v == recipe)
            || recipe.level == 0
            || recipe.level > 99
            || recipe.interval == 0
            || recipe.xp_tenths == 0
            || inventory.stack != StackType::Normal
            || inventory.placeholders
            || parts.is_empty()
            || parts.len() > 28
            || parts.iter().map(|v| v.1).collect::<BTreeSet<_>>().len() != parts.len()
            || parts.iter().any(|&(id, _)| {
                ![RUNE_ESSENCE, PURE_ESSENCE].contains(&id) || !items.ordinary_nonstackable(id)
            })
            || !items.stackable.contains(&recipe.output)
            || items.dummy.contains(&recipe.output)
            || items
                .notes
                .get(&recipe.output)
                .is_some_and(|t| t.template != 0)
            || items
                .placeholders
                .get(&recipe.output)
                .is_some_and(|t| t.template != 0)
            || items
                .transforms
                .get(&recipe.output)
                .is_some_and(|t| t.template != 0)
        {
            return Err(Error::Policy);
        }
        if inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| self.blocked_held.contains(&i.id))
            || worn
                .slots()
                .iter()
                .flatten()
                .any(|i| self.blocked_worn.contains(&i.id))
        {
            return Err(Error::Unsupported);
        }
        let level = stats.current(20) as u8;
        if level < recipe.level {
            return Err(Error::Level);
        }
        if level > 104 {
            return Err(Error::Policy);
        }
        if parts.iter().any(|&(id, slot)| {
            !inventory
                .get(slot)
                .is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0)
        }) {
            return Err(Error::Stale);
        }
        // Yield uses the visible level before this action's XP; extra runes give no extra XP.
        let produced = (parts.len() as i32)
            .checked_mul(i32::from(level / recipe.interval) + 1)
            .ok_or(Error::Policy)?;
        let mut next = inventory.clone();
        for &(id, slot) in parts {
            let mut debit = Delete::new(id, 1);
            debit.strict_slot = Some(slot);
            next.delete(items, debit).map_err(|_| Error::Stale)?;
        }
        next.insert(items, Insert::new(recipe.output, produced))
            .map_err(|_| Error::Capacity)?;
        let xp = parts.len() as f64 * f64::from(recipe.xp_tenths) / 10.0;
        let (stats, advancement) =
            assembly::prepare_reward_value(stats, 20, xp, rate).map_err(|_| Error::Reward)?;
        Ok(Prepared {
            inventory: next,
            stats,
            advancement,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    fn fixture() -> (Config, Recipe, Inventory, Inventory, Catalog, Stats) {
        let recipe = Recipe {
            output: 556,
            level: 1,
            xp_tenths: 50,
            interval: 11,
            name: "Air rune",
        };
        let c = Config {
            altars: [(34760, recipe)].into(),
            blocked_held: [24704].into(),
            blocked_worn: [26850].into(),
            animation: 791,
            spot: 186,
        };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: if slot % 2 == 0 { 1436 } else { 7936 },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let worn = Inventory::new(15, StackType::Normal, false);
        let mut items = Catalog::default();
        items.stackable.insert(556);
        let mut stats = Stats::default();
        stats.set_base(20, 10);
        stats.set_current(20, 11);
        stats
            .set_xp(20, crate::skills::xp_from_level(10).unwrap())
            .unwrap();
        (c, recipe, inv, worn, items, stats)
    }
    #[test]
    fn ordinary_altar_extra_runes_do_not_multiply_xp_and_boosted104_earth_is_not_clamped99() {
        let (mut c, r, inv, worn, mut items, mut stats) = fixture();
        let parts = c.capture(&inv).unwrap();
        let p = c
            .prepare(r, &parts, &inv, &worn, &items, &stats, 125)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 56);
        assert_eq!(p.inventory.free_space(), 27);
        assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 1750);
        let earth = Recipe {
            output: 557,
            level: 9,
            xp_tenths: 65,
            interval: 26,
            name: "Earth rune",
        };
        c.altars.insert(34763, earth);
        items.stackable.insert(557);
        stats.set_base(20, 99);
        stats.set_current(20, 104);
        stats
            .set_xp(20, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        let p = c
            .prepare(earth, &parts, &inv, &worn, &items, &stats, 100)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 140);
        assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 1820);
        stats.set_base(20, 10);
        stats.set_current(20, 10);
        stats
            .set_xp(20, crate::skills::xp_from_level(11).unwrap() - 1)
            .unwrap();
        let p = c
            .prepare(r, &parts, &inv, &worn, &items, &stats, 100)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 28);
        assert!(p.stats.base(20) >= 11);
    }
    #[test]
    fn ordinary_altar_all_slot_credit_reward_and_unsupported_failures_leave_both_states_unchanged()
    {
        for case in 0..10 {
            let (c, r, mut inv, mut worn, mut items, mut stats) = fixture();
            let mut parts = c.capture(&inv).unwrap().as_ref().clone();
            match case {
                0 => parts[1].1 = 0,
                1 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 7936,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap();
                }
                2 => {
                    items.dummy.insert(556);
                }
                3 => {
                    stats.set_current(20, 0);
                }
                4 => {
                    inv.restore_slot(1, None).unwrap();
                }
                5 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 24704,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                6 => {
                    worn.restore_slot(
                        0,
                        Some(Item {
                            id: 26850,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                7 => {
                    items.stackable.insert(1436);
                }
                9 => {
                    stats.set_current(20, 105);
                }
                _ => {}
            }
            let before = (inv.clone(), worn.clone(), stats.clone());
            assert!(
                c.prepare(
                    r,
                    &parts,
                    &inv,
                    &worn,
                    &items,
                    &stats,
                    if case == 8 { 0 } else { 100 }
                )
                .is_err(),
                "case{case}"
            );
            assert_eq!((inv, worn, stats), before);
        }
    }
}
