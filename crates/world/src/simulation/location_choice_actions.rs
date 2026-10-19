//! Actor-owned scenery choices reuse dialogue tokens and authoritative movement.
use super::location_swap_actions::{Continuation, Pending};
use super::*;
use crate::{location_content::Spiral, locations::Location};
impl World {
    pub(super) fn start_spiral_choice(&mut self, slot: usize, location: Location) {
        let p = self.players.get(slot).unwrap();
        let Some(revision) = self
            .locations
            .revision(location)
            .filter(|v| p.location_revision == Some(*v))
        else {
            return;
        };
        if p.saving || p.pending_logout || p.has_modal || p.data.stats.current(3) <= 0 {
            return;
        }
        let Some(ty) = self.location_types.get(&location.id) else {
            return;
        };
        // The source constructs both translations before suspending; destinations
        // are resolved by the server, never accepted from a client reply.
        let destinations = [1, -1].map(|delta_level| {
            Spiral {
                delta_level,
                arrival_delay: false,
            }
            .destination(location, ty.width, ty.length)
            .ok()
        });
        let p = self.players.get_mut(slot).unwrap();
        let Some(token) = p.dialog_generation.checked_add(1) else {
            return;
        };
        p.dialog_generation = token;
        p.dialogue = None;
        p.pending_location_action = Some(Pending {
            location,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            action: Continuation::Choice {
                token,
                destinations,
            },
        });
        p.protection.suspended = true;
        self.events.push(WorldEvent::Dialogue {
            actor: p.id,
            event: crate::dialogue::Event::LocationChoice {
                token,
                title: "Climb up or down the stairs?",
                options: &["Climb up the stairs.", "Climb down the stairs."],
            },
        });
    }
    pub(super) fn location_choice_token(&self, actor: ActorId) -> Option<u64> {
        let p = self.players.get(actor.slot as usize)?;
        if p.id != actor {
            return None;
        }
        match p.pending_location_action?.action {
            Continuation::Choice { token, .. } => Some(token),
            _ => None,
        }
    }
    pub(super) fn validate_location_choice(&mut self, slot: usize) {
        let Some(p) = self.players.get(slot) else {
            return;
        };
        let Some(pending) = p.pending_location_action else {
            return;
        };
        if !matches!(pending.action, Continuation::Choice { .. }) {
            return;
        }
        let invalid = p.saving
            || p.pending_logout
            || p.has_modal
            || p.data.stats.current(3) <= 0
            || p.movement.coords != pending.origin
            || p.movement.last_movement != pending.last_movement
            || p.location_interaction.is_some()
            || p.npc_interaction.is_some()
            || p.object_interaction.is_some()
            || self.locations.revision(pending.location) != Some(pending.revision);
        if invalid {
            let p = self.players.get_mut(slot).unwrap();
            p.pending_location_action = None;
            p.protection.suspended = false;
            self.events.push(WorldEvent::Dialogue {
                actor: p.id,
                event: crate::dialogue::Event::Closed,
            });
        }
    }
    pub(super) fn resume_location_choice(
        &mut self,
        actor: ActorId,
        token: u64,
        choice: i32,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        if self.location_choice_token(actor) != Some(token) {
            return Ok(());
        }
        self.validate_location_choice(actor.slot as usize);
        let p = self.player(actor)?;
        if self.location_choice_token(actor) != Some(token) {
            return Ok(());
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if !(1..=2).contains(&choice) {
            return Ok(());
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let pending = p.pending_location_action.take().unwrap();
        p.protection.suspended = false;
        let Continuation::Choice { destinations, .. } = pending.action else {
            unreachable!()
        };
        self.events.push(WorldEvent::Dialogue {
            actor,
            event: crate::dialogue::Event::Closed,
        });
        if let Some(destination) = destinations[(choice - 1) as usize] {
            self.finish_location_climb(
                actor.slot as usize,
                pending.location,
                pending.origin,
                destination,
            );
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        loc_collision::LocCollision, location_content::Handler, npc::RandomSource, stats::Stats,
    };
    struct Never;
    impl RandomSource for Never {
        fn below(&mut self, _: u32) -> u32 {
            panic!("scenery choice must not roll")
        }
    }
    fn fixture() -> (World, ActorId, Location) {
        let mut w = World::default();
        for level in 0..4 {
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, level);
                }
            }
        }
        let l = Location::new(CoordGrid::new(3204, 3204, 1).unwrap(), 16672, 10, 0).unwrap();
        w.locations.load(l);
        w.location_types.insert(
            l.id,
            LocCollision {
                width: 2,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
        );
        w.location_handlers.insert((l.id, 1), Handler::SpiralChoice);
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Choice".into(),
                CoordGrid::new(3203, 3204, 1).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a, l)
    }
    fn open(w: &mut World, a: ActorId, l: Location) -> u64 {
        w.submit(
            a,
            Action::OperateLocation {
                coords: l.coords,
                id: l.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(&mut Never).unwrap();
        let token = w.dialogue_token(a).unwrap().unwrap();
        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:crate::dialogue::Event::LocationChoice{title:"Climb up or down the stairs?",options,..},..}if *options==["Climb up the stairs.","Climb down the stairs."])));
        token
    }
    #[test]
    fn spiral_choice_waits_without_npc_and_commits_only_selected_server_landing_once() {
        for (choice, expected) in [(1, (3206, 3204, 2)), (2, (3205, 3203, 0))] {
            let (mut w, a, l) = fixture();
            let origin = w.data(a).unwrap().coords;
            let inv = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let token = open(&mut w, a, l);
            assert!(w.players.get(a.slot as usize).unwrap().dialogue.is_none());
            for _ in 0..3 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert_eq!(w.data(a).unwrap().coords, origin);
            assert_eq!(w.dialogue_token(a).unwrap(), Some(token));
            for (bad_token, bad_choice) in [(token + 1, choice), (token, 0), (token, 3)] {
                w.submit(
                    a,
                    Action::ResumeDialogue {
                        token: bad_token,
                        choice: bad_choice,
                    },
                )
                .unwrap();
                w.tick_with_random(&mut Never).unwrap();
                assert_eq!(w.dialogue_token(a).unwrap(), Some(token));
                assert_eq!(w.data(a).unwrap().coords, origin);
            }
            w.drain_events();
            w.submit(a, Action::ResumeDialogue { token, choice })
                .unwrap();
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(
                w.data(a).unwrap().coords,
                CoordGrid::new(expected.0, expected.1, expected.2).unwrap()
            );
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            let e = w.drain_events();
            assert_eq!(
                e.iter()
                    .filter(|e| matches!(e, WorldEvent::LocationClimbed { .. }))
                    .count(),
                1
            );
            assert!(!e.iter().any(|e| matches!(
                e,
                WorldEvent::PlayerAnimation { .. } | WorldEvent::PlayerSound { .. }
            )));
            w.submit(a, Action::ResumeDialogue { token, choice })
                .unwrap();
            w.tick_with_random(&mut Never).unwrap();
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
            assert_eq!(w.data(a).unwrap().inventory, inv);
            assert_eq!(w.data(a).unwrap().stats, stats);
        }
    }
    #[test]
    fn spiral_choice_close_stale_held_move_logout_death_and_modal_cancel_without_replay() {
        for mode in 0..7 {
            let (mut w, a, l) = fixture();
            let token = open(&mut w, a, l);
            let origin = w.data(a).unwrap().coords;
            match mode {
                0 => {
                    assert!(w.cancel_dialogue(a).unwrap());
                }
                1 => {
                    w.change_location(l, false).unwrap();
                    w.change_location(l, true).unwrap();
                }
                2 => {
                    w.submit(a, Action::EatItemSlot { slot: 0, item: 999 })
                        .unwrap();
                }
                3 => {
                    let p = w.players.get_mut(a.slot as usize).unwrap();
                    p.movement.last_movement += 1;
                }
                4 => {
                    w.request_logout(a).unwrap();
                }
                5 => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .data
                        .stats
                        .set_current(3, 0);
                }
                _ => {
                    w.players.get_mut(a.slot as usize).unwrap().has_modal = true;
                }
            }
            w.tick_with_random(&mut Never).unwrap();
            if w.data(a).is_ok() {
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                let _ = w.submit(a, Action::ResumeDialogue { token, choice: 1 });
                w.tick_with_random(&mut Never).unwrap();
                if let Ok(p) = w.data(a) {
                    assert_eq!(p.coords, origin);
                }
            }
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
        }
    }
}
