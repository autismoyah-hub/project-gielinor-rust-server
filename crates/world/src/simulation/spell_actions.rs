use super::*;
impl World {
    pub(super) fn request_npc_spell(
        &mut self,
        actor: ActorId,
        target: NpcId,
        spell: i32,
        control: bool,
    ) -> Result<(), ActionError> {
        if self.player(actor)?.protection.delayed() {
            self.events.push(WorldEvent::ClearMapFlag(actor));
            return Ok(());
        }
        if !self
            .elemental_spells
            .as_ref()
            .is_some_and(|c| c.spells.contains_key(&spell))
        {
            return Err(ActionError::UnsupportedCombat);
        }
        self.request_npc_inner(actor, target, 2, control, Some(spell))
    }
    pub(super) fn cast_manual_npc(
        &mut self,
        actor: ActorId,
        target: NpcId,
        spell: i32,
        random: &mut impl RandomSource,
    ) -> Result<bool, ActionError> {
        let p = self.player(actor)?;
        let definition = &self
            .elemental_spells
            .as_ref()
            .and_then(|c| c.spells.get(&spell))
            .ok_or(ActionError::UnsupportedCombat)?
            .selection;
        let access = self
            .autocast_selection
            .as_ref()
            .ok_or(ActionError::UnsupportedCombat)?;
        let wear = self
            .combat_wear
            .as_ref()
            .ok_or(ActionError::UnsupportedCombat)?;
        let failure = crate::spell_cast::check(
            &wear.runes,
            crate::spell_cast::Request {
                spell: &definition.spell,
                required_book: definition.book,
                active_book: access
                    .book
                    .get(&p.data.variables)
                    .map_err(|_| ActionError::InvalidItem)?,
                required_level: definition.level,
                visible_level: p.data.stats.current(access.magic_stat) as u8 as i32,
                sunfire_item: -1,
                logged: p.protection.processed_clock > 0,
            },
            &p.data.inventory,
            &p.data.equipment,
            &p.data.variables,
        )
        .map_err(|_| ActionError::InvalidItem)?;
        if let Some(failure) = failure {
            if let Some(message) = failure.message {
                self.events.push(WorldEvent::CombatFeedback {
                    actor,
                    events: vec![crate::autocast::Event::Message(message)],
                });
            }
            self.clear_npc_route(actor.slot as usize);
            return Ok(false);
        }
        if let Some((message, _)) = self.npc_combat_denial(actor, target) {
            if let Some(message) = message {
                self.events.push(WorldEvent::NpcInteraction {
                    actor,
                    event: crate::npc_interaction::Event::Message(message),
                });
            }
            self.clear_npc_route(actor.slot as usize);
            return Ok(false);
        }
        if !self.can_operate_npc(actor, target, 2) {
            self.events.push(WorldEvent::NpcInteraction {
                actor,
                event: crate::npc_interaction::Event::Message("You can't attack this npc."),
            });
            self.clear_npc_route(actor.slot as usize);
            return Ok(false);
        }
        // Manual selection resolves with null style, never defensive autocast.
        self.attack_npc_elemental(actor, target, spell, false, random)
    }
    /// SpellAttackManager.attemptCast: consume requirements, then award cast XP.
    /// Only trusted spell handlers call this after reach/cooldown validation.
    pub fn attempt_spell_cast(
        &mut self,
        actor: ActorId,
        spell: &crate::spell_cast::Definition,
        sunfire_item: i32,
        global_xp_rate: f64,
    ) -> Result<crate::spell_cast::Outcome, ActionError> {
        let p = self.player(actor)?;
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        let access = self
            .autocast_selection
            .as_ref()
            .ok_or(ActionError::UnsupportedCombat)?;
        let wear = self
            .combat_wear
            .as_ref()
            .ok_or(ActionError::UnsupportedCombat)?;
        let book = access
            .book
            .get(&p.data.variables)
            .map_err(|_| ActionError::InvalidItem)?;
        let skill = access.magic_stat;
        let level = p.data.stats.current(skill) as u8 as i32;
        let logged = p.protection.processed_clock > 0;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let result = crate::spell_cast::attempt(
            &wear.runes,
            crate::spell_cast::Request {
                spell: &spell.spell,
                required_book: spell.book,
                active_book: book,
                required_level: spell.level,
                visible_level: level,
                sunfire_item,
                logged,
            },
            &mut p.data.inventory,
            &p.data.equipment,
            &mut p.data.variables,
            &self.object_types,
        )
        .map_err(|_| ActionError::InvalidItem)?;
        if !result.updates.is_empty() {
            self.events.push(WorldEvent::CombatVariables {
                actor,
                updates: result.updates.clone(),
            });
        }
        if let Some(message) = &result.message {
            self.events.push(WorldEvent::CombatFeedback {
                actor,
                events: vec![crate::autocast::Event::Message(message.clone())],
            });
        }
        if result.success() {
            let advancement = crate::skills::advance(
                &mut p.data.stats,
                skill,
                spell.cast_xp,
                global_xp_rate * p.data.xp_rate_hundreds as f64 / 100.0,
                99,
            )
            .map_err(|_| ActionError::InvalidStat)?;
            if self.tutorial_rules_completed_varp.is_some() {
                p.stat_events
                    .advance(&crate::tutorial_rules::bindings(), skill, advancement)
                    .map_err(|_| ActionError::QueueFull)?;
            }
            self.events.push(WorldEvent::SkillAdvanced {
                actor,
                skill,
                advancement,
            });
        } else {
            self.clear_npc_route(actor.slot as usize);
        }
        Ok(result)
    }
}
