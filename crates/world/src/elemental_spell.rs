//! Existing ElementalSpells metadata. Rune/level overrides are supplied by the
//! content resolver; max damage scales by family and current magic level.
use crate::{projectile, spell_cast};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug)]
pub enum Tier {
    Strike,
    Bolt,
    Blast,
    Wave,
    Surge,
}
impl Tier {
    pub fn max_hit(self, level: i32) -> i32 {
        let (thresholds, hits) = match self {
            Self::Strike => ([5, 9, 13], [2, 4, 6, 8]),
            Self::Bolt => ([23, 29, 35], [9, 10, 11, 12]),
            Self::Blast => ([47, 53, 59], [13, 14, 15, 16]),
            Self::Wave => ([65, 70, 75], [17, 18, 19, 20]),
            Self::Surge => ([85, 90, 95], [21, 22, 23, 24]),
        };
        hits[thresholds
            .into_iter()
            .filter(|&threshold| level >= threshold)
            .count()]
    }
}
#[derive(Clone)]
pub struct Spell {
    /// Original registry metadata for manual-selection checks, before cooldown.
    pub selection: spell_cast::Definition,
    /// Elemental handler requirements after optional Kronos substitution.
    pub definition: spell_cast::Definition,
    pub tier: Tier,
    pub staff_animation: u16,
    pub unarmed_animation: u16,
    pub launch: u16,
    pub travel: u16,
    pub impact: u16,
    pub cast_sound: u16,
    pub hit_sound: u16,
}
pub struct Catalog {
    pub spells: BTreeMap<i32, Spell>,
    pub staves: BTreeSet<i32>,
    pub projectile: projectile::Style,
    pub splash_spot: u16,
    pub splash_sound: u16,
    pub sunfire: i32,
    pub global_xp_rate: f64,
}
