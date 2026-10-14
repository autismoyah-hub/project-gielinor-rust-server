use super::*;
impl World {
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
