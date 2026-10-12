//! CombatTabScript stance validation, persistence, and Bulwark transition rule.
use crate::{
    bank_actions::Binding,
    variables::{Update, Variables, Varp},
};
use std::collections::BTreeMap;
pub fn category(id: Option<u8>) -> i32 {
    match id {
        Some(id @ (0..=24 | 27 | 28)) => id as i32,
        _ => 0,
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Weapon {
    pub category: u8,
    pub bulwark: bool,
}
#[derive(Clone)]
pub struct Config {
    pub stance: Varp,
    pub melee: Varp,
    pub retaliate: Varp,
    pub saved: BTreeMap<i32, Binding>,
    pub styles: BTreeMap<i32, i32>,
    pub weapons: BTreeMap<i32, Weapon>,
}
pub struct Change {
    pub variables: Vec<Update>,
    pub bulwark_delay: bool,
}
impl Config {
    /// First phase of CombatTabScript.onWearposChange. Autocast restoration runs
    /// between this phase and validation, and its packet order is observable.
    pub fn load_saved(
        &self,
        vars: &mut Variables,
        weapon: Option<i32>,
        logged_in: bool,
    ) -> Result<Vec<Update>, String> {
        let category = category(
            weapon
                .and_then(|id| self.weapons.get(&id))
                .map(|w| w.category),
        );
        let mut updates = Vec::new();
        if let Some(saved) = self.saved.get(&category) {
            let value = vars.bit(saved.bit).map_err(|e| format!("{e:?}"))?;
            let stance = if (0..=3).contains(&value) { value } else { 0 };
            updates.extend(vars.set(self.stance, stance, logged_in));
        }
        Ok(updates)
    }
    /// Wear-position validation always writes a resolved melee style. Clicking
    /// a stance uses a different source path that suppresses unchanged styles.
    pub fn validate_worn(
        &self,
        vars: &mut Variables,
        weapon: Option<i32>,
        logged_in: bool,
    ) -> Result<Vec<Update>, String> {
        let category = category(
            weapon
                .and_then(|id| self.weapons.get(&id))
                .map(|w| w.category),
        );
        let value = vars.get(self.stance.id);
        let stance = if (0..=3).contains(&value) { value } else { 0 };
        let style =
            crate::bonuses::attack_style(self.styles.get(&category).copied().unwrap_or(0), stance)
                .map_err(str::to_owned)?;
        let mut updates = Vec::new();
        updates.extend(vars.set(
            self.stance,
            if style.is_some() { stance } else { 0 },
            logged_in,
        ));
        if let Some(style @ 1..=4) = style {
            updates.extend(vars.set(self.melee, style as i32 - 1, logged_in));
        }
        Ok(updates)
    }
    pub fn set(
        &self,
        vars: &mut Variables,
        weapon: Option<i32>,
        stance: u8,
        logged_in: bool,
    ) -> Result<Change, String> {
        if stance > 3 {
            return Err("Invalid combat stance".into());
        }
        let weapon = weapon.and_then(|id| self.weapons.get(&id));
        let category = category(weapon.map(|w| w.category));
        let previous = vars.get(self.stance.id);
        let bulwark_delay = weapon.is_some_and(|w| w.bulwark) && previous == 3 && stance != 3;
        let mut updates = Vec::new();
        updates.extend(vars.set(self.stance, stance as i32, logged_in));
        let style = crate::bonuses::attack_style(
            self.styles.get(&category).copied().unwrap_or(0),
            stance as i32,
        )
        .map_err(str::to_owned)?;
        let validated = if style.is_none() { 0 } else { stance as i32 };
        // Source deliberately writes com_mode twice, even if both writes have the same value.
        updates.extend(vars.set(self.stance, validated, logged_in));
        if let Some(style @ 1..=4) = style {
            let melee = vars.get(self.melee.id);
            let current = if (0..=3).contains(&melee) { melee } else { 0 };
            let target = style as i32 - 1;
            if current != target {
                updates.extend(vars.set(self.melee, target, logged_in));
            }
        }
        if let Some(saved) = self.saved.get(&category) {
            updates.extend(
                vars.set_bit(saved.base, saved.bit, validated, logged_in)
                    .map_err(|e| format!("{e:?}"))?,
            );
        }
        Ok(Change {
            variables: updates,
            bulwark_delay,
        })
    }
    pub fn toggle_retaliate(&self, vars: &mut Variables, logged_in: bool) -> Option<Update> {
        let value = i32::from(vars.get(self.retaliate.id) != 1);
        vars.set(self.retaliate, value, logged_in)
    }
}
