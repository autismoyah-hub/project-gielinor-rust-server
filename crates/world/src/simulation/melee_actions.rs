use super::*;
use crate::{
    attack_profile::Attack,
    hits::{Hit, Style, Target},
};
impl World {
    pub(super) fn npc_combat_denial(
        &self,
        actor: ActorId,
        target: NpcId,
    ) -> Option<(Option<&'static str>, bool)> {
        let p = self.player(actor).ok()?;
        if let Some(message)=self.cinderworks_denial(actor,target){
            let shell=message==crate::tower_cinderworks::SHELL_HINT;
            return Some(((!shell).then_some(message),!shell));
        }
        if let Some(config) = &self.combat_stance {
            let weapon = p
                .data
                .equipment
                .get(3)
                .and_then(|i| config.weapons.get(&i.id));
            let stance = p.data.variables.get(config.stance.id);
            if weapon.is_some_and(|w| w.bulwark)
                && self
                    .attack_profiles
                    .selection(p.data.equipment.get(3).map(|i| i.id), stance)
                    .is_ok_and(|(_, style)| style == Some(3))
            {
                return Some((Some("Your bulwark gets in the way."), true));
            }
        }
        if p.queues.contains(3) {
            return Some((None, true));
        }
        let multi = self
            .multiway_area
            .is_some_and(|id| self.areas.at(p.movement.coords).contains(&id));
        if !multi {
            let player_busy = if let Some(config) = self.combat_state {
                config.denied(
                    &p.data.variables,
                    self.clock.0,
                    crate::combat_state::npc_uid(target.slot, self.npc(target)?.type_id),
                )
            } else {
                p.last_pvp_combat.wrapping_add(8) >= self.clock.0
                    || (p.last_npc_combat.wrapping_add(8) >= self.clock.0
                        && p.aggressive_npc.is_some_and(|id| id != target))
            };
            if player_busy {
                return Some((Some("I'm already under attack."), false));
            }
            let npc = self.npc(target)?;
            if npc.last_combat.wrapping_add(8) > self.clock.0
                && npc.aggressive_player.is_some_and(|source| source != actor)
            {
                return Some((Some("Someone else is fighting that."), false));
            }
        }
        None
    }
    /// Default PvN melee pipeline. Damage is generated exclusively by server RNG.
    /// Specialized weapon scripts and retaliation use separate dispatch hooks.
    pub fn attack_npc_melee(
        &mut self,
        actor: ActorId,
        target: NpcId,
        attack: Attack,
        random: &mut impl RandomSource,
    ) -> Result<bool, ActionError> {
        let Attack::Melee {
            kind,
            style,
            stance,
        } = attack
        else {
            return Err(ActionError::InvalidItem);
        };
        if let Some(config) = &self.special_attacks {
            config.validate().map_err(|_| ActionError::InvalidItem)?;
        }
        let p = self.player(actor)?;
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        if self.npc_combat_denial(actor, target).is_some() {
            return Err(ActionError::Protected);
        }
        let npc = self
            .npc(target)
            .filter(|n| n.visible && !n.delayed && n.stats[3] > 0)
            .ok_or(ActionError::StaleActor)?;
        if !self
            .npc_options
            .has_operation(npc.type_id, &p.data.variables, 2)
        {
            return Err(ActionError::InvalidItem);
        }
        if self.is_attack_delayed(actor)? {
            return Ok(false);
        }
        let profile = self
            .attack_profiles
            .profile(p.data.equipment.get(3).map(|i| i.id), stance as i32, 0)
            .map_err(|_| ActionError::InvalidItem)?;
        let source = p.movement.coords;
        let dest = npc.movement.coords;
        let size = npc.movement.size;
        let touching = crate::reach::Rectangle {
            coords: dest,
            width: size,
            length: size,
            angle: 0,
            blocked: 0,
            exclusive: true,
        }
        .reached(&self.collision, source, 1);
        let at_range = profile.range > 1
            && source.level() == dest.level()
            && source.x() >= dest.x() - profile.range
            && source.x() <= dest.x() + size - 1 + profile.range
            && source.z() >= dest.z() - profile.range
            && source.z() <= dest.z() + size - 1 + profile.range
            && crate::raycast::clear(
                &self.collision,
                crate::raycast::Bounds {
                    coords: source,
                    width: 1,
                    length: 1,
                },
                crate::raycast::Bounds {
                    coords: dest,
                    width: size,
                    length: size,
                },
                true,
                0,
            );
        if !touching && !at_range {
            return Err(ActionError::OutOfRange);
        }
        let speed = profile.speed;
        let config = self.melee.as_ref().ok_or(ActionError::InvalidItem)?;
        let definition = self
            .npc_combat
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        let mut input = config
            .inputs(&p.data, npc, definition, kind, style, speed, -1)
            .map_err(|_| ActionError::InvalidItem)?;
        let special_vars = self.special_attacks.as_ref().map(|c| (c.energy, c.armed));
        let armed = self.special_attacks.as_ref().is_some_and(|c| p.data.variables.get(c.armed.id) == 1);
        let special = self.special_attacks.as_ref().and_then(|c| {
            armed.then(|| p.data.equipment.get(3).and_then(|item| c.melee.get(&item.id).copied())).flatten()
        });
        let insufficient = special.zip(special_vars).is_some_and(|(s, (energy, _))| p.data.variables.get(energy.id) < s.cost);
        let special = special.filter(|_| !insufficient);
        if let Some(special) = special {
            // Cleave keeps offensive type/style, but blocks against Slash.
            input.defence_bonus = definition.defence[usize::from(special.block_type - 1)];
        }
        let fx = config.effects(p.data.equipment.get(3).map(|i| i.id), stance);
        let defend = config.npc_defend.get(&npc.type_id).copied();
        let mut mark = *config
            .npc_marks
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        let threshold = config
            .hit_threshold
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?
            .max(10);
        let max_hit_varp = config.max_hit;
        let prior_max_hit = p.data.variables.get(max_hit_varp.id);
        let rate = p.data.xp_rate_hundreds as f64 / 100.0;
        let uuid = p.data.character_id;
        let logged = p.protection.processed_clock > 0;
        self.set_next_attack_delay(actor, speed)?;
        // PvNCombat resets Weapon before attempting the registered handler. Low
        // energy returns to the ordinary attack under the same weapon delay.
        if armed {
            let (_, armed) = special_vars.unwrap();
            if let Some(update) = self.data_mut(actor)?.variables.set(armed, 0, logged) {
                self.events.push(WorldEvent::CombatVariables { actor, updates: vec![update] });
            }
        }
        if insufficient {
            self.events.push(WorldEvent::GameMessage { actor, text: "You don't have enough power left.".into() });
        }
        if let Some(s) = special {
            self.events.push(WorldEvent::PlayerCombatAnimation { actor, sequence: s.animation });
            self.events.push(WorldEvent::PlayerSpotAnimation { actor, spot: s.spot, delay: 0, height: s.spot_height, slot: s.spot_slot });
        }
        let roll = match special {
            Some(s) => input.roll_with_special(random, s.accuracy, s.max_hit),
            None => input.roll(random),
        }.map_err(|_| ActionError::InvalidStat)?;
        if let Some(max_hit) = roll.max_hit {
            let update = self
                .data_mut(actor)?
                .variables
                .set(max_hit_varp, max_hit, logged);
            if let Some(update) = update {
                self.events.push(WorldEvent::CombatVariables {
                    actor,
                    updates: vec![update],
                });
            }
        }
        self.give_npc_combat_xp(actor, target, attack, roll.damage, rate)?;
        if special.is_none() {
            self.events.push(WorldEvent::CombatAnimation { actor, sequence: fx.0, sound: fx.1 });
        }
        mark.source_threshold = threshold.max(roll.max_hit.unwrap_or(prior_max_hit));
        let clock = self.clock.0;
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        npc.aggressive_player = Some(actor);
        npc.retaliate_due.push_back(clock.wrapping_add(1));
        npc.last_combat = npc.last_combat.max(clock);
        // The reference applies flat armour after awarding rolled-damage XP.
        let damage = self.modify_npc_combat_damage(Some(actor), target, roll.damage)?;
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        npc.hero_points.credit(uuid, damage.min(npc.stats[3]));
        let hit = Hit {
            display: Some(mark.build(damage)),
            source: Some(Target::Player(actor)),
            style: Style::Melee,
            damage,
        };
        self.queue_hit(Target::Npc(target), hit, 1)
            .map_err(|_| ActionError::QueueFull)?;
        if let Some(sequence) = defend {
            self.events.push(WorldEvent::NpcCombatAnimation {
                actor: target,
                sequence,
            });
        }
        // Source charges only after the special handler successfully queues its
        // hit and continuation. Misses still charge; rejected actions do not.
        if let Some(s) = special {
            let (energy, _) = special_vars.unwrap();
            let p = self.data_mut(actor)?;
            let remaining = p.variables.get(energy.id) - s.cost;
            if let Some(update) = p.variables.set(energy, remaining, logged) {
                self.events.push(WorldEvent::CombatVariables { actor, updates: vec![update] });
            }
        }
        Ok(true)
    }
}
