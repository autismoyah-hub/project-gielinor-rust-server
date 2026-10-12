//! PlayerTimerMap insertion order and PlayerTimerProcessor snapshot semantics.
//! A callback can clear/reschedule itself or another timer. Expired callbacks
//! already captured for this pass still run, even when another callback clears them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: u16,
    pub expiry: i32,
    pub interval: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Timers {
    entries: Vec<Entry>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidInterval;
impl Timers {
    pub fn schedule(&mut self, id: u16, clock: i32, interval: i32) -> Result<(), InvalidInterval> {
        if interval <= 0 {
            return Err(InvalidInterval);
        }
        let entry = Entry {
            id,
            expiry: clock.wrapping_add(interval),
            interval,
        };
        if let Some(old) = self.entries.iter_mut().find(|e| e.id == id) {
            *old = entry;
        } else {
            self.entries.push(entry);
        }
        Ok(())
    }
    pub fn remove(&mut self, id: u16) {
        self.entries.retain(|e| e.id != id);
    }
    pub fn get(&self, id: u16) -> Option<Entry> {
        self.entries.iter().find(|e| e.id == id).copied()
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// The caller checks access protection for normal timers; soft timers pass true.
    /// Returning false from launch stops this normal-timer pass before that callback.
    pub fn process(&mut self, clock: i32, mut launch: impl FnMut(&mut Self, u16) -> bool) {
        let expired = self
            .entries
            .iter()
            .filter(|e| clock >= e.expiry)
            .map(|e| e.id)
            .collect::<Vec<_>>();
        for id in expired {
            if !launch(self, id) {
                break;
            }
            if let Some(entry) = self.get(id) {
                self.schedule(id, clock, entry.interval)
                    .expect("stored positive interval");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expiry_snapshot_retains_cleared_keys_but_not_new_timers() {
        let mut timers = Timers::default();
        timers.schedule(8, 0, 1).unwrap();
        timers.schedule(4, 0, 1).unwrap();
        let mut fired = Vec::new();
        timers.process(1, |t, id| {
            fired.push(id);
            if id == 8 {
                t.remove(4);
                t.schedule(3, 0, 1).unwrap();
            }
            true
        });
        assert_eq!(fired, vec![8, 4]);
        assert_eq!(
            timers.entries().iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![8, 3]
        );
        assert_eq!(timers.get(8).unwrap().expiry, 2);
        assert_eq!(timers.get(3).unwrap().expiry, 1);
    }
    #[test]
    fn rescheduling_inside_callback_uses_new_interval_from_current_tick() {
        let mut timers = Timers::default();
        timers.schedule(1, 0, 1).unwrap();
        timers.process(5, |t, id| {
            t.schedule(id, 99, 50).unwrap();
            true
        });
        assert_eq!(timers.get(1).unwrap().expiry, 55);
        timers.process(55, |t, id| {
            t.remove(id);
            true
        });
        assert!(timers.entries().is_empty());
    }
    #[test]
    fn protected_timer_is_not_rescheduled_and_wrapping_matches_kotlin_int() {
        let mut timers = Timers::default();
        timers.schedule(1, 0, 1).unwrap();
        timers.process(9, |_, _| false);
        assert_eq!(timers.get(1).unwrap().expiry, 1);
        timers.schedule(2, i32::MAX, 1).unwrap();
        assert_eq!(timers.get(2).unwrap().expiry, i32::MIN);
        assert_eq!(timers.schedule(3, 0, 0), Err(InvalidInterval));
    }
}
