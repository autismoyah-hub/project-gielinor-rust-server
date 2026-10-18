//! Instant, explicitly counted exchanges between two selected ordinary stacks.
//! The existing inventory clone/debit/credit transaction supplies atomicity.
use crate::{
    assembly::Outcome,
    held_use,
    inventory::{Catalog, Delete, Insert, Inventory, StackType},
    stats::Stats,
    CoordGrid,
};
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub first: i32,
    pub second: i32,
    pub output: i32,
    pub skill: u8,
    pub level: u8,
    pub members: bool,
    pub message: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub recipe: Recipe,
    pub first_slot: usize,
    pub second_slot: usize,
    pub origin: CoordGrid,
    pub last_movement: i32,
}
impl Recipe {
    fn valid(self) -> bool {
        self.first >= 0
            && self.second >= 0
            && self.output >= 0
            && self.first != self.second
            && self.output != self.first
            && self.output != self.second
            && self.skill < 23
            && self.level <= 99
    }
}
impl Request {
    pub fn bind(
        recipe: Recipe,
        held: held_use::Request,
        inv: &Inventory,
        origin: CoordGrid,
        last_movement: i32,
    ) -> Option<Self> {
        if recipe.first < 0
            || recipe.second < 0
            || recipe.output < 0
            || recipe.first == recipe.second
            || recipe.output == recipe.first
            || recipe.output == recipe.second
            || recipe.skill >= 23
            || recipe.level > 99
        {
            return None;
        }
        let (a, b) = if held.selected_id == recipe.first && held.target_id == recipe.second {
            (held.selected_slot, held.target_slot)
        } else if held.target_id == recipe.first && held.selected_id == recipe.second {
            (held.target_slot, held.selected_slot)
        } else {
            return None;
        };
        let r = Self {
            recipe,
            first_slot: usize::try_from(a).ok()?,
            second_slot: usize::try_from(b).ok()?,
            origin,
            last_movement,
        };
        (r.available(inv) > 0).then_some(r)
    }
    pub fn available(self, inv: &Inventory) -> i32 {
        if inv.stack != StackType::Normal || inv.placeholders || self.first_slot == self.second_slot
        {
            return 0;
        }
        let Some(a) = inv.get(self.first_slot) else {
            return 0;
        };
        let Some(b) = inv.get(self.second_slot) else {
            return 0;
        };
        if a.id != self.recipe.first
            || b.id != self.recipe.second
            || a.vars != 0
            || b.vars != 0
            || a.count <= 0
            || b.count <= 0
        {
            return 0;
        }
        a.count.min(b.count)
    }
    pub fn complete(
        self,
        inv: &mut Inventory,
        cat: &Catalog,
        stats: &Stats,
        member: bool,
        count: i32,
    ) -> Result<i32, Outcome> {
        if !self.recipe.valid() {
            return Err(Outcome::Stale);
        }
        if self.recipe.members && !member {
            return Err(Outcome::Members);
        }
        if i32::from(stats.current(self.recipe.skill)) < i32::from(self.recipe.level) {
            return Err(Outcome::Level);
        }
        if count <= 0 {
            return Err(Outcome::Stale);
        }
        let amount = count.min(self.available(inv));
        if amount <= 0 {
            return Err(Outcome::Stale);
        }
        for id in [self.recipe.first, self.recipe.second, self.recipe.output] {
            if !cat.stackable.contains(&id)
                || cat.dummy.contains(&id)
                || cat.notes.get(&id).is_some_and(|v| v.template != 0)
                || cat.placeholders.get(&id).is_some_and(|v| v.template != 0)
                || cat.transforms.get(&id).is_some_and(|v| v.template != 0)
            {
                return Err(Outcome::Capacity);
            }
        }
        let mut next = inv.clone();
        for (id, slot) in [
            (self.recipe.first, self.first_slot),
            (self.recipe.second, self.second_slot),
        ] {
            let mut debit = Delete::new(id, amount);
            debit.strict_slot = Some(slot);
            next.delete(cat, debit).map_err(|_| Outcome::Capacity)?;
        }
        next.insert(cat, Insert::new(self.recipe.output, amount))
            .map_err(|_| Outcome::Capacity)?;
        *inv = next;
        Ok(amount)
    }
}
