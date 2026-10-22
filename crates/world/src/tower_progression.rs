//! Permanent solo floor accomplishments, independent of active encounters.
//! Definitions are supplied by content; this module grants no items, XP or credits.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FloorId(u16);
impl FloorId {
    pub const CINDERWORKS: Self = Self(1);
    pub const fn new(id: u16) -> Option<Self> {
        if id == 0 { None } else { Some(Self(id)) }
    }
    pub const fn get(self) -> u16 { self.0 }
}

/// Content-owned objectives. Optional objectives never become implicit gates.
#[derive(Clone, Copy, Debug)]
pub struct Definition<'a> {
    pub id: FloorId,
    pub prerequisites: &'a [FloorId],
    pub required: u64,
    pub optional: u64,
}
impl Definition<'_> {
    pub fn valid(self) -> bool {
        self.required != 0 && self.required & self.optional == 0
            && !self.prerequisites.contains(&self.id)
    }
    pub fn eligible(self, progress: &Progress) -> bool {
        self.valid() && self.prerequisites.iter().all(|id| progress.completed(*id))
    }
}

/// Attempt objectives are separate from permanent Progress. Content decides when
/// to start/reset an attempt and supplies only committed authoritative actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objectives {
    floor: FloorId,
    completed: u64,
}
impl Objectives {
    pub fn restore(definition: Definition<'_>, completed: u64) -> Result<Self, &'static str> {
        if !definition.valid() || completed & !(definition.required | definition.optional) != 0 {
            return Err("Invalid floor objective state");
        }
        Ok(Self { floor: definition.id, completed })
    }
    pub fn floor(&self) -> FloorId { self.floor }
    pub fn mask(&self) -> u64 { self.completed }
    pub fn new(definition: Definition<'_>) -> Result<Self, &'static str> {
        if !definition.valid() { return Err("Invalid floor definition"); }
        Ok(Self { floor: definition.id, completed: 0 })
    }
    pub fn credit(&mut self, definition: Definition<'_>, objective: u64) -> bool {
        if !definition.valid() || self.floor != definition.id || !objective.is_power_of_two()
            || objective & (definition.required | definition.optional) == 0
            || self.completed & objective != 0 { return false; }
        self.completed |= objective;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conquest { First, Replay }

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    completed: BTreeSet<FloorId>,
    achieved: BTreeMap<FloorId, u64>,
}
impl Progress {
    pub fn restore(completed: impl IntoIterator<Item = FloorId>) -> Self {
        Self { completed: completed.into_iter().collect(), achieved: BTreeMap::new() }
    }
    /// Storage restores historical masks without requiring today's content
    /// catalogue. Removed/changed content must not erase prior accomplishments.
    pub fn restore_history(&mut self, floor: FloorId, mask: u64) {
        if mask != 0 { *self.achieved.entry(floor).or_default() |= mask; }
    }
    pub fn history(&self) -> impl Iterator<Item = (FloorId, u64)> + '_ {
        self.achieved.iter().map(|(floor, mask)| (*floor, *mask))
    }
    pub fn achieved(&self, floor: FloorId) -> u64 { self.achieved.get(&floor).copied().unwrap_or(0) }
    pub fn observe(&mut self, definition: Definition<'_>, objectives: &Objectives) -> Result<bool, &'static str> {
        if !definition.eligible(self) || objectives.floor != definition.id
            || objectives.completed & !(definition.required | definition.optional) != 0 {
            return Err("Invalid or ineligible objective observation");
        }
        let before = self.achieved(definition.id);
        self.restore_history(definition.id, objectives.completed);
        Ok(self.achieved(definition.id) != before)
    }
    pub fn floors(&self) -> impl Iterator<Item = FloorId> + '_ {
        self.completed.iter().copied()
    }
    pub fn completed(&self, floor: FloorId) -> bool { self.completed.contains(&floor) }
    pub fn complete(&mut self, definition: Definition<'_>, objectives: &Objectives)
        -> Result<Conquest, &'static str>
    {
        if !definition.eligible(self) || objectives.floor != definition.id
            || objectives.completed & definition.required != definition.required {
            return Err("Floor completion prerequisites not met");
        }
        self.observe(definition, objectives)?;
        Ok(if self.completed.insert(definition.id) { Conquest::First } else { Conquest::Replay })
    }
    /// Import only an existing proven victory. Replay resets and reward claims
    /// must never remove conquest, grant another reward or manufacture milestones.
    pub fn reconcile_cinderworks(&mut self, run: crate::tower_run::Run) {
        if !run.valid() { return; }
        // An existing completed run proves all five mandatory objectives. An
        // incomplete run proves only its committed service/guardian flags.
        let mask = if run.clears > 0 { 31 } else { run.objective_mask() };
        self.restore_history(FloorId::CINDERWORKS, mask);
        if run.clears > 0 { self.completed.insert(FloorId::CINDERWORKS); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn second_floor_scaffold_is_gated_and_optional_work_does_not_gate_completion() {
        // Synthetic contract content only; neither a live Floor Two nor earned play.
        let second = FloorId::new(2).unwrap();
        let first = Definition { id: FloorId::CINDERWORKS, prerequisites: &[], required: 1, optional: 2 };
        let next = Definition { id: second, prerequisites: &[FloorId::CINDERWORKS], required: 4, optional: 8 };
        let mut progress = Progress::default();
        let mut a = Objectives::new(first).unwrap();
        let mut b = Objectives::new(next).unwrap();
        assert!(!next.eligible(&progress));
        assert!(b.credit(next, 4));
        assert!(progress.complete(next, &b).is_err());
        assert!(!a.credit(first, 3));
        assert!(!a.credit(next, 4));
        assert!(a.credit(first, 1));
        assert!(!a.credit(first, 1));
        assert_eq!(progress.complete(first, &a), Ok(Conquest::First));
        assert!(next.eligible(&progress));
        assert_eq!(progress.complete(next, &b), Ok(Conquest::First));
        assert_eq!(progress.complete(first, &a), Ok(Conquest::Replay));
        assert_eq!(progress.floors().count(), 2);
        assert_eq!(Objectives::new(first).unwrap().completed, 0);
        assert!(progress.completed(second));
    }
    #[test]
    fn invalid_definitions_and_unearned_run_never_unlock() {
        assert_eq!(FloorId::new(0), None);
        let invalid = Definition { id: FloorId::CINDERWORKS, prerequisites: &[], required: 1, optional: 1 };
        assert!(Objectives::new(invalid).is_err());
        let mut progress = Progress::default();
        let mut run = crate::tower_run::Run::default();
        run.begin();
        progress.reconcile_cinderworks(run);
        assert!(!progress.completed(FloorId::CINDERWORKS));
        for service in [crate::tower_run::Service::Forge, crate::tower_run::Service::Kitchen, crate::tower_run::Service::Altar] { run.service(service); }
        run.defeat_imp(); run.defeat_warden();
        progress.reconcile_cinderworks(run);
        assert!(progress.completed(FloorId::CINDERWORKS));
        assert!(run.reward_pending);
        run.paid(run.id); run.begin();
        progress.reconcile_cinderworks(run);
        assert!(progress.completed(FloorId::CINDERWORKS));
        assert_eq!(progress.floors().count(), 1);
    }
}
