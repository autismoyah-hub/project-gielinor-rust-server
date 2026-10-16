//! Tool-preserving ingredient grinding. The host must attest recipes and membership.
//! World scheduling and live action admission are intentionally owned by the caller.
use crate::inventory::{Catalog, Delete, Insert, Inventory};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub tool: i32,
    pub input: i32,
    pub output: i32,
    pub members: bool,
    pub animation: Option<u16>,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub tool_slot: usize,
    pub input_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made,
    Members,
    Stale,
    InvalidRecipe,
    Capacity,
}
impl Job {
    /// Revalidate both exact slots before an atomic one-for-one replacement.
    /// The tool is retained, and a full inventory does not itself forbid grinding.
    pub fn complete(&self, inv: &mut Inventory, catalog: &Catalog, member: bool) -> Outcome {
        let r = self.recipe;
        if r.tool == r.input || r.output == r.input || r.output == r.tool {
            return Outcome::InvalidRecipe;
        }
        if r.members && !member {
            return Outcome::Members;
        }
        if self.tool_slot == self.input_slot
            || inv
                .get(self.tool_slot)
                .is_none_or(|i| i.id != r.tool || i.count != 1 || i.vars != 0)
            || inv
                .get(self.input_slot)
                .is_none_or(|i| i.id != r.input || i.count != 1 || i.vars != 0)
        {
            return Outcome::Stale;
        }
        let mut next = inv.clone();
        let mut debit = Delete::new(r.input, 1);
        debit.strict_slot = Some(self.input_slot);
        if next.delete(catalog, debit).is_err() {
            return Outcome::Capacity;
        }
        let mut credit = Insert::new(r.output, 1);
        credit.strict_slot = Some(self.input_slot);
        if next.insert(catalog, credit).is_err() {
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
    fn fixture() -> (Job, Inventory, Catalog) {
        let job = Job {
            recipe: Recipe {
                tool: 1,
                input: 2,
                output: 3,
                members: false,
                animation: Some(364),
            },
            tool_slot: 0,
            input_slot: 1,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for slot in 0..2 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: slot as i32 + 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        (job, inv, Catalog::default())
    }
    #[test]
    fn full_inventory_retains_tool_and_selected_output() {
        let (job, mut inv, catalog) = fixture();
        let tool = inv.get(0);
        assert_eq!(job.complete(&mut inv, &catalog, false), Outcome::Made);
        assert_eq!(inv.get(0), tool);
        assert_eq!(
            inv.get(1),
            Some(Item {
                id: 3,
                count: 1,
                vars: 0
            })
        );
        let after = inv.clone();
        assert_eq!(job.complete(&mut inv, &catalog, false), Outcome::Stale);
        assert_eq!(inv, after);
    }
    #[test]
    fn members_and_stale_tool_require_live_authorization() {
        let (mut job, mut inv, catalog) = fixture();
        job.recipe.members = true;
        let before = inv.clone();
        assert_eq!(job.complete(&mut inv, &catalog, false), Outcome::Members);
        assert_eq!(inv, before);
        job.tool_slot = 1;
        assert_eq!(job.complete(&mut inv, &catalog, true), Outcome::Stale);
        assert_eq!(inv, before);
        job.tool_slot = 0;
        inv.restore_slot(0, None).unwrap();
        let before = inv.clone();
        assert_eq!(job.complete(&mut inv, &catalog, true), Outcome::Stale);
        assert_eq!(inv, before);
    }
    #[test]
    fn rejected_output_does_not_consume_ingredient_or_change_tool() {
        let (mut job, mut inv, mut catalog) = fixture();
        catalog.dummy.insert(3);
        let before = inv.clone();
        assert_eq!(job.complete(&mut inv, &catalog, false), Outcome::Capacity);
        assert_eq!(inv, before);
        job.recipe.output = 1;
        assert_eq!(
            job.complete(&mut inv, &catalog, false),
            Outcome::InvalidRecipe
        );
        assert_eq!(inv, before);
    }
}

pub(crate) const QUEUE: i32 = -2391005;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub origin: crate::CoordGrid,
    pub last_movement: i32,
}
