//! Resolved NPC combat metadata. Spawn XP is fixed at creation; target attributes
//! include current health and the attacking player's slayer assignment.
use crate::melee_modifiers::Npc;
#[derive(Clone, Debug)]
pub struct Definition {
    pub xp_multiplier: i32,
    pub defence: [i32; 3],
    pub attributes: u64,
    pub attack_rate: i32,
    pub retreat: i32,
    pub aggression_range: i32,
}
impl Definition {
    pub fn attributes(&self, health: i32, max_health: i32, slayer: bool) -> u64 {
        self.attributes
            | if health < max_health / 4 {
                1u64 << Npc::QuarterHealth as u8
            } else {
                0
            }
            | if slayer {
                1u64 << Npc::SlayerTask as u8
            } else {
                0
            }
    }
}
