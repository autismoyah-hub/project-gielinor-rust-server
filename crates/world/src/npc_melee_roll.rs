//! NPC-to-player melee: zero-inclusive damage draws and ordered equipment reductions.
use crate::{combat, npc::RandomSource};
#[derive(Clone, Copy)]
pub struct Inputs {
    pub attack_level: i32,
    pub strength_level: i32,
    pub attack_bonus: i32,
    pub strength_bonus: i32,
    pub defence_level: i32,
    pub defence_bonus: i32,
    pub defence_style: Option<u8>,
    pub defence_prayer: f64,
    pub defence_armour: f64,
    pub elysian: bool,
    pub bulwark: bool,
    pub justiciar: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roll {
    pub chance: i32,
    pub max_hit: Option<i32>,
    pub damage: i32,
}
pub fn reductions(
    mut damage: i32,
    defence: i32,
    elysian: bool,
    bulwark: bool,
    justiciar: bool,
) -> i32 {
    if elysian {
        damage = damage.wrapping_mul(3) / 4;
    }
    if bulwark {
        damage = damage.wrapping_mul(4) / 5;
    }
    if justiciar {
        let reduction = ((damage as f64 * (defence as f64 / 3000.0)) as i32).max(1);
        damage = damage.wrapping_sub(reduction).max(0);
    }
    damage
}
impl Inputs {
    pub fn chance(self) -> Result<i32, &'static str> {
        let style = match self.defence_style {
            Some(4 | 7) => 11,
            Some(1) => 9,
            _ => 8,
        };
        let defence = combat::effective_defence(
            self.defence_level,
            style,
            self.defence_prayer,
            self.defence_armour,
        )
        .map_err(|_| "Invalid defence multiplier")?;
        Ok(combat::hit_chance(
            combat::base_roll(self.attack_level.wrapping_add(9), self.attack_bonus),
            combat::base_roll(defence, self.defence_bonus),
            false,
        ))
    }
    pub fn roll(self, random: &mut impl RandomSource) -> Result<Roll, &'static str> {
        let chance = self.chance()?;
        if chance <= random.below(10000) as i32 {
            return Ok(Roll {
                chance,
                max_hit: None,
                damage: 0,
            });
        }
        let base = combat::base_damage(self.strength_level.wrapping_add(9), self.strength_bonus);
        let elysian = self.elysian && random.below(10) < 7;
        let max_hit = reductions(
            base,
            self.defence_bonus,
            elysian,
            self.bulwark,
            self.justiciar,
        );
        if max_hit < 0 || max_hit == i32::MAX {
            return Err("Invalid NPC damage range");
        }
        Ok(Roll {
            chance,
            max_hit: Some(max_hit),
            damage: random.below(max_hit as u32 + 1) as i32,
        })
    }
}
