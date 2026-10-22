//! Content definitions, objective dependencies and read-only boss discovery.
//! This does not admit boss-only replay, allocate instances or choose credit rules.
use crate::tower_progression::{Definition, FloorId, Objectives, Progress};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
pub struct Objective {
    pub bit: u64,
    pub name: &'static str,
    pub requires: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Boss {
    /// Stable Tower content identity, independent of cache NPC IDs.
    pub id: u16,
    pub name: &'static str,
    pub objective: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Floor<'a> {
    pub name: &'static str,
    pub definition: Definition<'a>,
    pub objectives: &'a [Objective],
    pub bosses: &'a [Boss],
}
impl Floor<'_> {
    pub fn valid(self) -> bool {
        if !self.definition.valid() || self.name.is_empty() { return false; }
        let all = self.definition.required | self.definition.optional;
        let mut seen = 0;
        for objective in self.objectives {
            if !objective.bit.is_power_of_two() || objective.name.is_empty()
                || seen & objective.bit != 0 || objective.requires & !all != 0
                || objective.requires & objective.bit != 0
                || objective.bit & self.definition.required != 0
                    && objective.requires & self.definition.optional != 0 { return false; }
            seen |= objective.bit;
        }
        if seen != all { return false; }
        // Resolve the dependency DAG, including optional objectives.
        let mut resolved = 0;
        loop {
            let before = resolved;
            for objective in self.objectives {
                if objective.requires & resolved == objective.requires { resolved |= objective.bit; }
            }
            if resolved == all { break; }
            if before == resolved { return false; }
        }
        let mut bosses = BTreeSet::new();
        let mut boss_objectives = BTreeSet::new();
        self.bosses.iter().all(|boss| boss.id != 0 && !boss.name.is_empty()
            && boss.objective.is_power_of_two() && boss.objective & all != 0
            && bosses.insert(boss.id) && boss_objectives.insert(boss.objective))
    }
    pub fn restore(self, mask: u64) -> Result<Objectives, &'static str> {
        if !self.valid() { return Err("Invalid floor definition"); }
        let attempt = Objectives::restore(self.definition, mask)?;
        if self.objectives.iter().any(|o| mask & o.bit != 0 && mask & o.requires != o.requires) {
            return Err("Objective dependencies are incomplete");
        }
        Ok(attempt)
    }
    pub fn credit(self, attempt: &mut Objectives, bit: u64) -> bool {
        if self.restore(attempt.mask()).is_err() { return false; }
        let Some(objective) = self.objectives.iter().find(|o| o.bit == bit) else { return false; };
        if attempt.mask() & objective.requires != objective.requires { return false; }
        attempt.credit(self.definition, bit)
    }
    pub fn boss_records(self, progress: &Progress) -> Vec<BossRecord> {
        self.bosses.iter().map(|boss| BossRecord {
            floor: self.definition.id, id: boss.id, name: boss.name,
            defeated: progress.achieved(self.definition.id) & boss.objective != 0,
        }).collect()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BossRecord {
    pub floor: FloorId,
    pub id: u16,
    pub name: &'static str,
    pub defeated: bool,
}

pub struct Catalogue<'a> { floors: &'a [Floor<'a>] }
impl<'a> Catalogue<'a> {
    /// Reject unknown prerequisites, duplicate identities and cyclic floor graphs.
    pub fn new(floors: &'a [Floor<'a>]) -> Result<Self, &'static str> {
        let mut ids = BTreeSet::new();
        let mut bosses = BTreeSet::new();
        for floor in floors {
            if !floor.valid() || !ids.insert(floor.definition.id)
                || !floor.bosses.iter().all(|b| bosses.insert(b.id)) {
                return Err("Invalid Tower floor or duplicate content identity");
            }
        }
        for floor in floors {
            let prerequisites = floor.definition.prerequisites;
            if prerequisites.iter().any(|id| !ids.contains(id))
                || prerequisites.iter().copied().collect::<BTreeSet<_>>().len() != prerequisites.len() {
                return Err("Unknown or duplicate floor prerequisite");
            }
        }
        let mut reachable = Progress::default();
        loop {
            let before = reachable.floors().count();
            for floor in floors {
                if floor.definition.eligible(&reachable) {
                    let objectives = Objectives::restore(floor.definition,
                        floor.definition.required | floor.definition.optional)?;
                    reachable.complete(floor.definition, &objectives)?;
                }
            }
            if reachable.floors().count() == floors.len() { return Ok(Self { floors }); }
            if before == reachable.floors().count() { return Err("Cyclic floor prerequisites"); }
        }
    }
    pub fn floor(&self, id: FloorId) -> Option<Floor<'a>> {
        self.floors.iter().find(|floor| floor.definition.id == id).copied()
    }
    pub fn eligible(&self, id: FloorId, progress: &Progress) -> bool {
        self.floor(id).is_some_and(|floor| floor.definition.eligible(progress))
    }
    pub fn boss_records(&self, progress: &Progress) -> Vec<BossRecord> {
        self.floors.iter().flat_map(|floor| floor.boss_records(progress)).collect()
    }
}

pub const WORKSHOP: u64 = 1;
pub const KITCHEN: u64 = 2;
pub const ALTAR: u64 = 4;
pub const IMP: u64 = 8;
pub const WARDEN: u64 = 16;
pub const CINDERWORKS: Floor<'static> = Floor {
    name: "The Cinderworks",
    definition: Definition { id: FloorId::CINDERWORKS, prerequisites: &[], required: 31, optional: 0 },
    objectives: &[
        Objective { bit: WORKSHOP, name: "Restore the workshop", requires: 0 },
        Objective { bit: KITCHEN, name: "Restore the kitchen", requires: 0 },
        Objective { bit: ALTAR, name: "Restore the altar", requires: 0 },
        Objective { bit: IMP, name: "Silence the Ash Imp", requires: 7 },
        Objective { bit: WARDEN, name: "Quench the Ash Warden", requires: 15 },
    ],
    bosses: &[
        Boss { id: 1, name: "Ash Imp", objective: IMP },
        Boss { id: 2, name: "Ash Warden", objective: WARDEN },
    ],
};
pub const FLOORS: &[Floor<'static>] = &[CINDERWORKS];

/// The existing task overlay can show this in the lobby or any floor. Reading
/// it never grants discovery, starts a run or changes replay eligibility.
pub fn journal(progress: &Progress, run: crate::tower_run::Run) -> String {
    let mut known = progress.clone();
    known.reconcile_cinderworks(run);
    let milestones = (known.achieved(FloorId::CINDERWORKS) & 31).count_ones();
    let mut body = format!("{}<br><br><col=ffb83f>Permanent record</col><br>Floors conquered: {}<br>Cinderworks milestones: {milestones}/5<br><br>Boss catalogue", run.journal(), known.floors().count());
    for boss in CINDERWORKS.boss_records(&known) {
        body.push_str(&format!("<br>{}: {}", boss.name, if boss.defeated { "defeated" } else { "not defeated" }));
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_floor_dependencies_and_boss_discovery_are_separate_from_replay_admission() {
        let catalogue = Catalogue::new(FLOORS).unwrap();
        let mut objectives = Objectives::new(CINDERWORKS.definition).unwrap();
        assert!(!CINDERWORKS.credit(&mut objectives, WARDEN));
        assert!(!CINDERWORKS.credit(&mut objectives, IMP));
        assert!(CINDERWORKS.restore(WARDEN).is_err());
        for bit in [ALTAR, KITCHEN, WORKSHOP, IMP] { assert!(CINDERWORKS.credit(&mut objectives, bit)); }
        let mut progress = Progress::default();
        progress.observe(CINDERWORKS.definition, &objectives).unwrap();
        let bosses = catalogue.boss_records(&progress);
        assert!(bosses[0].defeated); assert!(!bosses[1].defeated);
        assert!(!progress.completed(FloorId::CINDERWORKS));
        assert!(!catalogue.eligible(FloorId::new(2).unwrap(), &progress));
        assert!(CINDERWORKS.credit(&mut objectives, WARDEN));
        progress.complete(CINDERWORKS.definition, &objectives).unwrap();
        assert!(catalogue.boss_records(&progress).iter().all(|b| b.defeated));
    }
    #[test]
    fn cycles_missing_prerequisites_duplicate_ids_and_optional_skill_gates_are_rejected() {
        let first = Floor { definition: Definition { prerequisites: &[FloorId::new(2).unwrap()], ..CINDERWORKS.definition }, ..CINDERWORKS };
        let second = Floor { definition: Definition { id: FloorId::new(2).unwrap(), prerequisites: &[FloorId::CINDERWORKS], ..CINDERWORKS.definition }, bosses: &[], ..CINDERWORKS };
        assert!(Catalogue::new(&[first, second]).is_err());
        assert!(Catalogue::new(&[first]).is_err());
        assert!(Catalogue::new(&[CINDERWORKS, CINDERWORKS]).is_err());
        let optional = Floor {
            definition: Definition { required: 1, optional: 2, ..CINDERWORKS.definition },
            objectives: &[Objective { bit: 1, name: "Required", requires: 2 }, Objective { bit: 2, name: "Optional", requires: 0 }],
            bosses: &[], ..CINDERWORKS
        };
        assert!(!optional.valid());
        let cycle = Floor {
            definition: Definition { required: 3, optional: 0, ..CINDERWORKS.definition },
            objectives: &[Objective { bit: 1, name: "A", requires: 2 }, Objective { bit: 2, name: "B", requires: 1 }],
            bosses: &[], ..CINDERWORKS
        };
        assert!(!cycle.valid());
    }
}
