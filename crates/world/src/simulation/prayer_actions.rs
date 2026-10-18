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
    pub(super) fn start_bury(&mut self, actor: ActorId, slot: usize, expected: Option<i32>) -> Result<(), ActionError> {
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
        if expected.is_some_and(|item|item!=bone) || !config.bones.contains_key(&bone) {
            return Err(ActionError::InvalidItem);
        }
        let selected=expected.map(|_|slot);
        if let Some(slot)=selected {
            if self.prayer_member_bones.contains(&bone) && !p.member {return Err(ActionError::InvalidItem);}
            config.prepare_selected(&p.data,&self.object_types,bone,slot).map_err(|_|ActionError::InvalidItem)?;
        }
        if config.requires_confirmation(p.movement.coords) && !(selected.is_some() && self.prayer_ashes.contains_key(&bone)) {
            p.dialog_generation = p
                .dialog_generation
                .checked_add(1)
                .ok_or(ActionError::QueueFull)?;
            let token = p.dialog_generation;
            p.prayer = Some(State::Confirm { token, bone, slot:selected });
            p.protection.suspended = true;
            self.events.push(WorldEvent::Prayer {
                actor,
                event: Event::Confirm { token },
            });
            return Ok(());
        }
        self.animate_bury(actor, bone, selected)
    }
    fn animate_bury(&mut self, actor: ActorId, bone: i32, slot: Option<usize>) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        let config = self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        let mut pending=if let Some(slot)=slot {
            if self.prayer_member_bones.contains(&bone) && !p.member {return Err(ActionError::InvalidItem);}
            config.prepare_selected(&p.data,&self.object_types,bone,slot)
        } else {config.prepare(&p.data,bone)}.map_err(|_|ActionError::InvalidItem)?;
        let ash_animation=slot.and_then(|_|self.prayer_ashes.get(&bone)).copied();
        if ash_animation.is_some(){pending.restore=0;}
        p.prayer = Some(State::Bury{pending,slot});
        p.protection.suspended = true;
        p.protection.delay(1);
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Animation(ash_animation.unwrap_or(config.animation)),
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
            slot,
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
            self.animate_bury(actor, bone, slot)?;
        }
        Ok(())
    }
    /// Start Prayer.offerOnAltar after the caller has verified location reach
    /// and its content gates. The existing player delay/cancellation machinery
    /// owns each three-cycle suspension. This is not a network action itself.
    pub fn begin_altar_offering(
        &mut self,
        actor: ActorId,
        bone: i32,
        animation: u16,
    ) -> Result<(), ActionError> {
        let xp = self
            .prayer
            .as_ref()
            .and_then(|c| c.bones.get(&bone))
            .copied()
            .ok_or(ActionError::InvalidItem)?;
        self.begin_offering(actor, bone, animation, Some(xp), None)
    }
    pub(super) fn begin_offering(
        &mut self,
        actor: ActorId,
        bone: i32,
        animation: u16,
        xp: Option<f64>,
        continuation: Option<crate::tower_offering::Continuation>,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        if self.prayer_ashes.contains_key(&bone){return Err(ActionError::InvalidItem);}
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        if !crate::prayer::contains(&p.data.inventory, bone) {
            self.events.push(WorldEvent::Prayer {
                actor,
                event: Event::OfferingFinished { offered: false },
            });
            return Ok(());
        }
        p.begin_held_action();
        p.prayer = Some(State::Offer {
            bone,
            animation,
            offered: false,
            xp,
            continuation,
        });
        p.protection.suspended = true;
        p.protection.current_clock = self.clock.0;
        p.protection.delay(3);
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Animation(animation),
        });
        Ok(())
    }
    pub(super) fn process_prayer(&mut self, slot: usize) {
        if self
            .players
            .get(slot)
            .is_some_and(|p| matches!(p.prayer, Some(State::CrateWalk { .. })))
        {
            self.process_tower_crate_walk(slot);
            return;
        }
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
        let (pending, selected, offering) = match p.prayer {
            Some(State::Bury{pending,slot}) => (pending, slot, None),
            Some(State::Offer {
                bone,
                animation,
                offered,
                xp,
                continuation,
            }) => (
                crate::prayer::Pending { bone, restore: 0 },
                None,
                Some((animation, offered, xp, continuation)),
            ),
            _ => return,
        };
        p.prayer = None;
        p.protection.suspended = false;
        let Some(config) = &self.prayer else { return };
        let mut tower_completion = None;
        let completed = if let Some((_, _, xp, _)) = offering {
            config.finish_offering_xp(&mut p.data, &self.object_types, pending.bone, xp)
        } else if let Some(slot)=selected {
            if p.pending_logout || p.has_modal || (self.prayer_member_bones.contains(&pending.bone) && !p.member) {Ok(None)}
            else {config.finish_selected(&mut p.data,&self.object_types,pending,slot)}
        } else {
            config.finish(&mut p.data, &self.object_types, pending)
        };
        match completed {
            Ok(Some(result)) => {
                if self.tutorial_rules_completed_varp.is_some() {
                    if p.stat_events
                        .advance(
                            &crate::tutorial_rules::bindings(),
                            config.skill,
                            result.advancement,
                        )
                        .is_err()
                    {
                        p.pending_logout = true;
                        self.events.push(WorldEvent::ActionRejected {
                            actor: p.id,
                            error: ActionError::QueueFull,
                        });
                    }
                }
                self.events.push(WorldEvent::Prayer {
                    actor: p.id,
                    event: Event::Completed {
                        skill: config.skill,
                        result,
                    },
                });
                self.events.push(WorldEvent::Prayer {
                    actor: p.id,
                    event: Event::Message(if offering.is_some() {
                        "The gods are pleased with your offering."
                    } else if selected.is_some() && self.prayer_ashes.contains_key(&pending.bone) {
                        "You scatter the ashes."
                    } else {
                        "You bury the bones."
                    }),
                });
                if let Some((animation, _, xp, continuation)) = offering {
                    if crate::prayer::contains(&p.data.inventory, pending.bone) {
                        p.prayer = Some(State::Offer {
                            bone: pending.bone,
                            animation,
                            offered: true,
                            xp,
                            continuation,
                        });
                        p.protection.suspended = true;
                        p.protection.delay(3);
                        self.events.push(WorldEvent::Prayer {
                            actor: p.id,
                            event: Event::Animation(animation),
                        });
                    } else {
                        tower_completion = continuation.map(|c| (p.id, c, true));
                        self.events.push(WorldEvent::Prayer {
                            actor: p.id,
                            event: Event::OfferingFinished { offered: true },
                        });
                    }
                }
            }
            Ok(None) => {
                if let Some((_, offered, _, continuation)) = offering {
                    tower_completion = continuation.map(|c| (p.id, c, offered));
                    self.events.push(WorldEvent::Prayer {
                        actor: p.id,
                        event: Event::OfferingFinished { offered },
                    });
                }
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::InvalidStat,
            }),
        }
        if let Some((actor, continuation, offered)) = tower_completion {
            self.finish_tower_offering(actor, continuation, offered);
        }
    }
}
