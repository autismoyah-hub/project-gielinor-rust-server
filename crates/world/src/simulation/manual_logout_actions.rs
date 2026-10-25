//! Logout-panel button (LogoutTabScript): the host closes dialogs and modals
//! (Player.ifClose), then [`World::request_manual_logout`] is the protected part
//! (ProtectedAccess.logOut). PlayerLogoutProcess.processManualLogout runs in the
//! PlayerLogout phase and queues the ordinary pending logout and save unless combat
//! (`logout_blocked_until`, the reference preventLogoutUntil) prevents it.
use super::*;

/// The reference preventLogout message; every source of `logout_blocked_until`
/// in this world is a player hit, where the reference sets this text.
pub const COMBAT_LOGOUT_MESSAGE: &str =
    "You can't log out until 10 seconds after the end of combat.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManualLogout {
    /// Marked; the PlayerLogout phase decides this tick.
    Requested,
    /// Access protected (delayed or suspended): the launch is refused silently.
    Protected,
    /// Already saving, pending, marked or disconnected (canLogout is false).
    AlreadyLoggingOut,
}

impl World {
    pub fn request_manual_logout(&mut self, id: ActorId) -> Result<ManualLogout, ActionError> {
        self.player(id)?;
        let clock = self.clock.0;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        if p.protection.access_protected(p.has_modal) {
            return Ok(ManualLogout::Protected);
        }
        // isBusy2 && mapClock > preventLogoutUntil: playerWalk(coords) stops the
        // interaction and route, as a walk to the current tile does.
        let busy = p.movement.interacting
            || p.location_interaction.is_some()
            || p.object_interaction.is_some()
            || p.npc_interaction.is_some()
            || p.movement.waypoint_count() > 0;
        if busy && clock > p.logout_blocked_until {
            if p.pending_location_action.take().is_some() {
                p.protection.suspended = false;
            }
            p.movement.interacting = false;
            p.location_interaction = None;
            p.object_interaction = None;
            p.npc_interaction = None;
            p.movement.abort();
        }
        if p.saving || p.pending_logout || p.manual_logout || p.disconnect_grace.disconnected() {
            return Ok(ManualLogout::AlreadyLoggingOut);
        }
        p.manual_logout = true;
        Ok(ManualLogout::Requested)
    }
    /// PlayerLogoutProcess.processManualLogout, after disconnect grace and before the
    /// pending logouts are processed.
    pub(super) fn process_manual_logouts(&mut self) {
        let marked: Vec<_> = self
            .players
            .iter()
            .filter(|(_, p)| p.manual_logout)
            .map(|(_, p)| p.id)
            .collect();
        for id in marked {
            let p = self.players.get_mut(id.slot as usize).unwrap();
            p.manual_logout = false;
            if self.clock.0 <= p.logout_blocked_until {
                self.events.push(WorldEvent::GameMessage {
                    actor: id,
                    text: COMBAT_LOGOUT_MESSAGE.into(),
                });
                continue;
            }
            self.request_logout(id).expect("current player");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> (World, ActorId) {
        let mut world = World::default();
        world.collision.allocate(3200, 3200, 0);
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        world
            .queue_login(1, PlayerData::new(1, "Logout tab".into(), coords, Stats::default()))
            .unwrap();
        world.tick().unwrap();
        world.drain_events();
        let actor = world.actors().next().unwrap();
        (world, actor)
    }
    fn saves(events: &[WorldEvent]) -> usize {
        events.iter().filter(|e| matches!(e, WorldEvent::Save { .. })).count()
    }
    fn messages(events: &[WorldEvent]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| match e {
                WorldEvent::GameMessage { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_button_queues_exactly_one_protected_logout_and_save() {
        let (mut w, actor) = world();
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::Requested));
        // A second click before the PlayerLogout phase is already a pending logout.
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::AlreadyLoggingOut));
        w.tick().unwrap();
        let events = w.drain_events();
        assert_eq!(saves(&events), 1);
        assert!(messages(&events).is_empty());
        let p = w.players.get(actor.slot as usize).unwrap();
        assert!(p.saving && !p.manual_logout && !p.pending_logout);
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::AlreadyLoggingOut));
        w.tick().unwrap();
        assert_eq!(saves(&w.drain_events()), 0, "no second save while saving");
    }

    #[test]
    fn combat_defers_with_the_reference_message_until_the_block_passes() {
        let (mut w, actor) = world();
        let until = w.clock.0 + 16;
        w.players.get_mut(actor.slot as usize).unwrap().logout_blocked_until = until;
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::Requested));
        w.tick().unwrap();
        let events = w.drain_events();
        assert_eq!(saves(&events), 0);
        assert_eq!(messages(&events), [COMBAT_LOGOUT_MESSAGE]);
        let p = w.players.get(actor.slot as usize).unwrap();
        assert!(!p.manual_logout && !p.pending_logout && !p.saving, "the click is consumed");
        while w.clock.0 <= until {
            w.tick().unwrap();
        }
        w.drain_events();
        assert_eq!(saves(&w.drain_events()), 0, "no logout without a new click");
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::Requested));
        w.tick().unwrap();
        let events = w.drain_events();
        assert_eq!((saves(&events), messages(&events).len()), (1, 0));
    }

    #[test]
    fn protected_disconnected_and_stale_players_cannot_request() {
        let (mut w, actor) = world();
        let clock = w.clock.0;
        let p = w.players.get_mut(actor.slot as usize).unwrap();
        p.protection.current_clock = clock;
        p.protection.delay(5);
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::Protected));
        w.tick().unwrap();
        assert_eq!(saves(&w.drain_events()), 0);
        assert!(!w.players.get(actor.slot as usize).unwrap().manual_logout);

        let (mut w, actor) = world();
        w.players.get_mut(actor.slot as usize).unwrap().disconnect_grace.disconnect();
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::AlreadyLoggingOut));
        assert!(!w.players.get(actor.slot as usize).unwrap().manual_logout);

        let (mut w, actor) = world();
        let stale = ActorId {
            slot: actor.slot,
            generation: actor.generation + 1,
        };
        assert_eq!(w.request_manual_logout(stale), Err(ActionError::StaleActor));
        w.tick().unwrap();
        assert_eq!(saves(&w.drain_events()), 0);
    }

    #[test]
    fn a_busy_player_is_stopped_only_outside_the_combat_block() {
        let (mut w, actor) = world();
        let route = [CoordGrid::new(3205, 3200, 0).unwrap()];
        let p = w.players.get_mut(actor.slot as usize).unwrap();
        p.movement.route(route, crate::movement::Speed::Walk);
        p.movement.interacting = true;
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::Requested));
        let p = w.players.get(actor.slot as usize).unwrap();
        assert_eq!(p.movement.waypoint_count(), 0);
        assert!(!p.movement.interacting);

        let (mut w, actor) = world();
        let clock = w.clock.0;
        let p = w.players.get_mut(actor.slot as usize).unwrap();
        p.logout_blocked_until = clock + 16;
        p.movement.route(route, crate::movement::Speed::Walk);
        assert_eq!(w.request_manual_logout(actor), Ok(ManualLogout::Requested));
        assert_eq!(w.players.get(actor.slot as usize).unwrap().movement.waypoint_count(), 1);
    }
}
