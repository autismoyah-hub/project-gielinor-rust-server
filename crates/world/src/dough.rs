//! Ordinary dough inventory exchange, separate from tutorial progression.
//! The host owns choice, quantity, cadence and revision-attested bindings.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    stats::Stats,
};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub skill: u8,
    pub flour: i32,
    pub water: i32,
    pub pot: i32,
    pub vessel: i32,
    pub dough: i32,
    pub level: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub flour_slot: usize,
    pub water_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made,
    InvalidRecipe,
    Stale,
    Level,
    Capacity,
}

impl Job {
    /// Preserve both selected containers. Capacity rejection is deliberately
    /// lossless; no ground-drop behavior is inferred from reference invAdd calls.
    pub fn complete(self, inv: &mut Inventory, catalog: &Catalog, stats: &Stats) -> Outcome {
        let r = self.recipe;
        let ids = [r.flour, r.water, r.pot, r.vessel, r.dough];
        if r.level == 0
            || ids.iter().any(|id| *id < 0)
            || ids.iter().enumerate().any(|(i, id)| ids[..i].contains(id))
        {
            return Outcome::InvalidRecipe;
        }
        if i32::from(stats.current(r.skill)) < i32::from(r.level) {
            return Outcome::Level;
        }
        if self.flour_slot == self.water_slot
            || [(self.flour_slot, r.flour), (self.water_slot, r.water)]
                .iter()
                .any(|&(slot, id)| {
                    inv.get(slot)
                        .is_none_or(|item| item.id != id || item.count != 1 || item.vars != 0)
                })
        {
            return Outcome::Stale;
        }
        let mut next = inv.clone();
        for (slot, input, output) in [
            (self.flour_slot, r.flour, r.pot),
            (self.water_slot, r.water, r.vessel),
        ] {
            let mut debit = Delete::new(input, 1);
            debit.strict_slot = Some(slot);
            let mut credit = Insert::new(output, 1);
            credit.strict_slot = Some(slot);
            if next.delete(catalog, debit).is_err() || next.insert(catalog, credit).is_err() {
                return Outcome::Capacity;
            }
        }
        if next.insert(catalog, Insert::new(r.dough, 1)).is_err() {
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
    fn fixture(capacity: usize) -> (Job, Inventory, Catalog, Stats) {
        let mut inv = Inventory::new(capacity, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 10,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 11,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        (
            Job {
                recipe: Recipe {
                    skill: 7,
                    flour: 10,
                    water: 11,
                    pot: 12,
                    vessel: 13,
                    dough: 14,
                    level: 1,
                },
                flour_slot: 0,
                water_slot: 1,
            },
            inv,
            Catalog::default(),
            Stats::default(),
        )
    }
    #[test]
    fn both_containers_return_to_selected_slots_without_xp() {
        let (job, mut inv, catalog, stats) = fixture(3);
        let before = stats.clone();
        assert_eq!(job.complete(&mut inv, &catalog, &stats), Outcome::Made);
        assert_eq!(
            inv.slots()
                .iter()
                .map(|i| i.unwrap().id)
                .collect::<Vec<_>>(),
            [12, 13, 14]
        );
        assert_eq!(stats, before);
    }
    #[test]
    fn full_inventory_and_invalid_output_are_atomic() {
        let (job, mut inv, catalog, stats) = fixture(2);
        let before = inv.clone();
        assert_eq!(job.complete(&mut inv, &catalog, &stats), Outcome::Capacity);
        assert_eq!(inv, before);
        let (mut job, mut inv, catalog, stats) = fixture(3);
        job.recipe.dough = job.recipe.water;
        let before = inv.clone();
        assert_eq!(
            job.complete(&mut inv, &catalog, &stats),
            Outcome::InvalidRecipe
        );
        assert_eq!(inv, before);
    }
    #[test]
    fn stale_selected_inputs_and_current_level_reject_without_debit() {
        for kind in 0..5 {
            let (mut job, mut inv, catalog, stats) = fixture(3);
            match kind {
                0 => job.water_slot = 0,
                1 => inv.restore_slot(1, None).unwrap(),
                2 => inv
                    .restore_slot(
                        1,
                        Some(Item {
                            id: 11,
                            count: 2,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                3 => inv
                    .restore_slot(
                        1,
                        Some(Item {
                            id: 11,
                            count: 1,
                            vars: 1,
                        }),
                    )
                    .unwrap(),
                _ => job.recipe.level = 35,
            }
            let before = inv.clone();
            assert_eq!(
                job.complete(&mut inv, &catalog, &stats),
                if kind == 4 {
                    Outcome::Level
                } else {
                    Outcome::Stale
                }
            );
            assert_eq!(inv, before);
        }
    }
}

pub(crate) const QUEUE: i32 = -2391006;
#[derive(Clone, Copy, Debug)]
pub struct Output {
    pub id: i32,
    pub level: u8,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub skill: u8,
    pub tutorial: Option<crate::tutorial_progression::Config>,
    pub flour: i32,
    pub pot: i32,
    pub waters: Vec<(i32, i32)>,
    pub outputs: [Output; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub flour_slot: usize,
    pub water_slot: usize,
    pub water: i32,
    pub origin: crate::CoordGrid,
    pub last_movement: i32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub request: Request,
    pub remaining: i32,
    pub queued_at: i32,
    pub index: u32,
}
impl Config {
    pub fn recipe(&self, water: i32, index: usize) -> Option<Recipe> {
        let vessel = self.waters.iter().find(|(id, _)| *id == water)?.1;
        let output = self.outputs.get(index)?;
        Some(Recipe {
            skill: self.skill,
            flour: self.flour,
            water,
            pot: self.pot,
            vessel,
            dough: output.id,
            level: output.level,
        })
    }
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
        let (flour_slot, water_slot, water) = if held.selected_id == self.flour {
            (held.selected_slot, held.target_slot, held.target_id)
        } else if held.target_id == self.flour {
            (held.target_slot, held.selected_slot, held.selected_id)
        } else {
            return None;
        };
        self.recipe(water, 0)?;
        Some(Request {
            flour_slot: flour_slot as usize,
            water_slot: water_slot as usize,
            water,
            origin,
            last_movement,
        })
    }
}

/// Successful commits use the raw world processing clock, not client display time.
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
            "{{\"actor_slot\":{},\"actor_generation\":{},\"flour_slot\":{},\"water_slot\":{},\"flour\":{},\"water\":{},\"pot\":{},\"vessel\":{},\"dough\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{},\"index\":{}}}",
            actor.slot,
            actor.generation,
            self.job.flour_slot,
            self.job.water_slot,
            self.job.recipe.flour,
            self.job.recipe.water,
            self.job.recipe.pot,
            self.job.recipe.vessel,
            self.job.recipe.dough,
            self.queued_at,
            self.committed_at,
            self.index
        )
    }
}
