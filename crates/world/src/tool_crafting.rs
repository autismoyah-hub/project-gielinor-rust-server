//! Manual tool crafting reuses the existing selected-slot tool-preserving exchange.
use crate::{
    grinding,
    inventory::{Catalog, Inventory, StackType},
    skills::{self, Advancement},
    stats::Stats,
    CoordGrid,
};
pub(crate) const QUEUE: i32 = -2391014;
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub exchange: grinding::Recipe,
    pub level: u8,
    pub xp_tenths: u16,
    pub ticks: u8,
    pub name: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub tool_slot: usize,
    pub input_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Policy,
    Level,
    Members,
    Exchange(grinding::Outcome),
}
pub struct Prepared {
    pub inventory: Inventory,
    pub stats: Stats,
    pub advancement: Advancement,
}
impl Job {
    pub fn prepare(
        &self,
        inv: &Inventory,
        catalog: &Catalog,
        stats: &Stats,
        rate: i32,
        member: bool,
    ) -> Result<Prepared, Error> {
        let r = self.recipe;
        if r.level == 0
            || r.level > 99
            || r.xp_tenths == 0
            || r.ticks == 0
            || rate <= 0
            || inv.stack != StackType::Normal
            || inv.placeholders
        {
            return Err(Error::Policy);
        }
        if r.exchange.members && !member {
            return Err(Error::Members);
        }
        if (stats.current(12) as u8) < r.level {
            return Err(Error::Level);
        }
        let mut inventory = inv.clone();
        let outcome = grinding::Job {
            recipe: r.exchange,
            tool_slot: self.tool_slot,
            input_slot: self.input_slot,
        }
        .complete(&mut inventory, catalog, member);
        if outcome != grinding::Outcome::Made {
            return Err(Error::Exchange(outcome));
        }
        let mut stats = stats.clone();
        let advancement = skills::advance(
            &mut stats,
            12,
            r.xp_tenths as f64 / 10.0,
            rate as f64 / 100.0,
            99,
        )
        .map_err(|_| Error::Policy)?;
        Ok(Prepared {
            inventory,
            stats,
            advancement,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub origin: CoordGrid,
    pub last_movement: i32,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    #[test]
    fn tool_and_full_bag_selected_exchange_are_atomic_with_fractional_xp() {
        let mut inv = Inventory::new(28, StackType::Normal, false);
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
        inv.restore_slot(
            1,
            Some(Item {
                id: 1755,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            20,
            Some(Item {
                id: 1621,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut stats = Stats::default();
        stats.set_base(12, 27);
        stats.set_current(12, 27);
        let job = Job {
            recipe: Recipe {
                exchange: grinding::Recipe {
                    tool: 1755,
                    input: 1621,
                    output: 1605,
                    members: false,
                    animation: Some(8833),
                },
                level: 27,
                xp_tenths: 675,
                ticks: 2,
                name: "emerald",
            },
            tool_slot: 1,
            input_slot: 20,
        };
        let mut cat = Catalog::default();
        let prepared = job.prepare(&inv, &cat, &stats, 125, false).unwrap();
        assert_eq!(prepared.inventory.get(1), inv.get(1));
        assert_eq!(prepared.inventory.get(20).unwrap().id, 1605);
        assert_eq!(prepared.stats.fine_xp(12), 843);
        assert_eq!(inv.get(20).unwrap().id, 1621);
        assert_eq!(stats.fine_xp(12), 0);
        cat.dummy.insert(1605);
        assert!(matches!(
            job.prepare(&inv, &cat, &stats, 125, false),
            Err(Error::Exchange(_))
        ));
        cat.dummy.clear();
        stats.set_current(12, 26);
        assert!(matches!(
            job.prepare(&inv, &cat, &stats, 125, false),
            Err(Error::Level)
        ));
        stats.set_current(12, 27);
        assert!(matches!(
            job.prepare(&inv, &cat, &stats, 0, false),
            Err(Error::Policy)
        ));
        inv.restore_slot(
            1,
            Some(Item {
                id: 1755,
                count: 1,
                vars: 1,
            }),
        )
        .unwrap();
        assert!(matches!(
            job.prepare(&inv, &cat, &stats, 125, false),
            Err(Error::Exchange(_))
        ));
    }
}
