//! Quest-gated unfinished-potion exchange using the shared selected-slot transaction.
//! Wiki Guam potion (unf) revision15185604 and Marrentill potion (unf)
//! revision15345595: boostable requirements, members, zero XP. Timing and
//! Make-X routing are deliberately not registered by this primitive.
use crate::{
    assembly,
    inventory::{Catalog, Inventory},
    stats::Stats,
};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub herb: i32,
    pub vessel: i32,
    pub output: i32,
    pub level: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub herb_slot: usize,
    pub vessel_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made,
    InvalidRecipe,
    Members,
    Quest,
    Level,
    Stale,
    Capacity,
}
impl Job {
    /// Eligibility is supplied at commit time. No quest state or XP is awarded.
    pub fn complete(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        member: bool,
        quest_complete: bool,
    ) -> Outcome {
        let r = self.recipe;
        if r.herb < 0
            || r.vessel < 0
            || r.output < 0
            || r.level == 0
            || r.level > 99
            || r.herb == r.vessel
            || r.herb == r.output
            || r.vessel == r.output
        {
            return Outcome::InvalidRecipe;
        }
        if !member {
            return Outcome::Members;
        }
        if !quest_complete {
            return Outcome::Quest;
        }
        // Ordinary herbs and vials must occupy exactly one unmodified item slot.
        if [(self.herb_slot, r.herb), (self.vessel_slot, r.vessel)]
            .iter()
            .any(|&(slot, id)| {
                inv.get(slot)
                    .is_none_or(|i| i.id != id || i.count != 1 || i.vars != 0)
            })
        {
            return Outcome::Stale;
        }
        let job = assembly::Job {
            recipe: assembly::Recipe {
                first: r.herb,
                second: r.vessel,
                output: r.output,
                skill: 15,
                level: r.level,
                base_xp: 0,
                members: true,
            },
            first_slot: self.herb_slot,
            second_slot: self.vessel_slot,
        };
        match job.complete_awarding(inv, catalog, stats, 500, member).0 {
            assembly::Outcome::Made => Outcome::Made,
            assembly::Outcome::Level => Outcome::Level,
            assembly::Outcome::Stale => Outcome::Stale,
            assembly::Outcome::Capacity => Outcome::Capacity,
            assembly::Outcome::Members => Outcome::Members,
            assembly::Outcome::InvalidReward => unreachable!("unfinished potions grant no XP"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn fixture(reverse: bool) -> (Job, Inventory, Catalog, Stats) {
        let (herb_slot, vessel_slot) = if reverse { (1, 0) } else { (0, 1) };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            let id = if slot == herb_slot {
                249
            } else if slot == vessel_slot {
                227
            } else {
                1931
            };
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
        stats.set_base(15, 1);
        stats.set_current(15, 3);
        stats.set_fine_xp(15, 123).unwrap();
        (
            Job {
                recipe: Recipe {
                    herb: 249,
                    vessel: 227,
                    output: 91,
                    level: 3,
                },
                herb_slot,
                vessel_slot,
            },
            inv,
            Catalog::default(),
            stats,
        )
    }
    #[test]
    fn full_backpack_boosted_exchange_preserves_all_stats_and_cannot_replay() {
        for reverse in [false, true] {
            let (job, mut inv, catalog, mut stats) = fixture(reverse);
            let before_stats = stats.clone();
            assert_eq!(
                job.complete(&mut inv, &catalog, &mut stats, true, true),
                Outcome::Made
            );
            assert_eq!(
                inv.get(0),
                Some(Item {
                    id: 91,
                    count: 1,
                    vars: 0
                })
            );
            assert_eq!(inv.get(1), None);
            for slot in 2..28 {
                assert_eq!(
                    inv.get(slot),
                    Some(Item {
                        id: 1931,
                        count: 1,
                        vars: 0
                    })
                );
            }
            assert_eq!(stats, before_stats);
            let made = inv.clone();
            assert_eq!(
                job.complete(&mut inv, &catalog, &mut stats, true, true),
                Outcome::Stale
            );
            assert_eq!(inv, made);
            assert_eq!(stats, before_stats);
        }
    }
    #[test]
    fn live_refusals_and_failed_credit_conserve_every_item_and_stat() {
        for mode in 0..9 {
            let (mut job, mut inv, mut catalog, mut stats) = fixture(false);
            let expected = match mode {
                0 => Outcome::Members,
                1 => Outcome::Quest,
                2 => {
                    stats.set_current(15, 2);
                    Outcome::Level
                }
                3 => {
                    job.herb_slot = 2;
                    Outcome::Stale
                }
                4 => {
                    inv.restore_slot(
                        0,
                        Some(Item {
                            id: 249,
                            count: 1,
                            vars: 1,
                        }),
                    )
                    .unwrap();
                    Outcome::Stale
                }
                5 => {
                    catalog.dummy.insert(91);
                    Outcome::Capacity
                }
                6 => {
                    job.recipe.output = 249;
                    Outcome::InvalidRecipe
                }
                7 => {
                    job.vessel_slot = 0;
                    Outcome::Stale
                }
                _ => {
                    job.herb_slot = 28;
                    Outcome::Stale
                }
            };
            let before = (inv.clone(), stats.clone());
            assert_eq!(
                job.complete(&mut inv, &catalog, &mut stats, mode != 0, mode != 1),
                expected
            );
            assert_eq!((inv, stats), before);
        }
    }
}
