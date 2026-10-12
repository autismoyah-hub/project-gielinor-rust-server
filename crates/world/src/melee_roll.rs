//! PvN melee roll pipeline. Attribute collection and random draws are explicit
//! inputs so a Kotlin trace can be replayed without relying on RNG seed parity.
use crate::{
    combat,
    melee_modifiers::{Attributes, Melee},
    npc::RandomSource,
};
#[derive(Clone, Copy)]
pub struct Inputs {
    pub attack_level: i32,
    pub strength_level: i32,
    pub defence_level: i32,
    pub attack_bonus: i32,
    pub strength_bonus: i32,
    pub defence_bonus: i32,
    pub style: Option<u8>,
    pub attack_prayer: f64,
    pub strength_prayer: f64,
    pub void: f64,
    pub soul: f64,
    pub invocation: i32,
    pub speed: i32,
    pub health: i32,
    pub max_health: i32,
    pub attributes: Attributes,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Roll {
    pub damage: i32,
    pub max_hit: Option<i32>,
    pub chance: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Multiplier,
    EmptyDamageRange,
}
impl Inputs {
    pub fn chance(self) -> Result<i32, Error> {
        let style = match self.style {
            Some(1) => 9,
            Some(2) => 11,
            _ => 8,
        };
        let attack =
            combat::effective_attack(self.attack_level, style, self.attack_prayer, self.void)
                .map_err(|_| Error::Multiplier)?;
        let attack = self
            .attributes
            .attack_roll(combat::base_roll(attack, self.attack_bonus));
        let defence = self.attributes.defence_roll(
            combat::base_roll(self.defence_level.wrapping_add(9), self.defence_bonus),
            self.invocation,
        );
        Ok(self
            .attributes
            .hit_chance(combat::hit_chance(attack, defence, false), attack, defence))
    }
    pub fn max_hit(self, attributes: Attributes) -> Result<i32, Error> {
        let style = match self.style {
            Some(1) => 9,
            Some(3) => 11,
            _ => 8,
        };
        let strength = combat::effective_strength(
            self.strength_level,
            style,
            self.strength_prayer,
            self.void,
            self.soul,
        )
        .map_err(|_| Error::Multiplier)?;
        let damage = attributes.damage(combat::base_damage(strength, self.strength_bonus));
        Ok(attributes.post_special(damage, self.speed, self.health, self.max_health))
    }
    pub fn roll(self, random: &mut impl RandomSource) -> Result<Roll, Error> {
        let chance = self.chance()?;
        if chance <= random.below(10000) as i32 {
            return Ok(Roll {
                damage: 0,
                max_hit: None,
                chance,
            });
        }
        let mut attrs = self.attributes;
        let has = |m: Melee| attrs.melee & (1u64 << m as u8) != 0;
        if [
            Melee::KerisWeapon,
            Melee::KerisBreachPartisan,
            Melee::KerisSunPartisan,
        ]
        .into_iter()
        .any(has)
            && random.below(51) == 0
        {
            attrs.melee |= 1u64 << Melee::KerisProc as u8;
        }
        if attrs.melee & (1u64 << Melee::Gadderhammer as u8) != 0 && random.below(20) == 0 {
            attrs.melee |= 1u64 << Melee::GadderhammerProc as u8;
        }
        let max_hit = self.max_hit(attrs)?;
        if max_hit < 1 || max_hit == i32::MAX {
            return Err(Error::EmptyDamageRange);
        }
        Ok(Roll {
            damage: 1 + random.below(max_hit as u32) as i32,
            max_hit: Some(max_hit),
            chance,
        })
    }
}
