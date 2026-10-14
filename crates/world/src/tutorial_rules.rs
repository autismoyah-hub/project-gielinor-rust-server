//! TutorialIslandRulesScript handlers. Call on delivery of the corresponding
//! engine event, not immediately when experience or damage is applied.
use crate::{engine_queues::Kind, skills::xp_from_level, stats::Stats};

pub const CAPPED_STATS: [u8; 12] = [0, 2, 1, 4, 5, 6, 14, 13, 10, 7, 11, 8];
pub const CAP_MESSAGE: &str = "You can't advance beyond level 3 while on Tutorial Island.";
pub const FLOOR_MESSAGE: &str = "You would have died, but the gods of Tutorial Island protect you.";

pub fn bindings() -> crate::engine_queues::Bindings {
    let mut bindings = crate::engine_queues::Bindings::default();
    for stat in CAPPED_STATS.into_iter().chain([3]) {
        bindings.add_labelled(Kind::AdvanceStat, stat as i32);
    }
    bindings.add_labelled(Kind::ChangeStat, 3);
    bindings
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub clear_death_queue: bool,
    pub message: Option<&'static str>,
}

/// `completed` is the current tutorial completion variable. Only exactly one
/// disables the reference rules; this check is deliberately independent of map.
pub fn deliver(stats: &mut Stats, completed: i32, kind: Kind, stat: u8) -> Effects {
    if completed == 1 {
        return Effects::default();
    }
    match kind {
        Kind::AdvanceStat if CAPPED_STATS.contains(&stat) && stats.base(stat) as u8 > 3 => {
            stats.set_xp(stat, xp_from_level(3).unwrap()).unwrap();
            stats.set_base(stat, 3);
            stats.set_current(stat, 3);
            Effects {
                message: Some(CAP_MESSAGE),
                ..Effects::default()
            }
        }
        Kind::AdvanceStat if stat == 3 => {
            stats.set_xp(3, xp_from_level(10).unwrap()).unwrap();
            stats.set_base(3, 10);
            if stats.current(3) as u8 > 10 {
                stats.set_current(3, 10);
            }
            Effects::default()
        }
        Kind::ChangeStat if stat == 3 && stats.current(3) as u8 == 0 => {
            stats.set_current(3, 1);
            Effects {
                clear_death_queue: true,
                message: Some(FLOOR_MESSAGE),
            }
        }
        _ => Effects::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsigned_hp_and_exact_completion_flag() {
        for completed in [-1, 0, 1, 2] {
            for hp in [0u8, 1, 10, 11, 127, 128, 255] {
                let mut stats = Stats::default();
                stats.set_current(3, hp as i8);
                let effects = deliver(&mut stats, completed, Kind::ChangeStat, 3);
                let rescued = completed != 1 && hp == 0;
                assert_eq!(effects.clear_death_queue, rescued);
                assert_eq!(stats.current(3) as u8, if rescued { 1 } else { hp });
            }
        }
    }
    #[test]
    fn queued_delivery_observes_current_state_and_preserves_sublevel_hp_xp() {
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 7);
        stats.set_xp(3, 1200).unwrap();
        deliver(&mut stats, 0, Kind::ChangeStat, 3);
        assert_eq!(stats.xp(3), 1200);
        deliver(&mut stats, 0, Kind::AdvanceStat, 3);
        assert_eq!(stats.xp(3), xp_from_level(10).unwrap());
        assert_eq!(stats.current(3), 7);
        stats.set_base(0, 4);
        stats.set_current(0, 9);
        stats.set_xp(0, 500).unwrap();
        deliver(&mut stats, 1, Kind::AdvanceStat, 0);
        assert_eq!(stats.base(0), 4);
        assert_eq!(
            deliver(&mut stats, 0, Kind::AdvanceStat, 0).message,
            Some(CAP_MESSAGE)
        );
        assert_eq!((stats.base(0), stats.current(0), stats.xp(0)), (3, 3, 174));
    }
}
