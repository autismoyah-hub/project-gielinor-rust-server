//! Sealed, prevalidated character snapshot for a Tower attempt commit (#119 C1).
//!
//! [`SnapshotRequest::new`] runs the save path's static checks, then encodes every [`Save`]
//! field once into an exact, versioned byte image (`TWSV`). The image becomes part of the
//! commit's retry identity, so the same commit with a different `Save` is a `Conflict`, never a
//! silent `AlreadyApplied`. No hash or digest stands in for the bytes.
//!
//! Layout v1, little-endian, in `Save` declaration order: tag, version; the optional diary
//! progress (presence byte, then its counts, flag arrays, bounty and two flags); chemistry
//! (remaining, stop); the Tower run (id, services, four flags, clears); the arrival state (three
//! flags, bones offered as 0 none / 1 false / 2 true); account and character ids; display name,
//! known device, modlevel (each with a presence byte; strings as a u32 length and UTF-8); the
//! packed coordinate; the varps JSON; last login; run energy; XP rate; stats as a u16 count of
//! ascending `(id, visible, base, fine_xp)`; inventories as a u16 count of ascending
//! `(kind, u32 item count, ascending (slot, id, count, vars))`.
use super::CommitError;
use crate::accounts::{Save, Stat};
use gielinor_world::{
    inventory::Item, tower_arrival::State as ArrivalState, tower_diary::Progress, tower_run::Run,
};

pub const SNAPSHOT_TAG: [u8; 4] = *b"TWSV";
pub const SNAPSHOT_VERSION: u16 = 1;
/// PROPOSED technical safety cap on one encoded snapshot, pending lead A's shared review.
/// Not a gameplay number.
pub const MAX_SNAPSHOT_BYTES: usize = 1 << 20;

#[derive(Clone, Debug)]
pub struct SnapshotRequest {
    save: Save,
    bytes: Vec<u8>,
}

impl SnapshotRequest {
    /// Validates the save with the save path's own static checks, then encodes it once.
    pub fn new(save: Save) -> Result<Self, CommitError> {
        crate::accounts::validate_save(&save).map_err(|e| CommitError::InvalidSnapshot(e.message))?;
        // The cap is enforced while measuring, before the image is allocated.
        let bytes = canonical_save(&save)?;
        Ok(Self { save, bytes })
    }
    pub fn character(&self) -> i64 {
        self.save.character_id
    }
    pub fn account(&self) -> i64 {
        self.save.account_id
    }
    /// The exact canonical image that joins the commit's retry identity.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub(crate) fn save(&self) -> &Save {
        &self.save
    }
}

/// The encoder's sink. Every append checks the running length against
/// [`MAX_SNAPSHOT_BYTES`] before it happens, so nothing past the cap is ever copied. A counting
/// sink (`buf: None`) measures without allocating; a writing sink is preallocated exactly.
struct Out {
    buf: Option<Vec<u8>>,
    len: usize,
}
impl Out {
    fn counting() -> Self {
        Self { buf: None, len: 0 }
    }
    fn writing(exact: usize) -> Self {
        Self { buf: Some(Vec::with_capacity(exact)), len: 0 }
    }
    fn put(&mut self, bytes: &[u8]) -> Result<(), CommitError> {
        let next = self
            .len
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_SNAPSHOT_BYTES)
            .ok_or(CommitError::SnapshotTooLarge)?;
        if let Some(buf) = &mut self.buf {
            buf.extend_from_slice(bytes);
        }
        self.len = next;
        Ok(())
    }
    fn u8(&mut self, v: u8) -> Result<(), CommitError> {
        self.put(&[v])
    }
    fn flag(&mut self, v: bool) -> Result<(), CommitError> {
        self.put(&[u8::from(v)])
    }
    fn i8(&mut self, v: i8) -> Result<(), CommitError> {
        self.put(&v.to_le_bytes())
    }
    fn u16(&mut self, v: u16) -> Result<(), CommitError> {
        self.put(&v.to_le_bytes())
    }
    fn u32(&mut self, v: u32) -> Result<(), CommitError> {
        self.put(&v.to_le_bytes())
    }
    fn i32(&mut self, v: i32) -> Result<(), CommitError> {
        self.put(&v.to_le_bytes())
    }
    fn i64(&mut self, v: i64) -> Result<(), CommitError> {
        self.put(&v.to_le_bytes())
    }
    fn len(&mut self, n: usize) -> Result<(), CommitError> {
        let n = u32::try_from(n).map_err(|_| CommitError::SnapshotTooLarge)?;
        self.u32(n)
    }
    fn count(&mut self, n: usize) -> Result<(), CommitError> {
        let n = u16::try_from(n).map_err(|_| CommitError::SnapshotTooLarge)?;
        self.u16(n)
    }
    /// The length prefix and the bytes; a string that would pass the cap is refused before
    /// it is copied.
    fn text(&mut self, v: &str) -> Result<(), CommitError> {
        self.len(v.len())?;
        self.put(v.as_bytes())
    }
    fn opt_text(&mut self, v: &Option<String>) -> Result<(), CommitError> {
        match v {
            None => self.u8(0),
            Some(v) => {
                self.u8(1)?;
                self.text(v)
            }
        }
    }
}

/// Measures first (no allocation, refused at the cap), then encodes into an exact buffer.
pub(super) fn canonical_save(s: &Save) -> Result<Vec<u8>, CommitError> {
    let mut probe = Out::counting();
    encode(s, &mut probe)?;
    let mut out = Out::writing(probe.len);
    encode(s, &mut out)?;
    let bytes = out.buf.unwrap_or_default();
    assert_eq!(bytes.len(), probe.len, "both encoder passes follow one layout");
    Ok(bytes)
}

/// Every struct is destructured without `..`, so adding a field to `Save`, `Run`, `Progress`,
/// the arrival `State`, `Stat` or `Item` fails to compile here until the identity covers it.
/// The one exception is chemistry `State`, whose fields are private to the world crate: it is
/// encoded through its existing `remaining()` and `stop_on_crumble()` getters, and a new
/// chemistry field would have to be added here by hand (the golden test pins today's two).
fn encode(s: &Save, o: &mut Out) -> Result<(), CommitError> {
    let Save {
        tower_diary,
        chemistry,
        tower_run,
        tower_arrival,
        account_id,
        character_id,
        display_name,
        known_device,
        modlevel,
        coords,
        persistent_varps_json,
        last_login,
        run_energy,
        xp_rate_hundreds,
        stats,
        inventories,
    } = s;
    o.put(&SNAPSHOT_TAG)?;
    o.u16(SNAPSHOT_VERSION)?;
    match tower_diary {
        None => o.u8(0)?,
        Some(Progress { counts, actions, done, pending, bounty, schema11, imp_bones }) => {
            o.u8(1)?;
            for n in counts {
                o.i32(*n)?;
            }
            for f in actions.iter().chain(done).chain(pending) {
                o.flag(*f)?;
            }
            o.i32(*bounty)?;
            o.flag(*schema11)?;
            o.flag(*imp_bones)?;
        }
    }
    o.u8(chemistry.remaining())?;
    o.flag(chemistry.stop_on_crumble())?;
    let Run { id, services, imp, victory, reward_pending, first_reward_paid, clears } = tower_run;
    o.u32(*id)?;
    o.u8(*services)?;
    o.flag(*imp)?;
    o.flag(*victory)?;
    o.flag(*reward_pending)?;
    o.flag(*first_reward_paid)?;
    o.u32(*clears)?;
    let ArrivalState { kit_given, staff_taken, diary_progress, bones_offered } = tower_arrival;
    o.flag(*kit_given)?;
    o.flag(*staff_taken)?;
    o.flag(*diary_progress)?;
    o.u8(match bones_offered {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    })?;
    o.i64(*account_id)?;
    o.i64(*character_id)?;
    o.opt_text(display_name)?;
    match known_device {
        None => o.u8(0)?,
        Some(d) => {
            o.u8(1)?;
            o.i64(*d)?;
        }
    }
    o.opt_text(modlevel)?;
    o.i32(coords.packed())?;
    o.text(persistent_varps_json)?;
    o.opt_text(last_login)?;
    o.i32(*run_energy)?;
    o.i32(*xp_rate_hundreds)?;
    o.count(stats.len())?;
    for (stat_id, Stat { visible, base, fine_xp }) in stats {
        o.u8(*stat_id)?;
        o.i8(*visible)?;
        o.i8(*base)?;
        o.i32(*fine_xp)?;
    }
    o.count(inventories.len())?;
    for (kind, items) in inventories {
        o.i32(*kind)?;
        o.len(items.len())?;
        for (slot, Item { id, count, vars }) in items {
            let slot = u32::try_from(*slot).map_err(|_| CommitError::SnapshotTooLarge)?;
            o.u32(slot)?;
            o.i32(*id)?;
            o.i32(*count)?;
            o.i32(*vars)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::Stat;
    use gielinor_world::{inventory::Item, CoordGrid};
    use std::collections::BTreeMap;

    fn fixture() -> Save {
        Save {
            tower_diary: None,
            chemistry: Default::default(),
            tower_run: Default::default(),
            tower_arrival: Default::default(),
            account_id: 7,
            character_id: 9,
            display_name: Some("Pad".into()),
            known_device: None,
            modlevel: None,
            coords: CoordGrid::new(3222, 3218, 0).unwrap(),
            persistent_varps_json: "{}".into(),
            last_login: None,
            run_energy: 10000,
            xp_rate_hundreds: 100,
            stats: BTreeMap::from([(3, Stat { visible: 10, base: 10, fine_xp: 11540 })]),
            inventories: BTreeMap::from([(93, BTreeMap::from([(0, Item { id: 995, count: 25, vars: 0 })]))]),
        }
    }

    #[test]
    fn golden_snapshot_bytes_are_pinned() {
        let s = fixture();
        let mut e = b"TWSV".to_vec();
        e.extend_from_slice(&1u16.to_le_bytes());
        e.push(0); // no diary progress
        e.extend_from_slice(&[s.chemistry.remaining(), 0]);
        e.extend_from_slice(&0u32.to_le_bytes());
        e.extend_from_slice(&[0, 0, 0, 0, 0]);
        e.extend_from_slice(&0u32.to_le_bytes());
        e.extend_from_slice(&[0, 0, 0, 0]);
        e.extend_from_slice(&7i64.to_le_bytes());
        e.extend_from_slice(&9i64.to_le_bytes());
        e.push(1);
        e.extend_from_slice(&3u32.to_le_bytes());
        e.extend_from_slice(b"Pad");
        e.push(0);
        e.push(0);
        e.extend_from_slice(&s.coords.packed().to_le_bytes());
        e.extend_from_slice(&2u32.to_le_bytes());
        e.extend_from_slice(b"{}");
        e.push(0);
        e.extend_from_slice(&10000i32.to_le_bytes());
        e.extend_from_slice(&100i32.to_le_bytes());
        e.extend_from_slice(&1u16.to_le_bytes());
        e.extend_from_slice(&[3, 10, 10]);
        e.extend_from_slice(&11540i32.to_le_bytes());
        e.extend_from_slice(&1u16.to_le_bytes());
        e.extend_from_slice(&93i32.to_le_bytes());
        e.extend_from_slice(&1u32.to_le_bytes());
        e.extend_from_slice(&0u32.to_le_bytes());
        e.extend_from_slice(&995i32.to_le_bytes());
        e.extend_from_slice(&25i32.to_le_bytes());
        e.extend_from_slice(&0i32.to_le_bytes());
        let request = SnapshotRequest::new(s.clone()).unwrap();
        assert_eq!(request.bytes(), e.as_slice());
        assert_eq!(SnapshotRequest::new(s).unwrap().bytes(), request.bytes(), "deterministic");
    }

    #[test]
    fn any_single_field_change_changes_the_bytes() {
        let base = SnapshotRequest::new(fixture()).unwrap().bytes().to_vec();
        let mut variants: Vec<(&str, Save)> = vec![];
        let mut v = |name, f: &dyn Fn(&mut Save)| {
            let mut s = fixture();
            f(&mut s);
            variants.push((name, s));
        };
        v("diary", &|s| s.tower_diary = Some(Default::default()));
        v("chemistry", &|s| s.chemistry = gielinor_world::chemistry::State::restore(2, 1).unwrap());
        v("run id", &|s| s.tower_run.id = 1);
        v("run flag", &|s| s.tower_run.imp = true);
        v("arrival kit", &|s| s.tower_arrival.kit_given = true);
        v("arrival bones", &|s| s.tower_arrival.bones_offered = Some(false));
        v("account", &|s| s.account_id = 8);
        v("character", &|s| s.character_id = 10);
        v("display name", &|s| s.display_name = Some("Pae".into()));
        v("display none", &|s| s.display_name = None);
        v("device", &|s| s.known_device = Some(0));
        v("modlevel", &|s| s.modlevel = Some(String::new()));
        v("coords", &|s| s.coords = CoordGrid::new(3223, 3218, 0).unwrap());
        v("varps", &|s| s.persistent_varps_json = "{ }".into());
        v("last login", &|s| s.last_login = Some(String::new()));
        v("energy", &|s| s.run_energy = 9999);
        v("xp rate", &|s| s.xp_rate_hundreds = 101);
        v("stat xp", &|s| s.stats.get_mut(&3).unwrap().fine_xp = 11541);
        v("stat added", &|s| {
            s.stats.insert(4, Stat { visible: 1, base: 1, fine_xp: 0 });
        });
        v("item count", &|s| s.inventories.get_mut(&93).unwrap().get_mut(&0).unwrap().count = 26);
        v("item vars", &|s| s.inventories.get_mut(&93).unwrap().get_mut(&0).unwrap().vars = 1);
        v("item slot", &|s| {
            let item = s.inventories.get_mut(&93).unwrap().remove(&0).unwrap();
            s.inventories.get_mut(&93).unwrap().insert(1, item);
        });
        v("inventory kind", &|s| {
            let items = s.inventories.remove(&93).unwrap();
            s.inventories.insert(94, items);
        });
        v("empty inventory", &|s| {
            s.inventories.insert(95, BTreeMap::new());
        });
        let mut all = vec![base];
        for (name, save) in variants {
            let bytes = SnapshotRequest::new(save).unwrap().bytes().to_vec();
            assert!(!all.contains(&bytes), "{name} collides");
            all.push(bytes);
        }
    }

    #[test]
    fn invalid_or_oversized_saves_are_refused_before_any_database_use() {
        let refuse = |f: &dyn Fn(&mut Save)| {
            let mut s = fixture();
            f(&mut s);
            SnapshotRequest::new(s)
        };
        assert!(matches!(refuse(&|s| s.coords = CoordGrid::NULL), Err(CommitError::InvalidSnapshot(_))));
        assert!(matches!(refuse(&|s| s.run_energy = 10001), Err(CommitError::InvalidSnapshot(_))));
        assert!(matches!(refuse(&|s| s.stats.get_mut(&3).unwrap().fine_xp = -1), Err(CommitError::InvalidSnapshot(_))));
        assert!(matches!(
            refuse(&|s| s.inventories.get_mut(&93).unwrap().get_mut(&0).unwrap().count = 0),
            Err(CommitError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            refuse(&|s| {
                s.inventories.get_mut(&93).unwrap().insert(65536, Item { id: 1, count: 1, vars: 0 });
            }),
            Err(CommitError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            refuse(&|s| s.persistent_varps_json = "x".repeat(MAX_SNAPSHOT_BYTES)),
            Err(CommitError::SnapshotTooLarge)
        ));
    }

    #[test]
    fn the_cap_is_inclusive_and_refuses_before_encoding_past_it() {
        let mut s = fixture();
        s.persistent_varps_json = String::new();
        let base = SnapshotRequest::new(s.clone()).unwrap().bytes().len();
        assert!(base < MAX_SNAPSHOT_BYTES);
        s.persistent_varps_json = "x".repeat(MAX_SNAPSHOT_BYTES - base);
        assert_eq!(SnapshotRequest::new(s.clone()).unwrap().bytes().len(), MAX_SNAPSHOT_BYTES);
        s.persistent_varps_json.push('x');
        assert!(matches!(SnapshotRequest::new(s.clone()), Err(CommitError::SnapshotTooLarge)));
        // Measuring stops at the cap: a counting pass over an oversized save allocates nothing
        // and records no length beyond the cap.
        let mut probe = Out::counting();
        assert!(matches!(encode(&s, &mut probe), Err(CommitError::SnapshotTooLarge)));
        assert!(probe.buf.is_none() && probe.len <= MAX_SNAPSHOT_BYTES);
        // Collections are bounded the same way: many inventory items past the cap are refused.
        let mut many = fixture();
        let per_item = 16;
        let items = MAX_SNAPSHOT_BYTES / per_item + 1;
        many.inventories.insert(
            94,
            (0..items.min(65535)).map(|slot| (slot, Item { id: 1, count: 1, vars: 0 })).collect(),
        );
        many.inventories.insert(
            95,
            (0..items.min(65535)).map(|slot| (slot, Item { id: 1, count: 1, vars: 0 })).collect(),
        );
        assert!(matches!(SnapshotRequest::new(many), Err(CommitError::SnapshotTooLarge)));
    }
}
