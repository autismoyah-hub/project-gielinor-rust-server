//! PlayerCommons' attack-time autocast check. A failed requirement disables
//! casting and clears the weapon's saved choice, preserving active spell vars.
use crate::{
    autocast::{self, Event, Spell},
    autocast_select,
    variables::Variables,
};
pub struct Outcome {
    pub spell: Option<i32>,
    pub events: Vec<Event>,
}
pub fn resolve(
    autocast: &autocast::Config,
    access: &autocast_select::Config,
    vars: &mut Variables,
    weapon: Option<i32>,
    level: i32,
    logged: bool,
    mut missing_runes: impl FnMut(&Spell) -> Result<Option<String>, String>,
) -> Result<Outcome, String> {
    let empty = || Outcome {
        spell: None,
        events: Vec::new(),
    };
    if autocast.enabled.get(vars).map_err(|e| format!("{e:?}"))? != 1 {
        return Ok(empty());
    }
    let Some(weapon) = weapon.and_then(|id| autocast.weapons.get(&id)) else {
        return Ok(empty());
    };
    let id = autocast.spell.get(vars).map_err(|e| format!("{e:?}"))?;
    let Some(spell) = autocast.spells.get(&id) else {
        return Ok(empty());
    };
    let required = access
        .spells
        .get(&id)
        .ok_or_else(|| format!("Missing spell access {id}"))?;
    let book = access.book.get(vars).map_err(|e| format!("{e:?}"))?;
    let mut events = Vec::new();
    let failure =
        if required.book != Some(book) {
            Some("You can't autocast that spell with your current active spellbook.".to_owned())
        } else if !autocast.autocast_items.get(&id).is_some_and(|item| {
            !autocast.restricted.contains(item) || weapon.additional.contains(item)
        }) {
            // An incompatible staff emits a message without clearing any settings.
            return Ok(Outcome {
                spell: None,
                events: vec![Event::Message(
                    "You can't autocast that spell with this staff.".into(),
                )],
            });
        } else if level < required.level {
            Some("Your Magic level is not high enough for this spell.".to_owned())
        } else {
            missing_runes(spell)?
        };
    if let Some(message) = failure {
        events.push(Event::Message(message));
        let mut next = vars.clone();
        let saved = match weapon.category {
            18 => Some(autocast.staff),
            21 => Some(autocast.bladed),
            _ => None,
        };
        let mut bindings = vec![autocast.enabled];
        if let Some(saved) = saved {
            bindings.extend([saved.spell, saved.defensive]);
        }
        for binding in bindings {
            events.extend(
                next.set_bit(binding.base, binding.bit, 0, logged)
                    .map_err(|e| format!("{e:?}"))?
                    .map(Event::Variable),
            );
        }
        *vars = next;
        Ok(Outcome {
            spell: None,
            events,
        })
    } else {
        Ok(Outcome {
            spell: Some(id),
            events,
        })
    }
}
