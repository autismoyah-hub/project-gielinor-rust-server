//! Selected-slot food assembly with atomic inventory replacement and live levels.
//! Timing and held-item registration are owned by the existing world scheduler.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    stats::Stats,
};

pub(crate) const QUEUE: i32 = -2391002;

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub first: i32,
    pub second: i32,
    pub output: i32,
    pub skill: u8,
    pub level: u8,
    pub base_xp: u16,
    pub members: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub first_slot: usize,
    pub second_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made,
    Level,
    Stale,
    Capacity,
    InvalidReward,
    Members,
    Tools,
}
impl Job {
    /// Commit inventory and skill advancement together. Failed credit, stale
    /// inputs or invalid reward policy cannot consume an ingredient or grant XP.
    pub fn complete_awarding(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        rate_hundreds: i32,
        member: bool,
    ) -> (Outcome, Option<crate::skills::Advancement>) {
        self.complete_awarding_value(
            inv,
            catalog,
            stats,
            rate_hundreds,
            member,
            f64::from(self.recipe.base_xp),
        )
    }
    /// Precise rewards use tenths of XP; the existing inventory/level transaction
    /// and legacy integer-XP entry point retain identical behavior.
    pub fn complete_awarding_tenths(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        rate_hundreds: i32,
        member: bool,
        xp_tenths: u16,
    ) -> (Outcome, Option<crate::skills::Advancement>) {
        self.complete_awarding_value(
            inv,
            catalog,
            stats,
            rate_hundreds,
            member,
            f64::from(xp_tenths) / 10.0,
        )
    }
    fn complete_awarding_value(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        rate_hundreds: i32,
        member: bool,
        xp: f64,
    ) -> (Outcome, Option<crate::skills::Advancement>) {
        if self.recipe.members && !member {
            return (Outcome::Members, None);
        }
        let mut next_inv = inv.clone();
        let outcome = self.exchange(&mut next_inv, catalog, stats);
        if outcome != Outcome::Made {
            return (outcome, None);
        }
        let (next_stats, advancement) =
            match prepare_reward_value(stats, self.recipe.skill, xp, rate_hundreds) {
                Ok(v) => v,
                Err(outcome) => return (outcome, None),
            };
        *inv = next_inv;
        *stats = next_stats;
        (Outcome::Made, advancement)
    }

    fn exchange(&self, inv: &mut Inventory, catalog: &Catalog, stats: &Stats) -> Outcome {
        let r = self.recipe;
        if i32::from(stats.current(r.skill)) < i32::from(r.level) {
            return Outcome::Level;
        }
        match exchange_slots(
            inv,
            catalog,
            &[(r.first, self.first_slot), (r.second, self.second_slot)],
            &[r.output],
            false,
        ) {
            Ok(next) => *inv = next,
            Err(outcome) => return outcome,
        }

        Outcome::Made
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn fixture() -> (Job, Inventory, Catalog, Stats) {
        let recipe = Recipe {
            first: 10,
            second: 11,
            output: 12,
            skill: 7,
            level: 1,
            base_xp: 0,
            members: false,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for (slot, id) in [(0, 10), (1, 11)] {
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
        (
            Job {
                recipe,
                first_slot: 0,
                second_slot: 1,
            },
            inv,
            Catalog::default(),
            Stats::default(),
        )
    }
    #[test]
    fn precise_reward_keeps_fractional_xp_atomic_on_bad_credit_or_live_policy() {
        for case in 0..5 {
            let (mut job, mut inv, mut cat, mut stats) = fixture();
            job.recipe.skill = 12;
            job.recipe.level = 58;
            job.recipe.members = true;
            stats.set_base(12, 58);
            stats.set_current(12, 58);
            let mut rate = 125;
            let mut member = true;
            match case {
                1 => {
                    cat.dummy.insert(12);
                }
                2 => rate = 0,
                3 => member = false,
                4 => stats.set_current(12, 57),
                _ => {}
            }
            let before = (inv.clone(), stats.clone());
            let (outcome, advanced) =
                job.complete_awarding_tenths(&mut inv, &cat, &mut stats, rate, member, 1125);
            if case == 0 {
                assert_eq!(outcome, Outcome::Made);
                assert_eq!(stats.fine_xp(12), 1406);
                assert!(advanced.is_some());
            } else {
                assert_ne!(outcome, Outcome::Made);
                assert!(advanced.is_none());
                assert_eq!((inv, stats), before);
            }
        }
    }
    #[test]
    fn exact_slots_and_live_level_gate_atomic_exchange() {
        let (mut job, mut inv, catalog, stats) = fixture();
        job.recipe.level = 1;
        let before = inv.clone();
        job.second_slot = 0;
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Stale);
        assert_eq!(inv, before);
        job.second_slot = 1;
        job.recipe.level = 255;
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Level);
        assert_eq!(inv, before);
        job.recipe.level = 1;
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Made);
        assert_eq!(
            inv.slots()
                .iter()
                .flatten()
                .map(|i| (i.id, i.count))
                .collect::<Vec<_>>(),
            vec![(12, 1)]
        );
    }
    #[test]
    fn output_failure_keeps_both_inputs_and_modified_slots() {
        let (mut job, mut inv, mut catalog, stats) = fixture();
        job.recipe.level = 1;
        catalog.dummy.insert(12);
        let before = inv.clone();
        assert_eq!(job.exchange(&mut inv, &catalog, &stats), Outcome::Capacity);
        assert_eq!(inv, before);
    }
    #[test]
    fn reward_and_inventory_commit_together_at_account_rate() {
        for (base_xp, rate, fine) in [
            (26, 100, 260),
            (26, 500, 1300),
            (39, 100, 390),
            (39, 500, 1950),
        ] {
            let (mut job, mut inv, catalog, mut stats) = fixture();
            job.recipe.base_xp = base_xp;
            let (outcome, advancement) =
                job.complete_awarding(&mut inv, &catalog, &mut stats, rate, false);
            assert_eq!(outcome, Outcome::Made);
            assert_eq!(stats.fine_xp(7), fine);
            assert_eq!(advancement.unwrap().added_xp, fine / 10);
            assert_eq!(inv.get(0).unwrap().id, 12);
            assert!(inv.get(1).is_none());
            let before = (inv.clone(), stats.clone());
            assert_eq!(
                job.complete_awarding(&mut inv, &catalog, &mut stats, rate, false),
                (Outcome::Stale, None)
            );
            assert_eq!((inv, stats), before);
        }
    }
    #[test]
    fn failed_exchange_or_reward_policy_preserves_inventory_and_stats() {
        for failure in 0..5 {
            let (mut job, mut inv, mut catalog, mut stats) = fixture();
            job.recipe.base_xp = 26;
            let mut rate = 500;
            let expected = match failure {
                0 => {
                    catalog.dummy.insert(12);
                    Outcome::Capacity
                }
                1 => {
                    stats.set_current(7, -1);
                    Outcome::Level
                }
                2 => {
                    job.second_slot = 0;
                    Outcome::Stale
                }
                3 => {
                    rate = 0;
                    Outcome::InvalidReward
                }
                _ => {
                    rate = -500;
                    Outcome::InvalidReward
                }
            };
            let before = (inv.clone(), stats.clone());
            assert_eq!(
                job.complete_awarding(&mut inv, &catalog, &mut stats, rate, false),
                (expected, None)
            );
            assert_eq!((inv, stats), before);
        }
        let (mut job, mut inv, catalog, mut stats) = fixture();
        job.recipe.base_xp = 39;
        stats.set_fine_xp(7, Stats::MAX_FINE_XP - 7).unwrap();
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false)
                .0,
            Outcome::Made
        );
        assert_eq!(stats.fine_xp(7), Stats::MAX_FINE_XP);
    }
    #[test]
    fn member_recipe_checks_entitlement_and_boosted_level_before_awarding() {
        let (mut job, mut inv, catalog, mut stats) = fixture();
        job.recipe.members = true;
        job.recipe.level = 65;
        job.recipe.base_xp = 45;
        stats.set_current(7, 65);
        let before = (inv.clone(), stats.clone());
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false),
            (Outcome::Members, None)
        );
        assert_eq!((inv.clone(), stats.clone()), before);
        stats.set_current(7, 64);
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
            (Outcome::Level, None)
        );
        assert_eq!(inv, before.0);
        assert_eq!(stats.fine_xp(7), 0);
        stats.set_current(7, 65);
        assert_eq!(
            job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true)
                .0,
            Outcome::Made
        );
        assert_eq!(stats.fine_xp(7), 2250);
        assert_eq!(stats.current(7), 65);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub job: Job,
    pub queued_at: i32,
    pub xp_tenths: Option<u16>,
    pub required_tools: &'static [ToolRequirement],
}
/// Registry-owned manual assembly metadata. `xp_tenths` is authoritative;
/// `recipe.base_xp` belongs only to the unchanged legacy integer-XP API.
#[derive(Clone, Copy, Debug)]
pub struct PreciseRecipe {
    pub recipe: Recipe,
    pub xp_tenths: u16,
    pub ticks: u8,
    pub animation: Option<u16>,
    pub spot: Option<u16>,
    pub required_tools: &'static [ToolRequirement],
    pub tool_animation: Option<(ToolRequirement, u16)>,
}
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub job: Job,
    pub queued_at: i32,
    pub committed_at: i32,
}
impl Transition {
    pub fn json(self, actor: crate::simulation::ActorId) -> String {
        format!(
            "{{\"actor_slot\":{},\"actor_generation\":{},\"first_slot\":{},\"second_slot\":{},\"first\":{},\"second\":{},\"output\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{}}}",
            actor.slot,
            actor.generation,
            self.job.first_slot,
            self.job.second_slot,
            self.job.recipe.first,
            self.job.recipe.second,
            self.job.recipe.output,
            self.queued_at,
            self.committed_at
        )
    }
}

// Both pair and multi-material assembly share this inventory transaction and XP
// preparation. Existing pair behavior is retained; ordinary multi-material jobs
// additionally require each captured slot to contain a single unmodified item.
pub(crate) fn exchange_slots(
    inv: &Inventory,
    catalog: &Catalog,
    inputs: &[(i32, usize)],
    outputs: &[i32],
    single: bool,
) -> Result<Inventory, Outcome> {
    let mut slots = std::collections::BTreeSet::new();
    if inputs.iter().any(|&(id, slot)| {
        !slots.insert(slot)
            || inv.get(slot).is_none_or(|i| {
                i.id != id || i.count <= 0 || (single && i.count != 1) || i.vars != 0
            })
    }) {
        return Err(Outcome::Stale);
    }
    let mut next = inv.clone();
    for &(id, slot) in inputs {
        let mut debit = Delete::new(id, 1);
        debit.strict_slot = Some(slot);
        next.delete(catalog, debit).map_err(|_| Outcome::Capacity)?;
    }
    for &id in outputs {
        next.insert(catalog, Insert::new(id, 1))
            .map_err(|_| Outcome::Capacity)?;
    }
    Ok(next)
}
fn prepare_reward(
    stats: &Stats,
    skill: u8,
    base_xp: u16,
    rate: i32,
) -> Result<(Stats, Option<crate::skills::Advancement>), Outcome> {
    prepare_reward_value(stats, skill, f64::from(base_xp), rate)
}
pub(crate) fn prepare_reward_value(
    stats: &Stats,
    skill: u8,
    base_xp: f64,
    rate: i32,
) -> Result<(Stats, Option<crate::skills::Advancement>), Outcome> {
    let mut next = stats.clone();
    let advancement = if base_xp == 0.0 {
        None
    } else {
        if rate <= 0 {
            return Err(Outcome::InvalidReward);
        }
        Some(
            crate::skills::advance(
                &mut next,
                skill,
                f64::from(base_xp),
                f64::from(rate) / 100.0,
                99,
            )
            .map_err(|_| Outcome::InvalidReward)?,
        )
    };
    Ok((next, advancement))
}
#[derive(Clone, Debug)]
pub struct BatchRecipe {
    pub inputs: Vec<i32>,
    pub outputs: Vec<i32>,
    pub skill: u8,
    pub level: u8,
    pub base_xp: u16,
    pub members: bool,
    pub ticks: u8,
}
#[derive(Clone, Debug)]
pub struct BatchJob {
    pub recipe: std::sync::Arc<BatchRecipe>,
    pub slots: Vec<usize>,
}
impl BatchRecipe {
    fn valid(&self) -> bool {
        self.inputs.len() >= 2
            && !self.outputs.is_empty()
            && self.ticks > 0
            && self.level > 0
            && self.inputs.iter().all(|v| *v >= 0)
            && self.outputs.iter().all(|v| *v >= 0)
            && self
                .inputs
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == self.inputs.len()
    }
    /// Retain both client-selected slots; resolve the remaining ingredients once
    /// at admission. Completion cannot substitute a different matching item.
    pub fn bind(
        self: &std::sync::Arc<Self>,
        request: crate::held_use::Request,
        inv: &Inventory,
    ) -> Option<BatchJob> {
        if !self.valid()
            || request.selected_id == request.target_id
            || request.selected_slot == request.target_slot
            || !self.inputs.contains(&request.selected_id)
            || !self.inputs.contains(&request.target_id)
        {
            return None;
        }
        let selected = usize::try_from(request.selected_slot).ok()?;
        let target = usize::try_from(request.target_slot).ok()?;
        let mut slots = Vec::with_capacity(self.inputs.len());
        for &id in &self.inputs {
            let slot = if id == request.selected_id {
                selected
            } else if id == request.target_id {
                target
            } else {
                inv.slots()
                    .iter()
                    .position(|v| v.is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0))?
            };
            if slots.contains(&slot)
                || inv
                    .get(slot)
                    .is_none_or(|i| i.id != id || i.count != 1 || i.vars != 0)
            {
                return None;
            }
            slots.push(slot);
        }
        Some(BatchJob {
            recipe: self.clone(),
            slots,
        })
    }
}
impl BatchJob {
    pub fn complete_awarding(
        &self,
        inv: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        rate: i32,
        member: bool,
    ) -> (Outcome, Option<crate::skills::Advancement>) {
        let r = &self.recipe;
        if inv.stack == crate::inventory::StackType::Always {
            return (Outcome::Capacity, None);
        }
        if !r.valid() || self.slots.len() != r.inputs.len() {
            return (Outcome::Stale, None);
        }
        if r.members && !member {
            return (Outcome::Members, None);
        }
        if i32::from(stats.current(r.skill)) < i32::from(r.level) {
            return (Outcome::Level, None);
        }
        let ordinary = |id| {
            !catalog.stackable.contains(&id)
                && !catalog.dummy.contains(&id)
                && !catalog.notes.get(&id).is_some_and(|v| v.template != 0)
                && !catalog
                    .placeholders
                    .get(&id)
                    .is_some_and(|v| v.template != 0)
                && !catalog.transforms.get(&id).is_some_and(|v| v.template != 0)
        };
        if !r.inputs.iter().chain(&r.outputs).copied().all(ordinary) {
            return (Outcome::Capacity, None);
        }
        let inputs: Vec<_> = r
            .inputs
            .iter()
            .copied()
            .zip(self.slots.iter().copied())
            .collect();
        let next = match exchange_slots(inv, catalog, &inputs, &r.outputs, true) {
            Ok(v) => v,
            Err(e) => return (e, None),
        };
        let (next_stats, advancement) = match prepare_reward(stats, r.skill, r.base_xp, rate) {
            Ok(v) => v,
            Err(e) => return (e, None),
        };
        *inv = next;
        *stats = next_stats;
        (Outcome::Made, advancement)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct BatchPending {
    pub job: BatchJob,
    pub queued_at: i32,
}
#[derive(Clone, Debug)]
pub struct BatchTransition {
    pub job: BatchJob,
    pub queued_at: i32,
    pub committed_at: i32,
}
impl BatchTransition {
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        format!("{{\"actor_slot\":{},\"actor_generation\":{},\"inputs\":{:?},\"slots\":{:?},\"outputs\":{:?},\"queued_at\":{},\"committed_at\":{}}}",actor.slot,actor.generation,self.job.recipe.inputs,self.job.slots,self.job.recipe.outputs,self.queued_at,self.committed_at)
    }
}

/// Admission and completion use the recipe's actual skill without changing legacy Cooking text.
pub fn level_message(skill: u8) -> &'static str {
    match skill {
        7 => "You need a higher Cooking level to make that.",
        12 => "You need a higher Crafting level to make that.",
        13 => "You need a higher Smithing level to make that.",
        _ => "You need a higher skill level to make that.",
    }
}
pub fn outcome_message_for_skill(outcome: Outcome, skill: u8) -> &'static str {
    if outcome == Outcome::Level {
        level_message(skill)
    } else {
        outcome_message(outcome)
    }
}
pub fn outcome_message(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Made => "You combine the ingredients.",
        Outcome::Tools => "You don't have all the required tools.",
        Outcome::Level => "You need a higher Cooking level to make that.",
        Outcome::Stale => "You no longer have those ingredients in the selected slots.",
        Outcome::Capacity => "You don't have enough inventory space to make that.",
        Outcome::InvalidReward => "You cannot make that item right now.",
        Outcome::Members => "You need to be a member to make that item.",
    }
}

#[cfg(test)]
mod batch_tests {
    use super::*;
    use crate::{
        held_use::Request,
        inventory::{Item, StackType},
    };
    use std::sync::Arc;
    #[test]
    fn batch_selected_duplicate_fullbag_returns_and_late_refusals_are_atomic() {
        for mode in 0..7 {
            let mut recipe = BatchRecipe {
                inputs: vec![10, 11, 12, 13],
                outputs: vec![20, 21, 22],
                skill: 7,
                level: 40,
                base_xp: 0,
                members: false,
                ticks: 2,
            };
            if mode == 5 {
                recipe.base_xp = 30;
            }
            if mode == 6 {
                recipe.members = true;
            }
            let recipe = Arc::new(recipe);
            let mut inv = Inventory::new(6, StackType::Normal, false);
            for (slot, id) in [(0, 10), (1, 11), (2, 12), (3, 13), (4, 10), (5, 99)] {
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
            let request = Request {
                selected_slot: 4,
                selected_id: 10,
                target_slot: 1,
                target_id: 11,
            };
            let job = recipe.bind(request, &inv).unwrap();
            assert_eq!(job.slots, vec![4, 1, 2, 3]);
            let mut stats = Stats::default();
            stats.set_base(7, 39);
            stats.set_current(7, 40);
            let mut catalog = Catalog::default();
            match mode {
                1 => {
                    catalog.dummy.insert(22);
                }
                2 => {
                    let item = inv.get(2);
                    inv.restore_slot(2, None).unwrap();
                    inv.restore_slot(5, item).unwrap();
                }
                3 => stats.set_current(7, 39),
                4 => inv.stack = StackType::Always,
                _ => {}
            }
            let before = (inv.clone(), stats.clone());
            let (outcome, _) = job.complete_awarding(
                &mut inv,
                &catalog,
                &mut stats,
                if mode == 5 { 0 } else { 500 },
                false,
            );
            if mode == 0 {
                assert_eq!(outcome, Outcome::Made);
                assert_eq!(stats, before.1);
                for id in [10, 20, 21, 22, 99] {
                    assert_eq!(
                        inv.slots().iter().flatten().filter(|i| i.id == id).count(),
                        1
                    );
                }
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false)
                        .0,
                    Outcome::Stale
                );
            } else {
                assert_ne!(outcome, Outcome::Made);
                assert_eq!((inv, stats), before);
            }
        }
    }
}

/// Durable alternatives can be carried or explicitly allowed in worn slots.
#[derive(Clone, Copy, Debug)]
pub struct ToolRequirement {
    pub inventory: &'static [i32],
    pub worn: &'static [i32],
}
impl ToolRequirement {
    pub fn available(self, inv: &Inventory, worn: &Inventory, cat: &Catalog) -> bool {
        let holds = |items: &Inventory, id| {
            cat.ordinary_nonstackable(id)
                && items
                    .slots()
                    .iter()
                    .flatten()
                    .any(|i| i.id == id && i.count == 1 && i.vars == 0)
        };
        self.inventory.iter().any(|&id| holds(inv, id))
            || self.worn.iter().any(|&id| holds(worn, id))
    }
}
pub fn tools_available(
    tools: &[ToolRequirement],
    inv: &Inventory,
    worn: &Inventory,
    cat: &Catalog,
) -> bool {
    tools.iter().all(|tool| tool.available(inv, worn, cat))
}
impl PreciseRecipe {
    pub fn animation_for(self, inv: &Inventory, worn: &Inventory, cat: &Catalog) -> Option<u16> {
        self.tool_animation
            .and_then(|(tool, sequence)| tool.available(inv, worn, cat).then_some(sequence))
            .or(self.animation)
    }
}
