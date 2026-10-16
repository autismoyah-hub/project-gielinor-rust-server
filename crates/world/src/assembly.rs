//! Selected-slot food assembly with atomic inventory replacement and live levels.
//! Timing and held-item registration are owned by the existing world scheduler.
use crate::{
    inventory::{Catalog, Delete, Error, Insert, Inventory},
    stats::Stats,
};

pub(crate) const QUEUE: i32 = -2391002;

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub first: i32,
    pub second: i32,
    pub output: i32,
    pub skill: u8,
    pub level: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub first_slot: usize,
    pub second_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made,
    Level,
    Stale,
    Capacity,
}
impl Job {
    pub fn complete(&self, inv: &mut Inventory, catalog: &Catalog, stats: &Stats) -> Outcome {
        let r = self.recipe;
        if (stats.current(r.skill) as u8) < r.level {
            return Outcome::Level;
        }
        if self.first_slot == self.second_slot
            || inv
                .get(self.first_slot)
                .is_none_or(|i| i.id != r.first || i.count <= 0 || i.vars != 0)
            || inv
                .get(self.second_slot)
                .is_none_or(|i| i.id != r.second || i.count <= 0 || i.vars != 0)
        {
            return Outcome::Stale;
        }
        let mut next = inv.clone();
        let result = (|| -> Result<(), Error> {
            for (id, slot) in [(r.first, self.first_slot), (r.second, self.second_slot)] {
                let mut debit = Delete::new(id, 1);
                debit.strict_slot = Some(slot);
                next.delete(catalog, debit)?;
            }
            next.insert(catalog, Insert::new(r.output, 1))?;
            Ok(())
        })();
        if result.is_err() {
            return Outcome::Capacity;
        }
        *inv = next;
        Outcome::Made
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn fixture() -> (Job, Inventory, Catalog, Stats) {
        let recipe = Recipe {
            first: 10,
            second: 11,
            output: 12,
            skill: 7,
            level: 1,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for (slot, id) in [(0, 10), (1, 11)] {
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
        (
            Job {
                recipe,
                first_slot: 0,
                second_slot: 1,
            },
            inv,
            Catalog::default(),
            Stats::default(),
        )
    }
    #[test]
    fn exact_slots_and_live_level_gate_atomic_exchange() {
        let (mut job, mut inv, catalog, stats) = fixture();
        job.recipe.level = 1;
        let before = inv.clone();
        job.second_slot = 0;
        assert_eq!(job.complete(&mut inv, &catalog, &stats), Outcome::Stale);
        assert_eq!(inv, before);
        job.second_slot = 1;
        job.recipe.level = 255;
        assert_eq!(job.complete(&mut inv, &catalog, &stats), Outcome::Level);
        assert_eq!(inv, before);
        job.recipe.level = 1;
        assert_eq!(job.complete(&mut inv, &catalog, &stats), Outcome::Made);
        assert_eq!(
            inv.slots()
                .iter()
                .flatten()
                .map(|i| (i.id, i.count))
                .collect::<Vec<_>>(),
            vec![(12, 1)]
        );
    }
    #[test]
    fn output_failure_keeps_both_inputs_and_modified_slots() {
        let (mut job, mut inv, mut catalog, stats) = fixture();
        job.recipe.level = 1;
        catalog.dummy.insert(12);
        let before = inv.clone();
        assert_eq!(job.complete(&mut inv, &catalog, &stats), Outcome::Capacity);
        assert_eq!(inv, before);
    }
}
