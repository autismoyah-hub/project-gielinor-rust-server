//! CombatTabScript's explicit spell selection. Weapon swaps use autocast::restore
//! and deliberately skip the spellbook and level checks performed here.
use crate::{
    autocast::{self, Event, Spell},
    bank_actions::Binding,
    variables::Variables,
};
use std::collections::BTreeMap;
#[derive(Clone, Copy)]
pub struct Access {
    pub book: Option<i32>,
    pub level: i32,
}
pub struct Config {
    pub book: Binding,
    pub magic_stat: u8,
    pub spells: BTreeMap<i32, Access>,
}
pub struct Outcome {
    pub restore_tab: bool,
    pub events: Vec<Event>,
}
impl Config {
    pub fn select(
        &self,
        autocast: &autocast::Config,
        vars: &mut Variables,
        weapon: Option<i32>,
        choice: i32,
        defensive: bool,
        level: i32,
        logged: bool,
        mut missing_runes: impl FnMut(&Spell) -> Result<Option<String>, String>,
    ) -> Result<Outcome, String> {
        let mut next = vars.clone();
        let mut events = Vec::new();
        let weapon = weapon.and_then(|id| autocast.weapons.get(&id));
        let saved = weapon.and_then(|w| match w.category {
            18 => Some(autocast.staff),
            21 => Some(autocast.bladed),
            _ => None,
        });
        let mut write = |binding: Binding, value: i32| -> Result<(), String> {
            events.extend(
                next.set_bit(binding.base, binding.bit, value, logged)
                    .map_err(|e| format!("{e:?}"))?
                    .map(Event::Variable),
            );
            Ok(())
        };
        if choice == 0 {
            if let Some(saved) = saved {
                write(saved.spell, 0)?;
                write(saved.defensive, 0)?;
            }
            write(autocast.enabled, 0)?;
            write(autocast.spell, 0)?;
            write(autocast.defensive, 0)?;
            *vars = next;
            return Ok(Outcome {
                restore_tab: true,
                events,
            });
        }
        let (Some(spell), Some(access), Some(weapon), Some(saved)) = (
            autocast.spells.get(&choice),
            self.spells.get(&choice),
            weapon,
            saved,
        ) else {
            return Ok(Outcome {
                restore_tab: false,
                events,
            });
        };
        let book = self.book.get(vars).map_err(|e| format!("{e:?}"))?;
        let failure = if access.book.is_some_and(|required| required != book) {
            Some("You can't autocast that spell with your current active spellbook.".to_owned())
        } else if !autocast.autocast_items.get(&choice).is_some_and(|item| {
            !autocast.restricted.contains(item) || weapon.additional.contains(item)
        }) {
            Some("You can't autocast that spell with this staff.".to_owned())
        } else if access.book.is_none() || !(0..=3).contains(&book) {
            // MagicRuneManager.validateAccess rejects a null spellbook silently.
            return Ok(Outcome {
                restore_tab: false,
                events,
            });
        } else if level < access.level {
            Some("Your Magic level is not high enough for this spell.".to_owned())
        } else {
            missing_runes(spell)?
        };
        if let Some(message) = failure {
            events.push(Event::Message(message));
            return Ok(Outcome {
                restore_tab: false,
                events,
            });
        }
        write(saved.spell, choice)?;
        write(saved.defensive, i32::from(defensive))?;
        write(autocast.spell, choice)?;
        write(autocast.defensive, i32::from(defensive))?;
        write(autocast.enabled, 1)?;
        *vars = next;
        Ok(Outcome {
            restore_tab: true,
            events,
        })
    }
}
