use super::*;
impl World {
    pub fn set_main_modal(
        &mut self,
        actor: ActorId,
        interface: Option<u16>,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        self.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .main_modal = interface;
        Ok(())
    }
    pub(super) fn start_tutorial_fire(
        &mut self,
        actor: ActorId,
        config: crate::tutorial_fire::Config,
    ) {
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if let Some(pending) = config.begin(
            &mut p.data,
            &self.object_types,
            p.movement.coords,
            p.main_modal,
        ) {
            p.tutorial_fire = Some(pending);
            p.protection.current_clock = self.clock.0;
            p.protection.delay(4);
            p.protection.suspended = true;
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: config.animation,
            });
        }
    }
    pub(super) fn resume_tutorial_fire(&mut self, slot: usize) {
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
        let Some(pending) = p.tutorial_fire.take() else {
            return;
        };
        p.protection.suspended = false;
        // ProtectedAccess.delay resumes only with the same main-modal identity.
        if p.main_modal != pending.main_modal {
            return;
        }
        let actor = p.id;
        let config = pending.config;
        let advancement = match crate::skills::advance(
            &mut p.data.stats,
            config.skill,
            40.0,
            p.data.xp_rate_hundreds as f64 / 100.0 * config.global_xp_rate,
            config.max_level,
        ) {
            Ok(value) => value,
            Err(_) => {
                self.events.push(WorldEvent::ActionRejected {
                    actor,
                    error: ActionError::InvalidStat,
                });
                return;
            }
        };
        if self.tutorial_rules_completed_varp.is_some()
            && p.stat_events
                .advance(
                    &crate::tutorial_rules::bindings(),
                    config.skill,
                    advancement,
                )
                .is_err()
        {
            p.pending_logout = true;
            self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::QueueFull,
            });
        }
        self.events.push(WorldEvent::SkillAdvanced {
            actor,
            skill: config.skill,
            advancement,
        });
        // Player.walk replaces route destinations and forces walking, without
        // changing the persistent run preference or teleporting off blocked tiles.
        if let Ok(west) = pending.coords.translate(-1, 0, 0) {
            p.movement.temporary_speed = None;
            p.movement.route([west], Speed::Walk);
        }
        let fire = match crate::locations::Location::new(pending.coords, config.location, 10, 0) {
            Ok(fire) => fire,
            Err(error) => {
                self.events.push(WorldEvent::LocationError(error));
                return;
            }
        };
        if let Err(error) = self.change_location_for(fire, true, 200) {
            self.events.push(WorldEvent::LocationError(error));
            return;
        }
        self.events.push(WorldEvent::LocationMessage {
            actor,
            message: "The fire catches and the logs begin to burn.",
        });
        let p = self.players.get_mut(slot).unwrap();
        let effects = config.progression.advance(
            &mut p.data.variables,
            crate::tutorial_steps::Step::SurvivalCook,
            None,
            p.protection.processed_clock > 0,
        );
        self.events
            .push(WorldEvent::TutorialInventoryProgress { actor, effects });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request, Scope, Type},
        inventory::{Insert, Item},
        loc_collision::LocCollision,
        tutorial_fire::Config,
        tutorial_progression,
        variables::{Transmit, Varp},
    };
    fn setup() -> (World, ActorId, Config, Request) {
        let mut w = World::default();
        for x in (3192..3216).step_by(8) {
            for z in (3192..3216).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let config = Config {
            progression: tutorial_progression::Config {
                step: varp(1),
                completed: varp(2),
            },
            logs: 11,
            location: 2,
            animation: 733,
            skill: 11,
            max_level: 99,
            global_xp_rate: 1.0,
        };
        for id in [10, 11] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        w.held_use_handlers
            .register(Scope::Types, 10, 11, Handler::TutorialFire(config))
            .unwrap();
        w.location_types.insert(
            2,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
        let mut p = PlayerData::new(
            1,
            "Firemaker".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            crate::stats::Stats::default(),
        );
        p.variables.set(varp(1), 11, false);
        for id in [10, 11] {
            p.inventory
                .insert(&w.object_types, Insert::new(id, 1))
                .unwrap();
        }
        w.queue_login(1, p).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.drain_events();
        let request = Request {
            selected_slot: 0,
            selected_id: 10,
            target_slot: 1,
            target_id: 11,
        };
        (w, actor, config, request)
    }
    fn begin(w: &mut World, actor: ActorId, r: Request) {
        assert!(w.prepare_held_use(actor, r).unwrap().is_some());
        w.finish_held_use(actor, r).unwrap();
    }
    #[test]
    fn delay_then_walk_fire_xp_progress_and_exact_duration() {
        let (mut w, actor, config, r) = setup();
        let origin = w.data(actor).unwrap().coords;
        let start = w.clock.0;
        begin(&mut w, actor, r);
        assert!(w.data(actor).unwrap().inventory.get(1).is_none());
        assert_eq!(w.data(actor).unwrap().stats.xp(11), 0);
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 733, .. })));
        while w.clock.0 < start + 4 {
            w.tick().unwrap();
            assert!(w.locations.find_type(origin, config.location).is_none());
            assert_eq!(w.data(actor).unwrap().coords, origin);
        }
        w.tick().unwrap();
        assert_eq!(
            w.data(actor).unwrap().coords,
            origin.translate(-1, 0, 0).unwrap()
        );
        assert_eq!(w.data(actor).unwrap().stats.xp(11), 40);
        assert_eq!(w.data(actor).unwrap().variables.get(1), 12);
        assert!(w.locations.find_type(origin, config.location).is_some());
        // Location timers run in WorldPostTick after Late increments the clock.
        while w.clock.0 < start + 203 {
            w.tick().unwrap();
            w.drain_events();
        }
        assert!(w.locations.find_type(origin, config.location).is_some());
        w.tick().unwrap();
        assert_eq!(w.clock.0, start + 204);
        assert!(w.locations.find_type(origin, config.location).is_none());
    }
    #[test]
    fn lenient_held_input_cancels_fire_but_not_its_delay_or_log_consumption() {
        let (mut w, actor, config, r) = setup();
        let origin = w.data(actor).unwrap().coords;
        begin(&mut w, actor, r);
        let until = w.protection(actor).unwrap().delay_until;
        w.submit(actor, Action::EatSlot { slot: 0 }).unwrap();
        w.tick().unwrap();
        assert_eq!(w.protection(actor).unwrap().delay_until, until);
        assert!(w.protection(actor).unwrap().delayed());
        for _ in 0..6 {
            w.tick().unwrap();
        }
        assert!(w.locations.find_type(origin, config.location).is_none());
        assert_eq!(w.data(actor).unwrap().stats.xp(11), 0);
        assert!(w.data(actor).unwrap().inventory.get(1).is_none());
    }
    #[test]
    fn only_changed_main_modal_loses_delayed_access() {
        for main in [None, Some(12)] {
            let (mut w, actor, config, r) = setup();
            let origin = w.data(actor).unwrap().coords;
            begin(&mut w, actor, r);
            w.set_modal_open(actor, true).unwrap();
            w.set_main_modal(actor, main).unwrap();
            for _ in 0..6 {
                w.tick().unwrap();
            }
            assert_eq!(
                w.locations.find_type(origin, config.location).is_some(),
                main.is_none()
            );
            assert_eq!(
                w.data(actor).unwrap().stats.xp(11),
                if main.is_none() { 40 } else { 0 }
            );
            assert!(!w.protection(actor).unwrap().suspended);
        }
    }
    #[test]
    fn early_progress_keeps_logs_and_blocked_west_does_not_prevent_fire() {
        let (mut w, actor, config, r) = setup();
        w.data_mut(actor)
            .unwrap()
            .variables
            .set(config.progression.step, 10, true);
        begin(&mut w, actor, r);
        assert_eq!(
            w.data(actor).unwrap().inventory.get(1),
            Some(Item {
                id: 11,
                count: 1,
                vars: 0
            })
        );
        assert!(!w.protection(actor).unwrap().delayed());
        w.data_mut(actor)
            .unwrap()
            .variables
            .set(config.progression.step, 11, true);
        let origin = w.data(actor).unwrap().coords;
        let west = origin.translate(-1, 0, 0).unwrap();
        w.collision.add(west.x(), west.z(), west.level(), 0x200000);
        begin(&mut w, actor, r);
        for _ in 0..6 {
            w.tick().unwrap();
        }
        assert!(w.locations.find_type(origin, config.location).is_some());
        assert_eq!(w.data(actor).unwrap().stats.xp(11), 40);
        assert_eq!(w.data(actor).unwrap().coords, origin);
    }
}
