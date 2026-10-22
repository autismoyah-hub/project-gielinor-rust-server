//! Actor-owned scenery choices reuse dialogue tokens and authoritative movement.
use super::location_swap_actions::{ChoiceAfter, Continuation, Pending};
use super::*;
use crate::{location_content::Spiral, locations::Location};
impl World {
    pub(super) fn start_spiral_choice(&mut self, slot: usize, location: Location) {
        let Some(ty) = self.location_types.get(&location.id) else {
            return;
        };
        let destinations = [1, -1].map(|delta_level| {
            Spiral {
                delta_level,
                arrival_delay: false,
            }
            .destination(location, ty.width, ty.length)
            .ok()
        });
        self.start_location_choice(
            slot,
            location,
            "Climb up or down the stairs?",
            &["Climb up the stairs.", "Climb down the stairs."],
            destinations,
            ChoiceAfter::Jump,
        );
    }
    pub(super) fn start_windmill_choice(&mut self, slot: usize, location: Location) {
        let origin = self.players.get(slot).unwrap().movement.coords;
        let destinations = [1, -1].map(|level| origin.translate(0, 0, level).ok());
        self.start_location_choice(
            slot,
            location,
            "Climb up or down the ladder?",
            &["Climb Up.", "Climb Down."],
            destinations,
            ChoiceAfter::Climb {
                animation: 828,
                delay: 2,
            },
        );
    }
    pub(super) fn start_location_choice(
        &mut self,
        slot: usize,
        location: Location,
        title: &'static str,
        options: &'static [&'static str],
        destinations: [Option<CoordGrid>; 2],
        after: ChoiceAfter,
    ) {
        let p = self.players.get(slot).unwrap();
        let Some(revision) = self
            .locations
            .revision(location)
            .filter(|v| p.location_revision == Some(*v))
        else {
            return;
        };
        if p.saving
            || p.pending_logout
            || p.has_modal
            || (p.death_queued || p.data.stats.current(3) as u8 == 0)
            || options.len() != 2
        {
            return;
        }
        if let ChoiceAfter::Climb { delay, .. } = after {
            if delay <= 0 || self.clock.0 < 0 || self.clock.0.checked_add(delay).is_none() {
                return;
            }
        }
        let p = self.players.get_mut(slot).unwrap();
        let Some(token) = p.dialog_generation.checked_add(1) else {
            return;
        };
        p.dialog_generation = token;
        p.dialogue = None;
        p.pending_location_action = Some(Pending {
            dialogue_modal: None,
            location,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            action: Continuation::Choice {
                token,
                destinations,
                after,
            },
        });
        p.protection.suspended = true;
        self.events.push(WorldEvent::Dialogue {
            actor: p.id,
            event: crate::dialogue::Event::LocationChoice {
                token,
                title,
                options,
            },
        });
    }
    pub(super) fn location_choice_token(&self, actor: ActorId) -> Option<u64> {
        let p = self.players.get(actor.slot as usize)?;
        if p.id != actor {
            return None;
        }
        match p.pending_location_action?.action {
            Continuation::Choice { token, .. } | Continuation::Message { token } => Some(token),
            _ => None,
        }
    }
    /// A successfully projected page owns only its exact main-modal interface.
    /// Opening an unrelated modal must still invalidate the continuation.
    pub fn location_dialogue_presented(
        &mut self,
        actor: ActorId,
        token: u64,
        interface: u16,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        if self.location_choice_token(actor) == Some(token) {
            self.players
                .get_mut(actor.slot as usize)
                .unwrap()
                .pending_location_action
                .as_mut()
                .unwrap()
                .dialogue_modal = Some(interface);
        }
        Ok(())
    }
    pub(super) fn validate_location_choice(&mut self, slot: usize) {
        let Some(p) = self.players.get(slot) else {
            return;
        };
        let Some(pending) = p.pending_location_action else {
            return;
        };
        if !matches!(
            pending.action,
            Continuation::Choice { .. } | Continuation::Message { .. }
        ) {
            return;
        }
        let invalid = p.saving
            || p.pending_logout
            || (p.has_modal
                && (pending.dialogue_modal.is_none() || p.main_modal != pending.dialogue_modal))
            || (p.death_queued || p.data.stats.current(3) as u8 == 0)
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
        let message = matches!(
            p.pending_location_action.unwrap().action,
            Continuation::Message { .. }
        );
        if message {
            if !(0..=1).contains(&choice) {
                return Ok(());
            }
            let p = self.players.get_mut(actor.slot as usize).unwrap();
            p.pending_location_action = None;
            p.protection.suspended = false;
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: crate::dialogue::Event::Closed,
            });
            return Ok(());
        }
        if !(1..=2).contains(&choice) {
            return Ok(());
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let pending = p.pending_location_action.take().unwrap();
        p.protection.suspended = false;
        let Continuation::Choice {
            destinations,
            after,
            ..
        } = pending.action
        else {
            unreachable!()
        };
        if let ChoiceAfter::Banknote { slot, note } = after {
            return self.finish_banknote_choice(actor, pending, slot, note, choice == 1);
        }
        self.events.push(WorldEvent::Dialogue {
            actor,
            event: crate::dialogue::Event::Closed,
        });
        if let Some(destination) = destinations[(choice - 1) as usize] {
            match after {
                ChoiceAfter::Banknote { .. } => {
                    unreachable!("banknotes do not select a destination")
                }
                ChoiceAfter::Jump => self.finish_location_climb(
                    actor.slot as usize,
                    pending.location,
                    pending.origin,
                    destination,
                ),
                ChoiceAfter::Climb { animation, delay } => self.start_fixed_location_climb(
                    actor.slot as usize,
                    pending.location,
                    destination,
                    animation,
                    delay,
                ),
            }
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
    fn scenery_choice_accepts_unsigned_high_hitpoints_but_death_remains_terminal() {
        use crate::hits::{Hit, Style, Target};
        for mill in [false, true] {
            let (mut w, a, l) = if mill { windmill_fixture() } else { fixture() };
            w.data_mut(a).unwrap().stats.set_current(3, 150u8 as i8);
            let token = if mill {
                open_windmill(&mut w, a, l)
            } else {
                open(&mut w, a, l)
            };
            w.queue_hit(
                Target::Player(a),
                Hit {
                    display: None,
                    source: None,
                    style: Style::Typeless,
                    damage: 255,
                },
                0,
            )
            .unwrap();
            w.tick().unwrap();
            assert!(w
                .drain_events()
                .iter()
                .any(|e| matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)));
            w.data_mut(a).unwrap().stats.set_current(3, 10);
            let origin = w.data(a).unwrap().coords;
            w.submit(a, Action::ResumeDialogue { token, choice: 1 })
                .unwrap();
            w.tick().unwrap();
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            assert_eq!(w.data(a).unwrap().coords, origin);
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
        }
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
    fn windmill_fixture() -> (World, ActorId, Location) {
        let mut w = World::default();
        for level in 0..4 {
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, level);
                }
            }
        }
        let l = Location::new(CoordGrid::new(3204, 3204, 1).unwrap(), 12965, 10, 0).unwrap();
        w.locations.load(l);
        w.location_types.insert(
            l.id,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
        );
        w.location_handlers
            .insert((l.id, 1), Handler::WindmillChoice);
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Mill".into(),
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
    fn open_windmill(w: &mut World, a: ActorId, l: Location) -> u64 {
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
        let t = w.dialogue_token(a).unwrap().unwrap();
        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:crate::dialogue::Event::LocationChoice{title:"Climb up or down the ladder?",options,..},..}if *options==["Climb Up.","Climb Down."])));
        t
    }
    #[test]
    fn windmill_choice_selected_actor_relative_landing_waits_two_ticks_and_replay_cannot_change_it()
    {
        for choice in [1, 2] {
            let (mut w, a, l) = windmill_fixture();
            let origin = w.data(a).unwrap().coords;
            let inv = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let token = open_windmill(&mut w, a, l);
            w.submit(a, Action::ResumeDialogue { token, choice })
                .unwrap();
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(w.data(a).unwrap().coords, origin);
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            let events = w.drain_events();
            assert!(events
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerAnimation { sequence: 828, .. })));
            assert!(!events
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
            w.submit(
                a,
                Action::ResumeDialogue {
                    token,
                    choice: 3 - choice,
                },
            )
            .unwrap();
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(w.data(a).unwrap().coords, origin);
            w.drain_events();
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(
                w.data(a).unwrap().coords,
                origin
                    .translate(0, 0, if choice == 1 { 1 } else { -1 })
                    .unwrap()
            );
            let events = w.drain_events();
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::LocationClimbed { .. }))
                    .count(),
                1
            );
            assert!(!events
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerAnimation { .. })));
            assert_eq!(w.data(a).unwrap().inventory, inv);
            assert_eq!(w.data(a).unwrap().stats, stats);
        }
    }
    #[test]
    fn windmill_choice_close_or_post_selection_held_stale_logout_and_modal_cancel_without_landing()
    {
        for mode in 0..5 {
            let (mut w, a, l) = windmill_fixture();
            let token = open_windmill(&mut w, a, l);
            if mode == 0 {
                assert!(w.cancel_dialogue(a).unwrap());
            } else {
                w.submit(a, Action::ResumeDialogue { token, choice: 1 })
                    .unwrap();
                w.tick_with_random(&mut Never).unwrap();
                match mode {
                    1 => {
                        w.submit(a, Action::EatItemSlot { slot: 0, item: 999 })
                            .unwrap();
                    }
                    2 => {
                        w.change_location(l, false).unwrap();
                        w.change_location(l, true).unwrap();
                    }
                    3 => {
                        w.request_logout(a).unwrap();
                    }
                    _ => {
                        w.players.get_mut(a.slot as usize).unwrap().has_modal = true;
                    }
                }
            }
            w.drain_events();
            for _ in 0..4 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
        }
    }
}
