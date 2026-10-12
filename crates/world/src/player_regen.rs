use crate::{
    player_timers::Timers,
    stat_ops::{self, Regen, Update},
    stats::Stats,
};
#[derive(Clone, Debug)]
pub struct Config {
    pub skills: Vec<u8>,
    pub hitpoints: u8,
    pub prayer: u8,
    pub bracelet: i32,
    pub drained_timer: u16,
    pub boosted_timer: u16,
    pub health_timer: u16,
    pub rapid_timer: u16,
}
impl Config {
    /// PrayerTab's ordinary toggles. Preserve additionally requires its queued
    /// activation to be canceled/replaced by the world before this call.
    pub fn prayer_toggle(
        &self,
        timers: &mut Timers,
        clock: i32,
        kind: crate::prayer_book::Regen,
        enabled: bool,
    ) {
        use crate::prayer_book::Regen;
        match (kind, enabled) {
            (Regen::RapidRestore, true) => {
                timers.schedule(self.rapid_timer, clock, 100).unwrap();
            }
            (Regen::RapidRestore, false) => timers.remove(self.rapid_timer),
            (Regen::RapidHeal, enabled) => {
                timers
                    .schedule(self.health_timer, clock, if enabled { 50 } else { 100 })
                    .unwrap();
            }
            (Regen::Preserve, false) => {
                timers.schedule(self.boosted_timer, clock, 100).unwrap();
            }
            (Regen::Preserve, true) => {}
        }
    }
    pub fn activate_preserve(&self, timers: &mut Timers, clock: i32) {
        timers.schedule(self.boosted_timer, clock, 150).unwrap();
    }
    pub fn login(&self, timers: &mut Timers, clock: i32) {
        for id in [self.drained_timer, self.boosted_timer, self.health_timer] {
            timers.schedule(id, clock, 100).unwrap();
        }
    }
    pub fn tick(&self, id: u16, stats: &mut Stats, hands: Option<i32>) -> Vec<Update> {
        let kind = if id == self.drained_timer || id == self.rapid_timer {
            Regen::Drained
        } else if id == self.boosted_timer {
            Regen::Boosted
        } else if id == self.health_timer {
            Regen::Health
        } else {
            return Vec::new();
        };
        stat_ops::regenerate(
            stats,
            &self.skills,
            self.hitpoints,
            self.prayer,
            kind,
            hands == Some(self.bracelet),
        )
    }
}
