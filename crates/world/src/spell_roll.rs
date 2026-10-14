//! PvN spell formula stages. Attribute collection must run separately for the
//! accuracy and damage stages: Kotlin rolls Ahrim/Brimstone again on a hit.
use crate::{
    combat, melee_modifiers,
    melee_roll::Error,
    npc::RandomSource,
    spell_modifiers::{Attributes, Spell},
};
#[derive(Clone, Copy)]
pub struct Inputs {
    pub magic: i32,
    pub attack_bonus: i32,
    /// Caller selects current defence or magic using the NPC's type parameter.
    pub defence_level: i32,
    pub defence_bonus: i32,
    pub accuracy_prayer: f64,
    pub accuracy_void: f64,
    pub damage_bonus: i32,
    pub damage_prayer: i32,
    pub invocation: i32,
    pub weakness_percent: i32,
    pub speed: i32,
}
impl Inputs {
    pub fn chance(self, attributes: Attributes) -> Result<i32, Error> {
        // Spell attacks pass a null style, whose base bonus is nine.
        let level =
            combat::effective_attack(self.magic, 9, self.accuracy_prayer, self.accuracy_void)
                .map_err(|_| Error::Multiplier)?;
        let attack = attributes.accuracy(
            combat::base_roll(level, self.attack_bonus),
            self.weakness_percent,
        );
        let mut defence = melee_modifiers::Attributes {
            melee: 0,
            npc: attributes.npc,
        }
        .defence_roll(
            combat::base_roll(self.defence_level.wrapping_add(9), self.defence_bonus),
            self.invocation,
        );
        if attributes.spell & (1 << Spell::BrimstonePassive as u8) != 0 {
            defence = defence.wrapping_mul(9) / 10;
        }
        Ok(combat::hit_chance(attack, defence, false))
    }
    pub fn accurate(
        self,
        attributes: Attributes,
        random: &mut impl RandomSource,
    ) -> Result<bool, Error> {
        Ok(self.chance(attributes)? > random.below(10000) as i32)
    }
    pub fn damage_range(self, attributes: Attributes, base: i32) -> (i32, i32) {
        let modified = attributes.damage(base, self.magic, self.damage_bonus, self.damage_prayer);
        attributes.damage_range(modified, base, self.speed, self.weakness_percent)
    }
    /// Separate from the maximum-hit varp write so callers preserve its ordering
    /// even if malformed metadata produces an invalid inclusive range.
    pub fn damage(range: (i32, i32), random: &mut impl RandomSource) -> Result<i32, Error> {
        let (min, max) = range;
        let count = max as i64 - min as i64 + 1;
        // DefaultGameRandom adds one in Int before calling nextInt; MAX overflows.
        if max == i32::MAX || count <= 0 || count > u32::MAX as i64 {
            return Err(Error::EmptyDamageRange);
        }
        Ok((min as i64 + random.below(count as u32) as i64) as i32)
    }
}
/// Flags ordered as mystic will, lore, might, augury, vigour unlocked.
pub fn prayer_bonuses([will, lore, might, augury, vigour]: [i32; 5]) -> (f64, i32) {
    let accuracy = if will == 1 {
        1.05
    } else if lore == 1 {
        1.1
    } else if might == 1 {
        if vigour == 1 {
            1.18
        } else {
            1.15
        }
    } else if augury == 1 {
        1.25
    } else {
        1.0
    };
    let damage = if lore == 1 {
        10
    } else if might == 1 {
        if vigour == 1 {
            30
        } else {
            20
        }
    } else if augury == 1 {
        40
    } else {
        0
    };
    (accuracy, damage)
}
