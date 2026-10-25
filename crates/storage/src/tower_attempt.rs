//! Tower attempt records and their transition ledger (#119 slice B, V14).
//!
//! The only write input is a sealed [`DurableCommit`] built by the pure attempt model. One
//! `BEGIN IMMEDIATE` transaction validates it, checks the exact retry identity, checks the
//! stored base, lets a [`ValueWriter`] apply value effects, then writes the attempt row and
//! the ledger row. Any refusal or failure rolls everything back.
//!
//! Two write paths share one commit core: [`RecordOnly`] (#137) and the snapshot path below
//! (C1). Both refuse every money or item intent and every start. Committing value effects and
//! actual starts still needs typed external value policies (C3) and a per-character
//! pending-write/acknowledgement barrier in the runtime (C2). Neither exists here, and no
//! runtime code calls this module yet.
//!
//! **Snapshot commits (#119 C1).** [`Accounts::commit_tower_attempt_with_snapshot`] carries the
//! character's post-transition absolute [`crate::accounts::Save`] as a sealed, prevalidated
//! [`SnapshotRequest`] and writes it with the unchanged save transaction body
//! (`write_snapshot`) inside the same commit transaction. The snapshot's exact bytes join the
//! retry identity, and its owner is checked before that identity, so a different or mismatched
//! `Save` can never be acknowledged as `AlreadyApplied`. Value intents and starts stay refused;
//! a later stale absolute save overwriting committed state is the runtime barrier's job (C2).
//!
//! The writer, outcome and error types are public so a caller of
//! [`Accounts::commit_tower_attempt`] can name them. This compiles outside the crate and
//! touches no database:
//!
//! ```
//! use gielinor_storage::tower_attempt::{CommitError, Outcome, RecordOnly, ValueWriter};
//! fn record_only() -> Box<dyn ValueWriter> {
//!     Box::new(RecordOnly)
//! }
//! fn classify(result: Result<Outcome, CommitError>) -> &'static str {
//!     match result {
//!         Ok(Outcome::Committed) => "committed",
//!         Ok(Outcome::AlreadyApplied) => "already applied",
//!         Err(CommitError::StartUnsupported) => "start refused",
//!         Err(_) => "refused",
//!     }
//! }
//! let _writer = record_only();
//! assert_eq!(classify(Ok(Outcome::AlreadyApplied)), "already applied");
//! assert_eq!(classify(Err(CommitError::StartUnsupported)), "start refused");
//! ```
use crate::accounts::Accounts;
use crate::sqlite::{Database, Error, Value};
use gielinor_world::tower_run::attempt::{
    AttemptId, AttemptRecord, AttemptState, CommitStep, DurableCommit, DurableIntent, FloorPlan,
    RecordError, StepError,
};

mod snapshot;
pub use snapshot::{SnapshotRequest, MAX_SNAPSHOT_BYTES, SNAPSHOT_TAG, SNAPSHOT_VERSION};

/// Separates a snapshot-carrying request from a record-only request for the same commit.
const SNAPSHOT_MARKER: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Committed,
    /// The exact same request was committed before; nothing was written or re-applied.
    AlreadyApplied,
}

#[derive(Debug)]
pub enum CommitError {
    UnknownCharacter,
    /// An attempt or transition id is zero or above `i64::MAX`.
    IdOutOfRange,
    Invalid(StepError),
    Record(RecordError),
    /// A stored row disagrees with its own record (attempt id, state or last transition).
    Corrupt(&'static str),
    LiveAttemptExists,
    AttemptIdNotNewer,
    /// Starts can't be committed until the snapshot writer and ack barrier exist.
    StartUnsupported,
    /// The same (attempt, transition) was committed with a different request.
    Conflict,
    /// The stored attempt isn't the commit's base.
    Stale,
    /// The value writer refused this intent.
    Unsupported(DurableIntent),
    /// The snapshot failed the save path's static checks.
    InvalidSnapshot(String),
    /// The snapshot belongs to another character or account.
    SnapshotMismatch,
    /// The encoded snapshot exceeds [`MAX_SNAPSHOT_BYTES`].
    SnapshotTooLarge,
    Database(Error),
}
impl From<Error> for CommitError {
    fn from(e: Error) -> Self {
        Self::Database(e)
    }
}

/// Applies a commit's value intents inside the commit transaction.
pub trait ValueWriter {
    fn apply(
        &mut self,
        db: &Database,
        character: i64,
        intents: &[DurableIntent],
    ) -> Result<(), CommitError>;
}

/// The only writer in this slice. It accepts exactly `[]` or
/// `[MoveRewardToLobbyClaimChest]` (the claim right is the stored attempt state; reward
/// contents are OPEN) and refuses everything else, `CreateAttempt` and repeats included.
pub struct RecordOnly;
impl ValueWriter for RecordOnly {
    fn apply(&mut self, _: &Database, _: i64, intents: &[DurableIntent]) -> Result<(), CommitError> {
        const CLAIM: DurableIntent = DurableIntent::MoveRewardToLobbyClaimChest;
        match intents {
            [] | [CLAIM] => Ok(()),
            _ => Err(CommitError::Unsupported(
                intents.iter().copied().find(|i| *i != CLAIM).unwrap_or(CLAIM),
            )),
        }
    }
}

fn sql_id(id: u64) -> Result<i64, CommitError> {
    i64::try_from(id)
        .ok()
        .filter(|id| *id >= 1)
        .ok_or(CommitError::IdOutOfRange)
}
/// The index's state codes; a test pins them to the record encoding.
fn state_code(state: AttemptState) -> i64 {
    match state {
        AttemptState::Active => 0,
        AttemptState::Completed => 1,
        AttemptState::RewardPaid => 2,
        AttemptState::RewardStoredInLobby => 3,
        AttemptState::Abandoned(_) => 4,
        AttemptState::FailedByDeath => 5,
        AttemptState::Reclaimed => 6,
    }
}
fn decode_row(row: &[Value]) -> Result<AttemptRecord, CommitError> {
    let [Value::Integer(attempt), Value::Blob(bytes), Value::Integer(state), Value::Integer(last)] =
        row
    else {
        return Err(CommitError::Corrupt("unexpected tower attempt row shape"));
    };
    let record = AttemptRecord::decode(bytes).map_err(CommitError::Record)?;
    if sql_id(record.id.0)? != *attempt {
        return Err(CommitError::Corrupt("attempt id differs from its record"));
    }
    if state_code(record.state) != *state {
        return Err(CommitError::Corrupt("state differs from its record"));
    }
    if sql_id(record.last.0)? != *last {
        return Err(CommitError::Corrupt("last transition differs from its record"));
    }
    Ok(record)
}
fn load(db: &Database, character: i64, attempt: i64) -> Result<Option<AttemptRecord>, CommitError> {
    let rows = db.query(
        "SELECT attempt_id,record,state,last_transition FROM tower_attempts WHERE character_id=? AND attempt_id=?",
        &[Value::Integer(character), Value::Integer(attempt)],
        1,
    )?;
    rows.first().map(|row| decode_row(row)).transpose()
}
/// What a commit writes besides the attempt and ledger rows.
enum Payload<'a> {
    /// The #137 value-writer path; the request bytes are exactly the record-only form.
    Record(&'a mut dyn ValueWriter),
    /// The absolute character snapshot; its bytes join the request identity.
    Snapshot(&'a SnapshotRequest),
}
/// The single commit flow for both payloads.
fn commit_core(
    db: &Database,
    character: i64,
    plan: &FloorPlan,
    commit: &DurableCommit,
    mut payload: Payload<'_>,
) -> Result<Outcome, CommitError> {
    if db
        .query("SELECT id FROM characters WHERE id=?", &[Value::Integer(character)], 1)?
        .is_empty()
    {
        return Err(CommitError::UnknownCharacter);
    }
    // The snapshot's owner is checked before anything else, the retry identity included,
    // because an AlreadyApplied answer would never reach a later check.
    if let Payload::Snapshot(snapshot) = &payload {
        if snapshot.character() != character
            || db
                .query(
                    "SELECT id FROM characters WHERE id=? AND account_id=?",
                    &[Value::Integer(character), Value::Integer(snapshot.account())],
                    1,
                )?
                .is_empty()
        {
            return Err(CommitError::SnapshotMismatch);
        }
    }
    commit.validate(plan).map_err(CommitError::Invalid)?;
    let attempt = sql_id(commit.attempt().0)?;
    let transition = sql_id(commit.transition_id().0)?;
    let result = commit.result().encode().map_err(CommitError::Record)?;
    let c = Value::Integer(character);
    let a = Value::Integer(attempt);
    match commit.step() {
        CommitStep::Start { .. } => {
            let live = db.query(
                "SELECT attempt_id FROM tower_attempts WHERE character_id=? AND state IN (0,1)",
                &[c.clone()],
                1,
            )?;
            if !live.is_empty() {
                return Err(CommitError::LiveAttemptExists);
            }
            let newest = db.query(
                "SELECT MAX(attempt_id) FROM tower_attempts WHERE character_id=?",
                &[c],
                1,
            )?;
            if let Some(Value::Integer(newest)) = newest.first().and_then(|r| r.first()) {
                if attempt <= *newest {
                    return Err(CommitError::AttemptIdNotNewer);
                }
            }
            // No actual starts in this slice: no writer and no snapshot is written for one, so
            // no side effect can escape the rollback.
            Err(CommitError::StartUnsupported)
        }
        CommitStep::Transition(_) => {
            if let Payload::Snapshot(_) = &payload {
                // C1 carries no value policy: the snapshot path accepts the RecordOnly set only.
                RecordOnly.apply(db, character, commit.intents())?;
            }
            let mut request = character.to_le_bytes().to_vec();
            request.extend(
                commit
                    .request_bytes()
                    .ok_or(CommitError::Corrupt("transition without request"))?
                    .map_err(CommitError::Record)?,
            );
            if let Payload::Snapshot(snapshot) = &payload {
                let len = u32::try_from(snapshot.bytes().len())
                    .map_err(|_| CommitError::SnapshotTooLarge)?;
                request.push(SNAPSHOT_MARKER);
                request.extend_from_slice(&len.to_le_bytes());
                request.extend_from_slice(snapshot.bytes());
            }
            let t = Value::Integer(transition);
            let prior = db.query(
                "SELECT request FROM tower_attempt_transitions WHERE character_id=? AND attempt_id=? AND transition_id=?",
                &[c.clone(), a.clone(), t.clone()],
                1,
            )?;
            if let Some(row) = prior.first() {
                return match row.first() {
                    Some(Value::Blob(stored)) if *stored == request => Ok(Outcome::AlreadyApplied),
                    _ => Err(CommitError::Conflict),
                };
            }
            let base = commit.base().ok_or(CommitError::Stale)?;
            if load(db, character, attempt)?.as_ref() != Some(base) {
                return Err(CommitError::Stale);
            }
            match &mut payload {
                Payload::Record(value) => value.apply(db, character, commit.intents())?,
                Payload::Snapshot(snapshot) => {
                    crate::accounts::write_snapshot(db, snapshot.save(), false)?
                }
            }
            db.execute(
                "UPDATE tower_attempts SET record=?,state=?,last_transition=? WHERE character_id=? AND attempt_id=?",
                &[
                    Value::Blob(result),
                    Value::Integer(state_code(commit.result().state)),
                    t.clone(),
                    c.clone(),
                    a.clone(),
                ],
            )?;
            db.execute(
                "INSERT INTO tower_attempt_transitions(character_id,attempt_id,transition_id,request) VALUES(?,?,?,?)",
                &[c, a, t, Value::Blob(request)],
            )?;
            Ok(Outcome::Committed)
        }
    }
}

impl Accounts {
    pub fn load_tower_attempt(
        &mut self,
        character: i64,
        attempt: AttemptId,
    ) -> Result<Option<AttemptRecord>, CommitError> {
        let attempt = sql_id(attempt.0)?;
        load(self.tower_database(), character, attempt)
    }
    pub fn latest_tower_attempt(&mut self, character: i64) -> Result<Option<AttemptRecord>, CommitError> {
        let rows = self.tower_database().query(
            "SELECT attempt_id,record,state,last_transition FROM tower_attempts WHERE character_id=? ORDER BY attempt_id DESC LIMIT 1",
            &[Value::Integer(character)],
            1,
        )?;
        rows.first().map(|row| decode_row(row)).transpose()
    }
    /// One transaction: validate, exact retry, base check, value writer, attempt and ledger rows.
    ///
    /// A retry must resubmit the same commit verbatim: the request bytes bind the full
    /// transition (id, attempt, instance, event and payload), the base, the result and the
    /// ordered intents, so any change, a different instance included, is a `Conflict`. Whether
    /// an in-run event came from the attempt's live instance stays the pure model's check;
    /// lobby steps (claims, reclaim, session end) aren't instance-bound there.
    pub fn commit_tower_attempt(
        &mut self,
        character: i64,
        plan: &FloorPlan,
        commit: &DurableCommit,
        value: &mut dyn ValueWriter,
    ) -> Result<Outcome, CommitError> {
        self.run_commit(character, plan, commit, Payload::Record(value))
    }
    /// The same transaction with the character's post-transition absolute snapshot written by
    /// the unchanged save body. Order: owner check, validate, RecordOnly intent set, exact retry
    /// identity (commit plus snapshot bytes), base check, snapshot, attempt and ledger rows.
    pub fn commit_tower_attempt_with_snapshot(
        &mut self,
        character: i64,
        plan: &FloorPlan,
        commit: &DurableCommit,
        snapshot: &SnapshotRequest,
    ) -> Result<Outcome, CommitError> {
        self.run_commit(character, plan, commit, Payload::Snapshot(snapshot))
    }
    fn run_commit(
        &mut self,
        character: i64,
        plan: &FloorPlan,
        commit: &DurableCommit,
        payload: Payload<'_>,
    ) -> Result<Outcome, CommitError> {
        let mut refused = None;
        let result = self.tower_database().transaction(|db| {
            commit_core(db, character, plan, commit, payload).map_err(|e| {
                refused = Some(e);
                Error::from("Tower attempt commit refused".to_string())
            })
        });
        match (result, refused) {
            (Ok(outcome), _) => Ok(outcome),
            (Err(_), Some(refusal)) => Err(refusal),
            (Err(e), None) => Err(CommitError::Database(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::tower_run::attempt::*;

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
    fn accounts() -> (Accounts, i64) {
        let mut a = Accounts::from_database(Database::memory().unwrap());
        a.initialize().unwrap();
        let (_, character) = a.create("tower attempt", "fixture-hash-only", 1).unwrap();
        (a, character)
    }
    /// Test-only: stores an attempt row directly, as a later starting slice would.
    fn insert_fixture(a: &mut Accounts, character: i64, r: &AttemptRecord) {
        a.tower_database()
            .execute(
                "INSERT INTO tower_attempts(character_id,attempt_id,record,state,last_transition) VALUES(?,?,?,?,?)",
                &[
                    Value::Integer(character),
                    Value::Integer(r.id.0 as i64),
                    Value::Blob(r.encode().unwrap()),
                    Value::Integer(state_code(r.state)),
                    Value::Integer(r.last.0 as i64),
                ],
            )
            .unwrap();
    }
    fn count(a: &mut Accounts, table: &str) -> i64 {
        match &a.tower_database().query(&format!("SELECT COUNT(*) FROM {table}"), &[], 1).unwrap()[0][0] {
            Value::Integer(n) => *n,
            _ => panic!(),
        }
    }
    /// Drives the pure model.
    struct Drive {
        attempt: Attempt,
        progress: Option<RunProgress>,
        next: u64,
    }
    impl Drive {
        fn start(mode: LifeMode, id: u64) -> (Self, DurableCommit) {
            let p = propose_start(None, &plan(), AttemptId(id), mode, InstanceId(90), TransitionId(1))
                .unwrap();
            (
                Self {
                    attempt: p.attempt.unwrap(),
                    progress: p.progress,
                    next: 2,
                },
                p.commit.unwrap(),
            )
        }
        fn go(&mut self, event: Event) -> Proposal {
            let t = Transition {
                id: TransitionId(self.next),
                attempt: self.attempt.id(),
                instance: self.attempt.instance(),
                event,
            };
            self.next += 1;
            let p = self.attempt.propose(&plan(), self.progress.as_ref(), t).unwrap();
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
    /// Counts calls and otherwise behaves like `RecordOnly`, or accepts everything.
    struct Counting {
        calls: usize,
        permissive: bool,
    }
    impl ValueWriter for Counting {
        fn apply(&mut self, db: &Database, c: i64, intents: &[DurableIntent]) -> Result<(), CommitError> {
            self.calls += 1;
            if self.permissive {
                Ok(())
            } else {
                RecordOnly.apply(db, c, intents)
            }
        }
    }

    #[test]
    fn v14_tables_have_no_room_columns_and_version_is_fourteen() {
        let (mut a, _) = accounts();
        let columns = |a: &mut Accounts, t: &str| {
            a.tower_database()
                .query(&format!("PRAGMA table_info({t})"), &[], 16)
                .unwrap()
                .into_iter()
                .map(|r| match &r[1] {
                    Value::Text(s) => s.clone(),
                    _ => panic!(),
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(columns(&mut a, "tower_attempts"), ["character_id", "attempt_id", "record", "state", "last_transition"]);
        assert_eq!(columns(&mut a, "tower_attempt_transitions"), ["character_id", "attempt_id", "transition_id", "request"]);
        let max = a.tower_database().query("SELECT MAX(version) FROM _rust_migrations", &[], 1).unwrap();
        assert!(matches!(max[0][0], Value::Integer(14)));
    }

    #[test]
    fn state_codes_match_the_record_encoding() {
        let mut records = vec![];
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        records.push(d.attempt.record());
        for end in [Event::SessionEnd(SessionEndCause::Crash), Event::Death] {
            let (mut d, _) = Drive::start(LifeMode::Hardcore, 9);
            d.go(end);
            records.push(d.attempt.record());
            if d.attempt.state() == AttemptState::FailedByDeath {
                d.go(Event::Reclaim);
                records.push(d.attempt.record());
            }
        }
        let (mut d, _) = Drive::start(LifeMode::Easy, 9);
        d.to_final();
        records.push(d.attempt.record());
        d.go(Event::SessionEnd(SessionEndCause::Leave));
        records.push(d.attempt.record());
        d.go(Event::ClaimReward(ClaimPath::LobbyClaimChest));
        records.push(d.attempt.record());
        assert_eq!(records.len(), 7);
        for r in records {
            // Byte 42 is the state code in the pinned slice A layout.
            assert_eq!(state_code(r.state), i64::from(r.encode().unwrap()[42]), "{:?}", r.state);
        }
    }

    #[test]
    fn starts_are_refused_and_write_nothing() {
        let (mut a, c) = accounts();
        let (_, start) = Drive::start(LifeMode::Softcore, 9);
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &start, &mut RecordOnly),
            Err(CommitError::StartUnsupported)
        ));
        // Even a writer that would accept everything is never called for a valid start.
        let mut permissive = Counting { calls: 0, permissive: true };
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &start, &mut permissive),
            Err(CommitError::StartUnsupported)
        ));
        assert_eq!(permissive.calls, 0);
        assert_eq!((count(&mut a, "tower_attempts"), count(&mut a, "tower_attempt_transitions")), (0, 0));
        // The one-live and id-order guards run first.
        let (live, _) = Drive::start(LifeMode::Softcore, 9);
        insert_fixture(&mut a, c, &live.attempt.record());
        let (_, newer) = Drive::start(LifeMode::Softcore, 10);
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &newer, &mut permissive),
            Err(CommitError::LiveAttemptExists)
        ));
        let (mut ended, _) = Drive::start(LifeMode::Softcore, 11);
        ended.go(Event::SessionEnd(SessionEndCause::Leave));
        let (mut b, cb) = accounts();
        insert_fixture(&mut b, cb, &ended.attempt.record());
        let (_, older) = Drive::start(LifeMode::Softcore, 11);
        assert!(matches!(
            b.commit_tower_attempt(cb, &plan(), &older, &mut permissive),
            Err(CommitError::AttemptIdNotNewer)
        ));
        assert_eq!(count(&mut b, "tower_attempts"), 1);
        assert_eq!(permissive.calls, 0, "guards refuse before any writer call");
    }

    #[test]
    fn a_retry_with_a_different_instance_conflicts() {
        let (mut a, c) = accounts();
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        let base = d.attempt.clone();
        insert_fixture(&mut a, c, &base.record());
        let end = |instance| {
            base.propose(
                &plan(),
                None,
                Transition {
                    id: TransitionId(5),
                    attempt: base.id(),
                    instance,
                    event: Event::SessionEnd(SessionEndCause::Disconnect),
                },
            )
            .unwrap()
            .commit
            .unwrap()
        };
        let first = end(base.instance());
        assert_eq!(a.commit_tower_attempt(c, &plan(), &first, &mut RecordOnly).unwrap(), Outcome::Committed);
        let after = a.load_tower_attempt(c, AttemptId(9)).unwrap();
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &end(InstanceId(91)), &mut RecordOnly),
            Err(CommitError::Conflict)
        ));
        assert_eq!(a.commit_tower_attempt(c, &plan(), &first, &mut RecordOnly).unwrap(), Outcome::AlreadyApplied);
        assert_eq!(a.load_tower_attempt(c, AttemptId(9)).unwrap(), after);
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn recovery_commits_once_and_an_exact_retry_is_already_applied() {
        let (mut a, c) = accounts();
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        insert_fixture(&mut a, c, &d.attempt.record());
        let restored = Attempt::restore(a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap(), &plan()).unwrap();
        let id = restored.recovery_transition_id().unwrap();
        let commit = restored.propose_recovery(&plan(), id).unwrap().commit.unwrap();
        let mut writer = Counting { calls: 0, permissive: false };
        assert_eq!(a.commit_tower_attempt(c, &plan(), &commit, &mut writer).unwrap(), Outcome::Committed);
        assert_eq!(writer.calls, 1);
        let stored = a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap();
        assert_eq!(stored.state, AttemptState::Abandoned(SessionEndCause::Crash));
        assert_eq!(stored, *commit.result());
        for _ in 0..2 {
            assert_eq!(a.commit_tower_attempt(c, &plan(), &commit, &mut writer).unwrap(), Outcome::AlreadyApplied);
        }
        assert_eq!(writer.calls, 1, "the writer is never called again");
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
        assert_eq!(a.latest_tower_attempt(c).unwrap(), Some(stored));
    }

    #[test]
    fn completed_recovery_stores_the_claim_and_payment_is_refused() {
        let (mut a, c) = accounts();
        let (mut d, _) = Drive::start(LifeMode::Hardcore, 9);
        d.to_final();
        insert_fixture(&mut a, c, &d.attempt.record());
        let commit = d.attempt.propose_recovery(&plan(), TransitionId(50)).unwrap().commit.unwrap();
        assert_eq!(commit.intents(), [DurableIntent::MoveRewardToLobbyClaimChest]);
        assert_eq!(a.commit_tower_attempt(c, &plan(), &commit, &mut RecordOnly).unwrap(), Outcome::Committed);
        let stored = Attempt::restore(a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap(), &plan()).unwrap();
        assert_eq!(stored.state(), AttemptState::RewardStoredInLobby);
        let claim = stored
            .propose(
                &plan(),
                None,
                Transition {
                    id: TransitionId(51),
                    attempt: stored.id(),
                    instance: stored.instance(),
                    event: Event::ClaimReward(ClaimPath::LobbyClaimChest),
                },
            )
            .unwrap()
            .commit
            .unwrap();
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &claim, &mut RecordOnly),
            Err(CommitError::Unsupported(DurableIntent::PayReward))
        ));
        assert_eq!(a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap(), stored.record());
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn every_value_intent_is_refused_and_rolls_back() {
        // Last-life death, floor-chest claim and reclaim, each from a stored base.
        let cases: Vec<(Attempt, Proposal, DurableIntent)> = {
            let mut v = vec![];
            let (mut d, _) = Drive::start(LifeMode::Softcore, 9);
            d.go(Event::Death);
            d.go(Event::Death);
            let base = d.attempt.clone();
            v.push((base, d.go(Event::Death), DurableIntent::MoveCarriedItemsToReclaimChest));
            let (mut d, _) = Drive::start(LifeMode::Easy, 9);
            d.to_final();
            let base = d.attempt.clone();
            v.push((base, d.go(Event::ClaimReward(ClaimPath::FloorChest)), DurableIntent::PayReward));
            let (mut d, _) = Drive::start(LifeMode::Hardcore, 9);
            d.go(Event::Death);
            let base = d.attempt.clone();
            v.push((base, d.go(Event::Reclaim), DurableIntent::ChargeReclaimPenalty));
            v
        };
        for (base, proposal, refused) in cases {
            let (mut a, c) = accounts();
            insert_fixture(&mut a, c, &base.record());
            let commit = proposal.commit.unwrap();
            match a.commit_tower_attempt(c, &plan(), &commit, &mut RecordOnly) {
                Err(CommitError::Unsupported(i)) => assert_eq!(i, refused),
                other => panic!("{other:?}"),
            }
            assert_eq!(a.load_tower_attempt(c, base.id()).unwrap().unwrap(), base.record());
            assert_eq!(count(&mut a, "tower_attempt_transitions"), 0);
        }
    }

    #[test]
    fn conflicting_retry_and_stale_base_change_nothing() {
        let (mut a, c) = accounts();
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        let base = d.attempt.clone();
        insert_fixture(&mut a, c, &base.record());
        let end = |cause, id| {
            base.propose(
                &plan(),
                None,
                Transition {
                    id: TransitionId(id),
                    attempt: base.id(),
                    instance: base.instance(),
                    event: Event::SessionEnd(cause),
                },
            )
            .unwrap()
            .commit
            .unwrap()
        };
        let crash = end(SessionEndCause::Crash, 5);
        assert_eq!(a.commit_tower_attempt(c, &plan(), &crash, &mut RecordOnly).unwrap(), Outcome::Committed);
        let after = a.load_tower_attempt(c, AttemptId(9)).unwrap();
        let mut writer = Counting { calls: 0, permissive: false };
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &end(SessionEndCause::Leave, 5), &mut writer),
            Err(CommitError::Conflict)
        ));
        assert!(matches!(
            a.commit_tower_attempt(c, &plan(), &end(SessionEndCause::Leave, 6), &mut writer),
            Err(CommitError::Stale)
        ));
        assert_eq!(writer.calls, 0);
        assert_eq!(a.load_tower_attempt(c, AttemptId(9)).unwrap(), after);
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn a_failure_inside_the_transaction_rolls_back_record_and_ledger() {
        let (mut a, c) = accounts();
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        insert_fixture(&mut a, c, &d.attempt.record());
        let commit = d.attempt.propose_recovery(&plan(), TransitionId(5)).unwrap().commit.unwrap();
        a.tower_database()
            .migration_script("CREATE TRIGGER reject_ledger BEFORE INSERT ON tower_attempt_transitions BEGIN SELECT RAISE(ABORT,'disposable ledger failure'); END;")
            .unwrap();
        assert!(matches!(a.commit_tower_attempt(c, &plan(), &commit, &mut RecordOnly), Err(CommitError::Database(_))));
        assert_eq!(a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap(), d.attempt.record());
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 0);
        a.tower_database().execute("DROP TRIGGER reject_ledger", &[]).unwrap();
        assert_eq!(a.commit_tower_attempt(c, &plan(), &commit, &mut RecordOnly).unwrap(), Outcome::Committed);
    }

    #[test]
    fn load_refuses_malformed_or_mismatched_rows() {
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        let good = d.attempt.record();
        let bytes = good.encode().unwrap();
        let mut bad_tag = bytes.clone();
        bad_tag[0] = b'X';
        for (attempt, record, state, last, expect_record_error) in [
            (9i64, bad_tag, 0i64, 1i64, true),
            (10, bytes.clone(), 0, 1, false),
            (9, bytes.clone(), 4, 1, false),
            (9, bytes.clone(), 0, 2, false),
        ] {
            let (mut a, c) = accounts();
            a.tower_database()
                .execute(
                    "INSERT INTO tower_attempts(character_id,attempt_id,record,state,last_transition) VALUES(?,?,?,?,?)",
                    &[Value::Integer(c), Value::Integer(attempt), Value::Blob(record), Value::Integer(state), Value::Integer(last)],
                )
                .unwrap();
            let loaded = a.load_tower_attempt(c, AttemptId(attempt as u64));
            match (loaded, expect_record_error) {
                (Err(CommitError::Record(RecordError::WrongTag)), true) => {}
                (Err(CommitError::Corrupt(_)), false) => {}
                (other, _) => panic!("{other:?}"),
            }
            assert!(a.latest_tower_attempt(c).is_err());
        }
    }

    #[test]
    fn latest_returns_the_highest_of_several_terminal_attempts() {
        let (mut a, c) = accounts();
        let mut ended = vec![];
        for (id, cause) in [(9, SessionEndCause::Leave), (11, SessionEndCause::Crash), (10, SessionEndCause::Logout)] {
            let (mut d, _) = Drive::start(LifeMode::Softcore, id);
            d.go(Event::SessionEnd(cause));
            insert_fixture(&mut a, c, &d.attempt.record());
            ended.push(d.attempt.record());
        }
        assert_eq!(count(&mut a, "tower_attempts"), 3);
        assert_eq!(a.latest_tower_attempt(c).unwrap(), Some(ended[1]));
        // Reading it leaves every row unchanged.
        for r in &ended {
            assert_eq!(a.load_tower_attempt(c, r.id).unwrap().as_ref(), Some(r));
        }
        assert_eq!(a.latest_tower_attempt(c).unwrap(), Some(ended[1]));
    }

    #[test]
    fn record_only_accepts_exactly_nothing_or_one_claim_move() {
        let db = Database::memory().unwrap();
        use DurableIntent::*;
        assert!(RecordOnly.apply(&db, 1, &[]).is_ok());
        assert!(RecordOnly.apply(&db, 1, &[MoveRewardToLobbyClaimChest]).is_ok());
        for (intents, refused) in [
            (vec![MoveRewardToLobbyClaimChest, MoveRewardToLobbyClaimChest], MoveRewardToLobbyClaimChest),
            (vec![MoveRewardToLobbyClaimChest, PayReward], PayReward),
            (vec![PayReward, MoveRewardToLobbyClaimChest], PayReward),
            (vec![CreateAttempt], CreateAttempt),
            (vec![ChargeEntryCost], ChargeEntryCost),
            (vec![MoveCarriedItemsToReclaimChest], MoveCarriedItemsToReclaimChest),
            (vec![ReturnReclaimedItems], ReturnReclaimedItems),
            (vec![ChargeReclaimPenalty], ChargeReclaimPenalty),
        ] {
            match RecordOnly.apply(&db, 1, &intents) {
                Err(CommitError::Unsupported(i)) => assert_eq!(i, refused, "{intents:?}"),
                other => panic!("{intents:?}: {other:?}"),
            }
        }
    }

    #[test]
    fn the_one_live_index_refuses_a_second_live_row() {
        let (mut a, c) = accounts();
        let (first, _) = Drive::start(LifeMode::Softcore, 9);
        insert_fixture(&mut a, c, &first.attempt.record());
        let (second, _) = Drive::start(LifeMode::Softcore, 10);
        let r = second.attempt.record();
        assert!(a
            .tower_database()
            .execute(
                "INSERT INTO tower_attempts(character_id,attempt_id,record,state,last_transition) VALUES(?,?,?,?,?)",
                &[Value::Integer(c), Value::Integer(10), Value::Blob(r.encode().unwrap()), Value::Integer(0), Value::Integer(1)],
            )
            .is_err());
        // A terminal row alongside the live one is allowed.
        let (mut ended, _) = Drive::start(LifeMode::Softcore, 11);
        ended.go(Event::SessionEnd(SessionEndCause::Leave));
        insert_fixture(&mut a, c, &ended.attempt.record());
        assert_eq!(count(&mut a, "tower_attempts"), 2);
    }

    #[test]
    fn ids_above_i64_and_unknown_characters_are_refused() {
        let (mut a, c) = accounts();
        let (_, huge) = Drive::start(LifeMode::Softcore, u64::MAX);
        assert!(matches!(a.commit_tower_attempt(c, &plan(), &huge, &mut RecordOnly), Err(CommitError::IdOutOfRange)));
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        insert_fixture(&mut a, c, &d.attempt.record());
        let commit = d.attempt.propose_recovery(&plan(), TransitionId(u64::MAX)).unwrap().commit.unwrap();
        assert!(matches!(a.commit_tower_attempt(c, &plan(), &commit, &mut RecordOnly), Err(CommitError::IdOutOfRange)));
        assert!(matches!(a.load_tower_attempt(c, AttemptId(u64::MAX)), Err(CommitError::IdOutOfRange)));
        let ok = d.attempt.propose_recovery(&plan(), TransitionId(5)).unwrap().commit.unwrap();
        assert!(matches!(a.commit_tower_attempt(c + 1000, &plan(), &ok, &mut RecordOnly), Err(CommitError::UnknownCharacter)));
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 0);
    }

    /// Removes exactly the files this test created, after every handle is closed.
    struct OwnedFiles(Vec<std::path::PathBuf>);
    impl Drop for OwnedFiles {
        fn drop(&mut self) {
            for path in &self.0 {
                if path.exists() {
                    std::fs::remove_file(path).unwrap();
                }
            }
        }
    }

    #[test]
    fn a_committed_recovery_survives_closing_and_reopening_the_database_file() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gielinor-tower-attempt-reopen-{}-{stamp}.sqlite",
            std::process::id()
        ));
        let journal = std::path::PathBuf::from(format!("{}-journal", path.display()));
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap(),
        );
        // Registered only after create_new succeeded, so a refused (pre-existing) file is never
        // removed; declared before any database handle so it is dropped after all of them.
        let _owned = OwnedFiles(vec![path.clone(), journal]);
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        let base = d.attempt.clone();
        let recovery = base.propose_recovery(&plan(), TransitionId(5)).unwrap().commit.unwrap();
        let changed = base
            .propose(
                &plan(),
                None,
                Transition {
                    id: TransitionId(5),
                    attempt: base.id(),
                    instance: base.instance(),
                    event: Event::SessionEnd(SessionEndCause::Leave),
                },
            )
            .unwrap()
            .commit
            .unwrap();
        let character;
        {
            let mut a = Accounts::from_database(Database::open(&path, false).unwrap());
            a.initialize().unwrap();
            character = a.create("tower reopen", "fixture-hash-only", 1).unwrap().1;
            insert_fixture(&mut a, character, &base.record());
            let mut writer = Counting { calls: 0, permissive: false };
            assert_eq!(
                a.commit_tower_attempt(character, &plan(), &recovery, &mut writer).unwrap(),
                Outcome::Committed
            );
            assert_eq!(writer.calls, 1);
        } // the connection is closed here
        let mut a = Accounts::from_database(Database::open(&path, false).unwrap());
        a.initialize().unwrap();
        let stored = a.load_tower_attempt(character, AttemptId(9)).unwrap().unwrap();
        assert_eq!(stored, *recovery.result());
        assert_eq!(stored.state, AttemptState::Abandoned(SessionEndCause::Crash));
        assert_eq!(a.latest_tower_attempt(character).unwrap(), Some(stored));
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
        let mut fresh = Counting { calls: 0, permissive: false };
        assert_eq!(
            a.commit_tower_attempt(character, &plan(), &recovery, &mut fresh).unwrap(),
            Outcome::AlreadyApplied
        );
        assert!(matches!(
            a.commit_tower_attempt(character, &plan(), &changed, &mut fresh),
            Err(CommitError::Conflict)
        ));
        assert_eq!(fresh.calls, 0, "no writer call after reopening");
        assert_eq!(a.load_tower_attempt(character, AttemptId(9)).unwrap(), Some(stored));
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
        drop(a);
    }

    #[test]
    fn a_v13_database_upgrades_to_v14_keeping_tower_run() {
        let (mut a, c) = accounts();
        let db = a.tower_database();
        db.execute("DROP TABLE tower_attempt_transitions", &[]).unwrap();
        db.execute("DROP TABLE tower_attempts", &[]).unwrap();
        db.execute("DELETE FROM _rust_migrations WHERE version=14", &[]).unwrap();
        db.execute(
            "INSERT INTO tower_run(character_id,run_id,services,flags,clears) VALUES(?,1,7,3,1)",
            &[Value::Integer(c)],
        )
        .unwrap();
        let before = db.query("SELECT * FROM tower_run", &[], 4).unwrap();
        a.initialize().unwrap();
        let db = a.tower_database();
        assert_eq!(db.query("SELECT * FROM tower_run", &[], 4).unwrap(), before);
        assert!(matches!(db.query("SELECT MAX(version) FROM _rust_migrations", &[], 1).unwrap()[0][0], Value::Integer(14)));
        assert_eq!(count(&mut a, "tower_attempts"), 0);
    }

    // ---- #119 C1: snapshot-carrying commits ----
    use crate::accounts::{Save, Stat};
    use gielinor_world::{inventory::Item, CoordGrid};
    use std::collections::BTreeMap;

    fn save_of(a: &Accounts, name: &str) -> Save {
        let c = a.load(name, 1).unwrap().unwrap();
        Save {
            tower_diary: c.tower_diary,
            chemistry: c.chemistry,
            tower_run: c.tower_run,
            tower_arrival: c.tower_arrival,
            account_id: c.account_id,
            character_id: c.character_id,
            display_name: c.display_name,
            known_device: c.known_device,
            modlevel: c.modlevel,
            coords: c.coords,
            persistent_varps_json: c.varps_json,
            last_login: c.last_login,
            run_energy: c.run_energy,
            xp_rate_hundreds: c.xp_rate_hundreds,
            stats: c.stats,
            inventories: c.inventories,
        }
    }
    /// A post-transition snapshot that differs from the stored character.
    fn changed_save(a: &Accounts) -> Save {
        let mut s = save_of(a, "tower attempt");
        s.coords = CoordGrid::new(3224, 3219, 0).unwrap();
        s.run_energy = 4321;
        s.stats.insert(3, Stat { visible: 12, base: 12, fine_xp: 15000 });
        s.inventories.insert(93, BTreeMap::from([(0, Item { id: 995, count: 25, vars: 0 })]));
        s
    }
    fn character_state(a: &mut Accounts, c: i64) -> Vec<Vec<Value>> {
        let db = a.tower_database();
        let mut rows = db
            .query(
                "SELECT x,z,level,varps,last_login,run_energy,xp_rate_in_hundreds,last_logout FROM characters WHERE id=?",
                &[Value::Integer(c)],
                1,
            )
            .unwrap();
        rows.extend(
            db.query(
                "SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=? ORDER BY stat_id",
                &[Value::Integer(c)],
                64,
            )
            .unwrap(),
        );
        rows.extend(
            db.query(
                "SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventories i JOIN inventory_objs o ON o.inventories_id=i.id WHERE i.character_id=? ORDER BY i.inv_type,o.slot",
                &[Value::Integer(c)],
                4096,
            )
            .unwrap(),
        );
        for table in ["tower_run", "tower_arrival", "chemistry_state", "tower_diary"] {
            rows.extend(
                db.query(&format!("SELECT * FROM {table} WHERE character_id=?"), &[Value::Integer(c)], 1)
                    .unwrap(),
            );
        }
        rows.extend(
            db.query(
                "SELECT a.display_name,a.known_device,a.modlevel FROM accounts a JOIN characters c ON c.account_id=a.id WHERE c.id=?",
                &[Value::Integer(c)],
                1,
            )
            .unwrap(),
        );
        rows
    }
    /// An active fixture attempt and its recovery commit (SessionEnd(Crash), id 5).
    fn active_with_recovery(a: &mut Accounts, c: i64) -> (Attempt, DurableCommit) {
        let (d, _) = Drive::start(LifeMode::Softcore, 9);
        insert_fixture(a, c, &d.attempt.record());
        let commit = d.attempt.propose_recovery(&plan(), TransitionId(5)).unwrap().commit.unwrap();
        (d.attempt, commit)
    }

    #[test]
    fn a_snapshot_commit_writes_the_character_attempt_and_ledger_together() {
        let (mut a, c) = accounts();
        let (_, commit) = active_with_recovery(&mut a, c);
        let snapshot = SnapshotRequest::new(changed_save(&a)).unwrap();
        assert_eq!(
            a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot).unwrap(),
            Outcome::Committed
        );
        let loaded = save_of(&a, "tower attempt");
        assert_eq!((loaded.coords, loaded.run_energy), (CoordGrid::new(3224, 3219, 0).unwrap(), 4321));
        assert_eq!(loaded.inventories.get(&93).and_then(|i| i.get(&0)), Some(&Item { id: 995, count: 25, vars: 0 }));
        assert_eq!(
            a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap().state,
            AttemptState::Abandoned(SessionEndCause::Crash)
        );
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn an_exact_snapshot_retry_is_already_applied_and_rewrites_nothing() {
        let (mut a, c) = accounts();
        let (_, commit) = active_with_recovery(&mut a, c);
        let snapshot = SnapshotRequest::new(changed_save(&a)).unwrap();
        a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot).unwrap();
        // Change the stored character behind the ledger's back: a retry must not restore it.
        a.tower_database()
            .execute("UPDATE characters SET run_energy=1 WHERE id=?", &[Value::Integer(c)])
            .unwrap();
        let before = character_state(&mut a, c);
        for _ in 0..2 {
            assert_eq!(
                a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot.clone()).unwrap(),
                Outcome::AlreadyApplied
            );
        }
        assert_eq!(character_state(&mut a, c), before);
        assert_eq!(save_of(&a, "tower attempt").run_energy, 1);
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn the_same_commit_with_a_changed_save_conflicts_and_writes_nothing() {
        let (mut a, c) = accounts();
        let (_, commit) = active_with_recovery(&mut a, c);
        let first = changed_save(&a);
        a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &SnapshotRequest::new(first.clone()).unwrap())
            .unwrap();
        let before = character_state(&mut a, c);
        let attempt = a.load_tower_attempt(c, AttemptId(9)).unwrap();
        let variants: Vec<(&str, Box<dyn Fn(&mut Save)>)> = vec![
            ("coords", Box::new(|s: &mut Save| s.coords = CoordGrid::new(3225, 3219, 0).unwrap())),
            ("run energy", Box::new(|s: &mut Save| s.run_energy = 4320)),
            ("item count", Box::new(|s: &mut Save| s.inventories.get_mut(&93).unwrap().get_mut(&0).unwrap().count = 26)),
            ("stat xp", Box::new(|s: &mut Save| s.stats.get_mut(&3).unwrap().fine_xp = 15001)),
            ("varps", Box::new(|s: &mut Save| s.persistent_varps_json = "{\"1\":1}".into())),
            ("display name", Box::new(|s: &mut Save| s.display_name = Some("Other".into()))),
            ("run flag", Box::new(|s: &mut Save| s.tower_run.first_reward_paid = !s.tower_run.first_reward_paid)),
            ("arrival flag", Box::new(|s: &mut Save| s.tower_arrival.kit_given = !s.tower_arrival.kit_given)),
        ];
        for (name, change) in variants {
            let mut save = first.clone();
            change(&mut save);
            let snapshot = SnapshotRequest::new(save).unwrap();
            assert!(
                matches!(a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot), Err(CommitError::Conflict)),
                "{name}"
            );
            assert_eq!(character_state(&mut a, c), before, "{name}");
            assert_eq!(a.load_tower_attempt(c, AttemptId(9)).unwrap(), attempt, "{name}");
        }
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn a_mismatched_snapshot_is_refused_before_the_retry_identity() {
        let (mut a, c) = accounts();
        let (_, other) = a.create("other character", "fixture-hash-only", 1).unwrap();
        let (_, commit) = active_with_recovery(&mut a, c);
        let good = SnapshotRequest::new(changed_save(&a)).unwrap();
        a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &good).unwrap();
        let before = character_state(&mut a, c);
        // Another character's save for this character's existing ledger row.
        let foreign = SnapshotRequest::new(save_of(&a, "other character")).unwrap();
        assert_eq!(foreign.character(), other);
        assert!(matches!(
            a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &foreign),
            Err(CommitError::SnapshotMismatch)
        ));
        // This character's id under a different account.
        let mut wrong_account = changed_save(&a);
        wrong_account.account_id += 1000;
        assert!(matches!(
            a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &SnapshotRequest::new(wrong_account).unwrap()),
            Err(CommitError::SnapshotMismatch)
        ));
        assert_eq!(character_state(&mut a, c), before);
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
    }

    #[test]
    fn record_only_and_snapshot_requests_for_one_commit_conflict_both_ways() {
        let (mut a, c) = accounts();
        let (_, commit) = active_with_recovery(&mut a, c);
        a.commit_tower_attempt(c, &plan(), &commit, &mut RecordOnly).unwrap();
        let snapshot = SnapshotRequest::new(changed_save(&a)).unwrap();
        assert!(matches!(
            a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot),
            Err(CommitError::Conflict)
        ));
        let (mut b, cb) = accounts();
        let (_, commit) = active_with_recovery(&mut b, cb);
        let snapshot = SnapshotRequest::new(changed_save(&b)).unwrap();
        b.commit_tower_attempt_with_snapshot(cb, &plan(), &commit, &snapshot).unwrap();
        assert!(matches!(b.commit_tower_attempt(cb, &plan(), &commit, &mut RecordOnly), Err(CommitError::Conflict)));
        assert_eq!((count(&mut a, "tower_attempt_transitions"), count(&mut b, "tower_attempt_transitions")), (1, 1));
    }

    #[test]
    fn the_snapshot_path_still_refuses_value_intents_and_starts() {
        let (mut a, c) = accounts();
        let (mut d, start) = Drive::start(LifeMode::Hardcore, 9);
        let snapshot = SnapshotRequest::new(changed_save(&a)).unwrap();
        let before = character_state(&mut a, c);
        assert!(matches!(
            a.commit_tower_attempt_with_snapshot(c, &plan(), &start, &snapshot),
            Err(CommitError::StartUnsupported)
        ));
        insert_fixture(&mut a, c, &d.attempt.record());
        let death = d.go(Event::Death).commit.unwrap();
        assert!(matches!(
            a.commit_tower_attempt_with_snapshot(c, &plan(), &death, &snapshot),
            Err(CommitError::Unsupported(DurableIntent::MoveCarriedItemsToReclaimChest))
        ));
        assert_eq!(character_state(&mut a, c), before);
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 0);
    }

    #[test]
    fn a_failure_on_either_side_rolls_back_snapshot_attempt_and_ledger() {
        for trigger in [
            "CREATE TRIGGER fail_c1 BEFORE INSERT ON tower_attempt_transitions BEGIN SELECT RAISE(ABORT,'ledger'); END;",
            "CREATE TRIGGER fail_c1 BEFORE INSERT ON inventory_objs BEGIN SELECT RAISE(ABORT,'snapshot'); END;",
        ] {
            let (mut a, c) = accounts();
            let (base, commit) = active_with_recovery(&mut a, c);
            let snapshot = SnapshotRequest::new(changed_save(&a)).unwrap();
            let before = character_state(&mut a, c);
            a.tower_database().migration_script(trigger).unwrap();
            assert!(matches!(
                a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot),
                Err(CommitError::Database(_))
            ));
            assert_eq!(character_state(&mut a, c), before, "{trigger}");
            assert_eq!(a.load_tower_attempt(c, AttemptId(9)).unwrap(), Some(base.record()));
            assert_eq!(count(&mut a, "tower_attempt_transitions"), 0);
            a.tower_database().execute("DROP TRIGGER fail_c1", &[]).unwrap();
            assert_eq!(
                a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &snapshot).unwrap(),
                Outcome::Committed
            );
        }
    }

    #[test]
    fn write_snapshot_matches_the_existing_checkpoint_save() {
        let (mut a, c) = accounts();
        let (mut b, cb) = accounts();
        assert_eq!(c, cb);
        let save = changed_save(&a);
        a.save_checkpoint(&save).unwrap();
        b.tower_database()
            .transaction(|db| crate::accounts::write_snapshot(db, &save, false))
            .unwrap();
        assert_eq!(character_state(&mut a, c), character_state(&mut b, cb));
    }

    #[test]
    fn a_snapshot_commit_survives_closing_and_reopening_the_database_file() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gielinor-tower-snapshot-reopen-{}-{stamp}.sqlite",
            std::process::id()
        ));
        let journal = std::path::PathBuf::from(format!("{}-journal", path.display()));
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap(),
        );
        // Registered only after create_new succeeded; dropped after every database handle.
        let _owned = OwnedFiles(vec![path.clone(), journal]);
        let (character, commit, snapshot);
        {
            let mut a = Accounts::from_database(Database::open(&path, false).unwrap());
            a.initialize().unwrap();
            character = a.create("tower attempt", "fixture-hash-only", 1).unwrap().1;
            let (_, c) = active_with_recovery(&mut a, character);
            commit = c;
            snapshot = SnapshotRequest::new(changed_save(&a)).unwrap();
            assert_eq!(
                a.commit_tower_attempt_with_snapshot(character, &plan(), &commit, &snapshot).unwrap(),
                Outcome::Committed
            );
        }
        let mut a = Accounts::from_database(Database::open(&path, false).unwrap());
        a.initialize().unwrap();
        let loaded = save_of(&a, "tower attempt");
        assert_eq!((loaded.coords, loaded.run_energy), (CoordGrid::new(3224, 3219, 0).unwrap(), 4321));
        assert_eq!(
            a.load_tower_attempt(character, AttemptId(9)).unwrap().unwrap().state,
            AttemptState::Abandoned(SessionEndCause::Crash)
        );
        assert_eq!(
            a.commit_tower_attempt_with_snapshot(character, &plan(), &commit, &snapshot).unwrap(),
            Outcome::AlreadyApplied
        );
        assert_eq!(count(&mut a, "tower_attempt_transitions"), 1);
        drop(a);
    }

    /// Documents the hazard C2 must close: storage can't tell an older absolute save from a
    /// newer one, so a stale ordinary save after a committed snapshot overwrites it.
    #[test]
    fn a_later_stale_absolute_save_still_overwrites_committed_state() {
        let (mut a, c) = accounts();
        let stale = save_of(&a, "tower attempt");
        let (_, commit) = active_with_recovery(&mut a, c);
        a.commit_tower_attempt_with_snapshot(c, &plan(), &commit, &SnapshotRequest::new(changed_save(&a)).unwrap())
            .unwrap();
        assert_eq!(save_of(&a, "tower attempt").run_energy, 4321);
        a.save_checkpoint(&stale).unwrap();
        assert_eq!(save_of(&a, "tower attempt").run_energy, stale.run_energy);
        // The attempt record itself is untouched by the ordinary save.
        assert_eq!(
            a.load_tower_attempt(c, AttemptId(9)).unwrap().unwrap().state,
            AttemptState::Abandoned(SessionEndCause::Crash)
        );
    }
}
