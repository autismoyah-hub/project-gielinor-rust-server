//! Pure solo attempt/life-mode transition model for the approved Tower framework
//! (`docs/tower/floor-raid-framework.md` §3, §6, §9). Nothing here is wired to the
//! game, storage or runtime, and the legacy Cinderworks [`super::Run`] is unchanged.
//!
//! Every function only *proposes*. A [`Proposal`] names the next durable [`Attempt`]
//! (when it changes), the next transient [`RunProgress`], the durable intents the
//! caller must commit in one transaction with that record, and the intents it may
//! present only after the commit is acknowledged. The model never debits currency,
//! moves items, allocates or exposes an instance, or acknowledges a commit.
//!
//! Left to external caller guards, deliberately unresolved here (framework OPEN):
//! entry-cost and reclaim-penalty amounts and payability, inventory capacity for a
//! claim, whether unclaimed reclaim items or a stored reward block a new entry,
//! reward queue limits, non-failing death costs, mode switching and groups. This
//! module carries no economic value and no permissive default for any of them.
//!
//! An attempt and its progress are bound at start to the identity of the validated
//! [`FloorPlan`] they were started on ([`PlanKey`]); every transition first rejects any
//! other plan. Storing plans, versioning floor definitions and migrating attempts across
//! definition changes are later work, not decided here.

mod commit;
mod record;
pub use commit::*;
pub use record::*;

/// Life counts are the only approved numbers in the framework.
pub const HARDCORE_LIVES: u8 = 1;
pub const SOFTCORE_LIVES: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttemptId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FloorId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomIndex(pub u16);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransitionId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifeMode {
    Hardcore,
    Softcore,
    Easy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lives {
    Limited(u8),
    Unlimited,
}
impl LifeMode {
    pub fn lives(self) -> Lives {
        match self {
            Self::Hardcore => Lives::Limited(HARDCORE_LIVES),
            Self::Softcore => Lives::Limited(SOFTCORE_LIVES),
            Self::Easy => Lives::Unlimited,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomKind {
    Room,
    Boss,
    /// Exactly one per floor; only its defeat completes the attempt.
    FinalBoss,
}
/// A room is enterable once every room in `requires` is cleared in this run.
/// Prerequisite sets express both ordered and any-order rooms; they select no content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomSpec {
    pub kind: RoomKind,
    pub requires: Vec<RoomIndex>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloorPlan {
    floor: FloorId,
    rooms: Vec<RoomSpec>,
    key: PlanKey,
}
/// Identity of a validated plan: its floor, its room count and a deterministic FNV-1a
/// digest of every room's kind and requirements. It is a content identity for
/// mismatch detection, not a security hash; the room count is checked separately, so a
/// digest collision still can't index outside the progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlanKey {
    pub floor: FloorId,
    pub rooms: u16,
    pub digest: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanError {
    Empty,
    TooManyRooms,
    FinalBossCount,
    FirstRoomHasRequirements,
    UnknownRequirement,
    /// A requirement must point to an earlier room, which also rules out cycles.
    ForwardRequirement,
}
impl FloorPlan {
    pub fn new(floor: FloorId, rooms: Vec<RoomSpec>) -> Result<Self, PlanError> {
        if rooms.is_empty() {
            return Err(PlanError::Empty);
        }
        if rooms.len() > usize::from(u16::MAX) {
            return Err(PlanError::TooManyRooms);
        }
        if rooms.iter().filter(|r| r.kind == RoomKind::FinalBoss).count() != 1 {
            return Err(PlanError::FinalBossCount);
        }
        if !rooms[0].requires.is_empty() {
            return Err(PlanError::FirstRoomHasRequirements);
        }
        for (i, room) in rooms.iter().enumerate() {
            for r in &room.requires {
                let r = usize::from(r.0);
                if r >= rooms.len() {
                    return Err(PlanError::UnknownRequirement);
                }
                if r >= i {
                    return Err(PlanError::ForwardRequirement);
                }
            }
        }
        let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for b in bytes {
                digest = (digest ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&floor.0.to_le_bytes());
        eat(&(rooms.len() as u16).to_le_bytes());
        for room in &rooms {
            eat(&[match room.kind {
                RoomKind::Room => 0,
                RoomKind::Boss => 1,
                RoomKind::FinalBoss => 2,
            }]);
            eat(&(room.requires.len() as u16).to_le_bytes());
            for r in &room.requires {
                eat(&r.0.to_le_bytes());
            }
        }
        let key = PlanKey {
            floor,
            rooms: rooms.len() as u16,
            digest,
        };
        Ok(Self { floor, rooms, key })
    }
    pub fn floor(&self) -> FloorId {
        self.floor
    }
    pub fn key(&self) -> PlanKey {
        self.key
    }
    fn room(&self, room: RoomIndex) -> Option<&RoomSpec> {
        self.rooms.get(usize::from(room.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionEndCause {
    Leave,
    Logout,
    Disconnect,
    Crash,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptState {
    Active,
    /// The final boss fell; the reward is pending.
    Completed,
    RewardPaid,
    RewardStoredInLobby,
    Abandoned(SessionEndCause),
    FailedByDeath,
    Reclaimed,
}
impl AttemptState {
    /// In a run: a new entry is refused while this holds (framework one live run).
    pub fn live(self) -> bool {
        matches!(self, Self::Active | Self::Completed)
    }
}

/// Durable-shaped attempt record. This slice adds no persistence. There is no
/// setter for the mode, so it cannot change, and lives never increase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    id: AttemptId,
    floor: FloorId,
    mode: LifeMode,
    lives: Lives,
    instance: InstanceId,
    state: AttemptState,
    last: TransitionId,
    plan: PlanKey,
}
impl Attempt {
    pub fn id(&self) -> AttemptId {
        self.id
    }
    pub fn floor(&self) -> FloorId {
        self.floor
    }
    pub fn mode(&self) -> LifeMode {
        self.mode
    }
    pub fn lives(&self) -> Lives {
        self.lives
    }
    pub fn instance(&self) -> InstanceId {
        self.instance
    }
    pub fn state(&self) -> AttemptState {
        self.state
    }
    pub fn last_transition(&self) -> TransitionId {
        self.last
    }
    pub fn plan(&self) -> PlanKey {
        self.plan
    }
}

/// Transient per-instance run state: the current room and the rooms cleared in this
/// run. It is created only by [`propose_start`] and is never derived from an
/// [`Attempt`], so stored attempt data cannot resume a run. Not for persistence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunProgress {
    attempt: AttemptId,
    instance: InstanceId,
    current: RoomIndex,
    cleared: Vec<bool>,
    last: TransitionId,
    plan: PlanKey,
}
impl RunProgress {
    pub fn current(&self) -> RoomIndex {
        self.current
    }
    pub fn cleared(&self, room: RoomIndex) -> bool {
        self.cleared.get(usize::from(room.0)).copied().unwrap_or(false)
    }
    pub fn cleared_count(&self) -> usize {
        self.cleared.iter().filter(|c| **c).count()
    }
}

/// Intents the caller commits in one database transaction together with
/// [`Proposal::attempt`]. Amounts, items and tables come from external policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurableIntent {
    CreateAttempt,
    ChargeEntryCost,
    MoveCarriedItemsToReclaimChest,
    MoveRewardToLobbyClaimChest,
    PayReward,
    ReturnReclaimedItems,
    ChargeReclaimPenalty,
}
/// Intents the caller may present only after the commit is acknowledged, or that
/// need no commit at all (transient run state, cleanup). Repeating cleanup is harmless.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentIntent {
    ExposePreparedInstance,
    RespawnAtRoomStart(RoomIndex),
    /// Floor content decides what resets; cleared rooms are never named here.
    ResetRoom(RoomIndex),
    DiscardInstance,
    ReturnToLobby,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposal {
    /// `Some` when the durable record changes; commit it with `durable`.
    pub attempt: Option<Attempt>,
    /// `None` when the instance is to be discarded.
    pub progress: Option<RunProgress>,
    pub durable: Vec<DurableIntent>,
    pub present: Vec<PresentIntent>,
    /// The sealed storage input, present exactly when `attempt` is (a durable change).
    pub commit: Option<DurableCommit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimPath {
    FloorChest,
    LobbyClaimChest,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    EnterRoom(RoomIndex),
    ClearRoom(RoomIndex),
    DefeatFinalBoss,
    Death,
    SessionEnd(SessionEndCause),
    ClaimReward(ClaimPath),
    Reclaim,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    pub id: TransitionId,
    pub attempt: AttemptId,
    pub instance: InstanceId,
    pub event: Event,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejected {
    LiveAttemptExists,
    AttemptIdNotNewer,
    WrongAttempt,
    StaleInstance,
    NoInstance,
    Duplicate,
    Stale,
    NotActive,
    Terminal,
    UnknownRoom,
    RoomLocked,
    AlreadyCurrent,
    AlreadyCleared,
    NotCurrentRoom,
    FinalBossRoom,
    NotFinalBoss,
    WrongClaimPath,
    /// The plan, or the progress, isn't the one this attempt was started on.
    PlanMismatch,
}

/// Proposes a new attempt at the floor's first room with nothing cleared. Only the
/// approved one-live-run guard is checked here; every OPEN entry policy is the
/// caller's. The caller prepares the private instance first, commits the attempt
/// and the entry charge, and exposes the instance only after the acknowledgement.
pub fn propose_start(
    previous: Option<&Attempt>,
    plan: &FloorPlan,
    id: AttemptId,
    mode: LifeMode,
    instance: InstanceId,
    transition: TransitionId,
) -> Result<Proposal, Rejected> {
    if let Some(previous) = previous {
        if previous.state.live() {
            return Err(Rejected::LiveAttemptExists);
        }
        if id <= previous.id {
            return Err(Rejected::AttemptIdNotNewer);
        }
    }
    let attempt = Attempt {
        id,
        floor: plan.floor,
        mode,
        lives: mode.lives(),
        instance,
        state: AttemptState::Active,
        last: transition,
        plan: plan.key,
    };
    let progress = RunProgress {
        attempt: id,
        instance,
        current: RoomIndex(0),
        cleared: vec![false; plan.rooms.len()],
        last: transition,
        plan: plan.key,
    };
    let durable = vec![DurableIntent::CreateAttempt, DurableIntent::ChargeEntryCost];
    let commit = DurableCommit::new(
        None,
        CommitStep::Start { transition },
        attempt.record(),
        durable.clone(),
    );
    Ok(Proposal {
        attempt: Some(attempt),
        progress: Some(progress),
        durable,
        present: vec![PresentIntent::ExposePreparedInstance],
        commit: Some(commit),
    })
}

impl Attempt {
    /// Validates `t` against this record and the transient progress, and proposes the
    /// result. Nothing is mutated; a rejection leaves both inputs as they were. A durable
    /// change also carries its sealed [`DurableCommit`], built from the same values.
    pub fn propose(
        &self,
        plan: &FloorPlan,
        progress: Option<&RunProgress>,
        t: Transition,
    ) -> Result<Proposal, Rejected> {
        let mut proposal = self.propose_step(plan, progress, t)?;
        proposal.commit = proposal.attempt.as_ref().map(|next| {
            DurableCommit::new(
                Some(self.record()),
                CommitStep::Transition(t),
                next.record(),
                proposal.durable.clone(),
            )
        });
        Ok(proposal)
    }
    fn propose_step(
        &self,
        plan: &FloorPlan,
        progress: Option<&RunProgress>,
        t: Transition,
    ) -> Result<Proposal, Rejected> {
        if t.attempt != self.id {
            return Err(Rejected::WrongAttempt);
        }
        if plan.key != self.plan || plan.floor != self.floor {
            return Err(Rejected::PlanMismatch);
        }
        if let Some(p) = progress {
            let shape_ok = p.plan == self.plan
                && p.cleared.len() == plan.rooms.len()
                && usize::from(p.current.0) < plan.rooms.len();
            if !shape_ok {
                return Err(Rejected::PlanMismatch);
            }
        }
        // Exiting after a paid reward is cleanup only: no durable change, and it
        // neither re-pays nor re-arms. It consumes no transition id, so it can repeat.
        if let (AttemptState::RewardPaid, Event::SessionEnd(_)) = (self.state, t.event) {
            return Ok(Proposal {
                attempt: None,
                progress: None,
                durable: vec![],
                present: vec![PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby],
                commit: None,
            });
        }
        let last = progress.map_or(self.last, |p| p.last.max(self.last));
        if t.id == last {
            return Err(Rejected::Duplicate);
        }
        if t.id < last {
            return Err(Rejected::Stale);
        }
        let in_run = || self.in_run(progress, &t);
        let next = |state: AttemptState, lives: Lives| Attempt {
            state,
            lives,
            last: t.id,
            ..self.clone()
        };
        // Completed still holds a pending reward, so it is "not active" rather than ended.
        let not_active = if self.state == AttemptState::Completed {
            Rejected::NotActive
        } else {
            Rejected::Terminal
        };
        let in_room_event = matches!(
            t.event,
            Event::EnterRoom(_) | Event::ClearRoom(_) | Event::DefeatFinalBoss | Event::Death
        );
        if in_room_event && self.state != AttemptState::Active {
            return Err(not_active);
        }
        match t.event {
            Event::EnterRoom(room) => {
                let p = in_run()?;
                let spec = plan.room(room).ok_or(Rejected::UnknownRoom)?;
                if p.current == room {
                    return Err(Rejected::AlreadyCurrent);
                }
                if !spec.requires.iter().all(|r| p.cleared(*r)) {
                    return Err(Rejected::RoomLocked);
                }
                let mut p = p.clone();
                p.current = room;
                p.last = t.id;
                Ok(proposal(None, Some(p), vec![], vec![]))
            }
            Event::ClearRoom(room) => {
                let p = in_run()?;
                let spec = plan.room(room).ok_or(Rejected::UnknownRoom)?;
                if p.current != room {
                    return Err(Rejected::NotCurrentRoom);
                }
                if spec.kind == RoomKind::FinalBoss {
                    return Err(Rejected::FinalBossRoom);
                }
                if p.cleared(room) {
                    return Err(Rejected::AlreadyCleared);
                }
                let mut p = p.clone();
                *p.cleared.get_mut(usize::from(room.0)).ok_or(Rejected::UnknownRoom)? = true;
                p.last = t.id;
                Ok(proposal(None, Some(p), vec![], vec![]))
            }
            Event::DefeatFinalBoss => {
                let p = in_run()?;
                let spec = plan.room(p.current).ok_or(Rejected::UnknownRoom)?;
                if spec.kind != RoomKind::FinalBoss {
                    return Err(Rejected::NotFinalBoss);
                }
                if p.cleared(p.current) {
                    return Err(Rejected::AlreadyCleared);
                }
                let mut p = p.clone();
                let current = usize::from(p.current.0);
                *p.cleared.get_mut(current).ok_or(Rejected::UnknownRoom)? = true;
                p.last = t.id;
                Ok(proposal(
                    Some(next(AttemptState::Completed, self.lives)),
                    Some(p),
                    vec![],
                    vec![],
                ))
            }
            Event::Death => {
                let p = in_run()?;
                let lives = match self.lives {
                    Lives::Limited(n) if n <= 1 => {
                        return Ok(proposal(
                            Some(next(AttemptState::FailedByDeath, Lives::Limited(0))),
                            None,
                            vec![DurableIntent::MoveCarriedItemsToReclaimChest],
                            vec![PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby],
                        ));
                    }
                    Lives::Limited(n) => Lives::Limited(n - 1),
                    Lives::Unlimited => Lives::Unlimited,
                };
                let mut present = vec![PresentIntent::RespawnAtRoomStart(p.current)];
                if !p.cleared(p.current) {
                    present.push(PresentIntent::ResetRoom(p.current));
                }
                let mut p = p.clone();
                p.last = t.id;
                // Easy changes no durable value; a limited life count must commit.
                let attempt = (lives != self.lives).then(|| next(AttemptState::Active, lives));
                Ok(proposal(attempt, Some(p), vec![], present))
            }
            Event::SessionEnd(cause) => {
                let cleanup = vec![PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby];
                match self.state {
                    AttemptState::Active => Ok(proposal(
                        Some(next(AttemptState::Abandoned(cause), self.lives)),
                        None,
                        vec![],
                        cleanup,
                    )),
                    AttemptState::Completed => Ok(proposal(
                        Some(next(AttemptState::RewardStoredInLobby, self.lives)),
                        None,
                        vec![DurableIntent::MoveRewardToLobbyClaimChest],
                        cleanup,
                    )),
                    _ => Err(Rejected::Terminal),
                }
            }
            Event::ClaimReward(path) => match (self.state, path) {
                (AttemptState::Completed, ClaimPath::FloorChest) => {
                    in_run()?;
                    Ok(proposal(
                        Some(next(AttemptState::RewardPaid, self.lives)),
                        progress.cloned(),
                        vec![DurableIntent::PayReward],
                        vec![],
                    ))
                }
                (AttemptState::RewardStoredInLobby, ClaimPath::LobbyClaimChest) => Ok(proposal(
                    Some(next(AttemptState::RewardPaid, self.lives)),
                    None,
                    vec![DurableIntent::PayReward],
                    vec![],
                )),
                (AttemptState::Completed | AttemptState::RewardStoredInLobby, _) => {
                    Err(Rejected::WrongClaimPath)
                }
                (AttemptState::Active, _) => Err(Rejected::NotActive),
                _ => Err(Rejected::Terminal),
            },
            Event::Reclaim => match self.state {
                AttemptState::FailedByDeath => Ok(proposal(
                    Some(next(AttemptState::Reclaimed, self.lives)),
                    None,
                    vec![
                        DurableIntent::ChargeReclaimPenalty,
                        DurableIntent::ReturnReclaimedItems,
                    ],
                    vec![],
                )),
                AttemptState::Active | AttemptState::Completed => Err(Rejected::NotActive),
                _ => Err(Rejected::Terminal),
            },
        }
    }
    fn in_run<'a>(
        &self,
        progress: Option<&'a RunProgress>,
        t: &Transition,
    ) -> Result<&'a RunProgress, Rejected> {
        let p = progress.ok_or(Rejected::NoInstance)?;
        if t.instance != self.instance || p.instance != self.instance || p.attempt != self.id {
            return Err(Rejected::StaleInstance);
        }
        Ok(p)
    }
}
fn proposal(
    attempt: Option<Attempt>,
    progress: Option<RunProgress>,
    durable: Vec<DurableIntent>,
    present: Vec<PresentIntent>,
) -> Proposal {
    Proposal {
        attempt,
        progress,
        durable,
        present,
        commit: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AttemptState::*;
    use Event::*;

    const ROOM: RoomIndex = RoomIndex(0);
    // Sanctuary, two any-order service rooms, intermediate boss, final boss.
    fn plan() -> FloorPlan {
        let r = |kind, requires: &[u16]| RoomSpec {
            kind,
            requires: requires.iter().map(|i| RoomIndex(*i)).collect(),
        };
        FloorPlan::new(
            FloorId(1),
            vec![
                r(RoomKind::Room, &[]),
                r(RoomKind::Room, &[0]),
                r(RoomKind::Room, &[0]),
                r(RoomKind::Boss, &[1, 2]),
                r(RoomKind::FinalBoss, &[3]),
            ],
        )
        .unwrap()
    }
    struct Harness {
        plan: FloorPlan,
        attempt: Attempt,
        progress: Option<RunProgress>,
        next: u64,
    }
    impl Harness {
        fn start(mode: LifeMode) -> Self {
            let plan = plan();
            let p = propose_start(None, &plan, AttemptId(7), mode, InstanceId(70), TransitionId(1))
                .unwrap();
            Self {
                plan,
                attempt: p.attempt.unwrap(),
                progress: p.progress,
                next: 2,
            }
        }
        fn transition(&mut self, event: Event) -> Transition {
            let id = TransitionId(self.next);
            self.next += 1;
            Transition {
                id,
                attempt: self.attempt.id(),
                instance: self.attempt.instance(),
                event,
            }
        }
        /// Proposes and, when accepted, applies the proposal as a caller would after the
        /// commit and acknowledgement. A rejection must leave both inputs unchanged.
        fn apply(&mut self, event: Event) -> Result<Proposal, Rejected> {
            let t = self.transition(event);
            self.apply_t(t)
        }
        /// Proposes against another plan; must never be accepted or mutate anything.
        fn against(&mut self, plan: &FloorPlan, event: Event) -> Result<Proposal, Rejected> {
            let t = self.transition(event);
            let (attempt, progress) = (self.attempt.clone(), self.progress.clone());
            let result = self.attempt.propose(plan, self.progress.as_ref(), t);
            assert_eq!((&self.attempt, &self.progress), (&attempt, &progress));
            result
        }
        fn apply_t(&mut self, t: Transition) -> Result<Proposal, Rejected> {
            let (attempt, progress) = (self.attempt.clone(), self.progress.clone());
            let result = self.attempt.propose(&self.plan, self.progress.as_ref(), t);
            assert_eq!((&self.attempt, &self.progress), (&attempt, &progress));
            if let Ok(p) = &result {
                if let Some(a) = &p.attempt {
                    self.attempt = a.clone();
                }
                self.progress = p.progress.clone();
            }
            result
        }
        fn ok(&mut self, event: Event) -> Proposal {
            self.apply(event).unwrap()
        }
        fn clear_to_final(&mut self) {
            self.ok(ClearRoom(ROOM));
            self.ok(EnterRoom(RoomIndex(2)));
            self.ok(ClearRoom(RoomIndex(2)));
            self.ok(EnterRoom(RoomIndex(1)));
            self.ok(ClearRoom(RoomIndex(1)));
            self.ok(EnterRoom(RoomIndex(3)));
            self.ok(ClearRoom(RoomIndex(3)));
            self.ok(EnterRoom(RoomIndex(4)));
        }
        fn rejects_all(&mut self) {
            for event in [
                EnterRoom(RoomIndex(1)),
                ClearRoom(ROOM),
                DefeatFinalBoss,
                Death,
                ClaimReward(ClaimPath::FloorChest),
                ClaimReward(ClaimPath::LobbyClaimChest),
            ] {
                assert!(self.apply(event).is_err(), "{event:?} in {:?}", self.attempt.state());
            }
        }
    }

    fn room(kind: RoomKind, requires: &[u16]) -> RoomSpec {
        RoomSpec {
            kind,
            requires: requires.iter().map(|i| RoomIndex(*i)).collect(),
        }
    }
    fn variants() -> Vec<(&'static str, FloorPlan)> {
        let base = || plan().rooms;
        let mut longer = base();
        longer.push(room(RoomKind::Room, &[]));
        let mut shorter = base();
        shorter.remove(2);
        shorter[2].requires = vec![RoomIndex(1)];
        shorter[3].requires = vec![RoomIndex(2)];
        let mut kind = base();
        kind[3].kind = RoomKind::Room;
        let mut reqs = base();
        reqs[3].requires = vec![RoomIndex(1)];
        vec![
            ("other floor", FloorPlan::new(FloorId(2), base()).unwrap()),
            ("longer", FloorPlan::new(FloorId(1), longer).unwrap()),
            ("shorter", FloorPlan::new(FloorId(1), shorter).unwrap()),
            ("room kind", FloorPlan::new(FloorId(1), kind).unwrap()),
            ("requirements", FloorPlan::new(FloorId(1), reqs).unwrap()),
        ]
    }

    #[test]
    fn plan_keys_identify_floor_and_shape() {
        assert_eq!(plan().key(), plan().key());
        for (name, other) in variants() {
            assert_ne!(other.key(), plan().key(), "{name}");
        }
        let run = Harness::start(LifeMode::Easy);
        assert_eq!(run.attempt.plan(), plan().key());
    }

    #[test]
    fn a_different_or_reshaped_plan_is_rejected_without_mutation_or_panic() {
        for (name, other) in variants() {
            let mut run = Harness::start(LifeMode::Softcore);
            run.ok(ClearRoom(ROOM));
            for event in [
                EnterRoom(RoomIndex(5)),
                ClearRoom(RoomIndex(5)),
                EnterRoom(RoomIndex(1)),
                ClearRoom(ROOM),
                DefeatFinalBoss,
                Death,
                SessionEnd(SessionEndCause::Leave),
                ClaimReward(ClaimPath::FloorChest),
                Reclaim,
            ] {
                assert_eq!(run.against(&other, event), Err(Rejected::PlanMismatch), "{name} {event:?}");
            }
            // The bound plan still works afterwards.
            run.ok(EnterRoom(RoomIndex(1)));
        }
    }

    #[test]
    fn the_extra_room_attack_from_review_cannot_index_progress() {
        let mut longer = plan().rooms;
        longer.push(room(RoomKind::Room, &[]));
        let longer = FloorPlan::new(FloorId(1), longer).unwrap();
        let mut run = Harness::start(LifeMode::Easy);
        assert_eq!(run.against(&longer, EnterRoom(RoomIndex(5))), Err(Rejected::PlanMismatch));
        assert_eq!(run.against(&longer, ClearRoom(RoomIndex(5))), Err(Rejected::PlanMismatch));
        assert_eq!(run.progress.as_ref().unwrap().current(), ROOM);
    }

    #[test]
    fn progress_from_another_plan_or_attempt_is_rejected() {
        let other = variants().remove(1).1;
        let foreign = propose_start(None, &other, AttemptId(7), LifeMode::Easy, InstanceId(70), TransitionId(1))
            .unwrap()
            .progress;
        let mut run = Harness::start(LifeMode::Easy);
        run.progress = foreign;
        assert_eq!(run.apply(EnterRoom(RoomIndex(1))), Err(Rejected::PlanMismatch));
        assert_eq!(run.apply(Death), Err(Rejected::PlanMismatch));
    }

    #[test]
    fn rewardpaid_cleanup_also_requires_the_bound_plan() {
        let mut run = Harness::start(LifeMode::Easy);
        run.clear_to_final();
        run.ok(DefeatFinalBoss);
        run.ok(ClaimReward(ClaimPath::FloorChest));
        let other = variants().remove(0).1;
        assert_eq!(run.against(&other, SessionEnd(SessionEndCause::Leave)), Err(Rejected::PlanMismatch));
        assert!(run.ok(SessionEnd(SessionEndCause::Leave)).durable.is_empty());
    }

    #[test]
    fn plans_validate_final_boss_and_requirements() {
        let r = |kind, requires: Vec<RoomIndex>| RoomSpec { kind, requires };
        let f = FloorId(1);
        assert_eq!(FloorPlan::new(f, vec![]), Err(PlanError::Empty));
        assert_eq!(
            FloorPlan::new(f, vec![r(RoomKind::Room, vec![])]),
            Err(PlanError::FinalBossCount)
        );
        assert_eq!(
            FloorPlan::new(
                f,
                vec![r(RoomKind::FinalBoss, vec![]), r(RoomKind::FinalBoss, vec![])]
            ),
            Err(PlanError::FinalBossCount)
        );
        assert_eq!(
            FloorPlan::new(f, vec![r(RoomKind::FinalBoss, vec![RoomIndex(0)])]),
            Err(PlanError::FirstRoomHasRequirements)
        );
        assert_eq!(
            FloorPlan::new(
                f,
                vec![r(RoomKind::Room, vec![]), r(RoomKind::FinalBoss, vec![RoomIndex(9)])]
            ),
            Err(PlanError::UnknownRequirement)
        );
        assert_eq!(
            FloorPlan::new(
                f,
                vec![
                    r(RoomKind::Room, vec![]),
                    r(RoomKind::Room, vec![RoomIndex(2)]),
                    r(RoomKind::FinalBoss, vec![RoomIndex(1)])
                ]
            ),
            Err(PlanError::ForwardRequirement)
        );
    }

    #[test]
    fn start_proposes_intents_without_applying_them_and_new_runs_carry_nothing() {
        let plan = plan();
        let p = propose_start(None, &plan, AttemptId(1), LifeMode::Softcore, InstanceId(9), TransitionId(1))
            .unwrap();
        assert_eq!(p.durable, [DurableIntent::CreateAttempt, DurableIntent::ChargeEntryCost]);
        assert_eq!(p.present, [PresentIntent::ExposePreparedInstance]);
        let a = p.attempt.unwrap();
        assert_eq!((a.state(), a.lives(), a.mode()), (Active, Lives::Limited(3), LifeMode::Softcore));
        let progress = p.progress.unwrap();
        assert_eq!((progress.current(), progress.cleared_count()), (ROOM, 0));
        assert_eq!(
            propose_start(Some(&a), &plan, AttemptId(2), LifeMode::Easy, InstanceId(10), TransitionId(1)),
            Err(Rejected::LiveAttemptExists)
        );

        let mut run = Harness::start(LifeMode::Easy);
        run.clear_to_final();
        assert!(run.progress.as_ref().unwrap().cleared_count() >= 4);
        run.ok(SessionEnd(SessionEndCause::Leave));
        let done = run.attempt.clone();
        assert_eq!(
            propose_start(Some(&done), &plan, AttemptId(7), LifeMode::Easy, InstanceId(71), TransitionId(1)),
            Err(Rejected::AttemptIdNotNewer)
        );
        let fresh = propose_start(Some(&done), &plan, AttemptId(8), LifeMode::Easy, InstanceId(71), TransitionId(1))
            .unwrap();
        let fresh_progress = fresh.progress.unwrap();
        assert_eq!((fresh_progress.current(), fresh_progress.cleared_count()), (ROOM, 0));
        assert_eq!(fresh.attempt.unwrap().state(), Active);
    }

    #[test]
    fn hardcore_first_death_fails_by_death_with_reclaim_intent() {
        let mut run = Harness::start(LifeMode::Hardcore);
        assert_eq!(run.attempt.lives(), Lives::Limited(1));
        let p = run.ok(Death);
        assert_eq!(run.attempt.state(), FailedByDeath);
        assert_eq!(p.durable, [DurableIntent::MoveCarriedItemsToReclaimChest]);
        assert_eq!(p.present, [PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby]);
        assert_eq!(run.progress, None);
        run.rejects_all();
        assert_eq!(run.apply(SessionEnd(SessionEndCause::Leave)), Err(Rejected::Terminal));
        assert_eq!(run.attempt.state(), FailedByDeath);
    }

    #[test]
    fn softcore_respawns_at_current_room_keeping_earlier_clears_then_fails_on_third_death() {
        let mut run = Harness::start(LifeMode::Softcore);
        run.ok(ClearRoom(ROOM));
        run.ok(EnterRoom(RoomIndex(1)));
        run.ok(ClearRoom(RoomIndex(1)));
        run.ok(EnterRoom(RoomIndex(2)));
        for lives in [2, 1] {
            let p = run.ok(Death);
            assert_eq!(run.attempt.lives(), Lives::Limited(lives));
            assert!(p.attempt.is_some(), "a limited life count is durable");
            assert!(p.durable.is_empty());
            assert_eq!(
                p.present,
                [PresentIntent::RespawnAtRoomStart(RoomIndex(2)), PresentIntent::ResetRoom(RoomIndex(2))]
            );
            let progress = run.progress.as_ref().unwrap();
            assert_eq!(progress.current(), RoomIndex(2));
            assert!(progress.cleared(ROOM) && progress.cleared(RoomIndex(1)));
            assert!(!progress.cleared(RoomIndex(2)));
        }
        let p = run.ok(Death);
        assert_eq!((run.attempt.state(), run.attempt.lives()), (FailedByDeath, Lives::Limited(0)));
        assert_eq!(p.durable, [DurableIntent::MoveCarriedItemsToReclaimChest]);
    }

    #[test]
    fn death_in_a_room_just_cleared_never_rearms_it() {
        let mut run = Harness::start(LifeMode::Softcore);
        run.ok(ClearRoom(ROOM));
        run.ok(EnterRoom(RoomIndex(1)));
        run.ok(EnterRoom(RoomIndex(2)));
        run.ok(EnterRoom(RoomIndex(1)));
        run.ok(ClearRoom(RoomIndex(1)));
        let p = run.ok(Death);
        assert_eq!(p.present, [PresentIntent::RespawnAtRoomStart(RoomIndex(1))]);
        assert!(run.progress.as_ref().unwrap().cleared(RoomIndex(1)));
        assert_eq!(run.apply(ClearRoom(RoomIndex(1))), Err(Rejected::AlreadyCleared));
    }

    #[test]
    fn easy_never_fails_by_death_and_commits_nothing_for_a_death() {
        let mut run = Harness::start(LifeMode::Easy);
        run.ok(ClearRoom(ROOM));
        run.ok(EnterRoom(RoomIndex(1)));
        for _ in 0..20 {
            let p = run.ok(Death);
            assert_eq!(p.attempt, None);
            assert!(p.durable.is_empty());
            assert_eq!(run.attempt.state(), Active);
            assert_eq!(run.attempt.lives(), Lives::Unlimited);
        }
        assert!(run.progress.as_ref().unwrap().cleared(ROOM));
    }

    #[test]
    fn only_the_current_final_boss_completes_and_locked_or_unknown_rooms_refuse() {
        let mut run = Harness::start(LifeMode::Easy);
        assert_eq!(run.apply(DefeatFinalBoss), Err(Rejected::NotFinalBoss));
        assert_eq!(run.apply(EnterRoom(RoomIndex(3))), Err(Rejected::RoomLocked));
        assert_eq!(run.apply(EnterRoom(RoomIndex(99))), Err(Rejected::UnknownRoom));
        assert_eq!(run.apply(EnterRoom(ROOM)), Err(Rejected::AlreadyCurrent));
        assert_eq!(run.apply(ClearRoom(RoomIndex(1))), Err(Rejected::NotCurrentRoom));
        run.ok(ClearRoom(ROOM));
        run.ok(EnterRoom(RoomIndex(2)));
        run.ok(ClearRoom(RoomIndex(2)));
        assert_eq!(run.apply(EnterRoom(RoomIndex(3))), Err(Rejected::RoomLocked));
        run.ok(EnterRoom(RoomIndex(1)));
        run.ok(ClearRoom(RoomIndex(1)));
        run.ok(EnterRoom(RoomIndex(3)));
        let p = run.ok(ClearRoom(RoomIndex(3)));
        assert_eq!(p.attempt, None, "an intermediate boss doesn't complete the run");
        assert_eq!(run.attempt.state(), Active);
        run.ok(EnterRoom(RoomIndex(4)));
        assert_eq!(run.apply(ClearRoom(RoomIndex(4))), Err(Rejected::FinalBossRoom));
        let p = run.ok(DefeatFinalBoss);
        assert_eq!(run.attempt.state(), Completed);
        assert!(p.durable.is_empty(), "reward stays pending");
        assert_eq!(run.apply(DefeatFinalBoss), Err(Rejected::NotActive));
        assert_eq!(run.apply(Death), Err(Rejected::NotActive));
        assert_eq!(run.apply(Reclaim), Err(Rejected::NotActive));
    }

    #[test]
    fn every_session_end_cause_abandons_with_no_reclaim_intent() {
        for cause in [
            SessionEndCause::Leave,
            SessionEndCause::Logout,
            SessionEndCause::Disconnect,
            SessionEndCause::Crash,
        ] {
            let mut run = Harness::start(LifeMode::Softcore);
            run.ok(ClearRoom(ROOM));
            run.ok(Death);
            let p = run.ok(SessionEnd(cause));
            assert_eq!(run.attempt.state(), Abandoned(cause));
            assert_eq!(run.attempt.lives(), Lives::Limited(2));
            assert!(p.durable.is_empty(), "no reclaim and no penalty");
            assert_eq!(p.present, [PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby]);
            assert_eq!(run.progress, None);
            run.rejects_all();
            assert_eq!(run.apply(Reclaim), Err(Rejected::Terminal));
            assert_eq!(run.apply(SessionEnd(cause)), Err(Rejected::Terminal));
        }
    }

    #[test]
    fn completed_session_end_stores_reward_and_only_the_lobby_claims_it_once() {
        let mut run = Harness::start(LifeMode::Hardcore);
        run.clear_to_final();
        run.ok(DefeatFinalBoss);
        let p = run.ok(SessionEnd(SessionEndCause::Disconnect));
        assert_eq!(run.attempt.state(), RewardStoredInLobby);
        assert_eq!(p.durable, [DurableIntent::MoveRewardToLobbyClaimChest]);
        assert_eq!(run.apply(SessionEnd(SessionEndCause::Crash)), Err(Rejected::Terminal));
        assert_eq!(run.apply(ClaimReward(ClaimPath::FloorChest)), Err(Rejected::WrongClaimPath));
        let p = run.ok(ClaimReward(ClaimPath::LobbyClaimChest));
        assert_eq!((run.attempt.state(), p.durable), (RewardPaid, vec![DurableIntent::PayReward]));
        assert_eq!(run.apply(ClaimReward(ClaimPath::LobbyClaimChest)), Err(Rejected::Terminal));
        run.rejects_all();
    }

    #[test]
    fn floor_claim_then_exit_cleans_up_without_repaying() {
        let mut run = Harness::start(LifeMode::Softcore);
        run.clear_to_final();
        run.ok(DefeatFinalBoss);
        assert_eq!(run.apply(ClaimReward(ClaimPath::LobbyClaimChest)), Err(Rejected::WrongClaimPath));
        let p = run.ok(ClaimReward(ClaimPath::FloorChest));
        assert_eq!(p.durable, [DurableIntent::PayReward]);
        assert!(run.progress.is_some(), "still in the reward room");
        let paid = run.attempt.clone();
        for cause in [SessionEndCause::Leave, SessionEndCause::Logout, SessionEndCause::Leave] {
            let p = run.ok(SessionEnd(cause));
            assert_eq!(p.attempt, None);
            assert!(p.durable.is_empty(), "never pays twice");
            assert_eq!(p.present, [PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby]);
            assert_eq!(run.attempt, paid);
            assert_eq!(run.progress, None);
        }
        run.rejects_all();
    }

    #[test]
    fn reclaim_happens_once_after_death_failure() {
        let mut run = Harness::start(LifeMode::Hardcore);
        run.ok(Death);
        assert_eq!(run.apply(ClaimReward(ClaimPath::LobbyClaimChest)), Err(Rejected::Terminal));
        let p = run.ok(Reclaim);
        assert_eq!(
            p.durable,
            [DurableIntent::ChargeReclaimPenalty, DurableIntent::ReturnReclaimedItems]
        );
        assert_eq!(run.attempt.state(), Reclaimed);
        assert_eq!(run.apply(Reclaim), Err(Rejected::Terminal));
        let mut active = Harness::start(LifeMode::Softcore);
        assert_eq!(active.apply(Reclaim), Err(Rejected::NotActive));
    }

    #[test]
    fn duplicate_stale_and_foreign_transitions_are_rejected_unchanged() {
        let mut run = Harness::start(LifeMode::Softcore);
        let t = run.transition(ClearRoom(ROOM));
        run.apply_t(t).unwrap();
        assert_eq!(run.apply_t(t), Err(Rejected::Duplicate));
        let enter = run.transition(EnterRoom(RoomIndex(1)));
        run.apply_t(enter).unwrap();
        assert_eq!(
            run.apply_t(Transition { event: Death, ..t }),
            Err(Rejected::Stale)
        );
        let mut foreign = run.transition(Death);
        foreign.attempt = AttemptId(99);
        assert_eq!(run.apply_t(foreign), Err(Rejected::WrongAttempt));
        let mut old_instance = run.transition(Death);
        old_instance.instance = InstanceId(1);
        assert_eq!(run.apply_t(old_instance), Err(Rejected::StaleInstance));
        let death = run.transition(Death);
        run.apply_t(death).unwrap();
        assert_eq!(run.apply_t(death), Err(Rejected::Duplicate));
        assert_eq!(run.attempt.lives(), Lives::Limited(2));
        // A durable record without its transient instance can't continue the run.
        run.progress = None;
        assert_eq!(run.apply(EnterRoom(ROOM)), Err(Rejected::NoInstance));
        assert_eq!(run.apply(Death), Err(Rejected::NoInstance));
        let p = run.ok(SessionEnd(SessionEndCause::Crash));
        assert_eq!(run.attempt.state(), Abandoned(SessionEndCause::Crash));
        assert!(p.durable.is_empty());
    }
}
