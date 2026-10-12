//! Ordered awards from PlayerAttackManager. Cast XP is awarded separately by spells.
use crate::attack_profile::Attack;
pub fn player_multiplier(combat_level: i32) -> f64 {
    (1.0 + 0.025 * (combat_level / 20) as f64).min(1.125)
}
/// AiSpawnScript's default multiplier, stored in thousandths on each NPC.
pub fn npc_multiplier(
    preset: Option<i32>,
    levels: [i32; 4],
    defence: [i32; 3],
    attack: i32,
    strength: i32,
) -> i32 {
    if let Some(value) = preset {
        return value;
    }
    let average = levels[0]
        .wrapping_add(levels[1])
        .wrapping_add(levels[2])
        .wrapping_add(levels[3].min(2000))
        / 4;
    let defence = defence[0].wrapping_add(defence[1]).wrapping_add(defence[2]) / 3;
    let scaled = average.wrapping_mul(defence.wrapping_add(strength).wrapping_add(attack)) / 5120;
    ((1.0 + 0.025 * scaled as f64) * 1000.0).round_ties_even() as i32
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skill {
    Attack,
    Strength,
    Defence,
    Ranged,
    Magic,
    Hitpoints,
}
impl Skill {
    pub fn id(self) -> u8 {
        match self {
            Self::Attack => 0,
            Self::Defence => 1,
            Self::Strength => 2,
            Self::Hitpoints => 3,
            Self::Ranged => 4,
            Self::Magic => 6,
        }
    }
}
/// Applies awards in the Kotlin manager's order. Each result identifies the
/// stat update/level-up work the world must publish, including zero-XP awards.
pub fn advance(
    stats: &mut crate::stats::Stats,
    attack: Attack,
    damage: i32,
    remaining_health: i32,
    multiplier: f64,
    rate: f64,
) -> Result<Vec<(Skill, crate::skills::Advancement)>, crate::stats::InvalidXp> {
    awards(attack, damage, remaining_health, multiplier)
        .into_iter()
        .map(|(skill, xp)| {
            crate::skills::advance(stats, skill.id(), xp, rate, 99).map(|change| (skill, change))
        })
        .collect()
}
pub fn awards(
    attack: Attack,
    damage: i32,
    remaining_health: i32,
    multiplier: f64,
) -> Vec<(Skill, f64)> {
    let damage = damage.min(remaining_health) as f64;
    let mut xp = Vec::new();
    let mut grant = |skill, rate| xp.push((skill, damage * rate * multiplier));
    match attack {
        Attack::Melee { style, .. } => match style {
            Some(1) => {
                grant(Skill::Attack, 1.33);
                grant(Skill::Strength, 1.33);
                grant(Skill::Defence, 1.33);
            }
            Some(2) => grant(Skill::Attack, 4.0),
            Some(3) => grant(Skill::Strength, 4.0),
            Some(4) => grant(Skill::Defence, 4.0),
            _ => {}
        },
        Attack::Ranged { style, .. } => match style {
            Some(5 | 6) => grant(Skill::Ranged, 4.0),
            Some(7) => {
                grant(Skill::Ranged, 2.0);
                grant(Skill::Defence, 2.0);
            }
            _ => {}
        },
        Attack::Spell { defensive: true } => {
            grant(Skill::Magic, 1.33);
            grant(Skill::Defence, 1.0);
        }
        Attack::Spell { defensive: false } | Attack::Staff { .. } => grant(Skill::Magic, 2.0),
    }
    grant(Skill::Hitpoints, 1.33);
    xp
}
