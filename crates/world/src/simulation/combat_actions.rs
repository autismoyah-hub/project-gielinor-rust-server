use super::*;
use crate::attack_profile::{self, Attack, Profile};
impl World {
    pub fn attack_profile(&self, actor: ActorId) -> Result<Profile, ActionError> {
        let p = self.player(actor)?;
        let stance = self
            .combat_stance
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let autocast = self
            .combat_wear
            .as_ref()
            .ok_or(ActionError::InvalidItem)?
            .autocast
            .spell
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?;
        self.attack_profiles
            .profile(
                p.data.equipment.get(3).map(|i| i.id),
                p.data.variables.get(stance.stance.id),
                autocast,
            )
            .map_err(|_| ActionError::InvalidItem)
    }
    /// Called when a combat interaction reaches its target. This validates the
    /// live equipment/spell state, but does not consume runes or start an attack.
    pub fn resolve_combat_attack(
        &mut self,
        actor: ActorId,
    ) -> Result<(Attack, Option<i32>), ActionError> {
        self.player(actor)?;
        let wear = self.combat_wear.as_ref().ok_or(ActionError::InvalidItem)?;
        let access = self
            .autocast_selection
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let stance_config = self
            .combat_stance
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        let weapon = p.data.equipment.get(3).map(|i| i.id);
        let stance = p.data.variables.get(stance_config.stance.id);
        let (kind, style) = self
            .attack_profiles
            .selection(weapon, stance)
            .map_err(|_| ActionError::InvalidItem)?;
        let old_vars = p.data.variables.clone();
        let selected = crate::autocast_attack::resolve(
            &wear.autocast,
            access,
            &mut p.data.variables,
            weapon,
            p.data.stats.current(access.magic_stat) as u8 as i32,
            p.protection.processed_clock > 0,
            |spell| {
                wear.runes
                    .missing_message(&old_vars, &p.data.inventory, &p.data.equipment, spell)
            },
        )
        .map_err(|_| ActionError::InvalidItem)?;
        let defensive = wear
            .autocast
            .defensive
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?
            == 1;
        let attack = attack_profile::resolve(
            weapon.is_some(),
            kind,
            style,
            selected.spell.is_some(),
            defensive,
            stance,
        )
        .map_err(|_| ActionError::InvalidItem)?;
        self.events.push(WorldEvent::CombatFeedback {
            actor,
            events: selected.events,
        });
        Ok((attack, selected.spell))
    }
    pub fn is_attack_delayed(&self, actor: ActorId) -> Result<bool, ActionError> {
        Ok(self.player(actor)?.action_delay > self.clock.0)
    }
    pub fn set_next_attack_delay(
        &mut self,
        actor: ActorId,
        cycles: i32,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        self.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .action_delay = self.clock.0.wrapping_add(cycles);
        Ok(())
    }
    /// Trusted content only: damage comes from the combat roll, never a client
    /// action. XP uses remaining target HP before queued damage is applied.
    pub fn give_npc_combat_xp(
        &mut self,
        actor: ActorId,
        target: NpcId,
        attack: Attack,
        damage: i32,
        rate: f64,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        let npc = self.npc(target).ok_or(ActionError::StaleActor)?;
        let health = npc.stats[3];
        let npc_type = npc.type_id;
        let npc_multiplier = npc.combat_xp_multiplier;
        let multiplier = npc.combat_xp_multiplier as f64 / 1000.0;
        let selected = self
            .npc_death
            .as_ref()
            .is_some_and(|config| config.chickens.contains_key(&npc_type));
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        let mut stats = p.data.stats.clone();
        let awards =
            crate::combat_xp::advance(&mut stats, attack, damage, health, multiplier, rate)
                .map_err(|_| ActionError::InvalidStat)?;
        if self.tutorial_rules_completed_varp.is_some() {
            p.stat_events
                .advance_batch(
                    &crate::tutorial_rules::bindings(),
                    awards.iter().map(|(skill, change)| (skill.id(), *change)),
                )
                .map_err(|_| ActionError::QueueFull)?;
        }
        let trace = if selected {
            if let Attack::Melee { style, stance, .. } = attack {
                Some(crate::combat_xp::MeleeTrace {
                    tick: self.clock.0,
                    actor,
                    target,
                    npc_type,
                    style,
                    stance,
                    damage,
                    target_health: health,
                    npc_multiplier,
                    rate,
                    awards: awards
                        .iter()
                        .map(|(skill, _)| {
                            (
                                skill.id(),
                                p.data.stats.fine_xp(skill.id()),
                                stats.fine_xp(skill.id()),
                            )
                        })
                        .collect(),
                })
            } else {
                None
            }
        } else {
            None
        };
        p.data.stats = stats;
        self.events.push(WorldEvent::CombatXp { actor, awards });
        if let Some(trace) = trace {
            self.events.push(WorldEvent::NpcMeleeXp(trace));
        }
        Ok(())
    }
}
