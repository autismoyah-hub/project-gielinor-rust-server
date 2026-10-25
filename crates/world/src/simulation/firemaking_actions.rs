//! Ordinary firemaking start paths (tinderbox on logs, logs on tinderbox, "Light"
//! on ground logs) and the queued lighting attempt from OpenRune BurnLogEvents.
use super::*;
use crate::firemaking::{Refusal, ATTEMPT, ATTEMPT_TICKS, CATCHES, QUEUE};
use crate::ground::{Handle, Object, Scope};
#[derive(Clone, Debug)]
pub(super) struct Job {
    log: i32,
    object: Handle,
}
impl World {
    /// canBurn for the player in `slot` lighting `log` on `tile`; members logs on a
    /// free account resolve to the engine's "Nothing interesting happens.".
    fn firemaking_refusal(&self, slot: usize, log: i32, tile: CoordGrid) -> Option<Refusal> {
        let c = self.firemaking.as_ref()?;
        let p = self.players.get(slot)?;
        let entry = c.logs.get(&log)?;
        let has_tinderbox = p
            .data
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| i.id == c.tinderbox);
        c.check(
            entry,
            has_tinderbox,
            p.data.stats.current(c.skill) as u8,
            crate::firemaking::tile_open(&self.locations, &self.collision, tile),
        )
        .err()
    }
    fn refuse_firemaking(&mut self, actor: ActorId, log: i32, refusal: Refusal) {
        let Some(c) = self.firemaking.as_ref() else {
            return;
        };
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: c.refusal_text(log, refusal),
        });
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor,
            sequence: u16::MAX,
        });
    }
    fn members_log_refused(&mut self, slot: usize, log: i32) -> bool {
        let p = self.players.get(slot).unwrap();
        let members = self
            .firemaking
            .as_ref()
            .and_then(|c| c.logs.get(&log))
            .is_some_and(|l| l.members);
        if members && !p.member {
            self.events.push(WorldEvent::EngineMessage {
                actor: p.id,
                message: "Nothing interesting happens.",
            });
            return true;
        }
        false
    }
    /// Tinderbox and logs used together in either direction: the log is dropped
    /// on the player's tile and lit there.
    pub(super) fn start_firemaking_held(&mut self, actor: ActorId, log: i32) {
        let slot = actor.slot as usize;
        let Some(c) = self.firemaking.as_ref() else {
            return;
        };
        let (object_ticks, clock) = (c.object_ticks, self.clock);
        if self.members_log_refused(slot, log) {
            return;
        }
        let coords = self.players.get(slot).unwrap().movement.coords;
        if let Some(refusal) = self.firemaking_refusal(slot, log, coords) {
            self.refuse_firemaking(actor, log, refusal);
            return;
        }
        let p = self.players.get_mut(slot).unwrap();
        if p.data
            .inventory
            .delete(&self.object_types, crate::inventory::Delete::new(log, 1))
            .is_err()
        {
            return;
        }
        let added = self.objects.add(
            Object {
                coords,
                item: log as u16,
                count: 1,
                scope: Scope::Temporary,
                created: clock.0,
                receiver: None,
                owner: None,
            },
            &self.object_types,
            clock,
            object_ticks,
            i32::MAX,
        );
        match added {
            Ok(mut handles) if !handles.is_empty() => self.begin_firemaking(slot, log, handles.remove(0)),
            Ok(_) => {}
            Err(error) => self.events.push(WorldEvent::GroundError(error)),
        }
    }
    /// Ground logs "Light" option: routed like Take, then lit where they lie.
    pub(super) fn request_light_object(
        &mut self,
        actor: ActorId,
        coords: CoordGrid,
        id: u16,
        control: bool,
    ) -> Result<(), ActionError> {
        if !self
            .firemaking
            .as_ref()
            .is_some_and(|c| c.logs.contains_key(&i32::from(id)))
        {
            return Err(ActionError::InvalidItem);
        }
        self.request_pickup(actor, coords, id, control)?;
        self.players.get_mut(actor.slot as usize).unwrap().object_light = true;
        Ok(())
    }
    /// process_pickup reached logs whose interaction is "Light".
    pub(super) fn start_firemaking_ground(&mut self, slot: usize, object: Handle) {
        let value = object.snapshot();
        let log = i32::from(value.item);
        let actor = self.players.get(slot).unwrap().id;
        if self.members_log_refused(slot, log) {
            return;
        }
        if let Some(refusal) = self.firemaking_refusal(slot, log, value.coords) {
            self.refuse_firemaking(actor, log, refusal);
            return;
        }
        self.begin_firemaking(slot, log, object);
    }
    fn begin_firemaking(&mut self, slot: usize, log: i32, object: Handle) {
        let Some(c) = self.firemaking.as_ref() else {
            return;
        };
        let animation = c.animation;
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        p.queues.clear_weak();
        p.queues.remove_all(QUEUE);
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                ATTEMPT_TICKS,
                Deferred::Firemaking(Job { log, object }),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor,
            sequence: animation,
        });
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: ATTEMPT.into(),
        });
    }
    /// One lighting attempt: re-check canBurn, roll, then either re-queue or
    /// replace the logs with a fire, award XP and step away from it.
    pub(super) fn complete_firemaking(
        &mut self,
        actor: ActorId,
        job: Job,
        random: &mut impl RandomSource,
    ) {
        if self.player(actor).is_err() {
            return;
        }
        let Some(c) = self.firemaking.as_ref() else {
            return;
        };
        let (fire_id, fire_ticks, skill, max_level) = (c.fire, c.fire_ticks, c.skill, c.max_level);
        let slot = actor.slot as usize;
        let p = self.players.get(slot).unwrap();
        if p.pending_logout
            || p.saving
            || p.has_modal
            || p.death_queued
            || p.data.stats.current(3) as u8 == 0
            || p.location_interaction.is_some()
            || p.npc_interaction.is_some()
            || p.object_interaction.is_some()
        {
            return;
        }
        let level = p.data.stats.current(skill) as u8;
        let value = job.object.snapshot();
        // ObjRegistry.isValid: the dropped logs must still lie there for this player.
        if !self.objects.at(value.coords).any(|h| h.id() == job.object.id())
            || !value.visible_to(p.observer_id)
        {
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: u16::MAX,
            });
            return;
        }
        if let Some(refusal) = self.firemaking_refusal(slot, job.log, value.coords) {
            self.refuse_firemaking(actor, job.log, refusal);
            return;
        }
        let threshold = self.firemaking.as_ref().unwrap().success_threshold(level);
        if f64::from(random.below(256)) >= threshold {
            let p = self.players.get_mut(slot).unwrap();
            if p.queues
                .add(
                    QUEUE,
                    crate::queues::Category::Weak,
                    ATTEMPT_TICKS,
                    Deferred::Firemaking(job),
                )
                .is_err()
            {
                self.events.push(WorldEvent::ActionRejected {
                    actor,
                    error: ActionError::QueueFull,
                });
            }
            return;
        }
        self.objects.delete(&job.object, self.clock, i32::MAX);
        let fire = match crate::locations::Location::new(value.coords, fire_id, 10, 0) {
            Ok(fire) => fire,
            Err(error) => {
                self.events.push(WorldEvent::LocationError(error));
                return;
            }
        };
        let span = (fire_ticks[1] - fire_ticks[0]).max(0) as u32 + 1;
        let duration = fire_ticks[0] + random.below(span) as i32;
        match self.change_location_for(fire, true, duration) {
            Ok(crate::locations::Changed::Spawned) => self.firemaking_fires.lit(fire),
            Ok(_) => {}
            Err(error) => {
                self.events.push(WorldEvent::LocationError(error));
                return;
            }
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor,
            sequence: u16::MAX,
        });
        let xp = f64::from(self.firemaking.as_ref().unwrap().logs[&job.log].xp_tenths) / 10.0;
        let step = crate::firemaking::step_away(&self.collision, fire.coords);
        let tutorial = self.tutorial_rules_completed_varp.is_some();
        let p = self.players.get_mut(slot).unwrap();
        match crate::skills::advance(
            &mut p.data.stats,
            skill,
            xp,
            p.data.xp_rate_hundreds as f64 / 100.0,
            max_level,
        ) {
            Ok(advancement) => {
                if tutorial
                    && p.stat_events
                        .advance(&crate::tutorial_rules::bindings(), skill, advancement)
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
                    skill,
                    advancement,
                });
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidStat,
            }),
        }
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: CATCHES.into(),
        });
        // Player.walk forces walking for this route without changing the run preference.
        if let Some(dest) = step {
            let waypoints = if p.movement.coords == fire.coords {
                vec![dest]
            } else {
                coordinate_route(
                    &self.collision,
                    p.movement.coords,
                    dest,
                    p.movement.size,
                    false,
                    25,
                    Strategy::Normal,
                )
                .waypoints
            };
            p.movement.temporary_speed = None;
            p.movement.route(waypoints, Speed::Walk);
        }
        self.events.push(WorldEvent::FaceLocation {
            actor,
            coords: fire.coords,
            width: 1,
            length: 1,
        });
    }
    /// A firemaking fire that timed out leaves ashes on its tile.
    pub(super) fn fire_burned_out(&mut self, loc: crate::locations::Location) {
        if !self.firemaking_fires.expired(loc) || self.locations.find_type(loc.coords, loc.id).is_some() {
            return;
        }
        let Some(c) = self.firemaking.as_ref() else {
            return;
        };
        let (ashes, ticks) = (c.ashes, c.object_ticks);
        if let Err(error) = self.objects.add(
            Object {
                coords: loc.coords,
                item: ashes as u16,
                count: 1,
                scope: Scope::Temporary,
                created: self.clock.0,
                receiver: None,
                owner: None,
            },
            &self.object_types,
            self.clock,
            ticks,
            i32::MAX,
        ) {
            self.events.push(WorldEvent::GroundError(error));
        }
    }
}

#[cfg(test)]
#[path = "firemaking_tests.rs"]
mod tests;
