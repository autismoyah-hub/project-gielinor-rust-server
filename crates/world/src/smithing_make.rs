//! MakeLoop and SmithingActions.continueJob's inventory/XP core. UI selection
//! and weak-queue ownership belong to the host; this never imposes a player delay.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    skills,
    stats::Stats,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub queue: i32,
    pub skill: u8,
    pub max_level: u8,
    pub xp_rate: f64,
    pub tutorial: Option<(crate::tutorial_progression::Config, i32, i32)>,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub product: i32,
    pub produced: i32,
    /// Kotlin's linked map iteration order, with unique resolved object ids.
    pub consumed: Vec<(i32, i32)>,
    pub xp: f64,
    pub animation: u16,
    pub cycles: i32,
}
#[derive(Clone, Debug)]
pub struct Job {
    pub request: Arc<Request>,
    pub remaining: i32,
}
#[derive(Debug)]
pub enum Outcome {
    MissingInputs,
    NoSpace,
    Made {
        product: i32,
        count: i32,
        advancement: skills::Advancement,
        next: Option<Job>,
    },
}
pub(crate) fn total(inv: &Inventory, catalog: &Catalog, id: i32) -> i32 {
    if inv.stack == crate::inventory::StackType::Always || catalog.stackable.contains(&id) {
        return inv
            .slots()
            .iter()
            .flatten()
            .find(|item| item.id == id)
            .map_or(0, |item| item.count);
    }
    inv.slots()
        .iter()
        .flatten()
        .filter(|i| i.id == id)
        .fold(0i32, |sum, i| sum.wrapping_add(i.count))
}
impl Request {
    pub fn affordable_count(
        &self,
        inv: &Inventory,
        catalog: &Catalog,
        limit: i32,
    ) -> Result<i32, &'static str> {
        let mut affordable = None;
        for &(id, amount) in &self.consumed {
            if amount == 0 {
                return Err("Zero ingredient amount");
            }
            let count = total(inv, catalog, id).wrapping_div(amount);
            affordable = Some(affordable.map_or(count, |previous: i32| previous.min(count)));
        }
        Ok(affordable.unwrap_or(0).min(limit))
    }
    pub fn has_inputs(&self, inv: &Inventory, catalog: &Catalog) -> bool {
        self.consumed
            .iter()
            .all(|&(id, amount)| total(inv, catalog, id) >= amount)
    }
    /// A start emits animation and schedules this job, before any consumption.
    pub fn start(self: &Arc<Self>, count: i32) -> Option<Job> {
        (count > 0).then(|| Job {
            request: self.clone(),
            remaining: count,
        })
    }
}
impl Job {
    pub fn complete(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        skill: u8,
        xp_rate: f64,
        max_level: u8,
    ) -> Result<Outcome, crate::stats::InvalidXp> {
        let request = &self.request;
        if !request.has_inputs(inv, catalog) {
            return Ok(Outcome::MissingInputs);
        }
        // Match the original's independent strict transactions and reinsertion
        // on failure. Refunds can relocate slots; this is not snapshot rollback.
        for &(id, amount) in &request.consumed {
            let _ = inv.delete(catalog, Delete::new(id, amount));
        }
        if inv
            .insert(catalog, Insert::new(request.product, request.produced))
            .is_err()
        {
            for &(id, amount) in &request.consumed {
                let _ = inv.insert(catalog, Insert::new(id, amount));
            }
            return Ok(Outcome::NoSpace);
        }
        let advancement = skills::advance(stats, skill, request.xp, xp_rate, max_level)?;
        let remaining = self.remaining.wrapping_sub(1);
        let next = (remaining > 0 && request.has_inputs(inv, catalog)).then(|| Job {
            request: request.clone(),
            remaining,
        });
        Ok(Outcome::Made {
            product: request.product,
            count: request.produced,
            advancement,
            next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn request(product: i32, produced: i32, consumed: Vec<(i32, i32)>) -> Arc<Request> {
        Arc::new(Request {
            product,
            produced,
            consumed,
            xp: 6.2,
            animation: 899,
            cycles: 5,
        })
    }
    fn inventory(items: &[Option<i32>]) -> Inventory {
        let mut inv = Inventory::new(items.len(), StackType::Normal, false);
        for (slot, &id) in items.iter().enumerate() {
            inv.restore_slot(
                slot,
                id.map(|id| Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        inv
    }
    #[test]
    fn missing_second_ingredient_never_consumes_first() {
        let mut inv = inventory(&[Some(436), Some(436), None]);
        let snapshot = inv.clone();
        let r = request(2349, 1, vec![(436, 1), (438, 1)]);
        assert_eq!(r.affordable_count(&inv, &Catalog::default(), 28), Ok(0));
        assert!(!r.has_inputs(&inv, &Catalog::default()));
        assert!(matches!(
            r.start(28)
                .unwrap()
                .complete(
                    &mut inv,
                    &Catalog::default(),
                    &mut Stats::default(),
                    13,
                    1.0,
                    99
                )
                .unwrap(),
            Outcome::MissingInputs
        ));
        assert_eq!(inv, snapshot);
    }
    #[test]
    fn full_inventory_consumes_before_output_and_keeps_fractional_xp() {
        let mut inv = inventory(&[Some(436), Some(438), Some(436), Some(438)]);
        let mut stats = Stats::default();
        let r = request(2349, 1, vec![(436, 1), (438, 1)]);
        assert_eq!(r.affordable_count(&inv, &Catalog::default(), 28), Ok(2));
        let Outcome::Made {
            next: Some(next), ..
        } = r
            .start(28)
            .unwrap()
            .complete(&mut inv, &Catalog::default(), &mut stats, 13, 1.0, 99)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(stats.fine_xp(13), 62);
        assert!(matches!(
            next.complete(&mut inv, &Catalog::default(), &mut stats, 13, 1.0, 99)
                .unwrap(),
            Outcome::Made { next: None, .. }
        ));
        assert_eq!(stats.fine_xp(13), 124);
        assert_eq!(total(&inv, &Catalog::default(), 2349), 2);
        assert_eq!(total(&inv, &Catalog::default(), 436), 0);
    }
    #[test]
    fn failed_output_refunds_in_map_order_without_restoring_original_slots() {
        let mut inv = inventory(&[Some(438), Some(436), Some(99)]);
        let mut stats = Stats::default();
        let r = request(2349, 3, vec![(436, 1), (438, 1)]);
        assert!(matches!(
            r.start(1)
                .unwrap()
                .complete(&mut inv, &Catalog::default(), &mut stats, 13, 1.0, 99)
                .unwrap(),
            Outcome::NoSpace
        ));
        assert_eq!(inv.get(0).unwrap().id, 436);
        assert_eq!(inv.get(1).unwrap().id, 438);
        assert_eq!(inv.get(2).unwrap().id, 99);
        assert_eq!(stats.fine_xp(13), 0);
    }
    #[test]
    fn stackable_products_allow_many_outputs_from_one_bar() {
        let mut inv = inventory(&[Some(2349), Some(99)]);
        let mut catalog = Catalog::default();
        catalog.stackable.insert(39);
        let r = request(39, 15, vec![(2349, 1)]);
        assert!(r.start(0).is_none());
        assert_eq!(r.affordable_count(&inv, &Catalog::default(), -5), Ok(-5));
        assert!(matches!(
            r.start(1)
                .unwrap()
                .complete(&mut inv, &catalog, &mut Stats::default(), 13, 1.0, 99)
                .unwrap(),
            Outcome::Made { count: 15, .. }
        ));
        assert_eq!(total(&inv, &Catalog::default(), 39), 15);
    }
    #[test]
    fn stackable_and_always_stack_inventory_count_only_first_matching_stack() {
        let mut inv = inventory(&[Some(436), Some(436)]);
        let r = request(2349, 1, vec![(436, 2)]);
        let mut catalog = Catalog::default();
        assert_eq!(r.affordable_count(&inv, &catalog, 28), Ok(1));
        catalog.stackable.insert(436);
        assert_eq!(r.affordable_count(&inv, &catalog, 28), Ok(0));
        assert!(!r.has_inputs(&inv, &catalog));
        catalog.stackable.clear();
        inv.stack = StackType::Always;
        assert_eq!(r.affordable_count(&inv, &catalog, 28), Ok(0));
    }
}
