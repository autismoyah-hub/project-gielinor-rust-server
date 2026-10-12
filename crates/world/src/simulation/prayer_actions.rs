use super::*;
use crate::prayer::{Event, State};
impl World {
    pub fn cancel_prayer_dialog(&mut self, actor: ActorId) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let cancelled = matches!(p.prayer, Some(State::Confirm { .. }));
        if cancelled {
            p.prayer = None;
            p.protection.suspended = false;
        }
        Ok(cancelled)
    }
    pub(super) fn start_bury(&mut self, actor: ActorId, slot: usize) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        p.begin_held_action();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let bone = p
            .data
            .inventory
            .get(slot)
            .ok_or(ActionError::InvalidItem)?
            .id;
        let config = self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        if !config.bones.contains_key(&bone) {
            return Err(ActionError::InvalidItem);
        }
        if config.requires_confirmation(p.movement.coords) {
            p.dialog_generation = p
                .dialog_generation
                .checked_add(1)
                .ok_or(ActionError::QueueFull)?;
            let token = p.dialog_generation;
            p.prayer = Some(State::Confirm { token, bone });
            p.protection.suspended = true;
            self.events.push(WorldEvent::Prayer {
                actor,
                event: Event::Confirm { token },
            });
            return Ok(());
        }
        self.animate_bury(actor, bone)
    }
    fn animate_bury(&mut self, actor: ActorId, bone: i32) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        let config = self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        let pending = config
            .prepare(&p.data, bone)
            .map_err(|_| ActionError::InvalidItem)?;
        p.prayer = Some(State::Bury(pending));
        p.protection.suspended = true;
        p.protection.delay(1);
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Animation(config.animation),
        });
        Ok(())
    }
    pub(super) fn resume_bury(
        &mut self,
        actor: ActorId,
        token: u64,
        choice: i32,
    ) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        let Some(State::Confirm {
            token: current,
            bone,
        }) = p.prayer
        else {
            return Ok(());
        };
        if token != current || !(1..=2).contains(&choice) {
            return Ok(());
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        p.prayer = None;
        p.protection.suspended = false;
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Closed,
        });
        if choice == 1 {
            self.animate_bury(actor, bone)?;
        }
        Ok(())
    }
    pub(super) fn process_prayer(&mut self, slot: usize) {
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        if p.saving {
            return;
        }
        p.protection.processed_clock = self.clock.0;
        if p.protection.delayed() {
            return;
        }
        let Some(State::Bury(pending)) = p.prayer else {
            return;
        };
        p.prayer = None;
        p.protection.suspended = false;
        let Some(config) = &self.prayer else { return };
        match config.finish(&mut p.data, &self.object_types, pending) {
            Ok(Some(result)) => {
                self.events.push(WorldEvent::Prayer {
                    actor: p.id,
                    event: Event::Completed {
                        skill: config.skill,
                        result,
                    },
                });
                self.events.push(WorldEvent::Prayer {
                    actor: p.id,
                    event: Event::Message("You bury the bones."),
                });
            }
            Ok(None) => (),
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::InvalidStat,
            }),
        }
    }
}
