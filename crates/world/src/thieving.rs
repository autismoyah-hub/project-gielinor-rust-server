//! Small pickpocket policy adapter; loot/modifier systems remain separate.
use crate::{
    inventory::{Catalog, Insert, Inventory, StackType},
    skills::{self, Advancement},
    stats::Stats,
};
use std::collections::BTreeMap;
pub const QUEUE: i32 = -2391013;
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub owner: &'static str,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub targets: BTreeMap<u16, Target>,
    pub skill: u8,
    pub pouch: i32,
    pub animation: u16,
    pub block: u16,
    pub spot: u16,
    pub sound: u16,
}
pub struct Prepared {
    pub inventory: Inventory,
    pub stats: Stats,
    pub advancement: Advancement,
    pub threshold: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPolicy,
    Level,
    Capacity,
    Cap,
    Metadata,
}
impl Config {
    /// Stage both outputs before any random draw. Nothing is published on failure.
    pub fn prepare(
        &self,
        inventory: &Inventory,
        stats: &Stats,
        catalog: &Catalog,
        rate: i32,
    ) -> Result<Prepared, Error> {
        if self.skill != 17
            || self.pouch != 22521
            || rate <= 0
            || inventory.stack != StackType::Normal
            || inventory.placeholders
            || !catalog.stackable.contains(&self.pouch)
        {
            return Err(Error::InvalidPolicy);
        }
        let level = stats.current(self.skill) as u8;
        if level < 1 {
            return Err(Error::Level);
        }
        let mut count = 0;
        let mut stacks = 0;
        for item in inventory
            .slots()
            .iter()
            .flatten()
            .filter(|v| v.id == self.pouch)
        {
            if item.vars != 0 || item.count <= 0 {
                return Err(Error::Metadata);
            }
            count = item.count;
            stacks += 1;
        }
        if stacks > 1 {
            return Err(Error::Metadata);
        }
        if count >= 28 {
            return Err(Error::Cap);
        }
        let mut inventory = inventory.clone();
        inventory
            .insert(catalog, Insert::new(self.pouch, 1))
            .map_err(|_| Error::Capacity)?;
        let mut stats = stats.clone();
        let advancement = skills::advance(&mut stats, self.skill, 8.0, rate as f64 / 100.0, 99)
            .map_err(|_| Error::InvalidPolicy)?;
        // Reuse the existing statRandom-compatible interpolation; higher visible boosts cap at99.
        let threshold = (skills::success_rate(180, 240, level.min(99) as i32, 99) * 256.0) as u32;
        Ok(Prepared {
            inventory,
            stats,
            advancement,
            threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    fn config() -> Config {
        Config {
            targets: BTreeMap::new(),
            skill: 17,
            pouch: 22521,
            animation: 881,
            block: 424,
            spot: 245,
            sound: 2581,
        }
    }
    #[test]
    fn stage_full_bag_merge_cap_and_fine_xp_without_mutating_input() {
        let c = config();
        let mut cat = Catalog::default();
        cat.stackable.insert(22521);
        let mut inv = Inventory::new(28, StackType::Normal, false);
        let mut stats = Stats::default();
        stats.set_base(17, 1);
        stats.set_current(17, 1);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 1511,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        assert!(matches!(
            c.prepare(&inv, &stats, &cat, 125),
            Err(Error::Capacity)
        ));
        inv.restore_slot(
            27,
            Some(Item {
                id: 22521,
                count: 27,
                vars: 0,
            }),
        )
        .unwrap();
        let prepared = c.prepare(&inv, &stats, &cat, 125).unwrap();
        assert_eq!(prepared.inventory.get(27).unwrap().count, 28);
        assert_eq!(prepared.stats.fine_xp(17), 100);
        assert_eq!(prepared.threshold, 181);
        assert_eq!(inv.get(27).unwrap().count, 27);
        assert_eq!(stats.fine_xp(17), 0);
        assert!(matches!(
            c.prepare(&prepared.inventory, &stats, &cat, 100),
            Err(Error::Cap)
        ));
        stats.set_current(17, 99);
        assert_eq!(c.prepare(&inv, &stats, &cat, 100).unwrap().threshold, 241);
    }
    #[test]
    fn invalid_rate_level_and_variable_stack_never_publish() {
        let c = config();
        let mut cat = Catalog::default();
        cat.stackable.insert(22521);
        let mut inv = Inventory::new(28, StackType::Normal, false);
        let mut stats = Stats::default();
        stats.set_current(17, 1);
        assert!(matches!(
            c.prepare(&inv, &stats, &cat, 0),
            Err(Error::InvalidPolicy)
        ));
        stats.set_current(17, 0);
        assert!(matches!(
            c.prepare(&inv, &stats, &cat, 100),
            Err(Error::Level)
        ));
        stats.set_current(17, 1);
        inv.restore_slot(
            3,
            Some(Item {
                id: 22521,
                count: 1,
                vars: 7,
            }),
        )
        .unwrap();
        assert!(matches!(
            c.prepare(&inv, &stats, &cat, 100),
            Err(Error::Metadata)
        ));
        assert_eq!(inv.get(3).unwrap().vars, 7);
        assert_eq!(stats.fine_xp(17), 0);
    }
}
