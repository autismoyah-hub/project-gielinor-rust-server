//! Four-input super combat mixing. Wiki Super combat potion revision15281186.
//! Runtime registration must attest the four-dose items and Torstol alternatives.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    skills,
    stats::Stats,
};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    /// Super attack(4), super strength(4), super defence(4), then Torstol/unf.
    pub inputs: [i32; 4],
    pub output: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub slots: [usize; 4],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made(skills::Advancement),
    InvalidRecipe,
    Members,
    Quest,
    Level,
    Stale,
    Capacity,
    InvalidReward,
}
impl Job {
    /// All selected inputs are rechecked before preparing either possible result.
    /// Goggles save only Torstol/unf. Always-four-dose output never rolls chemistry.
    pub fn complete(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        member: bool,
        quest: bool,
        rate: i32,
        goggles: bool,
        random: &mut impl crate::npc::RandomSource,
    ) -> (Outcome, bool) {
        let r = self.recipe;
        let ordinary = |id| {
            id >= 0
                && !catalog.stackable.contains(&id)
                && !catalog.dummy.contains(&id)
                && !catalog.notes.get(&id).is_some_and(|v| v.template != 0)
                && !catalog
                    .placeholders
                    .get(&id)
                    .is_some_and(|v| v.template != 0)
                && !catalog.transforms.get(&id).is_some_and(|v| v.template != 0)
        };
        let ids = [r.inputs[0], r.inputs[1], r.inputs[2], r.inputs[3], r.output];
        if !ids.into_iter().all(ordinary) || (0..5).any(|i| ids[..i].contains(&ids[i])) {
            return (Outcome::InvalidRecipe, false);
        }
        if !member {
            return (Outcome::Members, false);
        }
        if !quest {
            return (Outcome::Quest, false);
        }
        if stats.current(15) < 90 {
            return (Outcome::Level, false);
        }
        if rate <= 0 {
            return (Outcome::InvalidReward, false);
        }
        if (0..4).any(|i| {
            self.slots[..i].contains(&self.slots[i])
                || inventory
                    .get(self.slots[i])
                    .is_none_or(|v| v.id != r.inputs[i] || v.count != 1 || v.vars != 0)
        }) {
            return (Outcome::Stale, false);
        }
        let prepare = |save| {
            let mut next = inventory.clone();
            for i in 0..if save { 3 } else { 4 } {
                let mut debit = Delete::new(r.inputs[i], 1);
                debit.strict_slot = Some(self.slots[i]);
                if next.delete(catalog, debit).ok()?.completed != 1 {
                    return None;
                }
            }
            if next
                .insert(catalog, Insert::new(r.output, 1))
                .ok()?
                .completed
                != 1
            {
                return None;
            }
            Some(next)
        };
        let Some(normal) = prepare(false) else {
            return (Outcome::Capacity, false);
        };
        let Some(saved) = (if goggles {
            prepare(true)
        } else {
            Some(normal.clone())
        }) else {
            return (Outcome::Capacity, false);
        };
        let mut next_stats = stats.clone();
        let Ok(award) = skills::advance(&mut next_stats, 15, 150.0, f64::from(rate) / 100.0, 99)
        else {
            return (Outcome::InvalidReward, false);
        };
        let save = goggles && random.below(10) == 0;
        *inventory = if save { saved } else { normal };
        *stats = next_stats;
        (Outcome::Made(award), save)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    struct Draw {
        calls: usize,
        value: u32,
    }
    impl crate::npc::RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 10);
            self.calls += 1;
            self.value
        }
    }
    fn fixture(unf: bool, slots: [usize; 4]) -> (Job, Inventory, Catalog, Stats) {
        let recipe = Recipe {
            inputs: [2436, 2440, 2442, if unf { 111 } else { 269 }],
            output: 12695,
        };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: slots
                        .iter()
                        .position(|s| *s == slot)
                        .map(|i| recipe.inputs[i])
                        .unwrap_or(1931),
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut stats = Stats::default();
        stats.set_base(15, 89);
        stats.set_current(15, 90);
        stats
            .set_fine_xp(15, skills::xp_from_level(89).unwrap() * 10 + 1)
            .unwrap();
        (Job { recipe, slots }, inv, Catalog::default(), stats)
    }
    #[test]
    fn full_bag_boosted_both_torstols_and_selected_slot_orders() {
        for unf in [false, true] {
            for slots in [[0, 1, 2, 3], [17, 8, 24, 0], [27, 26, 25, 24]] {
                for goggles in [false, true] {
                    for roll in [0, 1] {
                        let (job, mut inv, catalog, mut stats) = fixture(unf, slots);
                        let before = inv.clone();
                        let fine = stats.fine_xp(15);
                        let mut rng = Draw {
                            calls: 0,
                            value: roll,
                        };
                        let (outcome, saved) = job.complete(
                            &mut inv, &catalog, &mut stats, true, true, 500, goggles, &mut rng,
                        );
                        assert!(matches!(outcome, Outcome::Made(_)));
                        assert_eq!(saved, goggles && roll == 0);
                        assert_eq!(rng.calls, usize::from(goggles));
                        assert_eq!(stats.fine_xp(15), fine + 7500);
                        for slot in 0..28 {
                            if !slots.contains(&slot) {
                                assert_eq!(inv.get(slot), before.get(slot));
                            }
                        }
                        assert_eq!(
                            inv.slots()
                                .iter()
                                .flatten()
                                .filter(|i| i.id == 12695)
                                .count(),
                            1
                        );
                        for id in job.recipe.inputs[..3].iter() {
                            assert!(!inv.slots().iter().flatten().any(|v| v.id == *id));
                        }
                        assert_eq!(
                            inv.get(slots[3])
                                .is_some_and(|v| v.id == job.recipe.inputs[3]),
                            saved
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn rejects_changed_inputs_and_gates_without_rng_or_partial_mutation() {
        for mode in 0..13 {
            let (mut job, mut inv, mut catalog, mut stats) = fixture(false, [0, 1, 2, 3]);
            let (mut member, mut quest, mut rate) = (true, true, 500);
            match mode {
                0 => member = false,
                1 => quest = false,
                2 => stats.set_current(15, 89),
                3 => rate = 0,
                4..=7 => {
                    inv.restore_slot(
                        mode - 4,
                        Some(Item {
                            id: 145,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                8 => job.slots[3] = job.slots[0],
                9 => {
                    catalog.stackable.insert(job.recipe.inputs[2]);
                }
                10 => {
                    catalog.dummy.insert(job.recipe.output);
                }
                11 => job.recipe.output = job.recipe.inputs[0],
                _ => {
                    inv.restore_slot(
                        3,
                        Some(Item {
                            id: 269,
                            count: 1,
                            vars: 1,
                        }),
                    )
                    .unwrap();
                }
            }
            let before = inv.clone();
            let fine = stats.fine_xp(15);
            let mut rng = Draw { calls: 0, value: 0 };
            let (outcome, saved) = job.complete(
                &mut inv, &catalog, &mut stats, member, quest, rate, true, &mut rng,
            );
            assert!(!matches!(outcome, Outcome::Made(_)), "mode{mode}");
            assert!(!saved);
            assert_eq!(inv, before);
            assert_eq!(stats.fine_xp(15), fine);
            assert_eq!(rng.calls, 0);
        }
    }
}

pub(crate) const QUEUE:i32=-2391011;
#[derive(Clone,Debug)]
pub struct Config {pub recipes:Vec<Recipe>,pub goggles:[i32;2],pub quest_progress:u16,pub quest_complete:i32,pub animation:u16}
#[derive(Clone,Copy,Debug)]
pub struct Request {pub recipe_index:usize,pub slots:[usize;4],pub origin:crate::CoordGrid,pub last_movement:i32}
impl Config {
    pub fn request(&self,held:crate::held_use::Request,inventory:&Inventory,origin:crate::CoordGrid,last_movement:i32)->Option<Request> {
        if held.selected_slot<0||held.target_slot<0||held.selected_slot==held.target_slot{return None}
        for (recipe_index,r) in self.recipes.iter().enumerate() {
            let (herb,potion)=if held.selected_id==r.inputs[3] {(held.selected_slot,held.target_slot)}else if held.target_id==r.inputs[3] {(held.target_slot,held.selected_slot)}else{continue};
            let potion_id=if held.selected_id==r.inputs[3] {held.target_id}else{held.selected_id};
            let Some(index)=r.inputs[..3].iter().position(|id|*id==potion_id)else{continue};
            let mut slots=[usize::MAX;4];slots[3]=herb as usize;slots[index]=potion as usize;
            for i in 0..3 {if i!=index {slots[i]=inventory.slots().iter().position(|v|v.is_some_and(|v|v.id==r.inputs[i]&&v.count==1&&v.vars==0))?;}}
            if (0..4).any(|i|inventory.get(slots[i]).is_none_or(|v|v.id!=r.inputs[i]||v.count!=1||v.vars!=0)){return None}
            return Some(Request{recipe_index,slots,origin,last_movement});
        }
        None
    }
}
#[derive(Clone,Copy,Debug)]
pub(crate) struct Pending {pub goggles:[i32;2],pub job:Job,pub request:Request,pub remaining:i32,pub queued_at:i32,pub index:u32,pub quest_progress:u16,pub quest_complete:i32,pub animation:u16}
#[derive(Clone,Copy,Debug)]
pub struct Transition {pub job:Job,pub secondary_saved:bool,pub queued_at:i32,pub committed_at:i32,pub index:u32,pub fine_before:i32,pub fine_after:i32,pub rate_hundreds:i32}
impl Transition {
    pub fn json(&self,actor:crate::simulation::ActorId)->String {
        format!("{{\"actor_slot\":{},\"actor_generation\":{},\"slots\":{:?},\"inputs\":{:?},\"output\":{},\"secondary_saved\":{},\"queued_at\":{},\"committed_at\":{},\"index\":{},\"base_fine_xp\":1500,\"rate_hundreds\":{},\"fine_before\":{},\"fine_after\":{}}}",actor.slot,actor.generation,self.job.slots,self.job.recipe.inputs,self.job.recipe.output,self.secondary_saved,self.queued_at,self.committed_at,self.index,self.rate_hundreds,self.fine_before,self.fine_after)
    }
}
