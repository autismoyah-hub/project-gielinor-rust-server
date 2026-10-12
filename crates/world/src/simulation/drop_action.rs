use super::*;
use crate::drop_actions::{Event, Kind, Pending, Stage};
impl World {
    /// Trusted gameplay override; like Kotlin this is transient player state.
    pub fn set_loot_drop_duration(
        &mut self,
        actor: ActorId,
        duration: Option<i32>,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        self.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .loot_drop_duration = duration;
        Ok(())
    }
    pub fn drop_dialog_token(&self, actor: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self.player(actor)?.pending_drop.map(|v| v.0))
    }
    pub fn cancel_drop_dialog(&mut self, actor: ActorId) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let cancelled = p.pending_drop.take().is_some();
        if cancelled {
            p.protection.suspended = false;
        }
        Ok(cancelled)
    }
    pub(super) fn start_drop(&mut self, actor: ActorId, slot: usize) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        p.begin_held_action();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let item = p.data.inventory.get(slot).ok_or(ActionError::InvalidItem)?;
        let config = self.dropping.as_ref().ok_or(ActionError::InvalidItem)?;
        let rule = config.rules.get(&item.id).ok_or(ActionError::InvalidItem)?;
        let enabled = config
            .warning_enabled
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?
            == 1;
        let threshold = config
            .warning_threshold
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?;
        let pending = Pending::begin(slot, item, rule, enabled, threshold);
        if let Some(prompt) = pending.prompt(rule) {
            p.dialog_generation = p
                .dialog_generation
                .checked_add(1)
                .ok_or(ActionError::QueueFull)?;
            let token = p.dialog_generation;
            p.pending_drop = Some((token, pending));
            p.protection.suspended = true;
            self.events.push(WorldEvent::Drop {
                actor,
                event: Event::Prompt { token, prompt },
            });
            Ok(())
        } else {
            self.commit_drop(actor, pending)
        }
    }
    pub(super) fn resume_drop(
        &mut self,
        actor: ActorId,
        token: u64,
        choice: i32,
    ) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        if p.pending_drop.is_none_or(|(current, _)| current != token) {
            return Ok(());
        }
        let (_, mut pending) = p.pending_drop.take().unwrap();
        p.protection.suspended = false;
        if p.protection.delayed() {
            self.events.push(WorldEvent::Drop {
                actor,
                event: Event::Closed,
            });
            return Err(ActionError::Delayed);
        }
        if pending.resume(choice).is_err() {
            self.events.push(WorldEvent::Drop {
                actor,
                event: Event::Closed,
            });
            return Err(ActionError::InvalidItem);
        }
        let rule = self
            .dropping
            .as_ref()
            .and_then(|c| c.rules.get(&pending.item.id))
            .ok_or(ActionError::InvalidItem)?;
        if let Some(prompt) = pending.prompt(rule) {
            // A fresh token prevents two queued replies from consuming both pages.
            p.dialog_generation = p
                .dialog_generation
                .checked_add(1)
                .ok_or(ActionError::QueueFull)?;
            let token = p.dialog_generation;
            p.pending_drop = Some((token, pending));
            p.protection.suspended = true;
            self.events.push(WorldEvent::Drop {
                actor,
                event: Event::Prompt { token, prompt },
            });
            return Ok(());
        }
        let result = if pending.stage == Stage::Ready {
            self.commit_drop(actor, pending)
        } else {
            Ok(())
        };
        self.events.push(WorldEvent::Drop {
            actor,
            event: Event::Closed,
        });
        result
    }
    fn commit_drop(&mut self, actor: ActorId, pending: Pending) -> Result<(), ActionError> {
        let config = self.dropping.as_ref().ok_or(ActionError::InvalidItem)?;
        let rule = config
            .rules
            .get(&pending.item.id)
            .ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        match pending.kind {
            Kind::Drop => {
                let observer = p.observer_id.ok_or(ActionError::Protected)?;
                crate::ground_transfer::drop_slot(
                    &mut self.objects,
                    &mut p.data.inventory,
                    &self.object_types,
                    self.clock,
                    crate::ground_transfer::Drop {
                        slot: pending.slot,
                        count: i32::MAX,
                        coords: p.movement.coords,
                        observer,
                        duration: p.loot_drop_duration.unwrap_or(200),
                        reveal: 100,
                    },
                )
                .map_err(|_| ActionError::InvalidItem)?;
                self.events.push(WorldEvent::Drop {
                    actor,
                    event: Event::Sound(config.sound),
                });
            }
            Kind::Destroy | Kind::Release => {
                pending
                    .delete(&mut p.data.inventory, &self.object_types)
                    .map_err(|_| ActionError::InvalidItem)?;
                if pending.kind == Kind::Release {
                    if let Some(message) = &rule.release_message {
                        self.events.push(WorldEvent::Drop {
                            actor,
                            event: Event::Message(message.clone()),
                        });
                    }
                }
            }
        }
        self.events.push(WorldEvent::Drop {
            actor,
            event: Event::Committed {
                item: pending.item,
                kind: pending.kind,
            },
        });
        Ok(())
    }
}
