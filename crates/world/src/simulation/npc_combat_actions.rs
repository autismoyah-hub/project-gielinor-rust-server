use super::*;
use crate::{
    hits::{Hit, Style, Target},
    npc::Restrict,
    reach::Rectangle,
};
impl World {
    fn reset_npc_combat(&mut self, id: NpcId) {
        if let Some(npc) = self.npc_mut(id) {
            npc.attacking_player = None;
            npc.escape_player = None;
            npc.face_player(None);
            npc.movement.interacting = false;
            npc.movement.abort();
        }
    }
    /// Returns true when the combat mode owns this NPC's movement for this tick.
    pub(super) fn process_npc_combat(&mut self, id: NpcId, random: &mut impl RandomSource) -> bool {
        let Some(config) = &self.npc_melee else {
            return false;
        };
        let Some(npc) = self.npc(id) else {
            return false;
        };
        let Some(definition) = config.definitions.get(&npc.type_id).copied() else {
            return false;
        };
        let Some(combat) = self.npc_combat.get(&npc.type_id).cloned() else {
            return false;
        };
        if npc.stats[3] <= 0 {
            self.reset_npc_combat(id);
            return true;
        }
        let clock = self.clock.0;
        let due = npc.retaliate_due.front().is_some_and(|due| *due <= clock);
        if due {
            let npc = self.npc_mut(id).unwrap();
            while npc.retaliate_due.front().is_some_and(|due| *due <= clock) {
                npc.retaliate_due.pop_front();
            }
            let can_retaliate = crate::npc_route::can_retaliate(
                npc.combat_action_delay,
                clock,
                npc.attacking_player.is_some() || npc.escape_player.is_some(),
            );
            let target = can_retaliate.then_some(npc.aggressive_player).flatten();
            if let Some(target) = target {
                if let Ok(p) = self.player(target) {
                    let coords = p.movement.coords;
                    let npc = self.npc_mut(id).unwrap();
                    npc.combat_action_delay = clock.wrapping_add(combat.attack_rate / 2);
                    npc.movement.abort();
                    let out_of_range = (coords.x() - npc.spawn.x())
                        .abs()
                        .max((coords.z() - npc.spawn.z()).abs())
                        > combat.aggression_range;
                    if npc.stats[3] <= combat.retreat || (npc.wander_radius > 0 && out_of_range) {
                        npc.escape_player = Some(target);
                        npc.attacking_player = None;
                    } else {
                        npc.attacking_player = Some(target);
                        npc.escape_player = None;
                    }
                    npc.movement.interacting = true;
                    npc.face_player(Some(target));
                }
            }
        }
        let npc = self.npc(id).unwrap();
        let Some(target) = npc.escape_player.or(npc.attacking_player) else {
            return false;
        };
        let Ok(p) = self.player(target) else {
            self.reset_npc_combat(id);
            return true;
        };
        let coords = p.movement.coords;
        if p.saving
            || p.movement.hidden
            || p.data.stats.current(3) as u8 == 0
            || coords.level() != npc.movement.coords.level()
        {
            self.reset_npc_combat(id);
            return true;
        }
        if npc.escape_player.is_some() {
            let origin = npc.movement.coords;
            if (origin.x() - coords.x())
                .abs()
                .max((origin.z() - coords.z()).abs())
                > 25
                || npc.restrict == Restrict::NoMove
            {
                self.reset_npc_combat(id);
                return true;
            }
            if let Some(dest) = crate::npc_route::retreat_destination(
                origin,
                npc.spawn,
                coords,
                definition.retreat_range,
            ) {
                if origin == dest {
                    return true;
                }
                let npc = &mut self.npcs.get_mut(id.slot as usize).unwrap().npc;
                npc.movement.route([dest], Speed::Walk);
                npc.process_movement(&mut self.collision, clock);
                if npc.movement.coords == origin
                    && clock.wrapping_sub(npc.movement.last_movement) > 3
                {
                    self.reset_npc_combat(id);
                }
            }
            return true;
        }
        if (npc.spawn.x() - coords.x())
            .abs()
            .max((npc.spawn.z() - coords.z()).abs())
            > combat.aggression_range
        {
            self.reset_npc_combat(id);
            return true;
        }
        let source = npc.movement.coords;
        let size = npc.movement.size;
        let overlap = coords.x() >= source.x()
            && coords.x() < source.x() + size
            && coords.z() >= source.z()
            && coords.z() < source.z() + size;
        let rect = Rectangle {
            coords,
            width: 1,
            length: 1,
            angle: 0,
            blocked: 0,
            exclusive: true,
        };
        if overlap {
            let (dx, dz) = [(0, -1), (0, 1), (-1, 0), (1, 0)][random.below(4) as usize];
            if let Ok(dest) = source.translate(dx, dz, 0) {
                let npc = &mut self.npcs.get_mut(id.slot as usize).unwrap().npc;
                npc.movement.route([dest], Speed::Walk);
                npc.process_movement(&mut self.collision, clock);
            }
            return true;
        }
        if !rect.reached(&self.collision, source, size) {
            let destination =
                crate::npc_route::entity_destination(source, (size, size), coords, (1, 1), 0)
                    .unwrap_or(coords);
            let npc = &mut self.npcs.get_mut(id.slot as usize).unwrap().npc;
            npc.movement.route([destination], Speed::Walk);
            npc.process_movement(&mut self.collision, clock);
            if npc.movement.coords != source && !definition.give_chase {
                self.reset_npc_combat(id);
                return true;
            }
        }
        let npc = self.npc(id).unwrap();
        if !rect.reached(&self.collision, npc.movement.coords, size) {
            return true;
        }
        if let Err(error) = self.npc_attack_player(id, target, random) {
            self.events
                .push(WorldEvent::NpcCombatError { actor: id, error });
            self.reset_npc_combat(id);
        }
        true
    }
    fn npc_attack_player(
        &mut self,
        source: NpcId,
        target: ActorId,
        random: &mut impl RandomSource,
    ) -> Result<(), ActionError> {
        let npc = self.npc(source).ok_or(ActionError::StaleActor)?;
        let p = self.player(target)?;
        let multi = self
            .multiway_area
            .is_some_and(|id| self.areas.at(npc.movement.coords).contains(&id));
        let uid = crate::combat_state::npc_uid(source.slot, npc.type_id);
        let player_busy = if let Some(config) = self.combat_state {
            config.denied(&p.data.variables, self.clock.0, uid)
        } else {
            p.last_pvp_combat.wrapping_add(8) >= self.clock.0
                || (p.last_npc_combat.wrapping_add(8) >= self.clock.0
                    && p.aggressive_npc.is_some_and(|id| id != source))
        };
        if !multi && player_busy {
            return Err(ActionError::Protected);
        }
        if npc.combat_action_delay > self.clock.0 {
            return Ok(());
        }
        let config = self.npc_melee.as_ref().ok_or(ActionError::InvalidItem)?;
        let definition = *config
            .definitions
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        let melee = self.melee.as_ref().ok_or(ActionError::InvalidItem)?;
        let stance = p.data.variables.get(melee.stance.id);
        let (_, style) = self
            .attack_profiles
            .selection(p.data.equipment.get(3).map(|i| i.id), stance)
            .map_err(|_| ActionError::InvalidItem)?;
        let inputs = config
            .inputs(npc, &p.data, melee, style, self.clock.0)
            .map_err(|_| ActionError::InvalidItem)?;
        let defend = config.defend_animation(&p.data);
        let mark = config.player_mark;
        let rate = self
            .npc_combat
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?
            .attack_rate;
        let clock = self.clock.0;
        let npc = self.npc_mut(source).unwrap();
        npc.combat_action_delay = clock.wrapping_add(rate);
        npc.last_attack = clock;
        self.events.push(WorldEvent::NpcCombatAnimation {
            actor: source,
            sequence: definition.animation,
        });
        if let Some(sound) = definition.sound {
            self.events.push(WorldEvent::CombatSound {
                actor: target,
                sound,
            });
        }
        let roll = inputs.roll(random).map_err(|_| ActionError::InvalidStat)?;
        let p = self.players.get_mut(target.slot as usize).unwrap();
        p.aggressive_npc = Some(source);
        p.last_npc_combat = clock;
        if let Some(config) = self.combat_state {
            let updates = config.record(
                &mut p.data.variables,
                clock,
                uid,
                p.protection.processed_clock > 0,
            );
            if !updates.is_empty() {
                self.events.push(WorldEvent::CombatVariables {
                    actor: target,
                    updates,
                });
            }
        }
        p.retaliate_npcs.push_back((clock, source));
        self.queue_hit(
            Target::Player(target),
            Hit {
                source: Some(Target::Npc(source)),
                style: Style::Melee,
                damage: roll.damage,
                display: Some(mark.build(roll.damage)),
            },
            1,
        )
        .map_err(|_| ActionError::QueueFull)?;
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: target,
            sequence: defend,
        });
        Ok(())
    }
    pub(super) fn process_player_retaliation(&mut self, actor: ActorId) {
        loop {
            let clock = self.clock.0;
            let Ok(p) = self.player(actor) else {
                return;
            };
            let Some(&(due, source)) = p.retaliate_npcs.front() else {
                return;
            };
            if due > clock || p.protection.access_protected(p.has_modal) {
                return;
            }
            if p.data.stats.current(3) as u8 == 0 {
                return;
            }
            let source_exists = self.npc(source).is_some();
            let p = self.players.get_mut(actor.slot as usize).unwrap();
            p.retaliate_npcs.pop_front();
            if !source_exists {
                continue;
            }
            let Some(config) = &self.combat_stance else {
                continue;
            };
            if p.data.variables.get(config.retaliate.id) == 1
                || p.movement.interacting
                || p.movement.destination().is_some()
            {
                continue;
            }
            let stance = p.data.variables.get(config.stance.id);
            let weapon = p.data.equipment.get(3).map(|i| i.id);
            let style = self
                .attack_profiles
                .selection(weapon, stance)
                .ok()
                .and_then(|(_, style)| style);
            let speed = weapon
                .and_then(|id| self.attack_profiles.weapons.get(&id))
                .map_or(4, |w| w.speed);
            if p.action_delay < clock {
                p.action_delay =
                    clock.wrapping_add(speed / 2 - if style == Some(6) { 1 } else { 0 });
            }
            let _ = self.request_npc(actor, source, 2, false);
        }
    }
}
