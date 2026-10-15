//! Ordered awards from PlayerAttackManager. Cast XP is awarded separately by spells.
use crate::attack_profile::Attack;
/// Selected melee award at the pre-impact XP boundary, not an applied-damage claim.
#[derive(Clone, Debug, PartialEq)]
pub struct MeleeTrace {
    pub tick: i32,
    pub actor: crate::simulation::ActorId,
    pub target: crate::simulation::NpcId,
    pub npc_type: u16,
    pub style: Option<u8>,
    pub stance: u8,
    pub damage: i32,
    pub target_health: i32,
    pub npc_multiplier: i32,
    pub rate: f64,
    /// Skill id, fine XP before, fine XP after, in award order.
    pub awards: Vec<(u8, i32, i32)>,
}
impl MeleeTrace {
    pub fn json(&self) -> String {
        let style = self
            .style
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".into());
        let rate = if self.rate.is_finite() {
            self.rate.to_string()
        } else {
            "null".into()
        };
        let awards = self
            .awards
            .iter()
            .map(|(skill, before, after)| format!("[{skill},{before},{after}]"))
            .collect::<Vec<_>>()
            .join(",");
        format!(concat!("{{\"schema\":1,\"tick\":{},\"actor_slot\":{},\"actor_generation\":{},",
            "\"npc_slot\":{},\"npc_generation\":{},\"npc_type\":{},\"policy\":\"selected_chicken_melee\",",
            "\"style\":{},\"stance\":{},\"damage\":{},\"target_health\":{},\"npc_multiplier\":{},\"xp_rate\":{},\"awards\":[{}]}}"),
            self.tick,self.actor.slot,self.actor.generation,self.target.slot,self.target.generation,
            self.npc_type,style,self.stance,self.damage,self.target_health,self.npc_multiplier,rate,awards)
    }
}
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

#[cfg(test)]
mod wiki_tests {
    use super::*;
    #[test]
    fn accurate_melee_truncates_each_hit_and_clamps_overkill() {
        let attack = Attack::Melee {
            kind: None,
            style: Some(2),
            stance: 0,
        };
        for (rate, per_hit_hp, combined_hp, attack_fine) in [(1.0, 13, 39, 40), (5.0, 66, 199, 200)]
        {
            let mut split = crate::stats::Stats::default();
            for hits in 1..=3 {
                let changes = advance(&mut split, attack, 1, 10, 1.0, rate).unwrap();
                assert_eq!(
                    changes.iter().map(|(skill, _)| *skill).collect::<Vec<_>>(),
                    [Skill::Attack, Skill::Hitpoints]
                );
                assert_eq!(split.fine_xp(0), attack_fine * hits);
                assert_eq!(split.fine_xp(3), per_hit_hp * hits);
            }
            let mut combined = crate::stats::Stats::default();
            advance(&mut combined, attack, 30, 3, 1.0, rate).unwrap();
            assert_eq!(combined.fine_xp(0), attack_fine * 3);
            assert_eq!(combined.fine_xp(3), combined_hp);
            let before = combined.clone();
            advance(&mut combined, attack, 30, 0, 1.0, rate).unwrap();
            assert_eq!(combined, before);
        }
    }
}
