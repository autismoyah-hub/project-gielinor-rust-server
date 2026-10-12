//! Weapon-swap autocast restoration from CombatTabScript. Rune validation is
//! supplied by the world spell service; swapping weapons does not check level
//! or spellbook and does not consume runes.
use crate::{
    bank_actions::Binding,
    runes::Requirement,
    variables::{Update, Variables},
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug)]
pub struct Spell {
    pub item: i32,
    pub requirements: Vec<Requirement>,
}
#[derive(Clone, Debug)]
pub struct Weapon {
    pub category: u8,
    pub additional: BTreeSet<i32>,
}
#[derive(Clone, Copy)]
pub struct Saved {
    pub spell: Binding,
    pub defensive: Binding,
}
pub struct Config {
    pub enabled: Binding,
    pub spell: Binding,
    pub defensive: Binding,
    pub staff: Saved,
    pub bladed: Saved,
    pub weapons: BTreeMap<i32, Weapon>,
    /// Registry of combat spells, indexed by autocast id.
    pub spells: BTreeMap<i32, Spell>,
    /// AutocastSpells has a separate map from MagicSpellRegistry.
    pub autocast_items: BTreeMap<i32, i32>,
    pub restricted: BTreeSet<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Variable(Update),
    Message(String),
}
impl Config {
    fn write(
        vars: &mut Variables,
        binding: Binding,
        value: i32,
        logged: bool,
        events: &mut Vec<Event>,
    ) -> Result<(), String> {
        events.extend(
            vars.set_bit(binding.base, binding.bit, value, logged)
                .map_err(|e| format!("{e:?}"))?
                .map(Event::Variable),
        );
        Ok(())
    }
    fn reset(
        &self,
        vars: &mut Variables,
        logged: bool,
        events: &mut Vec<Event>,
    ) -> Result<(), String> {
        for binding in [self.enabled, self.spell, self.defensive] {
            Self::write(vars, binding, 0, logged, events)?;
        }
        Ok(())
    }
    pub fn restore(
        &self,
        vars: &mut Variables,
        weapon: Option<i32>,
        logged: bool,
        mut missing_runes: impl FnMut(&Spell) -> Result<Option<String>, String>,
    ) -> Result<Vec<Event>, String> {
        let mut next = vars.clone();
        let mut events = Vec::new();
        self.restore_inner(&mut next, weapon, logged, &mut missing_runes, &mut events)?;
        *vars = next;
        Ok(events)
    }
    fn restore_inner(
        &self,
        vars: &mut Variables,
        weapon: Option<i32>,
        logged: bool,
        missing_runes: &mut impl FnMut(&Spell) -> Result<Option<String>, String>,
        events: &mut Vec<Event>,
    ) -> Result<(), String> {
        let Some(weapon) = weapon.and_then(|id| self.weapons.get(&id)) else {
            return self.reset(vars, logged, events);
        };
        let saved = match weapon.category {
            18 => self.staff,
            21 => self.bladed,
            _ => return self.reset(vars, logged, events),
        };
        let id = vars.bit(saved.spell.bit).map_err(|e| format!("{e:?}"))?;
        let Some(spell) = self.spells.get(&id).filter(|_| id != 0) else {
            return self.reset(vars, logged, events);
        };
        let valid_staff = self.autocast_items.get(&id).is_some_and(|item| {
            !self.restricted.contains(item) || weapon.additional.contains(item)
        });
        if !valid_staff {
            events.push(Event::Message(
                "You can't autocast that spell with this staff.".into(),
            ));
            return self.reset(vars, logged, events);
        }
        if let Some(message) = missing_runes(spell)? {
            events.push(Event::Message(message));
            Self::write(vars, saved.spell, 0, logged, events)?;
            Self::write(vars, saved.defensive, 0, logged, events)?;
            return self.reset(vars, logged, events);
        }
        let defensive = i32::from(
            vars.bit(saved.defensive.bit)
                .map_err(|e| format!("{e:?}"))?
                != 0,
        );
        Self::write(vars, self.spell, id, logged, events)?;
        Self::write(vars, self.defensive, defensive, logged, events)?;
        let enabled = i32::from(vars.bit(self.spell.bit).map_err(|e| format!("{e:?}"))? != 0);
        Self::write(vars, self.enabled, enabled, logged, events)
    }
}
