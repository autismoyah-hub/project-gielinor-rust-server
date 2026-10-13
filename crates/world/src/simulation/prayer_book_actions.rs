use super::*;
use crate::prayer_book::Event;
pub(super) const TOGGLE: i32 = 6;
pub(super) const PRESERVE: i32 = 7;
pub(super) fn publish(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    output: crate::prayer_runtime::Output,
    events: &mut Vec<WorldEvent>,
) -> Result<(), ActionError> {
    if let Some(enable) = output.preserve_queue {
        queues.remove_all(PRESERVE);
        if enable {
            queues
                .add(
                    PRESERVE,
                    crate::queues::Category::LongDiscard,
                    25,
                    Deferred::Preserve,
                )
                .map_err(|_| ActionError::QueueFull)?;
        }
    }
    if let Some(update) = output.stat {
        events.push(WorldEvent::Stat {
            actor: p.id,
            update,
        });
    }
    for event in output.events {
        match event {
            Event::Message { text, modal: true } => {
                p.dialog_generation = p.dialog_generation.wrapping_add(1);
                p.prayer_notice = Some(p.dialog_generation);
                p.protection.suspended = true;
                p.has_modal = true;
                events.push(WorldEvent::PrayerNotice {
                    actor: p.id,
                    token: p.dialog_generation,
                    text,
                });
            }
            Event::Overhead(icon) => {
                p.overhead = icon;
                events.push(WorldEvent::PrayerBook {
                    actor: p.id,
                    event: Event::Overhead(icon),
                });
            }
            event => events.push(WorldEvent::PrayerBook { actor: p.id, event }),
        }
    }
    Ok(())
}
pub(super) fn toggle(
    config: &crate::prayer_runtime::Config,
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    id: u8,
    clock: i32,
    events: &mut Vec<WorldEvent>,
) -> Result<(), ActionError> {
    p.begin_held_action();
    let output = config
        .toggle(
            id,
            &mut p.data.variables,
            &p.data.stats,
            p.overhead,
            &mut p.soft_timers,
            clock,
            p.protection.processed_clock > 0,
        )
        .map_err(|_| ActionError::InvalidStat)?;
    publish(p, queues, output, events)
}
impl World {
    pub(super) fn update_prayer_resistance(&mut self, actor: ActorId) {
        let Some(config) = &self.prayer_book else {
            return;
        };
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let bonus = self.melee.as_ref().map_or(0, |melee| {
            crate::bonuses::calculate(&p.data.equipment, &melee.bonuses, false).values[13]
        });
        match config.drain.update_resistance(
            &mut p.data.variables,
            bonus,
            p.protection.processed_clock > 0,
        ) {
            Ok(Some(update)) => self.events.push(WorldEvent::PrayerBook {
                actor,
                event: Event::Variable(update),
            }),
            Ok(None) => {}
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidStat,
            }),
        }
    }
    /// Caller applies Player.ifClose before selecting, just as the prayer-tab hook.
    pub fn select_prayer(&mut self, actor: ActorId, id: u8) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self.prayer_book.as_ref().ok_or(ActionError::InvalidStat)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            if let Some(events) = config
                .book
                .protected_failure(id, &p.data.variables, &p.data.stats)
                .map_err(|_| ActionError::InvalidStat)?
            {
                for event in events {
                    self.events.push(WorldEvent::PrayerBook { actor, event });
                }
            } else if p
                .queues
                .snapshot()
                .iter()
                .filter(|q| q.id == TOGGLE)
                .count()
                < 10
            {
                p.queues
                    .add(
                        TOGGLE,
                        crate::queues::Category::Strong,
                        1,
                        Deferred::PrayerToggle(id),
                    )
                    .map_err(|_| ActionError::QueueFull)?;
            }
            return Ok(());
        }
        let mut queues = std::mem::take(&mut p.queues);
        let result = toggle(config, p, &mut queues, id, self.clock.0, &mut self.events);
        p.queues = queues;
        result
    }
    pub fn prayer_notice_token(&self, actor: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self.player(actor)?.prayer_notice)
    }
    pub fn close_prayer_notice(&mut self, actor: ActorId, token: u64) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.prayer_notice != Some(token) {
            return Ok(false);
        }
        p.prayer_notice = None;
        p.protection.suspended = false;
        Ok(true)
    }
}
