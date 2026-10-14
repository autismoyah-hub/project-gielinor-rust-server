//! Stat-to-engine-queue adapter. XP thresholds enqueue ChangeStat followed by
//! AdvanceStat even if a boosted current level did not change. Handlers receive
//! current player state at publication time rather than an XP-award snapshot.
use crate::{
    engine_queues::{Bindings, Error, Kind, Queue, Queues},
    skills::Advancement,
};

#[derive(Default)]
pub struct Events {
    pending: Queues<u8>,
}
impl Events {
    pub fn len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn clear(&mut self) {
        self.pending.clear();
    }
    pub fn advance(
        &mut self,
        bindings: &Bindings,
        stat: u8,
        advancement: Advancement,
    ) -> Result<(), Error> {
        self.advance_batch(bindings, [(stat, advancement)])
    }
    pub fn advance_batch(
        &mut self,
        bindings: &Bindings,
        advancements: impl IntoIterator<Item = (u8, Advancement)>,
    ) -> Result<(), Error> {
        let mut pending = Vec::new();
        for (stat, change) in advancements {
            if change.old_level == change.new_level {
                continue;
            }
            for kind in [Kind::ChangeStat, Kind::AdvanceStat] {
                if bindings.select(kind, stat as i32).is_some() {
                    pending.push(Queue {
                        kind,
                        label: stat as i32,
                        args: stat,
                    });
                }
            }
        }
        self.pending.push_batch(pending)
    }
    pub fn changed(&mut self, bindings: &Bindings, stat: u8) -> Result<(), Error> {
        self.enqueue(bindings, stat, &[Kind::ChangeStat])
    }
    fn enqueue(&mut self, bindings: &Bindings, stat: u8, kinds: &[Kind]) -> Result<(), Error> {
        let pending = kinds
            .iter()
            .filter(|&&kind| bindings.select(kind, stat as i32).is_some())
            .map(|&kind| Queue {
                kind,
                label: stat as i32,
                args: stat,
            })
            .collect();
        self.pending.push_batch(pending)
    }
    pub fn process(
        &mut self,
        protected: bool,
        mut publish: impl FnMut(Kind, u8) -> bool,
    ) -> Result<(), Error> {
        self.pending
            .process(protected, |_, queue| publish(queue.kind, queue.args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{stats::Stats, tutorial_rules};
    #[test]
    fn threshold_events_wait_for_access_and_observe_completion_at_delivery() {
        let mut bindings = Bindings::default();
        bindings.add_labelled(Kind::AdvanceStat, 0);
        bindings.add_default(Kind::ChangeStat);
        let mut events = Events::default();
        let mut stats = Stats::default();
        stats.set_current(0, 20);
        let advancement = crate::skills::advance(&mut stats, 0, 500.0, 1.0, 99).unwrap();
        events.advance(&bindings, 0, advancement).unwrap();
        assert_eq!(events.len(), 2);
        events.process(true, |_, _| panic!("protected")).unwrap();
        assert_eq!(stats.current(0), 20);
        let mut seen = vec![];
        events
            .process(false, |kind, stat| {
                seen.push(kind);
                // The character finished the tutorial during protection.
                tutorial_rules::deliver(&mut stats, 1, kind, stat);
                false
            })
            .unwrap();
        assert_eq!(seen, [Kind::ChangeStat, Kind::AdvanceStat]);
        assert_eq!(stats.current(0), 20);
        assert!(events.is_empty());
    }
    #[test]
    fn no_threshold_no_handler_and_atomic_capacity() {
        let mut bindings = Bindings::default();
        bindings.add_default(Kind::ChangeStat);
        bindings.add_default(Kind::AdvanceStat);
        let mut events = Events::default();
        events
            .advance(
                &bindings,
                3,
                Advancement {
                    added_xp: 10,
                    old_level: 10,
                    new_level: 10,
                },
            )
            .unwrap();
        assert!(events.is_empty());
        events.changed(&Bindings::default(), 3).unwrap();
        assert!(events.is_empty());
        for _ in 0..1023 {
            events.changed(&bindings, 3).unwrap();
        }
        assert_eq!(
            events.advance(
                &bindings,
                3,
                Advancement {
                    added_xp: 1000,
                    old_level: 10,
                    new_level: 11
                }
            ),
            Err(Error::Capacity)
        );
        assert_eq!(events.len(), 1023);
        events.process(false, |_, _| true).unwrap();
        assert_eq!(events.len(), 1022);
    }
}
