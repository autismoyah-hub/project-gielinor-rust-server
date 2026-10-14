use super::*;
use crate::{
    attack_profile::Attack,
    hits::{Hit, Style, Target},
    projectile::{EntityIndex, Flight},
    raycast::Bounds,
};
impl World {
    /// Existing elemental PvN spells. Metadata gates activation until resolved
    /// and validated by the host; powered staves have their own attack handlers.
    pub fn attack_npc_elemental(
        &mut self,
        actor: ActorId,
        target: NpcId,
        spell_id: i32,
        defensive: bool,
        random: &mut impl RandomSource,
    ) -> Result<bool, ActionError> {
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
            > 10
            || !crate::raycast::clear(&self.collision, source, dest, true, 0)
        {
            return Err(ActionError::OutOfRange);
        }
        let catalog = self
            .elemental_spells
            .as_ref()
            .ok_or(ActionError::UnsupportedCombat)?;
        if self.spell_combat.is_none() || self.melee.is_none() {
            return Err(ActionError::UnsupportedCombat);
        }
        let spell = catalog.spells.get(&spell_id).cloned();
        let projectile = catalog.projectile;
        let (sunfire, rate, splash_spot, splash_sound) = (
            catalog.sunfire,
            catalog.global_xp_rate,
            catalog.splash_spot,
            catalog.splash_sound,
        );
        let staff = p
            .data
            .equipment
            .get(3)
            .is_some_and(|item| catalog.staves.contains(&item.id));
        self.set_next_attack_delay(actor, 5)?;
        let Some(spell) = spell else {
            self.clear_npc_route(actor.slot as usize);
            self.events.push(WorldEvent::CombatFeedback {
                actor,
                events: vec![crate::autocast::Event::Message(
                    "You attempt to cast the spell, but nothing happens.".into(),
                )],
            });
            return Ok(false);
        };
        let cast = self.attempt_spell_cast(actor, &spell.definition, sunfire, rate)?;
        if !cast.success() {
            return Ok(false);
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor,
            sequence: if staff {
                spell.staff_animation
            } else {
                spell.unarmed_animation
            },
        });
        self.events.push(WorldEvent::PlayerSpotAnimation {
            actor,
            spot: spell.launch,
            delay: 0,
            height: 92,
            slot: 0,
        });
        let flight = Flight::between(
            spell.travel as i32,
            projectile,
            EntityIndex::Player(actor.slot),
            EntityIndex::Npc(target.slot),
            source,
            dest,
        )
        .map_err(|_| ActionError::OutOfRange)?;
        self.emit_projectile(flight)?;
        let config = self.spell_combat.as_ref().unwrap();
        let common = self.melee.as_ref().unwrap();
        let p = self.player(actor)?;
        let npc = self.npc(target).ok_or(ActionError::StaleActor)?;
        let definition = self
            .npc_combat
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        let input = config
            .inputs(common, &p.data, npc, 5)
            .map_err(|_| ActionError::InvalidStat)?;
        let accuracy = config
            .attributes(
                &p.data,
                npc,
                definition,
                spell_id,
                spell.definition.book,
                cast.used_sunfire(),
                -1,
                // Kotlin InternalHelpers.isSlayerTask currently returns false.
                false,
                random,
            )
            .map_err(|_| ActionError::InvalidItem)?;
        let accurate = input
            .accurate(accuracy, random)
            .map_err(|_| ActionError::InvalidStat)?;
        let range = if accurate {
            // Kotlin collects equipment passives again; never reuse accuracy's draw.
            let damage_attrs = config
                .attributes(
                    &p.data,
                    npc,
                    definition,
                    spell_id,
                    spell.definition.book,
                    cast.used_sunfire(),
                    -1,
                    false,
                    random,
                )
                .map_err(|_| ActionError::InvalidItem)?;
            Some(input.damage_range(damage_attrs, spell.tier.max_hit(input.magic)))
        } else {
            None
        };
        let max_var = common.max_hit;
        let previous_max = p.data.variables.get(max_var.id);
        let logged = p.protection.processed_clock > 0;
        let mut mark = *common
            .npc_marks
            .get(&npc.type_id)
            .ok_or(ActionError::InvalidItem)?;
        mark.source_threshold = common
            .hit_threshold
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?
            .max(10)
            .max(range.map_or(previous_max, |r| r.1));
        let defend = common.npc_defend.get(&npc.type_id).copied();
        let uuid = p.data.character_id;
        let xp_rate = p.data.xp_rate_hundreds as f64 / 100.0;
        if let Some((_, max)) = range {
            if let Some(update) = self.data_mut(actor)?.variables.set(max_var, max, logged) {
                self.events.push(WorldEvent::CombatVariables {
                    actor,
                    updates: vec![update],
                });
            }
        }
        let damage = match range {
            Some(range) => crate::spell_roll::Inputs::damage(range, random)
                .map_err(|_| ActionError::InvalidStat)?,
            None => 0,
        };
        self.events.push(WorldEvent::CombatSound {
            actor,
            sound: spell.cast_sound,
        });
        self.events.push(WorldEvent::NpcSpotAnimation {
            actor: target,
            spot: if accurate { spell.impact } else { splash_spot },
            delay: flight.duration.client_delay as u16,
            height: 124,
            slot: 0,
        });
        self.emit_area_sound(crate::zones::Sound {
            coords: dest.coords,
            id: if accurate {
                spell.hit_sound
            } else {
                splash_sound
            } as i32,
            delay: flight.duration.client_delay,
            loops: 1,
            radius: 10,
            size: 0,
        })?;
        if accurate {
            self.give_npc_combat_xp(actor, target, Attack::Spell { defensive }, damage, xp_rate)?;
        }
        let clock = self.clock.0;
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        npc.aggressive_player = Some(actor);
        npc.retaliate_due.push_back(clock.wrapping_add(if accurate {
            flight.duration.server_delay
        } else {
            1
        }));
        npc.last_combat = npc.last_combat.max(clock);
        let damage = self.modify_npc_combat_damage(Some(actor), target, damage)?;
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        npc.hero_points.credit(uuid, damage.min(npc.stats[3]));
        self.queue_hit(
            Target::Npc(target),
            Hit {
                display: Some(mark.build(damage)),
                source: Some(Target::Player(actor)),
                style: Style::Magic,
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
        let wear = self
            .combat_wear
            .as_ref()
            .ok_or(ActionError::UnsupportedCombat)?;
        if wear
            .autocast
            .enabled
            .get(&self.player(actor)?.data.variables)
            .map_err(|_| ActionError::InvalidItem)?
            != 1
        {
            self.clear_npc_route(actor.slot as usize);
        } else if let Some(pending) = self
            .players
            .get_mut(actor.slot as usize)
            .and_then(|p| p.npc_interaction.as_mut())
        {
            // Kotlin re-engages OP2, resolving the current autocast selection.
            pending.spell = None;
        }
        Ok(true)
    }
}
