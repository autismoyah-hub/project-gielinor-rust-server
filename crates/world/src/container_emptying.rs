//! Ordinary selected-container emptying reuses assembly's staged slot exchange.
use crate::{
    assembly,
    inventory::{Catalog, Inventory, StackType},
};
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub output: i32,
    pub members: bool,
    pub button_operation: u8,
}
impl Rule {
    pub fn prepare(
        self,
        input: i32,
        slot: usize,
        inv: &Inventory,
        cat: &Catalog,
        member: bool,
    ) -> Result<Inventory, assembly::Outcome> {
        if inv.stack != StackType::Normal
            || inv.placeholders
            || (self.members && !member)
            || input == self.output
            || ![input, self.output].into_iter().all(|id| {
                id >= 0
                    && !cat.stackable.contains(&id)
                    && !cat.dummy.contains(&id)
                    && !cat.notes.get(&id).is_some_and(|t| t.template != 0)
                    && !cat.placeholders.get(&id).is_some_and(|t| t.template != 0)
                    && !cat.transforms.get(&id).is_some_and(|t| t.template != 0)
            })
        {
            return Err(assembly::Outcome::Stale);
        }
        assembly::exchange_slots(inv, cat, &[(input, slot)], &[self.output], true)
    }
}
