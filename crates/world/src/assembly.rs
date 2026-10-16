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
    pub base_xp: u16,
    pub members: bool,
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
    InvalidReward,
    Members,
}
impl Job {
    /// Commit inventory and skill advancement together. Failed credit, stale
    /// inputs or invalid reward policy cannot consume an ingredient or grant XP.
    pub fn complete_awarding(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        rate_hundreds: i32,
        member: bool,
    ) -> (Outcome, Option<crate::skills::Advancement>) {
        if self.recipe.members && !member {
            return (Outcome::Members, None);
        }
        let mut next_inv = inv.clone();
        let outcome = self.exchange(&mut next_inv, catalog, stats);
        if outcome != Outcome::Made {
            return (outcome, None);
        }
        let mut next_stats = stats.clone();
        let advancement = if self.recipe.base_xp == 0 {
            None
        } else {
            if rate_hundreds <= 0 {
                return (Outcome::InvalidReward, None);
            }
            match crate::skills::advance(
                &mut next_stats,
                self.recipe.skill,
                f64::from(self.recipe.base_xp),
                f64::from(rate_hundreds) / 100.0,
                99,
            ) {
                Ok(value) => Some(value),
                Err(_) => return (Outcome::InvalidReward, None),
            }
        };
        *inv = next_inv;
        *stats = next_stats;
        (Outcome::Made, advancement)
    }

    fn exchange(&self, inv: &mut Inventory, catalog: &Catalog, stats: &Stats) -> Outcome {
        let r = self.recipe;
        if i32::from(stats.current(r.skill)) < i32::from(r.level) {
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
            base_xp: 0,
            members: false,
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
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Stale);
        assert_eq!(inv, before);
        job.second_slot = 1;
        job.recipe.level = 255;
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Level);
        assert_eq!(inv, before);
        job.recipe.level = 1;
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Made);
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
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Capacity);
        assert_eq!(inv, before);
    }
    #[test]
    fn reward_and_inventory_commit_together_at_account_rate() {
        for (base_xp, rate, fine) in [
            (26, 100, 260),
            (26, 500, 1300),
            (39, 100, 390),
            (39, 500, 1950),
        ] {
            let (mut job, mut inv, catalog, mut stats) = fixture();
            job.recipe.base_xp = base_xp;
            let (outcome, advancement) =
                job.complete_awarding(&mut inv, &catalog, &mut stats, rate, false);
            assert_eq!(outcome, Outcome::Made);
            assert_eq!(stats.fine_xp(7), fine);
            assert_eq!(advancement.unwrap().added_xp, fine / 10);
            assert_eq!(inv.get(0).unwrap().id, 12);
            assert!(inv.get(1).is_none());
            let before = (inv.clone(), stats.clone());
            assert_eq!(
                job.complete_awarding(&mut inv, &catalog, &mut stats, rate, false),
                (Outcome::Stale, None)
            );
            assert_eq!((inv, stats), before);
        }
    }
    #[test]
    fn failed_exchange_or_reward_policy_preserves_inventory_and_stats() {
        for failure in 0..5 {
            let (mut job, mut inv, mut catalog, mut stats) = fixture();
            job.recipe.base_xp = 26;
            let mut rate = 500;
            let expected = match failure {
                0 => {
                    catalog.dummy.insert(12);
                    Outcome::Capacity
                }
                1 => {
                    stats.set_current(7, -1);
                    Outcome::Level
                }
                2 => {
                    job.second_slot = 0;
                    Outcome::Stale
                }
                3 => {
                    rate = 0;
                    Outcome::InvalidReward
                }
                _ => {
                    rate = -500;
                    Outcome::InvalidReward
                }
            };
            let before = (inv.clone(), stats.clone());
            assert_eq!(
                job.complete_awarding(&mut inv, &catalog, &mut stats, rate, false),
                (expected, None)
            );
            assert_eq!((inv, stats), before);
        }
        let (mut job, mut inv, catalog, mut stats) = fixture();
        job.recipe.base_xp = 39;
        stats.set_fine_xp(7, Stats::MAX_FINE_XP - 7).unwrap();
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false)
                .0,
            Outcome::Made
        );
        assert_eq!(stats.fine_xp(7), Stats::MAX_FINE_XP);
    }
    #[test]
    fn member_recipe_checks_entitlement_and_boosted_level_before_awarding() {
        let (mut job, mut inv, catalog, mut stats) = fixture();
        job.recipe.members = true;
        job.recipe.level = 65;
        job.recipe.base_xp = 45;
        stats.set_current(7, 65);
        let before = (inv.clone(), stats.clone());
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false),
            (Outcome::Members, None)
        );
        assert_eq!((inv.clone(), stats.clone()), before);
        stats.set_current(7, 64);
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
            (Outcome::Level, None)
        );
        assert_eq!(inv, before.0);
        assert_eq!(stats.fine_xp(7), 0);
        stats.set_current(7, 65);
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true)
                .0,
            Outcome::Made
        );
        assert_eq!(stats.fine_xp(7), 2250);
        assert_eq!(stats.current(7), 65);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub queued_at: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub job: Job,
    pub queued_at: i32,
    pub committed_at: i32,
}
impl Transition {
    pub fn json(self, actor: crate::simulation::ActorId) -> String {
        format!(
            "{{\"actor_slot\":{},\"actor_generation\":{},\"first_slot\":{},\"second_slot\":{},\"first\":{},\"second\":{},\"output\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{}}}",
            actor.slot,
            actor.generation,
            self.job.first_slot,
            self.job.second_slot,
            self.job.recipe.first,
            self.job.recipe.second,
            self.job.recipe.output,
            self.queued_at,
            self.committed_at
        )
    }
}
