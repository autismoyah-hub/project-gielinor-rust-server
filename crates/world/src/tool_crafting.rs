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
    pub skill: u8,
    pub exchange: grinding::Recipe,
    pub level: u8,
    pub xp_tenths: u16,
    pub output_count: u16,
    pub ticks: u8,
    pub name: &'static str,
    pub action: &'static str,
    pub failure: Option<Failure>,
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
    pub advancement: Option<Advancement>,
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
            || (r.xp_tenths > 0 && rate <= 0)
            || inv.stack != StackType::Normal
            || inv.placeholders
        {
            return Err(Error::Policy);
        }
        if r.exchange.members && !member {
            return Err(Error::Members);
        }
        if (stats.current(r.skill) as u8) < r.level {
            return Err(Error::Level);
        }
        let mut inventory = inv.clone();
        let outcome = grinding::Job {
            recipe: r.exchange,
            tool_slot: self.tool_slot,
            input_slot: self.input_slot,
        }
        .complete_count(&mut inventory, catalog, member, r.output_count);
        if outcome != grinding::Outcome::Made {
            return Err(Error::Exchange(outcome));
        }
        let mut stats = stats.clone();
        let advancement = if r.xp_tenths == 0 {
            None
        } else {
            Some(
                skills::advance(
                    &mut stats,
                    r.skill,
                    r.xp_tenths as f64 / 10.0,
                    rate as f64 / 100.0,
                    99,
                )
                .map_err(|_| Error::Policy)?,
            )
        };
        Ok(Prepared {
            inventory,
            stats,
            advancement,
        })
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub batch: Option<Batch>,
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
                skill: 12,
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
                action: "cut",
                output_count: 1,
                failure: None,
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

/// Immutable registry-owned choices and captured slots, never client recipe data.
#[derive(Clone, Debug)]
pub struct Request {
    pub recipes: std::sync::Arc<Vec<Recipe>>,
    pub tool_slot: usize,
    pub input_slot: usize,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub cycle_ticks: u8,
}
impl Request {
    pub fn bind(
        recipes: std::sync::Arc<Vec<Recipe>>,
        cycle_ticks: u8,
        request: crate::held_use::Request,
        inventory: &Inventory,
        origin: CoordGrid,
        last_movement: i32,
    ) -> Option<Self> {
        let first = *recipes.first()?;
        if inventory.stack != StackType::Normal
            || inventory.placeholders
            || cycle_ticks == 0
            || recipes.len() > 18
            || recipes.iter().any(|r| {
                r.exchange.tool != first.exchange.tool
                    || r.exchange.input != first.exchange.input
                    || r.ticks == 0
                    || r.level == 0
                    || r.level > 99
                    || r.output_count == 0
                    || r.xp_tenths == 0
            })
            || recipes
                .iter()
                .map(|r| r.exchange.output)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != recipes.len()
        {
            return None;
        }
        let (tool_slot, input_slot) = if request.selected_id == first.exchange.tool
            && request.target_id == first.exchange.input
        {
            (request.selected_slot, request.target_slot)
        } else if request.target_id == first.exchange.tool
            && request.selected_id == first.exchange.input
        {
            (request.target_slot, request.selected_slot)
        } else {
            return None;
        };
        let (tool_slot, input_slot) = (
            usize::try_from(tool_slot).ok()?,
            usize::try_from(input_slot).ok()?,
        );
        if tool_slot == input_slot
            || inventory
                .get(tool_slot)
                .is_none_or(|i| i.id != first.exchange.tool || i.count != 1 || i.vars != 0)
            || inventory
                .get(input_slot)
                .is_none_or(|i| i.id != first.exchange.input || i.count != 1 || i.vars != 0)
        {
            return None;
        }
        Some(Self {
            recipes,
            tool_slot,
            input_slot,
            origin,
            last_movement,
            cycle_ticks,
        })
    }
    pub fn available(&self, inventory: &Inventory) -> usize {
        let Some(recipe) = self.recipes.first() else {
            return 0;
        };
        let id = recipe.exchange.input;
        inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id && i.count == 1 && i.vars == 0)
            .count()
            .min(28)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Batch {
    pub slots: std::sync::Arc<Vec<usize>>,
    pub index: usize,
    pub cycle_ticks: u8,
}

/// Both possible resource/reward states are prepared before the only random draw.
#[derive(Clone, Copy, Debug)]
pub struct Failure {
    pub output: i32,
    pub xp_tenths: u16,
    pub low: i32,
    pub high: i32,
    pub message: &'static str,
}
impl Job {
    pub fn resolve(
        &self,
        inv: &Inventory,
        catalog: &Catalog,
        stats: &Stats,
        rate: i32,
        member: bool,
        random: &mut impl crate::npc::RandomSource,
    ) -> Result<(Prepared, bool), Error> {
        let success = self.prepare(inv, catalog, stats, rate, member)?;
        let Some(failure) = self.recipe.failure else {
            return Ok((success, true));
        };
        if failure.low < 0
            || failure.high > u16::MAX as i32
            || failure.low > failure.high
            || self.recipe.output_count != 1
        {
            return Err(Error::Policy);
        }
        let mut failed_job = *self;
        failed_job.recipe.exchange.output = failure.output;
        failed_job.recipe.xp_tenths = failure.xp_tenths;
        failed_job.recipe.failure = None;
        let failed = failed_job.prepare(inv, catalog, stats, rate, member)?;
        // Match source's clamped probability shape using the existing interpolation;
        // the host's authoritative uniform draw replaces source global Random.Default.
        let level = (stats.current(self.recipe.skill) as u8).clamp(1, 99) as i32;
        let threshold = (skills::success_rate(failure.low, failure.high, level, 99) * 256.0)
            .clamp(0.0, 256.0) as u32;
        if random.below(256) < threshold {
            Ok((success, true))
        } else {
            Ok((failed, false))
        }
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    use crate::inventory::Item;
    struct Draw {
        value: u32,
        calls: usize,
    }
    impl crate::npc::RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.calls += 1;
            self.value
        }
    }
    fn fixture() -> (Job, Inventory, Catalog, Stats) {
        let job = Job {
            recipe: Recipe {
                skill: 12,
                exchange: grinding::Recipe {
                    tool: 1755,
                    input: 1627,
                    output: 1611,
                    members: true,
                    animation: Some(891),
                },
                level: 13,
                xp_tenths: 200,
                ticks: 1,
                name: "Jade",
                action: "cut",
                output_count: 1,
                failure: Some(Failure {
                    output: 1633,
                    xp_tenths: 50,
                    low: 100,
                    high: 245,
                    message: "Crushed",
                }),
            },
            tool_slot: 0,
            input_slot: 1,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for (slot, id) in [(0, 1755), (1, 1627)] {
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
        let mut stats = Stats::default();
        stats.set_base(12, 13);
        stats.set_current(12, 13);
        (job, inv, Catalog::default(), stats)
    }
    #[test]
    fn zero_xp_failure_preserves_stats_without_fabricating_advancement() {
        let (mut job, inv, cat, stats) = fixture();
        job.recipe.failure.as_mut().unwrap().xp_tenths = 0;
        let mut rng = Draw {
            value: 255,
            calls: 0,
        };
        let (prepared, success) = job
            .resolve(&inv, &cat, &stats, 125, true, &mut rng)
            .unwrap();
        assert!(!success);
        assert_eq!(rng.calls, 1);
        assert_eq!(prepared.inventory.get(1).unwrap().id, 1633);
        assert_eq!(prepared.stats, stats);
        assert!(prepared.advancement.is_none());
    }
    #[test]
    fn roll_boundary_selects_exact_staged_reward_and_deterministic_profiles_draw_nothing() {
        for (value, success, id, fine) in [(118, true, 1611, 250), (119, false, 1633, 62)] {
            let (job, inv, cat, stats) = fixture();
            let mut r = Draw { value, calls: 0 };
            let (p, result) = job.resolve(&inv, &cat, &stats, 125, true, &mut r).unwrap();
            assert_eq!(result, success);
            assert_eq!(r.calls, 1);
            assert_eq!(p.inventory.get(1).unwrap().id, id);
            assert_eq!(p.inventory.get(0), inv.get(0));
            assert_eq!(p.stats.fine_xp(12), fine);
            assert_eq!(inv.get(1).unwrap().id, 1627);
            assert_eq!(stats.fine_xp(12), 0);
        }
        let (mut job, inv, cat, stats) = fixture();
        job.recipe.failure = None;
        let mut r = Draw {
            value: 255,
            calls: 0,
        };
        assert!(
            job.resolve(&inv, &cat, &stats, 125, true, &mut r)
                .unwrap()
                .1
        );
        assert_eq!(r.calls, 0);
    }
    #[test]
    fn either_invalid_outcome_or_live_policy_refuses_before_rng_and_never_mutates() {
        for case in 0..8 {
            let (mut job, mut inv, mut cat, mut stats) = fixture();
            let mut rate = 125;
            let mut member = true;
            match case {
                0 => {
                    cat.dummy.insert(1633);
                }
                1 => {
                    cat.dummy.insert(1611);
                }
                2 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 1627,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap();
                }
                3 => rate = 0,
                4 => member = false,
                5 => stats.set_current(12, 12),
                6 => job.recipe.failure.as_mut().unwrap().low = 246,
                _ => job.recipe.failure.as_mut().unwrap().high = 65536,
            };
            let before = (inv.clone(), stats.clone());
            let mut r = Draw { value: 0, calls: 0 };
            assert!(
                job.resolve(&inv, &cat, &stats, rate, member, &mut r)
                    .is_err(),
                "case{case}"
            );
            assert_eq!(r.calls, 0, "case{case}");
            assert_eq!((inv, stats), before, "case{case}");
        }
    }
}
