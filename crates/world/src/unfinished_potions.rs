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
    pub ticks: u8,
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
            || !(1..=2).contains(&r.ticks)
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
            assembly::Outcome::Stale | assembly::Outcome::Tools => Outcome::Stale,
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
                    ticks: 1,
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

    #[test]
    fn every_full_backpack_slot_pair_conserves_unrelated_state() {
        for herb_slot in 0..28 {
            for vessel_slot in 0..28 {
                if herb_slot == vessel_slot {
                    continue;
                }
                let (mut job, mut inv, catalog, mut stats) = fixture(false);
                for slot in 0..28 {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: 1931,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                inv.restore_slot(
                    herb_slot,
                    Some(Item {
                        id: 249,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                inv.restore_slot(
                    vessel_slot,
                    Some(Item {
                        id: 227,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                job.herb_slot = herb_slot;
                job.vessel_slot = vessel_slot;
                stats.set_fine_xp(15, Stats::MAX_FINE_XP).unwrap();
                let before = (inv.clone(), stats.clone());
                assert_eq!(
                    job.complete(&mut inv, &catalog, &mut stats, true, true),
                    Outcome::Made
                );
                assert_eq!(inv.free_space(), 1);
                assert_eq!(
                    inv.slots().iter().flatten().filter(|i| i.id == 91).count(),
                    1
                );
                assert!(
                    !inv.slots()
                        .iter()
                        .flatten()
                        .any(|i| i.id == 249 || i.id == 227)
                );
                for slot in 0..28 {
                    if slot != herb_slot && slot != vessel_slot {
                        assert_eq!(inv.get(slot), before.0.get(slot));
                    }
                }
                assert_eq!(stats, before.1);
            }
        }
    }
}

// Shared world scheduler content adapter; no independent tick engine.
pub(crate) const QUEUE: i32 = -2391007;
#[derive(Clone, Debug)]
pub struct Config {
    pub recipes: std::collections::BTreeMap<(i32, i32), Recipe>,
    pub quest_progress: u16,
    pub quest_complete: i32,
    pub animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub herb: i32,
    pub vessel: i32,
    pub herb_slot: usize,
    pub vessel_slot: usize,
    pub origin: crate::CoordGrid,
    pub last_movement: i32,
}
impl Config {
    pub fn request(
        &self,
        held: crate::held_use::Request,
        origin: crate::CoordGrid,
        last_movement: i32,
    ) -> Option<Request> {
        if held.selected_slot < 0 || held.target_slot < 0 || held.selected_slot == held.target_slot
        {
            return None;
        }
        let (herb, herb_slot, vessel, vessel_slot) = if self.recipes.contains_key(&(held.selected_id, held.target_id))
        {
            (
                held.selected_id,
                held.selected_slot,
                held.target_id,
                held.target_slot,
            )
        } else {
            (
                held.target_id,
                held.target_slot,
                held.selected_id,
                held.selected_slot,
            )
        };
        self.recipes.get(&(herb, vessel))?;
        Some(Request {
            herb,
            vessel,
            herb_slot: herb_slot as usize,
            vessel_slot: vessel_slot as usize,
            origin,
            last_movement,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub request: Request,
    pub remaining: i32,
    pub queued_at: i32,
    pub index: u32,
    pub quest_progress: u16,
    pub quest_complete: i32,
    pub animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub job: Job,
    pub queued_at: i32,
    pub committed_at: i32,
    pub index: u32,
}
impl Transition {
    pub fn json(self, actor: crate::simulation::ActorId) -> String {
        format!(
            "{{\"actor_slot\":{},\"actor_generation\":{},\"herb_slot\":{},\"vessel_slot\":{},\"herb\":{},\"vessel\":{},\"output\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{},\"index\":{}}}",
            actor.slot,
            actor.generation,
            self.job.herb_slot,
            self.job.vessel_slot,
            self.job.recipe.herb,
            self.job.recipe.vessel,
            self.job.recipe.output,
            self.queued_at,
            self.committed_at,
            self.index
        )
    }
}
