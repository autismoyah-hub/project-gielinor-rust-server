use super::*;
use crate::{
    attack_profile::Attack,
    hits::{Hit, Style, Target},
    projectile::{EntityIndex, Flight},
    ranged_ammo::Shot,
    ranged_weapon,
    raycast::Bounds,
};
impl World {
    /// Generic PvN ranged branch. Specialized weapon scripts must dispatch before this.
    pub fn attack_npc_ranged(
        &mut self,
        actor: ActorId,
        target: NpcId,
        attack: Attack,
        random: &mut impl RandomSource,
    ) -> Result<bool, ActionError> {
        let Attack::Ranged { kind, style } = attack else {
            return Err(ActionError::InvalidItem);
        };
        let p = self.player(actor)?;
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal)
            || self.npc_combat_denial(actor, target).is_some()
        {
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
        let profile = self.attack_profile(actor)?;
        let source = Bounds {
            coords: p.movement.coords,
            width: 1,
            length: 1,
        };
        let dest = Bounds {
            coords: npc.movement.coords,
            width: npc.movement.size,
            length: npc.movement.size,
        };
        if crate::projectile::bounds_distance(source, dest).map_err(|_| ActionError::OutOfRange)?
            > profile.range
            || !crate::raycast::clear(&self.collision, source, dest, true, 0)
        {
            return Err(ActionError::OutOfRange);
        }
        if self.ranged.is_none() || self.ranged_weapons.is_none() || self.melee.is_none() {
            return Err(ActionError::UnsupportedCombat);
        }
        // Failed weapon/ammo checks still consume the attack delay in the reference.
        self.set_next_attack_delay(actor, profile.speed)?;
        let prepared = self
            .ranged_weapons
            .as_ref()
            .unwrap()
            .prepare(&self.player(actor)?.data.equipment);
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(ranged_weapon::Error::Denied(message)) => {
                self.clear_npc_route(actor.slot as usize);
                self.events.push(WorldEvent::CombatFeedback {
                    actor,
                    events: vec![crate::autocast::Event::Message(message.into())],
                });
                return Ok(false);
            }
            Err(_) => return Err(ActionError::InvalidItem),
        };
        let flight = Flight::between(
            prepared.travel as i32,
            prepared.style,
            EntityIndex::Player(actor.slot),
            EntityIndex::Npc(target.slot),
            source,
            dest,
        )
        .map_err(|_| ActionError::OutOfRange)?;
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor,
            sequence: prepared.animation,
        });
        if let Some(sound) = prepared.sound {
            self.events.push(WorldEvent::CombatSound { actor, sound });
        }
        self.events.push(WorldEvent::PlayerSpotAnimation {
            actor,
            spot: prepared.launch,
            delay: 0,
            height: 96,
            slot: 1,
        });
        self.emit_projectile(flight)?;
        self.use_ranged_ammo(
            actor,
            Shot {
                slot: prepared.ammo_slot,
                id: prepared.ammo_id,
                recovery_rate: prepared.recovery,
                metallic_body: prepared.metallic,
                drop_coord: dest.coords,
                drop_delay: flight.duration.server_delay,
            },
            random,
        )?;

        // Consumption can unequip the last thrown weapon and run worn callbacks.
        // The formula must read that resulting equipment, rather than a launch snapshot.
        let p = self.player(actor)?;
        let npc = self.npc(target).ok_or(ActionError::StaleActor)?;
        let common = self.melee.as_ref().unwrap();
        let definition = self
            .npc_combat
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        let input = self
            .ranged
            .as_ref()
            .unwrap()
            .inputs(
                common,
                &p.data,
                npc,
                definition,
                Some(kind),
                style,
                self.attack_profile(actor)?.speed,
                -1,
                // Kotlin InternalHelpers.isSlayerTask currently returns false.
                false,
                source.coords,
            )
            .map_err(|_| ActionError::InvalidStat)?;
        let mut mark = *common
            .npc_marks
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        let defend = common.npc_defend.get(&npc.type_id).copied();
        let threshold = common
            .hit_threshold
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?
            .max(10);
        let max_hit_varp = common.max_hit;
        let previous_max = p.data.variables.get(max_hit_varp.id);
        let logged = p.protection.processed_clock > 0;
        let rate = p.data.xp_rate_hundreds as f64 / 100.0;
        let uuid = p.data.character_id;
        let roll = input.roll(random).map_err(|_| ActionError::InvalidStat)?;
        if let Some(max_hit) = roll.max_hit {
            if let Some(update) = self
                .data_mut(actor)?
                .variables
                .set(max_hit_varp, max_hit, logged)
            {
                self.events.push(WorldEvent::CombatVariables {
                    actor,
                    updates: vec![update],
                });
            }
        }
        self.give_npc_combat_xp(actor, target, attack, roll.damage, rate)?;
        mark.source_threshold = threshold.max(roll.max_hit.unwrap_or(previous_max));
        let clock = self.clock.0;
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        npc.aggressive_player = Some(actor);
        npc.retaliate_due
            .push_back(clock.wrapping_add(flight.duration.server_delay));
        npc.last_combat = npc.last_combat.max(clock);
        let damage = self.modify_npc_combat_damage(Some(actor), target, roll.damage)?;
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        npc.hero_points.credit(uuid, damage.min(npc.stats[3]));
        self.queue_hit(
            Target::Npc(target),
            Hit {
                display: Some(mark.build(damage)),
                source: Some(Target::Player(actor)),
                style: Style::Ranged,
                damage,
            },
            flight.duration.server_delay,
        )
        .map_err(|_| ActionError::QueueFull)?;
        if let Some(sequence) = defend {
            self.events.push(WorldEvent::NpcDelayedAnimation {
                actor: target,
                sequence,
                delay: flight.duration.client_delay as u8,
            });
        }
        if prepared.javelin {
            self.events.push(WorldEvent::NpcSpotAnimation {
                actor: target,
                spot: self.ranged_weapons.as_ref().unwrap().ballista_spot,
                delay: flight.duration.client_delay as u16,
                height: 146,
                slot: 0,
            });
        }
        if prepared.ammo_slot == 3 && self.player(actor)?.data.equipment.get(3).is_none() {
            self.clear_npc_route(actor.slot as usize);
            self.events.push(WorldEvent::CombatFeedback {
                actor,
                events: vec![crate::autocast::Event::Message(
                    "That was your last one!".into(),
                )],
            });
        }
        Ok(true)
    }
}
