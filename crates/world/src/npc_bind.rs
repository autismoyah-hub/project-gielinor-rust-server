//! Runtime data bindings, applied after cache definitions on spawn and respawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binding {
    pub stats: [i32; 6],
    /// `None` keeps the multiplier configured from NPC params (`npc_com_xp_multiplier`).
    pub xp_multiplier: Option<i32>,
    pub spawn_animation: Option<u16>,
    pub respawn_ticks: i32,
}
impl Binding {
    pub fn apply(self, npc: &mut crate::npc::Npc) {
        npc.stats = self.stats;
        npc.base_stats = self.stats;
        if let Some(multiplier) = self.xp_multiplier {
            npc.combat_xp_multiplier = multiplier;
        }
        npc.respawn_rate = self.respawn_ticks;
    }
}
