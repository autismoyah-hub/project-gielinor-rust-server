//! Captured ordinary input pairs for authoritative assembly quantity production.
use crate::{
    assembly::{Job, PreciseRecipe},
    held_use,
    inventory::{Inventory, StackType},
    CoordGrid,
};
pub(crate) const QUEUE: i32 = -2391015;
#[derive(Clone, Debug)]
pub struct Request {
    pub recipe: PreciseRecipe,
    pub first_slot: usize,
    pub second_slot: usize,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub cycle_ticks: u8,
}
impl Request {
    pub fn bind(
        recipe: PreciseRecipe,
        cycle_ticks: u8,
        held: held_use::Request,
        inventory: &Inventory,
        origin: CoordGrid,
        last_movement: i32,
    ) -> Option<Self> {
        let r = recipe.recipe;
        if inventory.stack != StackType::Normal
            || inventory.placeholders
            || recipe.ticks == 0
            || cycle_ticks == 0
            || !(1..=99).contains(&r.level)
            || r.first < 0
            || r.second < 0
            || r.output < 0
            || r.first == r.second
            || r.output == r.first
            || r.output == r.second
        {
            return None;
        }
        let (first, second) = if held.selected_id == r.first && held.target_id == r.second {
            (held.selected_slot, held.target_slot)
        } else if held.target_id == r.first && held.selected_id == r.second {
            (held.target_slot, held.selected_slot)
        } else {
            return None;
        };
        let (first_slot, second_slot) =
            (usize::try_from(first).ok()?, usize::try_from(second).ok()?);
        let exact = |slot, id| {
            inventory
                .get(slot)
                .is_some_and(|v| v.id == id && v.count == 1 && v.vars == 0)
        };
        if first_slot == second_slot || !exact(first_slot, r.first) || !exact(second_slot, r.second)
        {
            return None;
        }
        Some(Self {
            recipe,
            first_slot,
            second_slot,
            origin,
            last_movement,
            cycle_ticks,
        })
    }
    pub fn available(&self, inventory: &Inventory) -> usize {
        let count = |id| {
            inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == id && i.count == 1 && i.vars == 0)
                .count()
        };
        count(self.recipe.recipe.first)
            .min(count(self.recipe.recipe.second))
            .min(28)
    }
    pub(crate) fn jobs(&self, inventory: &Inventory, count: usize) -> Option<Vec<Job>> {
        let r = self.recipe.recipe;
        let exact = |slot, id| {
            inventory
                .get(slot)
                .is_some_and(|v| v.id == id && v.count == 1 && v.vars == 0)
        };
        if !exact(self.first_slot, r.first) || !exact(self.second_slot, r.second) {
            return None;
        }
        let slots = |selected, id| {
            let mut values = vec![selected];
            values.extend(inventory.slots().iter().enumerate().filter_map(|(s, i)| {
                if s != selected && i.is_some_and(|v| v.id == id && v.count == 1 && v.vars == 0) {
                    Some(s)
                } else {
                    None
                }
            }));
            values
        };
        Some(
            slots(self.first_slot, r.first)
                .into_iter()
                .zip(slots(self.second_slot, r.second))
                .take(count.min(28))
                .map(|(first_slot, second_slot)| Job {
                    recipe: r,
                    first_slot,
                    second_slot,
                })
                .collect(),
        )
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub request: Request,
    pub jobs: std::sync::Arc<Vec<Job>>,
    pub index: usize,
    pub queued_at: i32,
}
