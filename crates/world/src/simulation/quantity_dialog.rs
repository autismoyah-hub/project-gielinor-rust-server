use super::*;
impl World {
    /// Allocate a one-use protected continuation for skillmulti-style dialogs.
    /// The host validates the expected modal and pause-button permission first.
    pub fn start_quantity_dialog(&mut self, actor: ActorId) -> Result<u64, ActionError> {
        self.start_quantity_continuation(actor, false)
    }
    /// Host has validated the open modal and the button permission. A numeric
    /// input may suspend inside that modal; ordinary skillmulti access may not.
    pub fn start_modal_count_dialog(&mut self, actor: ActorId) -> Result<u64, ActionError> {
        self.start_quantity_continuation(actor, true)
    }
    fn start_quantity_continuation(
        &mut self,
        actor: ActorId,
        number: bool,
    ) -> Result<u64, ActionError> {
        let p = self.player(actor)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if if number {
            p.protection.modal_button_protected()
        } else {
            p.protection.access_protected(p.has_modal)
        } {
            return Err(ActionError::Protected);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let token = p
            .dialog_generation
            .checked_add(1)
            .ok_or(ActionError::QueueFull)?;
        p.begin_held_action();
        p.dialog_generation = token;
        p.quantity_dialog = Some(token);
        p.quantity_input = number;
        p.protection.suspended = true;
        Ok(token)
    }
    pub fn quantity_dialog_token(&self, actor: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self.player(actor)?.quantity_dialog)
    }
    pub fn cancel_quantity_dialog(&mut self, actor: ActorId) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let canceled = p.quantity_dialog.take().is_some();
        p.quantity_input = false;
        if canceled {
            p.protection.suspended = false;
        }
        Ok(canceled)
    }
    pub fn resume_quantity_dialog(
        &mut self,
        actor: ActorId,
        token: u64,
    ) -> Result<bool, ActionError> {
        let p = self.player(actor)?;
        if p.quantity_dialog != Some(token) {
            return Ok(false);
        }
        if p.protection.delayed() {
            self.cancel_quantity_dialog(actor)?;
            return Err(ActionError::Delayed);
        }
        self.cancel_quantity_dialog(actor)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_repeated_and_delayed_quantity_continuations_do_not_resume_new_work() {
        let mut world = World::default();
        world
            .queue_login(
                1,
                PlayerData::new(
                    1,
                    "Maker".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                ),
            )
            .unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        let old = world.start_quantity_dialog(actor).unwrap();
        assert!(world.close_dialogs(actor).unwrap());
        let new = world.start_quantity_dialog(actor).unwrap();
        assert_ne!(old, new);
        assert!(!world.resume_quantity_dialog(actor, old).unwrap());
        world.delay_player(actor, 2).unwrap();
        assert_eq!(
            world.resume_quantity_dialog(actor, new),
            Err(ActionError::Delayed)
        );
        assert_eq!(world.quantity_dialog_token(actor).unwrap(), None);
        assert!(!world.protection(actor).unwrap().suspended);
        for _ in 0..3 {
            world.tick().unwrap();
        }
        assert!(!world.resume_quantity_dialog(actor, new).unwrap());
        let latest = world.start_quantity_dialog(actor).unwrap();
        assert!(world.resume_quantity_dialog(actor, latest).unwrap());
        assert!(!world.resume_quantity_dialog(actor, latest).unwrap());
        assert!(!world.protection(actor).unwrap().suspended);
        world.set_modal_open(actor, true).unwrap();
        assert_eq!(
            world.start_quantity_dialog(actor),
            Err(ActionError::Protected)
        );
        let number = world.start_modal_count_dialog(actor).unwrap();
        assert!(world.cancel_count_dialog(actor).unwrap());
        assert!(!world.resume_quantity_dialog(actor, number).unwrap());
        world.set_modal_open(actor, false).unwrap();
        let pause = world.start_quantity_dialog(actor).unwrap();
        assert!(!world.cancel_count_dialog(actor).unwrap());
        assert_eq!(world.quantity_dialog_token(actor).unwrap(), Some(pause));
        assert!(world.resume_quantity_dialog(actor, pause).unwrap());
    }
}
