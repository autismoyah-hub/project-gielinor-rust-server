//! PlayerStatExtensions arithmetic, including Kotlin Int overflow and unsigned byte levels.
use crate::stats::Stats;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Add,
    Subtract,
    Boost,
    Drain,
    Heal,
    Restore,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NegativeConstant,
    InvalidPercent,
    HealAboveBase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Update {
    pub skill: u8,
    pub old: u8,
    pub level: u8,
    /// Mark the outgoing stat dirty, even for an unchanged Add/Subtract/Heal.
    pub notify: bool,
    /// Queue the engine stat-change trigger (compares the pre-byte-cast result).
    pub changed: bool,
    pub clear_hero_points: bool,
}
impl Stats {
    pub fn apply(
        &mut self,
        skill: u8,
        operation: Operation,
        constant: i32,
        percent: i32,
        hitpoints: bool,
    ) -> Result<Update, Error> {
        if operation != Operation::Restore {
            if constant < 0 {
                return Err(Error::NegativeConstant);
            }
            if !(0..=100).contains(&percent) {
                return Err(Error::InvalidPercent);
            }
        }
        let base = self.base(skill) as u8 as i32;
        let current = self.current(skill) as u8 as i32;
        let delta = if operation == Operation::Restore {
            0
        } else {
            constant.wrapping_add(base * percent / 100)
        };
        let add = |amount: i32| current.wrapping_add(amount).min(255);
        let subtract = |amount: i32| current.wrapping_sub(amount).max(0);
        let (level, healing) = match operation {
            Operation::Add => (Some(add(delta)), true),
            Operation::Subtract => (Some(subtract(delta)), false),
            Operation::Boost => {
                let amount = base
                    .wrapping_add(delta)
                    .min(current.wrapping_add(delta))
                    .wrapping_sub(current);
                ((amount > 0).then(|| add(amount)), true)
            }
            Operation::Drain => {
                let amount =
                    current.wrapping_sub(base.wrapping_sub(delta).max(current.wrapping_sub(delta)));
                ((amount > 0).then(|| subtract(amount)), false)
            }
            Operation::Heal => {
                // Kotlin coerceIn(current, base) throws when current > base.
                if current > base {
                    return Err(Error::HealAboveBase);
                }
                (Some(current.wrapping_add(delta).clamp(current, base)), true)
            }
            Operation::Restore => ((current != base).then_some(base), base > current),
        };
        let update = Update {
            skill,
            old: current as u8,
            level: level.unwrap_or(current) as u8,
            notify: level.is_some(),
            changed: level.is_some_and(|n| n != current),
            clear_hero_points: level
                .is_some_and(|n| healing && hitpoints && n as u8 as i32 >= base),
        };
        if level.is_some() {
            self.set_current(skill, update.level as i8);
        }
        Ok(update)
    }
}
/// Bodies of the three soft-timer handlers. Timer scheduling and prayer toggles are separate.
pub fn regenerate(
    stats: &mut Stats,
    skills: &[u8],
    hitpoints: u8,
    prayer: u8,
    kind: Regen,
    bracelet: bool,
) -> Vec<Update> {
    match kind {
        Regen::Health => {
            if (stats.current(hitpoints) as u8) >= (stats.base(hitpoints) as u8) {
                return vec![];
            }
            vec![stats
                .apply(
                    hitpoints,
                    Operation::Heal,
                    if bracelet { 2 } else { 1 },
                    0,
                    true,
                )
                .unwrap()]
        }
        Regen::Drained | Regen::Boosted => skills
            .iter()
            .copied()
            .filter(|id| *id != hitpoints && *id != prayer)
            .filter_map(|id| {
                let current = stats.current(id) as u8;
                let base = stats.base(id) as u8;
                let op = if kind == Regen::Drained && current < base {
                    Operation::Add
                } else if kind == Regen::Boosted && current > base {
                    Operation::Subtract
                } else {
                    return None;
                };
                Some(stats.apply(id, op, 1, 0, false).unwrap())
            })
            .collect(),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regen {
    Drained,
    Boosted,
    Health,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boosts_drains_and_regen_use_base_levels_and_do_not_stack_past_threshold() {
        let mut stats = Stats::default();
        stats.set_base(0, 80);
        stats.set_current(0, 80);
        assert_eq!(
            stats
                .apply(0, Operation::Boost, 5, 15, false)
                .unwrap()
                .level,
            97
        );
        assert!(
            !stats
                .apply(0, Operation::Boost, 3, 10, false)
                .unwrap()
                .notify
        );
        assert_eq!(
            stats
                .apply(0, Operation::Drain, 5, 10, false)
                .unwrap()
                .level,
            84
        );
        assert_eq!(
            stats
                .apply(0, Operation::Drain, 5, 10, false)
                .unwrap()
                .level,
            71
        );
        assert_eq!(
            stats
                .apply(0, Operation::Drain, 5, 10, false)
                .unwrap()
                .level,
            67
        );
        assert!(
            !stats
                .apply(0, Operation::Drain, 5, 10, false)
                .unwrap()
                .notify
        );
        stats.set_base(3, 10);
        stats.set_current(3, 9);
        stats.set_base(5, 50);
        stats.set_current(5, 1);
        assert_eq!(
            regenerate(&mut stats, &[0, 3, 5], 3, 5, Regen::Drained, false).len(),
            1
        );
        assert_eq!(stats.current(0), 68);
        assert_eq!(stats.current(5), 1);
        let health = regenerate(&mut stats, &[0, 3, 5], 3, 5, Regen::Health, true);
        assert_eq!(health[0].level, 10);
        assert!(health[0].clear_hero_points);
    }
    #[test]
    fn unsigned_caps_notifications_and_reference_overboost_heal_error() {
        let mut stats = Stats::default();
        stats.set_base(0, 99);
        stats.set_current(0, 250u8 as i8);
        assert_eq!(
            stats.apply(0, Operation::Heal, 1, 0, false),
            Err(Error::HealAboveBase)
        );
        assert_eq!(stats.current(0) as u8, 250);
        assert_eq!(
            stats.apply(0, Operation::Add, 20, 0, false).unwrap().level,
            255
        );
        let unchanged = stats.apply(0, Operation::Add, 1, 0, false).unwrap();
        assert!(unchanged.notify);
        assert!(!unchanged.changed);
        assert_eq!(
            stats
                .apply(0, Operation::Restore, 0, 0, false)
                .unwrap()
                .level,
            99
        );
        assert!(
            !stats
                .apply(0, Operation::Restore, 0, 0, false)
                .unwrap()
                .notify
        );
        assert_eq!(
            stats.apply(0, Operation::Subtract, -1, 0, false),
            Err(Error::NegativeConstant)
        );
    }
}
