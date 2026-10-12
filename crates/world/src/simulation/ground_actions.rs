use super::*;
use crate::ground_transfer::{self, Effect, Pickup, Taken};
impl World {
    pub(super) fn request_pickup(
        &mut self,
        actor: ActorId,
        coords: CoordGrid,
        id: u16,
        control: bool,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if coords.level() != p.movement.coords.level() {
            return Err(ActionError::WrongPlane);
        }
        let observer = p.observer_id.ok_or(ActionError::Protected)?;
        if self.pickup_effects.is_none() {
            return Err(ActionError::InvalidItem);
        }
        let Some(object) = self
            .objects
            .at(coords)
            .find(|h| {
                let value = h.snapshot();
                value.item == id && value.visible_to(Some(observer))
            })
            .cloned()
        else {
            return Ok(());
        };
        let route = coordinate_route(
            &self.collision,
            p.movement.coords,
            coords,
            p.movement.size,
            true,
            25,
            Strategy::Normal,
        );
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.pending_door = None;
        p.prayer = None;
        p.pending_pickup = None;
        p.pending_drop = None;
        p.location_interaction = None;
        p.bank_count = None;
        p.food_delay = false;
        p.protection.suspended = false;
        p.object_interaction = Some(object);
        p.npc_interaction = None;
        p.movement.interacting = true;
        p.movement.temporary_speed = if control {
            Some(if p.active_speed == Speed::Run || p.data.energy < 100 {
                Speed::Walk
            } else {
                Speed::Run
            })
        } else {
            None
        };
        p.movement.route(route.waypoints, p.active_speed);
        self.events.push(WorldEvent::FaceLocation {
            actor,
            coords,
            width: 1,
            length: 1,
        });
        Ok(())
    }
    pub(super) fn validate_pickup(&mut self, slot: usize) {
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        if p.saving || p.protection.access_protected(p.has_modal) {
            return;
        }
        if let Some(handle) = &p.object_interaction {
            let object = handle.snapshot();
            // Kotlin validates by visible type at the tile, retaining the original handle.
            if !self.objects.at(object.coords).any(|h| {
                let current = h.snapshot();
                current.item == object.item && current.visible_to(p.observer_id)
            }) {
                p.object_interaction = None;
                p.movement.interacting = false;
                p.movement.abort();
                self.events.push(WorldEvent::ClearMapFlag(p.id));
            }
        }
    }
    pub(super) fn resume_pickup(&mut self, slot: usize) {
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
        let Some(pickup) = p.pending_pickup.take() else {
            return;
        };
        p.protection.suspended = false;
        if let Some(effects) = self.pickup_effects {
            self.events.push(WorldEvent::GroundAction {
                actor: p.id,
                effect: Effect::Animation(effects.animation),
            });
            self.events.push(WorldEvent::GroundAction {
                actor: p.id,
                effect: Effect::Sound(effects.sound),
            });
        }
        self.finish_pickup(slot, pickup);
    }
    fn finish_pickup(&mut self, slot: usize, pickup: Pickup) {
        let p = self.players.get_mut(slot).unwrap();
        let result = pickup.finish(
            &mut self.objects,
            &mut p.data.inventory,
            &self.object_types,
            self.clock,
        );
        let message = match result {
            Ok(Taken::Added(_)) => None,
            Ok(Taken::Gone) => Some(ground_transfer::TAKEN),
            Ok(Taken::RemovedWithoutSpace) => Some(ground_transfer::NO_SPACE),
            Err(_) => Some(ground_transfer::NO_SPACE),
        };
        if let Some(message) = message {
            self.events.push(WorldEvent::GroundAction {
                actor: p.id,
                effect: Effect::Message(message),
            });
        }
    }
    pub(super) fn process_pickup(&mut self, slot: usize) {
        let Some(p) = self.players.get(slot) else {
            return;
        };
        if p.saving
            || p.protection.access_protected(p.has_modal)
            || p.movement.last_movement >= self.clock.0
        {
            return;
        }
        let Some(object) = p.object_interaction.as_ref() else {
            return;
        };
        let value = object.snapshot();
        let target = crate::reach::Rectangle {
            coords: value.coords,
            width: 1,
            length: 1,
            angle: 0,
            blocked: 0,
            exclusive: false,
        };
        let reached = target.reached(&self.collision, p.movement.coords, p.movement.size);
        if !reached && p.movement.destination().is_some() {
            return;
        }
        let actor = p.id;
        if p.movement.destination().is_none() {
            self.events.push(WorldEvent::ClearMapFlag(actor));
        }
        let p = self.players.get_mut(slot).unwrap();
        let object = p.object_interaction.take().unwrap();
        p.movement.interacting = false;
        p.movement.abort();
        if !reached {
            self.events.push(WorldEvent::ClearMapFlag(actor));
            self.events.push(WorldEvent::CannotReach(actor));
            return;
        }
        let Some(observer) = p.observer_id else {
            return;
        };
        let respawn = self
            .object_respawns
            .get(&value.item)
            .copied()
            .unwrap_or(i32::MAX);
        let pickup = Pickup::prepare(
            &object,
            observer,
            p.movement.coords,
            &p.data.inventory,
            &self.object_types,
            self.clock,
            respawn,
        );
        match pickup {
            Ok(pickup) => {
                self.events.push(WorldEvent::GroundAction {
                    actor,
                    effect: Effect::Animation(u16::MAX),
                });
                if pickup.ready_at() > self.clock.0 {
                    p.pending_pickup = Some(pickup);
                    p.protection.suspended = true;
                    p.protection.delay(1);
                } else {
                    if let Some(effects) = self.pickup_effects {
                        self.events.push(WorldEvent::GroundAction {
                            actor,
                            effect: Effect::Sound(effects.sound),
                        });
                    }
                    self.finish_pickup(slot, pickup);
                }
            }
            Err(ground_transfer::Error::NoSpace) => self.events.push(WorldEvent::GroundAction {
                actor,
                effect: Effect::Message(ground_transfer::NO_SPACE),
            }),
            Err(_) => {}
        }
        self.events.push(WorldEvent::ClearMapFlag(actor));
    }
}
