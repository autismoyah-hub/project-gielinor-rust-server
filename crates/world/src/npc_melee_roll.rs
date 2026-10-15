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

#[cfg(test)]
mod wiki_tests {
    use super::*;
    struct Draws(std::collections::VecDeque<(u32, u32)>);
    impl RandomSource for Draws {
        fn below(&mut self, bound: u32) -> u32 {
            let (expected, value) = self.0.pop_front().expect("unexpected draw");
            assert_eq!(bound, expected);
            assert!(value < bound);
            value
        }
    }
    #[test]
    fn prepared_chicken_bonuses_produce_hit_zero_distinct_from_a_miss() {
        // R13 prepared NPC1173/1174: Strength1, param10=-42 and65512=-47.
        // Owned param.sym binds these to melee_strength and attack_melee.
        // This is a formula vector; runtime cache loading remains separate.
        let input = Inputs {
            attack_level: 1,
            strength_level: 1,
            attack_bonus: -47,
            strength_bonus: -42,
            defence_level: 1,
            defence_bonus: 0,
            defence_style: None,
            defence_prayer: 1.0,
            defence_armour: 1.0,
            elysian: false,
            bulwark: false,
            justiciar: false,
        };
        assert!(input.chance().unwrap() > 0);
        let mut hit = Draws([(10000, 0), (1, 0)].into());
        let landed = input.roll(&mut hit).unwrap();
        assert_eq!((landed.max_hit, landed.damage), (Some(0), 0));
        assert!(hit.0.is_empty());
        let mut miss = Draws([(10000, 9999)].into());
        let missed = input.roll(&mut miss).unwrap();
        assert_eq!((missed.max_hit, missed.damage), (None, 0));
        assert!(miss.0.is_empty());
        assert_eq!(landed.chance, missed.chance);
    }
}
