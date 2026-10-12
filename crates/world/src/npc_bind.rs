//! Runtime data bindings, applied after cache definitions on spawn and respawn.
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub stats: [i32; 6],
    pub xp_multiplier: i32,
    pub spawn_animation: Option<u16>,
    pub respawn_ticks: i32,
}
impl Binding {
    pub fn apply(self, npc: &mut crate::npc::Npc) {
        npc.stats = self.stats;
        npc.base_stats = self.stats;
        npc.combat_xp_multiplier = self.xp_multiplier;
        npc.respawn_rate = self.respawn_ticks;
    }
}
