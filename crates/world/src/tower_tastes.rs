//! Existing FloorOneScript ash-timer movement and rune-spend observations.
use crate::{
    inventory::Inventory,
    tower_diary::{Progress, Sync},
    CoordGrid,
};
use std::collections::BTreeSet;
#[derive(Clone, Debug)]
pub struct State {
    pub last_move: Option<CoordGrid>,
    pub mind_seen: i32,
    pub mind_pending: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            last_move: None,
            mind_seen: -1,
            mind_pending: false,
        }
    }
}
impl State {
    pub fn sample_minds(&mut self, inv: &Inventory, ids: &BTreeSet<i32>) {
        let now = inv
            .slots()
            .iter()
            .flatten()
            .filter(|i| ids.contains(&i.id))
            .fold(0i32, |sum, i| sum.wrapping_add(i.count));
        if self.mind_seen >= 0 && now < self.mind_seen {
            self.mind_pending = true;
        }
        self.mind_seen = now;
    }
    /// Only called inside Tower bounds. Plane changes use the source's 2D
    /// distance: same x/z on a different plane still grants the minimum taste.
    pub fn movement_xp(&mut self, coords: CoordGrid) -> Option<f64> {
        let previous = self.last_move.replace(coords)?;
        if previous == coords {
            return None;
        }
        Some(previous.chebyshev_distance(coords).clamp(1, 16) as f64 * 0.25)
    }
    pub fn grant_mind_spend(
        &mut self,
        diary: Progress,
        levels: [i32; 3],
        staff: bool,
        protected: bool,
    ) -> Option<Sync> {
        if !self.mind_pending || diary.done[6] || protected {
            return None;
        }
        self.mind_pending = false;
        Some(diary.marks(levels, staff).note(2).session_sync())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn first_and_stationary_samples_award_nothing_then_cap_teleports() {
        let mut s = State::default();
        let c = CoordGrid::new(10424, 120, 0).unwrap();
        assert_eq!(s.movement_xp(c), None);
        assert_eq!(s.movement_xp(c), None);
        assert_eq!(s.movement_xp(c.translate(3, 4, 0).unwrap()), Some(1.0));
        assert_eq!(s.movement_xp(c.translate(60, 0, 0).unwrap()), Some(4.0));
        assert_eq!(s.movement_xp(c.translate(60, 0, 1).unwrap()), Some(0.25));
    }
    #[test]
    fn observed_reduction_is_deferred_while_protected_then_consumed_once() {
        let mut s = State::default();
        let mut inv = Inventory::new(1, StackType::Always, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 558,
                count: 3,
                vars: 0,
            }),
        )
        .unwrap();
        s.sample_minds(&inv, &[558].into());
        assert!(!s.mind_pending);
        inv.restore_slot(0, None).unwrap();
        s.sample_minds(&inv, &[558].into());
        let mut diary = Progress {
            schema11: true,
            ..Default::default()
        };
        diary.done[..6].fill(true);
        diary.counts[4] = 5;
        assert!(s.grant_mind_spend(diary, [1; 3], false, true).is_none());
        assert!(s.mind_pending);
        let out = s.grant_mind_spend(diary, [1; 3], false, false).unwrap();
        assert!(out.marks.actions[2]);
        assert!(!s.mind_pending);
        assert!(s.grant_mind_spend(diary, [1; 3], false, false).is_none());
    }
    #[test]
    fn paid_minds_row_preserves_pending_and_noneligible_spend_does_not_skip_rows() {
        let mut s = State {
            mind_pending: true,
            ..Default::default()
        };
        let mut diary = Progress {
            schema11: true,
            ..Default::default()
        };
        diary.done[6] = true;
        assert!(s.grant_mind_spend(diary, [1; 3], false, false).is_none());
        assert!(s.mind_pending);
        diary.done[6] = false;
        let out = s.grant_mind_spend(diary, [1; 3], false, false).unwrap();
        assert!(!out.marks.actions[2]);
        assert!(!s.mind_pending);
    }
}
