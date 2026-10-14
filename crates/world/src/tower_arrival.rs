//! FloorOneScript's Lumbridge recovery-pad and once-only arrival policy.
//! `diary_progress` is the persisted projection of hasDiaryProgress, not XP.
use crate::{simulation::PlayerData, CoordGrid};

#[derive(Clone, Copy)]
pub struct Config {
    pub origin: CoordGrid,
    pub progression: crate::floor_progression::Policy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub kit_given: bool,
    pub staff_taken: bool,
    pub diary_progress: bool,
    /// None preserves an unmigrated legacy bones sidecar as UNKNOWN.
    pub bones_offered: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrival {
    pub destination: CoordGrid,
    pub first: bool,
    pub strip: bool,
    pub fists_message: bool,
}

pub fn recovery_pad(coords: CoordGrid) -> bool {
    coords.level() == 0
        && (3219..=3223).contains(&coords.x())
        && (3216..=3220).contains(&coords.z())
}

impl State {
    pub fn arrival(self, origin: CoordGrid, packed: bool) -> Result<Arrival, crate::CoordError> {
        let first = !self.kit_given;
        let strip = first
            && !self.diary_progress
            && self.bones_offered != Some(true)
            && (packed || self.staff_taken);
        Ok(Arrival {
            destination: origin.translate(56, if first { 56 } else { 49 }, 0)?,
            first,
            strip,
            // A staff-only flag clear suppresses the fallback message in Kotlin.
            fists_message: first && (!strip || packed),
        })
    }
}

/// Apply only after the movement destination has been validated and committed.
pub fn apply(data: &mut PlayerData, arrival: Arrival) {
    if arrival.strip {
        for inv in [&mut data.inventory, &mut data.equipment] {
            for slot in 0..inv.slots().len() {
                inv.restore_slot(slot, None).expect("existing slot");
            }
        }
        data.tower_arrival.staff_taken = false;
    }
    if arrival.first {
        data.tower_arrival.kit_given = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, stats::Stats};
    #[test]
    fn pad_boundary_and_plane_are_exact() {
        for x in 3218..=3224 {
            for z in 3215..=3221 {
                for level in 0..4 {
                    assert_eq!(
                        recovery_pad(CoordGrid::new(x, z, level).unwrap()),
                        level == 0 && (3219..=3223).contains(&x) && (3216..=3220).contains(&z)
                    );
                }
            }
        }
    }
    #[test]
    fn kit_is_stripped_once_and_bank_is_preserved() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let mut data = PlayerData::new(
            1,
            "Arrival".into(),
            CoordGrid::new(3222, 3218, 0).unwrap(),
            Stats::default(),
        );
        let item = Item {
            id: 995,
            count: 1000,
            vars: 0,
        };
        for inv in [&mut data.inventory, &mut data.equipment, &mut data.bank] {
            inv.restore_slot(0, Some(item)).unwrap();
        }
        let first = data.tower_arrival.arrival(origin, true).unwrap();
        assert_eq!(first.destination, CoordGrid::new(10424, 120, 0).unwrap());
        apply(&mut data, first);
        assert!(data.inventory.get(0).is_none());
        assert!(data.equipment.get(0).is_none());
        assert_eq!(data.bank.get(0), Some(item));
        data.inventory.restore_slot(0, Some(item)).unwrap();
        let next = data.tower_arrival.arrival(origin, true).unwrap();
        assert_eq!(next.destination, CoordGrid::new(10424, 113, 0).unwrap());
        apply(&mut data, next);
        assert_eq!(data.inventory.get(0), Some(item));
    }
    #[test]
    fn independently_restored_bones_flag_preserves_the_arrival_pack() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        for flag in [None, Some(false), Some(true)] {
            let result = State {
                bones_offered: flag,
                ..State::default()
            }
            .arrival(origin, true)
            .unwrap();
            assert_eq!(result.strip, flag != Some(true));
        }
    }
    #[test]
    fn diary_progress_preserves_pack_and_staff_only_clear_has_no_fists_message() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let earned = State {
            diary_progress: true,
            ..State::default()
        }
        .arrival(origin, true)
        .unwrap();
        assert!(!earned.strip);
        assert!(earned.fists_message);
        let staff = State {
            staff_taken: true,
            ..State::default()
        }
        .arrival(origin, false)
        .unwrap();
        assert!(staff.strip);
        assert!(!staff.fists_message);
    }
}
