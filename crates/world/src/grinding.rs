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
        self.complete_count(inv, catalog, member, 1)
    }
    /// One selected raw may create a checked stack count; legacy grinding
    /// continues to call this with one output. Both debit and full credit stage.
    pub fn complete_count(&self, inv:&mut Inventory, catalog:&Catalog, member:bool, output_count:u16)->Outcome {
        let r = self.recipe;
        if output_count == 0 || (output_count > 1 && !catalog.stackable.contains(&r.output)) {
            return Outcome::InvalidRecipe;
        }
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
                .is_none_or(|i| i.id != r.input || i.count <= 0 || (i.count != 1 && !catalog.stackable.contains(&r.input)) || i.vars != 0)
        {
            return Outcome::Stale;
        }
        let mut next = inv.clone();
        let mut debit = Delete::new(r.input, 1);
        debit.strict_slot = Some(self.input_slot);
        if next.delete(catalog, debit).is_err() {
            return Outcome::Capacity;
        }
        let mut credit = Insert::new(r.output, i32::from(output_count));
        if !catalog.stackable.contains(&r.output) { credit.strict_slot = Some(self.input_slot); }
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
    fn stack_crushing_conserves_selected_input_on_capacity_and_overflow() {
        for mode in 0..5 {
            let (job,_,mut catalog)=fixture();catalog.stackable.extend([2,3]);
            let mut inv=Inventory::new(4,StackType::Normal,false);
            inv.restore_slot(0,Some(Item{id:1,count:1,vars:0})).unwrap();
            inv.restore_slot(1,Some(Item{id:2,count:if mode==4{1}else{3},vars:0})).unwrap();
            inv.restore_slot(2,match mode {0=>None,1|4=>Some(Item{id:995,count:1,vars:0}),_=>Some(Item{id:3,count:if mode==3{i32::MAX}else{10},vars:0})}).unwrap();
            inv.restore_slot(3,Some(Item{id:995,count:1,vars:0})).unwrap();let before=inv.clone();
            let outcome=job.complete(&mut inv,&catalog,false);
            if mode==1 || mode==3 {assert_eq!(outcome,Outcome::Capacity);assert_eq!(inv,before);}
            else {assert_eq!(outcome,Outcome::Made);assert_eq!(inv.get(0),before.get(0));assert_eq!(inv.get(3),before.get(3));
                assert_eq!(inv.slots().iter().flatten().find(|i|i.id==2).map(|i|i.count).unwrap_or(0),if mode==4{0}else{2});
                assert_eq!(inv.slots().iter().flatten().find(|i|i.id==3).unwrap().count,if mode==2{11}else{1});}
        }
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
    pub queued_at: i32,
    pub index: u32,
}

/// Successful authoritative commits only; failed or cancelled jobs emit none.
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
            "{{\"actor_slot\":{},\"actor_generation\":{},\"tool_slot\":{},\"input_slot\":{},\"tool\":{},\"input\":{},\"output\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{},\"index\":{}}}",
            actor.slot,
            actor.generation,
            self.job.tool_slot,
            self.job.input_slot,
            self.job.recipe.tool,
            self.job.recipe.input,
            self.job.recipe.output,
            self.queued_at,
            self.committed_at,
            self.index
        )
    }
}
