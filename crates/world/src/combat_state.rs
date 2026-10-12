//! Temporary player combat varps used by the reference's single-combat checks.
use crate::variables::{Update, Variables, Varp};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub last_npc: Varp,
    pub last_pvp: Varp,
    pub aggressive_npc: Varp,
}
pub fn npc_uid(slot: u16, kind: u16) -> i32 {
    ((kind as u32) << 16 | slot as u32) as i32
}
impl Config {
    pub fn denied(self, vars: &Variables, clock: i32, target: i32) -> bool {
        vars.get(self.last_pvp.id).wrapping_add(8) >= clock
            || (vars.get(self.last_npc.id).wrapping_add(8) >= clock
                && vars.get(self.aggressive_npc.id) != target)
    }
    pub fn record(
        self,
        vars: &mut Variables,
        clock: i32,
        source: i32,
        logged: bool,
    ) -> Vec<Update> {
        [(self.last_npc, clock), (self.aggressive_npc, source)]
            .into_iter()
            .filter_map(|(ty, value)| vars.set(ty, value, logged))
            .collect()
    }
    pub fn clear_on_death(self, vars: &mut Variables, source: i32, logged: bool) -> Option<Update> {
        if vars.get(self.aggressive_npc.id) == source {
            vars.set(self.last_npc, 0, logged)
        } else {
            None
        }
    }
}
