use super::*;
use crate::{doors, loc_reach::Target, locations::Location, reach::Rectangle};

impl World {
    /// Shared admission check for network sessions and headless actors.
    pub fn has_location_handler(&self, id: u32, operation: u8) -> bool {
        if self.facility_crafting.as_ref().is_some_and(|c|c.locations.get(&id)==Some(&operation)){return true;}
        if self
            .furnaces
            .get(&id)
            .is_some_and(|furnace| furnace.supports(operation))
        {
            return true;
        }
        (operation == 1
            && self
                .mining
                .as_ref()
                .is_some_and(|c| c.rocks.contains_key(&id)))
            || (matches!(operation, 1 | 3)
                && self
                    .woodcutting
                    .as_ref()
                    .is_some_and(|c| c.trees.contains_key(&id)))
            || (operation == 1
                && (self.tower_staff.as_ref().is_some_and(|c| c.location == id)
                    || self.tower_chest.as_ref().is_some_and(|c| c.location == id)
                    || self.anvils.contains(&id)
                    || self.tutorial_bank.is_some_and(|c| c.booth(id).is_some())
                    || self.doors.contains_key(&id)
                    || self.tutorial_bread.is_some_and(|c| c.location == id)
                    || self.tutorial_woodcutting.is_some_and(|c| c.contains(id))
                    || self.tutorial_mining.is_some_and(|c| c.ore(id).is_some())
                    || self
                        .tutorial_gates
                        .as_ref()
                        .is_some_and(|c| c.gates.contains_key(&id))))
            || self
                .milking
                .as_ref()
                .is_some_and(|c| c.cows.get(&id) == Some(&operation))
            || self.location_handlers.contains_key(&(id, operation))
    }
    fn door_target(&self, loc: Location) -> Result<Target, ActionError> {
        let ty = self
            .location_types
            .get(&loc.id)
            .ok_or(ActionError::InvalidLocation)?;
        let target = Target {
            rectangle: Rectangle {
                coords: loc.coords,
                width: ty.width as i32,
                length: ty.length as i32,
                angle: loc.angle,
                blocked: self.location_approach.get(&loc.id).copied().unwrap_or(0),
                exclusive: false,
            },
            shape: loc.shape as i8,
        };
        if !target.valid() {
            return Err(ActionError::InvalidLocation);
        }
        Ok(target)
    }

    pub(super) fn request_door(
        &mut self,
        actor: ActorId,
        coords: CoordGrid,
        id: u32,
        control: bool,
    ) -> Result<(), ActionError> {
        self.request_location(actor, coords, id, 1, control)
    }
    pub(super) fn request_location(
        &mut self,
        actor: ActorId,
        coords: CoordGrid,
        id: u32,
        operation: u8,
        control: bool,
    ) -> Result<(), ActionError> {
        self.request_location_action(
            actor,
            coords,
            id,
            LocationAction::Operation(operation),
            control,
        )
    }
    pub(super) fn request_location_use(
        &mut self,
        actor: ActorId,
        coords: CoordGrid,
        id: u32,
        slot: i32,
        object: i32,
        control: bool,
    ) -> Result<(), ActionError> {
        self.request_location_action(
            actor,
            coords,
            id,
            LocationAction::HeldUse { slot, object },
            control,
        )
    }
    fn request_location_action(
        &mut self,
        actor: ActorId,
        coords: CoordGrid,
        id: u32,
        action: LocationAction,
        control: bool,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if coords.level() != p.movement.coords.level() {
            return Err(ActionError::WrongPlane);
        }
        let Some(loc) = self.locations.find_type(coords, id) else {
            self.events.push(WorldEvent::ClearMapFlag(actor));
            return Ok(());
        };
        if let LocationAction::Operation(operation) = action {
            if !self.has_location_handler(id, operation) {
                return Err(ActionError::InvalidLocation);
            }
        }
        let target = self.door_target(loc)?;
        let route = crate::routing::location_route(
            &self.collision,
            p.movement.coords,
            target,
            p.movement.size,
            true,
            25,
            Strategy::Normal,
        );
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.queues.remove_all(crate::milking::QUEUE);
        p.queues.remove_all(crate::water_sources::QUEUE);
        p.queues.remove_all(crate::grinding::QUEUE);
        p.queues.remove_all(crate::dough::QUEUE);
        p.dough_request = None;
        p.pending_door = None;
        p.prayer = None;
        p.pending_pickup = None;
        p.pending_drop = None;
        p.object_interaction = None;
        p.npc_interaction = None;
        p.food_delay = false;
        p.tutorial_fire = None;
        p.quantity_dialog = None;
        p.quantity_input = false;
        p.bank_count = None;
        p.protection.suspended = false;
        p.location_revision = self.locations.revision(loc);
        p.location_interaction = Some((loc, action));
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
        let (width, length) = target.rectangle.dimensions();
        self.events.push(WorldEvent::FaceLocation {
            actor,
            coords,
            width: width as u8,
            length: length as u8,
        });
        Ok(())
    }

    // Resume before movement, matching the protected coroutine stage in PlayerMain.
    pub(super) fn resume_door(&mut self, slot: usize) {
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
        let Some(pending) = p.pending_door.take() else {
            return;
        };
        p.protection.suspended = false;
        self.finish_door(pending);
    }

    pub(super) fn validate_door(&mut self, slot: usize) {
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        if p.saving || p.protection.access_protected(p.has_modal) {
            return;
        }
        if let Some((loc, _)) = p.location_interaction {
            if self.locations.find_type(loc.coords, loc.id) != Some(loc) {
                p.location_interaction = None;
                p.movement.interacting = false;
                p.movement.abort();
                self.events.push(WorldEvent::ClearMapFlag(p.id));
            }
        }
    }

    fn finish_door(&mut self, pending: doors::Pending) {
        let catalog = std::mem::take(&mut self.doors);
        let result = doors::finish(self, &catalog, pending);
        self.doors = catalog;
        if let Err(error) = result {
            self.events.push(WorldEvent::LocationError(error));
        }
    }

    pub(super) fn process_door(&mut self, slot: usize, random: &mut impl RandomSource) {
        let Some(p) = self.players.get(slot) else {
            return;
        };
        if p.saving || p.protection.access_protected(p.has_modal) {
            return;
        }
        let Some((loc, action)) = p.location_interaction else {
            return;
        };
        let operation = match action {
            LocationAction::Operation(op) => op,
            LocationAction::HeldUse { .. } => 0,
        };
        let actor = p.id;
        if self.locations.find_type(loc.coords, loc.id) != Some(loc) {
            let p = self.players.get_mut(slot).unwrap();
            p.location_interaction = None;
            p.movement.interacting = false;
            p.movement.abort();
            return;
        }
        // Static targets cannot execute an op in a cycle with movement.
        if p.movement.last_movement >= self.clock.0 {
            return;
        }
        let Ok(target) = self.door_target(loc) else {
            return;
        };
        let (width, length) = target.rectangle.dimensions();
        let source = p.movement.coords;
        let overlap = source.x() < loc.coords.x() + width
            && source.x() + p.movement.size > loc.coords.x()
            && source.z() < loc.coords.z() + length
            && source.z() + p.movement.size > loc.coords.z();
        let reached = overlap || target.reached(&self.collision, source, p.movement.size);
        if !reached && p.movement.destination().is_some() {
            return;
        }
        // FloorOneScript.prayAltar calls arriveDelay, unlike takeVials. Preserve
        // its additional protected cycle when the actor moved last tick.
        if reached
            && p.movement.last_movement == self.clock.0 - 1
            && self
                .location_handlers
                .get(&(loc.id, operation))
                .is_some_and(|h| h.arrival_delay())
        {
            let p = self.players.get_mut(slot).unwrap();
            p.movement.abort();
            p.protection.delay(1);
            return;
        }
        if reached
            && operation == 1
            && self
                .mining
                .as_ref()
                .is_some_and(|c| c.rocks.contains_key(&loc.id))
        {
            self.process_mining(slot, loc, random);
            return;
        }
        if reached
            && self
                .woodcutting
                .as_ref()
                .is_some_and(|c| c.trees.contains_key(&loc.id))
        {
            self.process_woodcutting(slot, loc, action, random);
            return;
        }
        if p.movement.destination().is_none() {
            self.events.push(WorldEvent::ClearMapFlag(actor));
        }
        let p = self.players.get_mut(slot).unwrap();
        p.location_interaction = None;
        p.movement.interacting = false;
        p.movement.abort();
        if !reached {
            self.events.push(WorldEvent::ClearMapFlag(actor));
            self.events.push(WorldEvent::CannotReach(actor));
            return;
        }
        if let LocationAction::HeldUse {
            slot: inv_slot,
            object,
        } = action
        {
            self.process_location_use(slot, loc, inv_slot, object);
            return;
        }
        if self
            .milking
            .as_ref()
            .is_some_and(|c| c.cows.get(&loc.id) == Some(&operation))
        {
            self.start_milking(slot, loc, None);
            return;
        }
        if operation == 1
            && self
                .tutorial_gates
                .as_ref()
                .is_some_and(|c| c.gates.contains_key(&loc.id))
        {
            self.process_tutorial_gate(slot, loc);
            return;
        }
        if operation == 1
            && self
                .tutorial_woodcutting
                .is_some_and(|c| c.contains(loc.id))
        {
            self.process_tutorial_woodcutting(slot, loc.id);
            return;
        }
        if operation == 1 && self.tutorial_bread.is_some_and(|c| c.location == loc.id) {
            self.process_tutorial_bread(slot);
            return;
        }
        if operation == 1 {
            if let Some(ore) = self.tutorial_mining.and_then(|c| c.ore(loc.id)) {
                self.process_tutorial_mining(slot, ore);
                return;
            }
        }
        if operation == 1 {
            if let Some((config, booth)) = self
                .tutorial_bank
                .and_then(|c| c.booth(loc.id).map(|b| (c, b)))
            {
                if let Some(message) = config.denial(&p.data.variables, booth) {
                    self.events
                        .push(WorldEvent::LocationMessage { actor, message });
                } else {
                    if booth == crate::tutorial_bank::Booth::Bank {
                        config.grant(&mut p.data.bank, &self.object_types);
                    }
                    self.events.push(WorldEvent::TutorialBank {
                        actor,
                        booth,
                        next: config.next(&p.data.variables, booth),
                    });
                }
                return;
            }
        }
        if operation == 1
            && self
                .tower_chest
                .as_ref()
                .is_some_and(|c| c.location == loc.id)
        {
            if let Err(error) = self.claim_tower_chest(actor) {
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
            }
            return;
        }
        if operation == 1
            && self
                .tower_staff
                .as_ref()
                .is_some_and(|c| c.location == loc.id)
        {
            if let Err(error) = self.search_tower_staff(actor) {
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
            }
            return;
        }
        if self.facility_crafting.as_ref().is_some_and(|c|c.locations.get(&loc.id)==Some(&operation)) {
            self.request_facility_crafting(slot,loc,None);return;
        }
        if let Some(&handler) = self.location_handlers.get(&(loc.id, operation)) {
            match handler.apply(&mut p.data, &self.object_types) {
                Ok(Some(message)) => self
                    .events
                    .push(WorldEvent::LocationMessage { actor, message }),
                Ok(None) => (),
                Err(_) => self.events.push(WorldEvent::LocationError(
                    "Location inventory transaction failed",
                )),
            }
            return;
        }
        if self
            .furnaces
            .get(&loc.id)
            .is_some_and(|furnace| furnace.supports(operation))
        {
            self.process_smelting(slot, loc.id);
            return;
        }
        if operation == 1 && self.anvils.contains(&loc.id) {
            self.events.push(WorldEvent::AnvilRequested {
                actor,
                preferred: None,
            });
            return;
        }
        let Some(&door) = self.doors.get(&loc.id).filter(|_| operation == 1) else {
            return;
        };
        let started = match doors::begin(loc, door, source) {
            Ok(started) => started,
            Err(error) => {
                self.events.push(WorldEvent::LocationError(error));
                return;
            }
        };
        self.events.push(WorldEvent::DoorSound {
            actor,
            sound: started.sound,
        });
        if let Some(coords) = started.step_away {
            if let Err(error) =
                p.movement
                    .teleport(&mut self.collision, coords, self.clock.0, false)
            {
                self.events.push(WorldEvent::LocationError(error));
                return;
            }
            p.data.coords = coords;
            p.protection.delay(2);
            p.protection.suspended = true;
            p.pending_door = Some(started.pending);
        } else {
            self.finish_door(started.pending);
            self.events.push(WorldEvent::ClearMapFlag(actor));
        }
    }
}
