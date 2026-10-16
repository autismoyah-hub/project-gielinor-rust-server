//! Atomic selected-slot herb cleaning; automatic continuation remains unregistered.
//! The host must supply current membership and authoritative quest eligibility.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    skills,
    stats::Stats,
};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub input: i32,
    pub output: i32,
    pub skill: u8,
    pub level: u8,
    pub base_fine_xp: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Cleaned,
    Members,
    Quest,
    Level,
    Stale,
    InvalidRecipe,
    InvalidReward,
    Capacity,
}
impl Job {
    pub fn complete(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        rate_hundreds: i32,
        member: bool,
        quest_complete: bool,
    ) -> (Outcome, Option<skills::Advancement>) {
        let r = self.recipe;
        if r.input < 0 || r.output < 0 || r.input == r.output || r.level == 0 || r.base_fine_xp == 0
        {
            return (Outcome::InvalidRecipe, None);
        }
        if !member {
            return (Outcome::Members, None);
        }
        if !quest_complete {
            return (Outcome::Quest, None);
        }
        if i32::from(stats.current(r.skill)) < i32::from(r.level) {
            return (Outcome::Level, None);
        }
        if inv
            .get(self.slot)
            .is_none_or(|i| i.id != r.input || i.count != 1 || i.vars != 0)
        {
            return (Outcome::Stale, None);
        }
        if rate_hundreds <= 0 {
            return (Outcome::InvalidReward, None);
        }
        let mut next = inv.clone();
        let mut debit = Delete::new(r.input, 1);
        debit.strict_slot = Some(self.slot);
        let mut credit = Insert::new(r.output, 1);
        credit.strict_slot = Some(self.slot);
        if next.delete(catalog, debit).is_err() || next.insert(catalog, credit).is_err() {
            return (Outcome::Capacity, None);
        }
        let mut skills = stats.clone();
        let advancement = match skills::advance(
            &mut skills,
            r.skill,
            f64::from(r.base_fine_xp) / 10.0,
            f64::from(rate_hundreds) / 100.0,
            99,
        ) {
            Ok(a) => a,
            Err(_) => return (Outcome::InvalidReward, None),
        };
        *inv = next;
        *stats = skills;
        (Outcome::Cleaned, Some(advancement))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn fixture() -> (Job, Inventory, Catalog, Stats) {
        let job = Job {
            recipe: Recipe {
                input: 199,
                output: 249,
                skill: 15,
                level: 3,
                base_fine_xp: 25,
            },
            slot: 0,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 199,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 995,
                count: 1000,
                vars: 0,
            }),
        )
        .unwrap();
        let mut stats = Stats::default();
        stats.set_current(15, 3);
        (job, inv, Catalog::default(), stats)
    }
    #[test]
    fn full_backpack_replaces_only_selected_herb_and_awards_fractional_xp_once() {
        let (job, mut inv, catalog, mut stats) = fixture();
        let coin = inv.get(1);
        let mut expected = stats.clone();
        expected.set_fine_xp(15, 125).unwrap();
        assert_eq!(
            job.complete(&mut inv, &catalog, &mut stats, 500, true, true)
                .0,
            Outcome::Cleaned
        );
        assert_eq!(
            inv.get(0),
            Some(Item {
                id: 249,
                count: 1,
                vars: 0
            })
        );
        assert_eq!(inv.get(1), coin);
        assert_eq!(stats, expected);
        let committed = inv.clone();
        assert_eq!(
            job.complete(&mut inv, &catalog, &mut stats, 500, true, true),
            (Outcome::Stale, None)
        );
        assert_eq!(inv, committed);
        assert_eq!(stats, expected);
    }
    #[test]
    fn live_eligibility_and_failed_credit_preserve_both_inventory_and_stats() {
        for mode in 0..6 {
            let (mut job, mut inv, mut catalog, mut stats) = fixture();
            if mode == 2 {
                stats.set_current(15, 2);
            }
            if mode == 3 {
                catalog.dummy.insert(job.recipe.output);
            }
            if mode == 5 {
                job.recipe.output = job.recipe.input;
            }
            let before = inv.clone();
            let skills = stats.clone();
            let result = job.complete(
                &mut inv,
                &catalog,
                &mut stats,
                if mode == 4 { 0 } else { 500 },
                mode != 0,
                mode != 1,
            );
            assert_eq!(
                result,
                (
                    [
                        Outcome::Members,
                        Outcome::Quest,
                        Outcome::Level,
                        Outcome::Capacity,
                        Outcome::InvalidReward,
                        Outcome::InvalidRecipe
                    ][mode],
                    None
                )
            );
            assert_eq!(inv, before);
            assert_eq!(stats, skills);
        }
    }
    #[test]
    fn stale_slot_never_substitutes_a_matching_herb_elsewhere() {
        for mode in 0..4 {
            let (job, mut inv, catalog, mut stats) = fixture();
            inv.restore_slot(
                1,
                Some(Item {
                    id: 199,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            inv.restore_slot(
                0,
                match mode {
                    0 => None,
                    1 => Some(Item {
                        id: 200,
                        count: 1,
                        vars: 0,
                    }),
                    2 => Some(Item {
                        id: 199,
                        count: 2,
                        vars: 0,
                    }),
                    _ => Some(Item {
                        id: 199,
                        count: 1,
                        vars: 1,
                    }),
                },
            )
            .unwrap();
            let before = inv.clone();
            let skills = stats.clone();
            assert_eq!(
                job.complete(&mut inv, &catalog, &mut stats, 500, true, true),
                (Outcome::Stale, None)
            );
            assert_eq!(inv, before);
            assert_eq!(stats, skills);
        }
    }
    #[test]
    fn marrentill_preserves_tenths_and_uses_boosted_level() {
        for (rate, expected_fine) in [(100, 39), (500, 191)] {
            let (mut job, mut inv, catalog, mut stats) = fixture();
            job.recipe = Recipe {
                input: 201,
                output: 251,
                skill: 15,
                level: 5,
                base_fine_xp: 38,
            };
            inv.restore_slot(
                0,
                Some(Item {
                    id: 201,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            stats.set_current(15, 5);
            stats.set_fine_xp(15, 1).unwrap();
            let mut expected = stats.clone();
            expected.set_fine_xp(15, expected_fine).unwrap();
            assert_eq!(
                job.complete(&mut inv, &catalog, &mut stats, rate, true, true)
                    .0,
                Outcome::Cleaned
            );
            assert_eq!(inv.get(0).unwrap().id, 251);
            assert_eq!(stats, expected);
        }
    }
}

/// Selected manual cleaning only. Automatic continuation policy is not registered.
#[derive(Clone, Debug)]
pub struct Config {
    pub recipes: std::collections::BTreeMap<i32, Recipe>,
    pub quest_progress: u16,
    pub quest_complete: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub job: Job,
    pub clock: i32,
    pub fine_xp_before: i32,
    pub fine_xp_after: i32,
}
impl Transition {
    pub fn json(self, actor: crate::simulation::ActorId) -> String {
        format!(
            "{{\"actor_slot\":{},\"actor_generation\":{},\"slot\":{},\"input\":{},\"output\":{},\"count\":1,\"clock\":{},\"fine_xp_before\":{},\"fine_xp_after\":{}}}",
            actor.slot,
            actor.generation,
            self.job.slot,
            self.job.recipe.input,
            self.job.recipe.output,
            self.clock,
            self.fine_xp_before,
            self.fine_xp_after
        )
    }
}
