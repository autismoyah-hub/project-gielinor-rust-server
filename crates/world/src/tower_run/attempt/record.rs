//! Versioned attempt record, its canonical bytes, validated restore and restart recovery.
//! Pure data and validation only: nothing here is persisted, and no durability is claimed.
//!
//! The record holds the durable [`Attempt`] fields and nothing from [`RunProgress`]: no
//! current room and no cleared rooms. [`Attempt::restore`] returns an `Attempt` alone, so
//! a restored Active attempt rejects every in-room event (`NoInstance`) and can only end.
//!
//! **Identifier boundary.** The pure model ([`propose_start`]) doesn't check ids. The record
//! boundary is deliberately stricter: an attempt id or transition id of zero can't be
//! encoded, decoded or restored ([`RecordError::ZeroId`]). Model output with nonzero ids
//! round-trips exactly; an attempt started with id 0 is valid in the model but not
//! recordable, and [`AttemptRecord::encode`] refuses it, so nothing encodes successfully and
//! then fails to restore. Instance ids may be any value.
//!
//! **Recovery ids.** [`Attempt::recovery_transition_id`] is `last + 1` with checked overflow
//! (`None` at `u64::MAX`). Nothing allocates ids globally: a caller that retries a recovery
//! after an unknown outcome must reuse the same id, so the retry is the same request.
use super::*;

/// Domain tags and versions. Changing any byte layout needs a new version.
pub const RECORD_TAG: [u8; 4] = *b"TWAR";
pub const RECORD_VERSION: u16 = 1;
pub const REQUEST_TAG: [u8; 4] = *b"TWRQ";
pub const REQUEST_VERSION: u16 = 1;
/// Exact encoded record length (tag 4, version 2, fields 46).
pub const RECORD_LEN: usize = 52;
/// Upper bound on intents in one request; no current transition proposes more than 2.
pub const MAX_INTENTS: usize = 16;
/// Encoded transition: id 8, attempt 8, instance 8, event code 1, payload 2.
pub const TRANSITION_LEN: usize = 27;
const LIVES_UNLIMITED: u8 = 255;
const NO_CAUSE: u8 = 255;

/// Plain copy of an attempt's durable fields. Public fields mean any value can be
/// supplied, so encoding validates and is fallible.
///
/// Encode and decode validate structure only. A structurally valid record may still be
/// semantically impossible (for example easy FailedByDeath, hardcore `Limited(3)`, or a plan
/// that no longer matches); only [`Attempt::restore`] validates plan binding and
/// lives/state consistency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttemptRecord {
    pub version: u16,
    pub id: AttemptId,
    pub floor: FloorId,
    pub plan: PlanKey,
    pub mode: LifeMode,
    pub lives: Lives,
    pub instance: InstanceId,
    pub state: AttemptState,
    pub last: TransitionId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// Fewer than [`RECORD_LEN`] bytes.
    Truncated,
    /// More than [`RECORD_LEN`] bytes.
    Trailing,
    WrongTag,
    UnknownVersion,
    ZeroId,
    /// `Limited(n)` with n above the largest approved life count (3).
    UnsupportedLives,
    UnknownCode,
    /// A cause byte on a state other than Abandoned, or none on Abandoned.
    CauseMismatch,
    /// `floor` differs from `plan.floor`.
    FloorMismatch,
    TooManyIntents,
    /// The transition's attempt id differs from a record's id.
    AttemptMismatch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreError {
    Record(RecordError),
    /// The record's plan identity isn't the supplied plan. What the game does with an
    /// attempt whose floor definition changed is OPEN; this only refuses.
    PlanMismatch,
    /// Lives don't fit the mode and state (see [`Attempt::restore`]).
    InconsistentLives,
}

fn mode_code(mode: LifeMode) -> u8 {
    match mode {
        LifeMode::Hardcore => 0,
        LifeMode::Softcore => 1,
        LifeMode::Easy => 2,
    }
}
fn cause_code(cause: SessionEndCause) -> u8 {
    match cause {
        SessionEndCause::Leave => 0,
        SessionEndCause::Logout => 1,
        SessionEndCause::Disconnect => 2,
        SessionEndCause::Crash => 3,
    }
}
fn state_codes(state: AttemptState) -> (u8, u8) {
    match state {
        AttemptState::Active => (0, NO_CAUSE),
        AttemptState::Completed => (1, NO_CAUSE),
        AttemptState::RewardPaid => (2, NO_CAUSE),
        AttemptState::RewardStoredInLobby => (3, NO_CAUSE),
        AttemptState::Abandoned(cause) => (4, cause_code(cause)),
        AttemptState::FailedByDeath => (5, NO_CAUSE),
        AttemptState::Reclaimed => (6, NO_CAUSE),
    }
}
pub fn intent_code(intent: DurableIntent) -> u8 {
    match intent {
        DurableIntent::CreateAttempt => 0,
        DurableIntent::ChargeEntryCost => 1,
        DurableIntent::MoveCarriedItemsToReclaimChest => 2,
        DurableIntent::MoveRewardToLobbyClaimChest => 3,
        DurableIntent::PayReward => 4,
        DurableIntent::ReturnReclaimedItems => 5,
        DurableIntent::ChargeReclaimPenalty => 6,
    }
}

impl AttemptRecord {
    /// Structural checks shared by encode and decode, so decode accepts exactly the
    /// images of accepted records (the codec is injective on what it accepts).
    fn check(&self) -> Result<(), RecordError> {
        if self.version != RECORD_VERSION {
            return Err(RecordError::UnknownVersion);
        }
        if self.id.0 == 0 || self.last.0 == 0 {
            return Err(RecordError::ZeroId);
        }
        if let Lives::Limited(n) = self.lives {
            if n > SOFTCORE_LIVES {
                return Err(RecordError::UnsupportedLives);
            }
        }
        if self.floor != self.plan.floor {
            return Err(RecordError::FloorMismatch);
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>, RecordError> {
        self.check()?;
        let mut out = Vec::with_capacity(RECORD_LEN);
        out.extend_from_slice(&RECORD_TAG);
        out.extend_from_slice(&self.version.to_le_bytes());
        out.extend_from_slice(&self.id.0.to_le_bytes());
        out.extend_from_slice(&self.floor.0.to_le_bytes());
        out.extend_from_slice(&self.plan.floor.0.to_le_bytes());
        out.extend_from_slice(&self.plan.rooms.to_le_bytes());
        out.extend_from_slice(&self.plan.digest.to_le_bytes());
        out.push(mode_code(self.mode));
        out.push(match self.lives {
            Lives::Limited(n) => n,
            Lives::Unlimited => LIVES_UNLIMITED,
        });
        out.extend_from_slice(&self.instance.0.to_le_bytes());
        let (state, cause) = state_codes(self.state);
        out.push(state);
        out.push(cause);
        out.extend_from_slice(&self.last.0.to_le_bytes());
        debug_assert_eq!(out.len(), RECORD_LEN);
        Ok(out)
    }
    /// Reads exactly [`RECORD_LEN`] bytes; refuses shorter and longer input before parsing
    /// and allocates nothing.
    pub fn decode(bytes: &[u8]) -> Result<Self, RecordError> {
        if bytes.len() < RECORD_LEN {
            return Err(RecordError::Truncated);
        }
        if bytes.len() > RECORD_LEN {
            return Err(RecordError::Trailing);
        }
        if bytes[0..4] != RECORD_TAG {
            return Err(RecordError::WrongTag);
        }
        let u16_at = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        let u32_at = |at: usize| {
            u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
        };
        let u64_at = |at: usize| {
            let mut a = [0u8; 8];
            a.copy_from_slice(&bytes[at..at + 8]);
            u64::from_le_bytes(a)
        };
        // Offsets: tag 0, version 4, id 6, floor 14, plan floor 18, rooms 22, digest 24,
        // mode 32, lives 33, instance 34, state 42, cause 43, last 44..52.
        let version = u16_at(4);
        if version != RECORD_VERSION {
            return Err(RecordError::UnknownVersion);
        }
        let id = AttemptId(u64_at(6));
        let floor = FloorId(u32_at(14));
        let plan_floor = FloorId(u32_at(18));
        let rooms = u16_at(22);
        let digest = u64_at(24);
        let mode = match bytes[32] {
            0 => LifeMode::Hardcore,
            1 => LifeMode::Softcore,
            2 => LifeMode::Easy,
            _ => return Err(RecordError::UnknownCode),
        };
        let lives = match bytes[33] {
            LIVES_UNLIMITED => Lives::Unlimited,
            n if n <= SOFTCORE_LIVES => Lives::Limited(n),
            _ => return Err(RecordError::UnsupportedLives),
        };
        let instance = InstanceId(u64_at(34));
        let (state, cause) = (bytes[42], bytes[43]);
        let last = TransitionId(u64_at(44));
        let state = match (state, cause) {
            (4, c) => AttemptState::Abandoned(match c {
                0 => SessionEndCause::Leave,
                1 => SessionEndCause::Logout,
                2 => SessionEndCause::Disconnect,
                3 => SessionEndCause::Crash,
                NO_CAUSE => return Err(RecordError::CauseMismatch),
                _ => return Err(RecordError::UnknownCode),
            }),
            (s, NO_CAUSE) => match s {
                0 => AttemptState::Active,
                1 => AttemptState::Completed,
                2 => AttemptState::RewardPaid,
                3 => AttemptState::RewardStoredInLobby,
                5 => AttemptState::FailedByDeath,
                6 => AttemptState::Reclaimed,
                _ => return Err(RecordError::UnknownCode),
            },
            (0..=6, _) => return Err(RecordError::CauseMismatch),
            _ => return Err(RecordError::UnknownCode),
        };
        let record = Self {
            version,
            id,
            floor,
            plan: PlanKey {
                floor: plan_floor,
                rooms,
                digest,
            },
            mode,
            lives,
            instance,
            state,
            last,
        };
        record.check()?;
        Ok(record)
    }
}

fn event_codes(event: Event) -> (u8, u16) {
    match event {
        Event::EnterRoom(room) => (0, room.0),
        Event::ClearRoom(room) => (1, room.0),
        Event::DefeatFinalBoss => (2, 0),
        Event::Death => (3, 0),
        Event::SessionEnd(cause) => (4, u16::from(cause_code(cause))),
        Event::ClaimReward(ClaimPath::FloorChest) => (5, 0),
        Event::ClaimReward(ClaimPath::LobbyClaimChest) => (5, 1),
        Event::Reclaim => (6, 0),
    }
}

/// The world half of a durable transition's retry identity (slice B prepends the character).
/// Layout: request tag and version; the full transition (id, attempt, instance, event code,
/// event payload: fixed [`TRANSITION_LEN`]); a base presence byte, then a length-prefixed base
/// record if present; the length-prefixed result record; the intent count, then the ordered
/// intent codes. Every field is fixed-width or explicitly counted, so distinct requests can't
/// share bytes. Refuses zero ids, invalid records, an attempt-id mismatch and too many intents.
pub fn canonical_request(
    transition: &Transition,
    base: Option<&AttemptRecord>,
    result: &AttemptRecord,
    intents: &[DurableIntent],
) -> Result<Vec<u8>, RecordError> {
    if transition.attempt.0 == 0 || transition.id.0 == 0 {
        return Err(RecordError::ZeroId);
    }
    if result.id != transition.attempt || base.is_some_and(|b| b.id != transition.attempt) {
        return Err(RecordError::AttemptMismatch);
    }
    if intents.len() > MAX_INTENTS {
        return Err(RecordError::TooManyIntents);
    }
    let record_len = u16::try_from(RECORD_LEN).map_err(|_| RecordError::Trailing)?;
    let count = u16::try_from(intents.len()).map_err(|_| RecordError::TooManyIntents)?;
    let mut out =
        Vec::with_capacity(6 + TRANSITION_LEN + 1 + 2 * (2 + RECORD_LEN) + 2 + intents.len());
    out.extend_from_slice(&REQUEST_TAG);
    out.extend_from_slice(&REQUEST_VERSION.to_le_bytes());
    out.extend_from_slice(&transition.id.0.to_le_bytes());
    out.extend_from_slice(&transition.attempt.0.to_le_bytes());
    out.extend_from_slice(&transition.instance.0.to_le_bytes());
    let (code, payload) = event_codes(transition.event);
    out.push(code);
    out.extend_from_slice(&payload.to_le_bytes());
    match base {
        None => out.push(0),
        Some(base) => {
            out.push(1);
            out.extend_from_slice(&record_len.to_le_bytes());
            out.extend_from_slice(&base.encode()?);
        }
    }
    out.extend_from_slice(&record_len.to_le_bytes());
    out.extend_from_slice(&result.encode()?);
    out.extend_from_slice(&count.to_le_bytes());
    out.extend(intents.iter().map(|i| intent_code(*i)));
    Ok(out)
}

impl Attempt {
    pub fn record(&self) -> AttemptRecord {
        AttemptRecord {
            version: RECORD_VERSION,
            id: self.id,
            floor: self.floor,
            plan: self.plan,
            mode: self.mode,
            lives: self.lives,
            instance: self.instance,
            state: self.state,
            last: self.last,
        }
    }
    /// Validates a record against the plan and returns the attempt only, never progress.
    ///
    /// This is the semantic validation of any public record, including ones that encode and
    /// decode successfully.
    ///
    /// Lives must fit the mode and state, as the pure transitions produce them:
    /// - hardcore: exactly `Limited(1)` until FailedByDeath/Reclaimed, then exactly `Limited(0)`;
    /// - softcore: `Limited(1..=3)` until FailedByDeath/Reclaimed, then exactly `Limited(0)`;
    /// - easy: always `Unlimited`; FailedByDeath and Reclaimed are refused.
    pub fn restore(record: AttemptRecord, plan: &FloorPlan) -> Result<Attempt, RestoreError> {
        record.check().map_err(RestoreError::Record)?;
        if record.plan != plan.key() || record.floor != plan.floor() {
            return Err(RestoreError::PlanMismatch);
        }
        let failed = matches!(
            record.state,
            AttemptState::FailedByDeath | AttemptState::Reclaimed
        );
        let lives_ok = match (record.mode, record.lives, failed) {
            (LifeMode::Hardcore, Lives::Limited(n), false) => n == HARDCORE_LIVES,
            (LifeMode::Softcore, Lives::Limited(n), false) => (1..=SOFTCORE_LIVES).contains(&n),
            (LifeMode::Hardcore | LifeMode::Softcore, Lives::Limited(0), true) => true,
            (LifeMode::Easy, Lives::Unlimited, false) => true,
            _ => false,
        };
        if !lives_ok {
            return Err(RestoreError::InconsistentLives);
        }
        Ok(Attempt {
            id: record.id,
            floor: record.floor,
            mode: record.mode,
            lives: record.lives,
            instance: record.instance,
            state: record.state,
            last: record.last,
            plan: record.plan,
        })
    }
    /// `last + 1`, or `None` when `last` is `u64::MAX`. A convenience, not an allocator:
    /// retries must reuse the id they first used.
    pub fn recovery_transition_id(&self) -> Option<TransitionId> {
        self.last.0.checked_add(1).map(TransitionId)
    }
    /// Restart recovery is the existing `SessionEnd(Crash)` with no progress: an Active
    /// attempt becomes Abandoned with no durable intents (no reclaim, no penalty); a
    /// Completed one keeps its reward via the lobby claim chest; RewardPaid is cleanup
    /// only; every other state is rejected as Terminal.
    pub fn propose_recovery(
        &self,
        plan: &FloorPlan,
        transition: TransitionId,
    ) -> Result<Proposal, Rejected> {
        self.propose(
            plan,
            None,
            Transition {
                id: transition,
                attempt: self.id,
                instance: self.instance,
                event: Event::SessionEnd(SessionEndCause::Crash),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AttemptState::*;

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
    /// Drives real transitions; returns the attempt (and progress) after `events`.
    struct Drive {
        plan: FloorPlan,
        attempt: Attempt,
        progress: Option<RunProgress>,
        next: u64,
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
            }
        }
        fn go(&mut self, event: Event) -> Result<Proposal, Rejected> {
            let t = Transition {
                id: TransitionId(self.next),
                attempt: self.attempt.id(),
                instance: self.attempt.instance(),
                event,
            };
            self.next += 1;
            let p = self.attempt.propose(&self.plan, self.progress.as_ref(), t)?;
            if let Some(a) = &p.attempt {
                self.attempt = a.clone();
            }
            self.progress = p.progress.clone();
            Ok(p)
        }
        fn to_final(&mut self) {
            self.go(Event::ClearRoom(RoomIndex(0))).unwrap();
            self.go(Event::EnterRoom(RoomIndex(1))).unwrap();
            self.go(Event::ClearRoom(RoomIndex(1))).unwrap();
            self.go(Event::EnterRoom(RoomIndex(2))).unwrap();
            self.go(Event::DefeatFinalBoss).unwrap();
        }
    }
    /// Every reachable (mode, state) produced by real transitions.
    fn reachable() -> Vec<Attempt> {
        let mut out = vec![];
        for mode in [LifeMode::Hardcore, LifeMode::Softcore, LifeMode::Easy] {
            let d = Drive::start(mode);
            out.push(d.attempt.clone());
            let mut d = Drive::start(mode);
            d.go(Event::SessionEnd(SessionEndCause::Disconnect)).unwrap();
            out.push(d.attempt.clone());
            let mut d = Drive::start(mode);
            d.to_final();
            out.push(d.attempt.clone());
            d.go(Event::ClaimReward(ClaimPath::FloorChest)).unwrap();
            out.push(d.attempt.clone());
            let mut d = Drive::start(mode);
            d.to_final();
            d.go(Event::SessionEnd(SessionEndCause::Logout)).unwrap();
            out.push(d.attempt.clone());
            if mode != LifeMode::Easy {
                let mut d = Drive::start(mode);
                while d.attempt.state() == Active {
                    d.go(Event::Death).unwrap();
                }
                out.push(d.attempt.clone());
                d.go(Event::Reclaim).unwrap();
                out.push(d.attempt.clone());
            }
        }
        let mut d = Drive::start(LifeMode::Softcore);
        d.go(Event::Death).unwrap();
        d.go(Event::Death).unwrap();
        out.push(d.attempt.clone());
        out
    }

    #[test]
    fn every_reachable_attempt_round_trips_through_bytes_and_restore() {
        let all = reachable();
        assert_eq!(all.len(), 3 * 5 + 2 * 2 + 1);
        for attempt in all {
            let bytes = attempt.record().encode().unwrap();
            assert_eq!(bytes.len(), RECORD_LEN);
            let record = AttemptRecord::decode(&bytes).unwrap();
            assert_eq!(record, attempt.record());
            assert_eq!(Attempt::restore(record, &plan()).unwrap(), attempt);
        }
    }

    #[test]
    fn golden_record_and_request_bytes_are_pinned() {
        let mut d = Drive::start(LifeMode::Softcore);
        d.go(Event::Death).unwrap();
        let record = d.attempt.record();
        let bytes = record.encode().unwrap();
        let key = plan().key();
        // The plan digest is pinned too, so a FloorPlan identity change is visible.
        assert_eq!(key.digest, 0xfdc3_b647_0c62_d7e5);
        let mut expected = b"TWAR".to_vec();
        expected.extend_from_slice(&1u16.to_le_bytes());
        expected.extend_from_slice(&9u64.to_le_bytes());
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.extend_from_slice(&1u32.to_le_bytes());
        expected.extend_from_slice(&3u16.to_le_bytes());
        expected.extend_from_slice(&0xfdc3_b647_0c62_d7e5u64.to_le_bytes());
        expected.extend_from_slice(&[1, 2]);
        expected.extend_from_slice(&90u64.to_le_bytes());
        expected.extend_from_slice(&[0, 255]);
        expected.extend_from_slice(&2u64.to_le_bytes());
        assert_eq!(bytes, expected);
        assert_eq!(record.encode().unwrap(), bytes, "deterministic");

        let t = Transition {
            id: TransitionId(3),
            attempt: AttemptId(9),
            instance: InstanceId(90),
            event: Event::SessionEnd(SessionEndCause::Disconnect),
        };
        let request = canonical_request(
            &t,
            Some(&record),
            &record,
            &[DurableIntent::CreateAttempt, DurableIntent::ChargeEntryCost],
        )
        .unwrap();
        let mut expected_request = b"TWRQ".to_vec();
        expected_request.extend_from_slice(&1u16.to_le_bytes());
        expected_request.extend_from_slice(&3u64.to_le_bytes());
        expected_request.extend_from_slice(&9u64.to_le_bytes());
        expected_request.extend_from_slice(&90u64.to_le_bytes());
        expected_request.extend_from_slice(&[4, 2, 0]);
        expected_request.push(1);
        expected_request.extend_from_slice(&52u16.to_le_bytes());
        expected_request.extend_from_slice(&bytes);
        expected_request.extend_from_slice(&52u16.to_le_bytes());
        expected_request.extend_from_slice(&bytes);
        expected_request.extend_from_slice(&2u16.to_le_bytes());
        expected_request.extend_from_slice(&[0, 1]);
        assert_eq!(request, expected_request);
        assert_eq!(request.len(), 6 + TRANSITION_LEN + 1 + 2 * (2 + RECORD_LEN) + 2 + 2);
        // A request is never decodable as a record.
        assert!(AttemptRecord::decode(&request).is_err());
        assert!(AttemptRecord::decode(&request[..RECORD_LEN]).is_err());
    }

    #[test]
    fn decode_is_strict_and_bounded() {
        for attempt in reachable() {
            let good = attempt.record().encode().unwrap();
            for len in 0..RECORD_LEN {
                assert_eq!(AttemptRecord::decode(&good[..len]), Err(RecordError::Truncated), "len {len}");
            }
            for extra in 0..=255u8 {
                let mut long = good.clone();
                long.push(extra);
                assert_eq!(AttemptRecord::decode(&long), Err(RecordError::Trailing));
            }
        }
        let good = Drive::start(LifeMode::Softcore).attempt.record().encode().unwrap();
        let set = |at: usize, v: u8| {
            let mut b = good.clone();
            b[at] = v;
            AttemptRecord::decode(&b)
        };
        for at in 0..4 {
            assert_eq!(set(at, good[at] ^ 0xff), Err(RecordError::WrongTag));
        }
        assert_eq!(set(4, 2), Err(RecordError::UnknownVersion));
        assert_eq!(set(5, 1), Err(RecordError::UnknownVersion));
        for v in 0..=255u8 {
            // mode
            assert_eq!(set(32, v).is_ok(), v <= 2, "mode {v}");
            // lives: only 0..=3 and 255 decode; restore then judges consistency
            let lives = set(33, v);
            if v <= 3 || v == 255 {
                assert!(lives.is_ok(), "lives {v}");
            } else {
                assert_eq!(lives, Err(RecordError::UnsupportedLives), "lives {v}");
            }
            // state with no cause: 0..=3, 5, 6 decode; 4 needs a cause; others unknown
            let state = set(42, v);
            match v {
                0..=3 | 5 | 6 => assert!(state.is_ok(), "state {v}"),
                4 => assert_eq!(state, Err(RecordError::CauseMismatch)),
                _ => assert_eq!(state, Err(RecordError::UnknownCode), "state {v}"),
            }
            // a cause byte on Active is refused unless it is the no-cause marker
            let cause = set(43, v);
            if v == 255 {
                assert!(cause.is_ok());
            } else {
                assert!(cause.is_err(), "cause {v} on Active");
            }
            // Abandoned with every cause byte
            let mut abandoned = good.clone();
            abandoned[42] = 4;
            abandoned[43] = v;
            let r = AttemptRecord::decode(&abandoned);
            match v {
                0..=3 => assert!(r.is_ok()),
                255 => assert_eq!(r, Err(RecordError::CauseMismatch)),
                _ => assert_eq!(r, Err(RecordError::UnknownCode)),
            }
        }
        let mut zero = good.clone();
        zero[6..14].fill(0);
        assert_eq!(AttemptRecord::decode(&zero), Err(RecordError::ZeroId));
        let mut zero_last = good.clone();
        zero_last[44..52].fill(0);
        assert_eq!(AttemptRecord::decode(&zero_last), Err(RecordError::ZeroId));
        // The redundant floor fields must agree, in either byte.
        assert_eq!(set(14, 2), Err(RecordError::FloorMismatch));
        assert_eq!(set(18, 2), Err(RecordError::FloorMismatch));
    }

    #[test]
    fn encode_refuses_values_that_would_collide_or_violate_the_boundary() {
        let record = Drive::start(LifeMode::Softcore).attempt.record();
        for n in [4u8, 200, 254, 255] {
            let bad = AttemptRecord {
                lives: Lives::Limited(n),
                ..record
            };
            assert_eq!(bad.encode(), Err(RecordError::UnsupportedLives), "Limited({n})");
        }
        assert_eq!(
            AttemptRecord { version: 2, ..record }.encode(),
            Err(RecordError::UnknownVersion)
        );
        assert_eq!(
            AttemptRecord { floor: FloorId(5), ..record }.encode(),
            Err(RecordError::FloorMismatch)
        );
        assert_eq!(
            AttemptRecord { last: TransitionId(0), ..record }.encode(),
            Err(RecordError::ZeroId)
        );
    }

    #[test]
    fn zero_ids_are_valid_in_the_model_but_not_recordable() {
        let plan = plan();
        let p = propose_start(None, &plan, AttemptId(0), LifeMode::Easy, InstanceId(0), TransitionId(0))
            .unwrap();
        let attempt = p.attempt.unwrap();
        assert_eq!(attempt.record().encode(), Err(RecordError::ZeroId));
        assert_eq!(
            Attempt::restore(attempt.record(), &plan),
            Err(RestoreError::Record(RecordError::ZeroId))
        );
        // Instance 0 alone is recordable.
        let p = propose_start(None, &plan, AttemptId(1), LifeMode::Easy, InstanceId(0), TransitionId(1))
            .unwrap();
        let ok = p.attempt.unwrap();
        assert_eq!(Attempt::restore(AttemptRecord::decode(&ok.record().encode().unwrap()).unwrap(), &plan).unwrap(), ok);
    }

    #[test]
    fn restore_refuses_other_or_reshaped_plans() {
        let record = Drive::start(LifeMode::Softcore).attempt.record();
        let base = plan().rooms;
        let mut longer = base.clone();
        longer.push(RoomSpec { kind: RoomKind::Room, requires: vec![] });
        let mut shorter = base.clone();
        shorter.remove(1);
        shorter[1].requires = vec![RoomIndex(0)];
        let mut kind = base.clone();
        kind[1].kind = RoomKind::Room;
        let mut reqs = base.clone();
        reqs[2].requires = vec![RoomIndex(0)];
        for (name, other) in [
            ("floor", FloorPlan::new(FloorId(2), base.clone()).unwrap()),
            ("longer", FloorPlan::new(FloorId(1), longer).unwrap()),
            ("shorter", FloorPlan::new(FloorId(1), shorter).unwrap()),
            ("kind", FloorPlan::new(FloorId(1), kind).unwrap()),
            ("requirements", FloorPlan::new(FloorId(1), reqs).unwrap()),
        ] {
            assert_eq!(Attempt::restore(record, &other), Err(RestoreError::PlanMismatch), "{name}");
        }
    }

    #[test]
    fn restore_enforces_lives_for_every_mode_and_state() {
        let template = Drive::start(LifeMode::Softcore).attempt.record();
        let states = [
            Active,
            Completed,
            RewardPaid,
            RewardStoredInLobby,
            Abandoned(SessionEndCause::Leave),
            FailedByDeath,
            Reclaimed,
        ];
        let lives = [
            Lives::Limited(0),
            Lives::Limited(1),
            Lives::Limited(2),
            Lives::Limited(3),
            Lives::Unlimited,
        ];
        let mut accepted = 0;
        for mode in [LifeMode::Hardcore, LifeMode::Softcore, LifeMode::Easy] {
            for state in states {
                for l in lives {
                    let failed = matches!(state, FailedByDeath | Reclaimed);
                    let expect = match (mode, l) {
                        (LifeMode::Hardcore, Lives::Limited(1)) => !failed,
                        (LifeMode::Softcore, Lives::Limited(1..=3)) => !failed,
                        (LifeMode::Hardcore | LifeMode::Softcore, Lives::Limited(0)) => failed,
                        (LifeMode::Easy, Lives::Unlimited) => !failed,
                        _ => false,
                    };
                    let record = AttemptRecord {
                        mode,
                        lives: l,
                        state,
                        ..template
                    };
                    let result = Attempt::restore(record, &plan());
                    if expect {
                        accepted += 1;
                        assert!(result.is_ok(), "{mode:?} {state:?} {l:?}");
                    } else {
                        assert_eq!(result, Err(RestoreError::InconsistentLives), "{mode:?} {state:?} {l:?}");
                    }
                }
            }
        }
        // hardcore 5 + 2, softcore 15 + 2, easy 5.
        assert_eq!(accepted, 29);
    }

    #[test]
    fn a_restored_active_attempt_cannot_resume_its_run() {
        let mut d = Drive::start(LifeMode::Softcore);
        d.go(Event::ClearRoom(RoomIndex(0))).unwrap();
        let restored = Attempt::restore(d.attempt.record(), &plan()).unwrap();
        for (i, event) in [
            Event::EnterRoom(RoomIndex(1)),
            Event::ClearRoom(RoomIndex(0)),
            Event::DefeatFinalBoss,
            Event::Death,
        ]
        .into_iter()
        .enumerate()
        {
            let t = Transition {
                id: TransitionId(100 + i as u64),
                attempt: restored.id(),
                instance: restored.instance(),
                event,
            };
            assert_eq!(restored.propose(&plan(), None, t), Err(Rejected::NoInstance), "{event:?}");
        }
    }

    #[test]
    fn recovery_follows_the_existing_session_end() {
        let plan = plan();
        // Active -> Abandoned(Crash), no durable intent.
        let active = Drive::start(LifeMode::Hardcore).attempt;
        let restored = Attempt::restore(active.record(), &plan).unwrap();
        let p = restored.propose_recovery(&plan, TransitionId(50)).unwrap();
        let after = p.attempt.unwrap();
        assert_eq!(after.state(), Abandoned(SessionEndCause::Crash));
        assert_eq!(after.lives(), Lives::Limited(1));
        assert!(p.durable.is_empty());
        assert_eq!(p.present, [PresentIntent::DiscardInstance, PresentIntent::ReturnToLobby]);
        assert_eq!(after.propose_recovery(&plan, TransitionId(51)), Err(Rejected::Terminal));
        // A repeated recovery with the same id on the stored record is a duplicate.
        assert_eq!(after.propose_recovery(&plan, TransitionId(50)), Err(Rejected::Duplicate));

        // Completed -> RewardStoredInLobby; then only the lobby claim pays.
        let mut d = Drive::start(LifeMode::Softcore);
        d.to_final();
        let restored = Attempt::restore(d.attempt.record(), &plan).unwrap();
        let p = restored.propose_recovery(&plan, TransitionId(60)).unwrap();
        assert_eq!(p.durable, [DurableIntent::MoveRewardToLobbyClaimChest]);
        let stored = p.attempt.unwrap();
        assert_eq!(stored.state(), RewardStoredInLobby);
        let claim = |path, id| Transition {
            id: TransitionId(id),
            attempt: stored.id(),
            instance: stored.instance(),
            event: Event::ClaimReward(path),
        };
        assert_eq!(stored.propose(&plan, None, claim(ClaimPath::FloorChest, 61)), Err(Rejected::WrongClaimPath));
        let paid = stored.propose(&plan, None, claim(ClaimPath::LobbyClaimChest, 62)).unwrap();
        assert_eq!(paid.durable, [DurableIntent::PayReward]);

        // RewardPaid -> cleanup only; other terminal states -> Terminal.
        let mut d = Drive::start(LifeMode::Easy);
        d.to_final();
        d.go(Event::ClaimReward(ClaimPath::FloorChest)).unwrap();
        let p = d.attempt.propose_recovery(&plan, TransitionId(70)).unwrap();
        assert_eq!((p.attempt, p.durable.len()), (None, 0));
        for attempt in reachable() {
            let s = attempt.state();
            if matches!(s, RewardStoredInLobby | Abandoned(_) | FailedByDeath | Reclaimed) {
                assert_eq!(attempt.propose_recovery(&plan, TransitionId(999)), Err(Rejected::Terminal), "{s:?}");
            }
        }
    }

    #[test]
    fn canonical_requests_differ_for_any_change() {
        let a = Drive::start(LifeMode::Softcore).attempt.record();
        let mut d = Drive::start(LifeMode::Softcore);
        d.go(Event::Death).unwrap();
        let b = d.attempt.record();
        let i = [DurableIntent::CreateAttempt, DurableIntent::ChargeEntryCost];
        let t = Transition {
            id: TransitionId(5),
            attempt: AttemptId(9),
            instance: InstanceId(90),
            event: Event::Death,
        };
        let req = |t: &Transition, base: Option<&AttemptRecord>, result: &AttemptRecord, intents: &[DurableIntent]| {
            canonical_request(t, base, result, intents).unwrap()
        };
        let base = req(&t, Some(&a), &b, &i);
        assert_eq!(req(&t, Some(&a), &b, &i), base);
        let with = |event| Transition { event, ..t };
        let mut variants = vec![
            req(&Transition { id: TransitionId(6), ..t }, Some(&a), &b, &i),
            req(&Transition { instance: InstanceId(91), ..t }, Some(&a), &b, &i),
            req(&t, None, &b, &i),
            req(&t, Some(&b), &b, &i),
            req(&t, Some(&a), &a, &i),
            req(&t, Some(&a), &b, &i[..1]),
            req(&t, Some(&a), &b, &[i[1], i[0]]),
            req(&t, Some(&a), &b, &[i[0], DurableIntent::PayReward]),
            req(&t, Some(&a), &b, &[]),
        ];
        // Same result, same intents, different triggering event or payload.
        for event in [
            Event::EnterRoom(RoomIndex(1)),
            Event::EnterRoom(RoomIndex(2)),
            Event::ClearRoom(RoomIndex(1)),
            Event::DefeatFinalBoss,
            Event::SessionEnd(SessionEndCause::Leave),
            Event::SessionEnd(SessionEndCause::Crash),
            Event::ClaimReward(ClaimPath::FloorChest),
            Event::ClaimReward(ClaimPath::LobbyClaimChest),
            Event::Reclaim,
        ] {
            variants.push(req(&with(event), Some(&a), &b, &i));
        }
        variants.push(base.clone());
        let n = variants.len();
        let mut sorted = variants.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), n, "every variant is distinct from every other");

        let other = AttemptRecord { id: AttemptId(8), ..b };
        assert_eq!(canonical_request(&t, None, &other, &[]), Err(RecordError::AttemptMismatch));
        assert_eq!(canonical_request(&t, Some(&other), &b, &[]), Err(RecordError::AttemptMismatch));
        assert_eq!(
            canonical_request(&t, None, &b, &[DurableIntent::PayReward; MAX_INTENTS + 1]),
            Err(RecordError::TooManyIntents)
        );
        assert!(canonical_request(&t, None, &b, &[DurableIntent::PayReward; MAX_INTENTS]).is_ok());
        assert_eq!(
            canonical_request(&Transition { id: TransitionId(0), ..t }, None, &b, &[]),
            Err(RecordError::ZeroId)
        );
        let bad = AttemptRecord { lives: Lives::Limited(255), ..b };
        assert_eq!(canonical_request(&t, None, &bad, &[]), Err(RecordError::UnsupportedLives));
    }

    #[test]
    fn recovery_ids_are_checked_and_retries_reuse_the_same_request() {
        let plan = plan();
        let active = Drive::start(LifeMode::Softcore).attempt;
        let restored = Attempt::restore(active.record(), &plan).unwrap();
        assert_eq!(restored.recovery_transition_id(), Some(TransitionId(2)));
        // Ids at or below last are refused.
        assert_eq!(restored.propose_recovery(&plan, TransitionId(1)), Err(Rejected::Duplicate));
        let mut later = Drive::start(LifeMode::Softcore);
        later.go(Event::Death).unwrap();
        let later = Attempt::restore(later.attempt.record(), &plan).unwrap();
        assert_eq!(later.propose_recovery(&plan, TransitionId(1)), Err(Rejected::Stale));
        // The same inputs give the same proposal and the same request bytes.
        let id = restored.recovery_transition_id().unwrap();
        let p1 = restored.propose_recovery(&plan, id).unwrap();
        let p2 = restored.propose_recovery(&plan, id).unwrap();
        assert_eq!(p1, p2);
        let t = Transition {
            id,
            attempt: restored.id(),
            instance: restored.instance(),
            event: Event::SessionEnd(SessionEndCause::Crash),
        };
        let r1 = canonical_request(&t, Some(&restored.record()), &p1.attempt.as_ref().unwrap().record(), &p1.durable).unwrap();
        let r2 = canonical_request(&t, Some(&restored.record()), &p2.attempt.as_ref().unwrap().record(), &p2.durable).unwrap();
        assert_eq!(r1, r2);
        // last == u64::MAX: no next id, and recovery can't overflow.
        let max = AttemptRecord { last: TransitionId(u64::MAX), ..active.record() };
        let max = Attempt::restore(max, &plan).unwrap();
        assert_eq!(max.recovery_transition_id(), None);
        assert_eq!(max.propose_recovery(&plan, TransitionId(u64::MAX)), Err(Rejected::Duplicate));
        assert_eq!(max.propose_recovery(&plan, TransitionId(1)), Err(Rejected::Stale));
        // Recovery against a foreign plan is refused.
        let other = FloorPlan::new(FloorId(2), plan.rooms.clone()).unwrap();
        assert_eq!(restored.propose_recovery(&other, id), Err(Rejected::PlanMismatch));
    }

    #[test]
    fn named_lives_cells_from_review() {
        let plan = plan();
        // Softcore Limited(1) is reachable in Completed, RewardPaid and RewardStoredInLobby.
        for end in [None, Some(ClaimPath::FloorChest), Some(ClaimPath::LobbyClaimChest)] {
            let mut d = Drive::start(LifeMode::Softcore);
            d.go(Event::Death).unwrap();
            d.go(Event::Death).unwrap();
            d.to_final();
            match end {
                None => {}
                Some(ClaimPath::FloorChest) => {
                    d.go(Event::ClaimReward(ClaimPath::FloorChest)).unwrap();
                }
                Some(ClaimPath::LobbyClaimChest) => {
                    d.go(Event::SessionEnd(SessionEndCause::Leave)).unwrap();
                }
            }
            assert_eq!(d.attempt.lives(), Lives::Limited(1));
            let r = AttemptRecord::decode(&d.attempt.record().encode().unwrap()).unwrap();
            assert_eq!(Attempt::restore(r, &plan).unwrap(), d.attempt, "{:?}", d.attempt.state());
        }
        // Easy with any Limited value is refused in every state.
        let easy = Drive::start(LifeMode::Easy).attempt.record();
        for n in 0..=3 {
            for state in [Active, Completed, RewardPaid, RewardStoredInLobby, Abandoned(SessionEndCause::Crash)] {
                let r = AttemptRecord { lives: Lives::Limited(n), state, ..easy };
                assert_eq!(Attempt::restore(r, &plan), Err(RestoreError::InconsistentLives));
            }
        }
        // Every Abandoned cause round-trips.
        for cause in [SessionEndCause::Leave, SessionEndCause::Logout, SessionEndCause::Disconnect, SessionEndCause::Crash] {
            let mut d = Drive::start(LifeMode::Softcore);
            d.go(Event::SessionEnd(cause)).unwrap();
            let r = AttemptRecord::decode(&d.attempt.record().encode().unwrap()).unwrap();
            assert_eq!(Attempt::restore(r, &plan).unwrap().state(), Abandoned(cause));
        }
    }
}
