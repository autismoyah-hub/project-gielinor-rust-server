//! Sealed durable commit: the only input storage accepts for an attempt write.
//!
//! A [`DurableCommit`] is built only inside the attempt module, from a proposal the pure
//! model produced (`propose_start` and the durable branches of `Attempt::propose`). It holds
//! the stored base record, the step, the resulting record and the ordered durable intents,
//! with private fields and no setters. Non-durable proposals (room movement, clears, an easy
//! death, cleanup after a paid reward) carry no commit.
//!
//! [`DurableCommit::validate`] re-checks what is knowable without the transient
//! `RunProgress`: identity, ordering, unchanged invariants, non-increasing lives and that
//! both records restore. It doesn't re-derive the state machine; that stays the model's.
//!
//! A start has no triggering [`Event`], so it has no canonical request identity yet. Defining
//! one belongs to the slice that enables starts; until then storage refuses start commits.
//!
//! The constructor is not reachable outside the attempt module:
//!
//! ```compile_fail
//! use gielinor_world::tower_run::attempt::*;
//! fn forge(base: AttemptRecord, t: Transition, result: AttemptRecord) -> DurableCommit {
//!     DurableCommit::new(Some(base), CommitStep::Transition(t), result, vec![])
//! }
//! ```
use super::*;

/// What a commit records: the creation of an attempt, or one transition of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitStep {
    /// `propose_start` with its transition id and the prepared instance.
    Start { transition: TransitionId },
    Transition(Transition),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableCommit {
    base: Option<AttemptRecord>,
    step: CommitStep,
    result: AttemptRecord,
    intents: Vec<DurableIntent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    Record(RecordError),
    Restore(RestoreError),
    /// A start with a base, or a transition without one.
    BaseShape,
    AttemptMismatch,
    /// `result.last` isn't the step's transition id.
    ResultOrder,
    /// The step's id isn't greater than `base.last`.
    BaseOrder,
    /// Instance, mode, plan or floor changed between base and result, or differ from the plan.
    InvariantChanged,
    LivesIncreased,
    /// A transition that changes neither state nor lives isn't durable.
    NoChange,
    /// A start must yield an Active attempt with the mode's full lives.
    StartShape,
    TooManyIntents,
}

impl DurableCommit {
    pub(super) fn new(
        base: Option<AttemptRecord>,
        step: CommitStep,
        result: AttemptRecord,
        intents: Vec<DurableIntent>,
    ) -> Self {
        Self {
            base,
            step,
            result,
            intents,
        }
    }
    pub fn base(&self) -> Option<&AttemptRecord> {
        self.base.as_ref()
    }
    pub fn step(&self) -> CommitStep {
        self.step
    }
    pub fn result(&self) -> &AttemptRecord {
        &self.result
    }
    pub fn intents(&self) -> &[DurableIntent] {
        &self.intents
    }
    pub fn attempt(&self) -> AttemptId {
        self.result.id
    }
    pub fn transition_id(&self) -> TransitionId {
        match self.step {
            CommitStep::Start { transition } => transition,
            CommitStep::Transition(t) => t.id,
        }
    }
    /// The canonical request bytes (slice A framing) for a transition step. `None` for a
    /// start, which has no request identity yet.
    pub fn request_bytes(&self) -> Option<Result<Vec<u8>, RecordError>> {
        match self.step {
            CommitStep::Start { .. } => None,
            CommitStep::Transition(t) => Some(canonical_request(
                &t,
                self.base.as_ref(),
                &self.result,
                &self.intents,
            )),
        }
    }
    /// Checks everything knowable without the transient run progress.
    pub fn validate(&self, plan: &FloorPlan) -> Result<(), StepError> {
        if self.intents.len() > MAX_INTENTS {
            return Err(StepError::TooManyIntents);
        }
        let result = Attempt::restore(self.result, plan).map_err(StepError::Restore)?;
        if self.result.last != self.transition_id() {
            return Err(StepError::ResultOrder);
        }
        match (self.step, self.base) {
            (CommitStep::Start { .. }, None) => {
                if result.state() != AttemptState::Active || result.lives() != result.mode().lives() {
                    return Err(StepError::StartShape);
                }
                Ok(())
            }
            (CommitStep::Transition(t), Some(base_record)) => {
                if t.attempt.0 == 0 || t.id.0 == 0 {
                    return Err(StepError::Record(RecordError::ZeroId));
                }
                if self.result.id != t.attempt || base_record.id != t.attempt {
                    return Err(StepError::AttemptMismatch);
                }
                let base = Attempt::restore(base_record, plan).map_err(StepError::Restore)?;
                if base_record.last >= t.id {
                    return Err(StepError::BaseOrder);
                }
                let same = |r: &AttemptRecord| {
                    (r.instance, r.mode, r.plan, r.floor)
                };
                if same(&base_record) != same(&self.result) {
                    return Err(StepError::InvariantChanged);
                }
                let increased = match (base.lives(), result.lives()) {
                    (Lives::Limited(before), Lives::Limited(after)) => after > before,
                    (Lives::Unlimited, Lives::Unlimited) => false,
                    _ => true,
                };
                if increased {
                    return Err(StepError::LivesIncreased);
                }
                if (base.state(), base.lives()) == (result.state(), result.lives()) {
                    return Err(StepError::NoChange);
                }
                Ok(())
            }
            _ => Err(StepError::BaseShape),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AttemptState::*;
    use DurableIntent::*;

    fn plan() -> FloorPlan {
        let r = |kind, requires: &[u16]| RoomSpec {
            kind,
            requires: requires.iter().map(|i| RoomIndex(*i)).collect(),
        };
        FloorPlan::new(
            FloorId(1),
            vec![
                r(RoomKind::Room, &[]),
                r(RoomKind::Boss, &[0]),
                r(RoomKind::FinalBoss, &[1]),
            ],
        )
        .unwrap()
    }
    struct Drive {
        plan: FloorPlan,
        attempt: Attempt,
        progress: Option<RunProgress>,
        next: u64,
        commits: Vec<(Event, Option<DurableCommit>)>,
        start: DurableCommit,
    }
    impl Drive {
        fn start(mode: LifeMode) -> Self {
            let plan = plan();
            let p = propose_start(None, &plan, AttemptId(9), mode, InstanceId(90), TransitionId(1))
                .unwrap();
            Self {
                plan,
                attempt: p.attempt.unwrap(),
                progress: p.progress,
                next: 2,
                commits: vec![],
                start: p.commit.unwrap(),
            }
        }
        fn go(&mut self, event: Event) -> Proposal {
            let t = Transition {
                id: TransitionId(self.next),
                attempt: self.attempt.id(),
                instance: self.attempt.instance(),
                event,
            };
            self.next += 1;
            let p = self.attempt.propose(&self.plan, self.progress.as_ref(), t).unwrap();
            self.commits.push((event, p.commit.clone()));
            if let Some(a) = &p.attempt {
                self.attempt = a.clone();
            }
            self.progress = p.progress.clone();
            p
        }
        fn to_final(&mut self) {
            self.go(Event::ClearRoom(RoomIndex(0)));
            self.go(Event::EnterRoom(RoomIndex(1)));
            self.go(Event::ClearRoom(RoomIndex(1)));
            self.go(Event::EnterRoom(RoomIndex(2)));
            self.go(Event::DefeatFinalBoss);
        }
    }
    /// Specification oracle (test only): the durable steps of the PR #116 model, with the
    /// exact ordered intents. Production code never consults it.
    fn oracle(
        mode: LifeMode,
        base: AttemptState,
        base_lives: Lives,
        event: Event,
    ) -> Option<(AttemptState, Lives, Vec<DurableIntent>)> {
        let _ = mode;
        match (base, event) {
            (Active, Event::Death) => match base_lives {
                Lives::Limited(1) => Some((FailedByDeath, Lives::Limited(0), vec![MoveCarriedItemsToReclaimChest])),
                Lives::Limited(n) => Some((Active, Lives::Limited(n - 1), vec![])),
                Lives::Unlimited => None,
            },
            (Active, Event::DefeatFinalBoss) => Some((Completed, base_lives, vec![])),
            (Active, Event::SessionEnd(c)) => Some((Abandoned(c), base_lives, vec![])),
            (Completed, Event::SessionEnd(_)) => {
                Some((RewardStoredInLobby, base_lives, vec![MoveRewardToLobbyClaimChest]))
            }
            (Completed, Event::ClaimReward(ClaimPath::FloorChest))
            | (RewardStoredInLobby, Event::ClaimReward(ClaimPath::LobbyClaimChest)) => {
                Some((RewardPaid, base_lives, vec![PayReward]))
            }
            (FailedByDeath, Event::Reclaim) => {
                Some((Reclaimed, base_lives, vec![ChargeReclaimPenalty, ReturnReclaimedItems]))
            }
            _ => None,
        }
    }
    fn every_run() -> Vec<Drive> {
        let mut runs = vec![];
        for mode in [LifeMode::Hardcore, LifeMode::Softcore, LifeMode::Easy] {
            let mut d = Drive::start(mode);
            if mode != LifeMode::Hardcore {
                d.go(Event::Death);
            }
            d.go(Event::SessionEnd(SessionEndCause::Disconnect));
            runs.push(d);
            let mut d = Drive::start(mode);
            d.to_final();
            d.go(Event::ClaimReward(ClaimPath::FloorChest));
            d.go(Event::SessionEnd(SessionEndCause::Leave));
            runs.push(d);
            let mut d = Drive::start(mode);
            d.to_final();
            d.go(Event::SessionEnd(SessionEndCause::Logout));
            d.go(Event::ClaimReward(ClaimPath::LobbyClaimChest));
            runs.push(d);
            for cause in [SessionEndCause::Leave, SessionEndCause::Crash] {
                let mut d = Drive::start(mode);
                d.go(Event::SessionEnd(cause));
                runs.push(d);
            }
            if mode != LifeMode::Easy {
                let mut d = Drive::start(mode);
                while d.attempt.state() == Active {
                    d.go(Event::Death);
                }
                d.go(Event::Reclaim);
                runs.push(d);
            }
        }
        runs
    }

    #[test]
    fn start_commits_carry_the_start_and_no_request_identity() {
        for mode in [LifeMode::Hardcore, LifeMode::Softcore, LifeMode::Easy] {
            let d = Drive::start(mode);
            let c = &d.start;
            assert_eq!(c.base(), None);
            assert_eq!(c.step(), CommitStep::Start { transition: TransitionId(1) });
            assert_eq!(c.result(), &d.attempt.record());
            assert_eq!(c.intents(), [CreateAttempt, ChargeEntryCost]);
            assert_eq!(c.request_bytes(), None);
            c.validate(&d.plan).unwrap();
        }
    }

    #[test]
    fn every_model_commit_matches_the_oracle_and_validates() {
        let mut durable = 0;
        let mut non_durable = 0;
        for d in every_run() {
            let mut base = d.start.result().clone();
            for (event, commit) in &d.commits {
                match commit {
                    None => {
                        non_durable += 1;
                    }
                    Some(c) => {
                        durable += 1;
                        let (state, lives, intents) =
                            oracle(base.mode, base.state, base.lives, *event).expect("oracle step");
                        assert_eq!(c.base(), Some(&base));
                        assert_eq!((c.result().state, c.result().lives), (state, lives));
                        assert_eq!(c.intents(), intents.as_slice(), "{event:?}");
                        assert!(matches!(c.step(), CommitStep::Transition(t) if t.event == *event));
                        c.validate(&d.plan).unwrap();
                        let bytes = c.request_bytes().unwrap().unwrap();
                        let CommitStep::Transition(t) = c.step() else { unreachable!() };
                        assert_eq!(bytes, canonical_request(&t, Some(&base), c.result(), c.intents()).unwrap());
                        base = *c.result();
                    }
                }
            }
        }
        assert!(durable > 20 && non_durable > 10, "{durable} {non_durable}");
    }

    #[test]
    fn non_durable_proposals_carry_no_commit() {
        let mut d = Drive::start(LifeMode::Easy);
        assert_eq!(d.go(Event::ClearRoom(RoomIndex(0))).commit, None);
        assert_eq!(d.go(Event::EnterRoom(RoomIndex(1))).commit, None);
        assert_eq!(d.go(Event::Death).commit, None, "easy death");
        d.go(Event::ClearRoom(RoomIndex(1)));
        d.go(Event::EnterRoom(RoomIndex(2)));
        assert!(d.go(Event::DefeatFinalBoss).commit.is_some());
        assert!(d.go(Event::ClaimReward(ClaimPath::FloorChest)).commit.is_some());
        let cleanup = d.go(Event::SessionEnd(SessionEndCause::Leave));
        assert_eq!((cleanup.attempt, cleanup.commit), (None, None));
    }

    #[test]
    fn recovery_commits_reuse_the_session_end_proposal() {
        let plan = plan();
        let active = Drive::start(LifeMode::Softcore).attempt;
        let p = active.propose_recovery(&plan, TransitionId(5)).unwrap();
        let c = p.commit.unwrap();
        assert_eq!(c.base(), Some(&active.record()));
        assert_eq!(c.result().state, Abandoned(SessionEndCause::Crash));
        assert!(c.intents().is_empty());
        c.validate(&plan).unwrap();
        let mut d = Drive::start(LifeMode::Hardcore);
        d.to_final();
        let p = d.attempt.propose_recovery(&plan, TransitionId(50)).unwrap();
        assert_eq!(p.commit.unwrap().intents(), [MoveRewardToLobbyClaimChest]);
    }

    #[test]
    fn validate_refuses_tampered_commits() {
        let plan = plan();
        let mut d = Drive::start(LifeMode::Softcore);
        let good = d.go(Event::Death).commit.unwrap();
        let base = *good.base().unwrap();
        let result = *good.result();
        let CommitStep::Transition(t) = good.step() else { unreachable!() };
        let make = |base: Option<AttemptRecord>, t: Transition, result: AttemptRecord| {
            DurableCommit::new(base, CommitStep::Transition(t), result, vec![])
        };
        assert_eq!(make(Some(base), t, AttemptRecord { last: TransitionId(99), ..result }).validate(&plan), Err(StepError::ResultOrder));
        assert_eq!(
            make(Some(AttemptRecord { last: t.id, ..base }), t, result).validate(&plan),
            Err(StepError::BaseOrder)
        );
        assert_eq!(
            make(Some(base), t, AttemptRecord { instance: InstanceId(1), ..result }).validate(&plan),
            Err(StepError::InvariantChanged)
        );
        assert_eq!(
            make(Some(base), t, AttemptRecord { lives: Lives::Limited(3), ..result }).validate(&plan),
            Err(StepError::NoChange)
        );
        let up = make(Some(AttemptRecord { lives: Lives::Limited(1), ..base }), t, result);
        assert_eq!(up.validate(&plan), Err(StepError::LivesIncreased));
        assert_eq!(make(None, t, result).validate(&plan), Err(StepError::BaseShape));
        assert_eq!(
            make(Some(base), Transition { attempt: AttemptId(8), ..t }, result).validate(&plan),
            Err(StepError::AttemptMismatch)
        );
        assert_eq!(
            make(Some(base), t, AttemptRecord { lives: Lives::Unlimited, ..result }).validate(&plan),
            Err(StepError::Restore(RestoreError::InconsistentLives))
        );
        let other = FloorPlan::new(FloorId(2), plan.rooms.clone()).unwrap();
        assert_eq!(good.validate(&other), Err(StepError::Restore(RestoreError::PlanMismatch)));
        let start = DurableCommit::new(Some(base), CommitStep::Start { transition: TransitionId(1) }, base, vec![]);
        assert_eq!(start.validate(&plan), Err(StepError::BaseShape));
        let start = DurableCommit::new(None, CommitStep::Start { transition: result.last }, result, vec![]);
        assert_eq!(start.validate(&plan), Err(StepError::StartShape), "lives below full");
        let many = DurableCommit::new(Some(base), CommitStep::Transition(t), result, vec![PayReward; MAX_INTENTS + 1]);
        assert_eq!(many.validate(&plan), Err(StepError::TooManyIntents));
    }
}
