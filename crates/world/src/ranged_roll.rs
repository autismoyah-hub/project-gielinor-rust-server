//! PvN ranged operations with explicit attributes and replayable random draws.
use crate::{
    combat, melee_modifiers,
    melee_roll::{Error, Roll},
    npc::RandomSource,
    ranged_modifiers::Attributes,
};
#[derive(Clone, Copy)]
pub struct Inputs {
    pub level: i32,
    pub defence_level: i32,
    pub attack_bonus: i32,
    pub strength_bonus: i32,
    pub defence_bonus: i32,
    /// Global AttackStyle id: AccurateRanged = 5.
    pub style: Option<u8>,
    pub accuracy_prayer: f64,
    pub damage_prayer: f64,
    pub accuracy_void: f64,
    pub damage_void: f64,
    pub invocation: i32,
    pub speed: i32,
    pub magic: i32,
    pub distance: i32,
    pub accuracy_multiplier: f64,
    pub damage_multiplier: f64,
    pub bolt_damage: i32,
    pub attributes: Attributes,
}
impl Inputs {
    pub fn chance(self) -> Result<i32, Error> {
        let level = combat::effective_attack(
            self.level,
            if self.style == Some(5) { 11 } else { 8 },
            self.accuracy_prayer,
            self.accuracy_void,
        )
        .map_err(|_| Error::Multiplier)?;
        let attack = self.attributes.accuracy(
            combat::base_roll(level, self.attack_bonus),
            self.magic,
            self.distance,
        );
        let attack = (attack as f64 * self.accuracy_multiplier) as i32;
        let defence = melee_modifiers::Attributes {
            melee: 0,
            npc: self.attributes.npc,
        }
        .defence_roll(
            combat::base_roll(self.defence_level.wrapping_add(9), self.defence_bonus),
            self.invocation,
        );
        Ok(combat::hit_chance(attack, defence, false))
    }
    pub fn max_hit(self) -> Result<i32, Error> {
        let level = combat::effective_ranged(
            self.level,
            if self.style == Some(5) { 11 } else { 8 },
            self.damage_prayer,
            self.damage_void,
        )
        .map_err(|_| Error::Multiplier)?;
        let damage = self
            .attributes
            .damage(combat::base_damage(level, self.strength_bonus), self.magic);
        let damage = (damage as f64 * self.damage_multiplier) as i32;
        Ok(self
            .attributes
            .post_spec(damage, self.bolt_damage, self.speed))
    }
    pub fn roll(self, random: &mut impl RandomSource) -> Result<Roll, Error> {
        let chance = self.chance()?;
        if chance <= random.below(10000) as i32 {
            return Ok(Roll {
                chance,
                damage: 0,
                max_hit: None,
            });
        }
        let max_hit = self.max_hit()?;
        if max_hit < 1 || max_hit == i32::MAX {
            return Err(Error::EmptyDamageRange);
        }
        Ok(Roll {
            chance,
            damage: 1 + random.below(max_hit as u32) as i32,
            max_hit: Some(max_hit),
        })
    }
}
