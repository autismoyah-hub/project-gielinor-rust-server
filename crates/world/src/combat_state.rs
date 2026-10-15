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

#[cfg(test)]
mod wiki_tests {
    use super::*;
    #[test]
    fn single_combat_npc_guard_includes_eighth_tick_and_releases_ninth() {
        let varp = |id| Varp {
            id,
            permanent: false,
            transmit: crate::variables::Transmit::Never,
            bit_protect: false,
            client_code: 0,
        };
        let config = Config {
            last_npc: varp(1),
            last_pvp: varp(2),
            aggressive_npc: varp(3),
        };
        let mut vars = Variables::default();
        vars.set(config.last_pvp, -1000, false);
        let first = npc_uid(1, 41);
        let other = npc_uid(2, 41);
        config.record(&mut vars, 100, first, false);
        assert!(!config.denied(&vars, 108, first));
        assert!(config.denied(&vars, 108, other));
        assert!(!config.denied(&vars, 109, other));
        config.clear_on_death(&mut vars, other, false);
        assert_eq!(vars.get(config.last_npc.id), 100);
        config.clear_on_death(&mut vars, first, false);
        assert_eq!(vars.get(config.last_npc.id), 0);
    }
}
