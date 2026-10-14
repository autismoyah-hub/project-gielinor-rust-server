use super::*;

/// Captures ctrl-speed before modal-close callbacks, as MoveGameClickHandler does.
/// Fields are private so only validated input can cross the callback boundary.
pub struct MovementInput {
    actor: ActorId,
    destination: CoordGrid,
    temporary: Option<Speed>,
}
impl World {
    pub fn prepare_movement_input(
        &self,
        actor: ActorId,
        destination: CoordGrid,
        control: bool,
    ) -> Result<Option<MovementInput>, ActionError> {
        let p = self.player(actor)?;
        if !self.movement_input_allowed(actor)? {
            return Ok(None);
        }
        let temporary = control.then(|| {
            if p.active_speed == Speed::Run || p.data.energy < 100 {
                Speed::Walk
            } else {
                Speed::Run
            }
        });
        Ok(Some(MovementInput {
            actor,
            destination,
            temporary,
        }))
    }

    /// Called after ifClose has cancelled dialogs/weak work and run modal hooks.
    /// Do not clear weak queues again: a close hook may have scheduled new work.
    pub fn finish_movement_input(&mut self, input: MovementInput) -> Result<(), ActionError> {
        let actor = input.actor;
        self.cancel_location_target_action(actor)?;
        let p = self.player(actor)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if input.destination.level() != p.movement.coords.level() {
            return Err(ActionError::WrongPlane);
        }
        if (input.destination.x() - p.movement.coords.x()).abs() > 64
            || (input.destination.z() - p.movement.coords.z()).abs() > 64
        {
            return Err(ActionError::OutOfRange);
        }
        let route = coordinate_route(
            &self.collision,
            p.movement.coords,
            input.destination,
            p.movement.size,
            true,
            25,
            Strategy::Normal,
        );
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.movement.temporary_speed = input.temporary;
        p.movement.route(route.waypoints, p.active_speed);
        self.events.push(WorldEvent::ResetFaceEntity { actor });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn close_callback_work_survives_and_ctrl_speed_uses_pre_callback_state() {
        let mut world = World::default();
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        world.collision.allocate(3200, 3200, 0);
        world
            .queue_login(
                1,
                PlayerData::new(1, "Walker".into(), coords, Stats::default()),
            )
            .unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        let input = world
            .prepare_movement_input(actor, coords.translate(2, 0, 0).unwrap(), true)
            .unwrap()
            .unwrap();
        world.close_dialogs(actor).unwrap();
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        p.queues
            .add(900, crate::queues::Category::Weak, 10, Deferred::Preserve)
            .unwrap();
        p.set_active_speed(Speed::Run);
        p.protection.suspended = true;
        world.finish_movement_input(input).unwrap();
        let p = world.player(actor).unwrap();
        assert!(p.queues.contains(900));
        assert!(!p.protection.suspended);
        assert_eq!(p.movement.temporary_speed, Some(Speed::Run));
    }
}
