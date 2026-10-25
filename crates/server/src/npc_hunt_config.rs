//! Cache `.hunt` modes for the world's NPC aggression, plus the ordinary_aggression table.
use gielinor_cache::{npcs::NpcType, symbols::Symbols};
use gielinor_world::hunt::Mode;
use std::collections::BTreeMap;

pub struct Resolved {
    pub modes: BTreeMap<u16, Mode>,
    pub defaults: BTreeMap<u16, u16>,
    pub decoded: usize,
    /// Modes whose `check_notcombat_self` names a varn other than the NPC `lastcombat`,
    /// which is the only NPC combat clock the world keeps (`Npc::last_combat`).
    pub unsupported: usize,
    pub cache_types: usize,
}
pub fn resolve(
    decoded: BTreeMap<u32, Mode>,
    varns: &Symbols,
    types: &BTreeMap<u32, NpcType>,
    defaults: BTreeMap<u16, u16>,
) -> Result<Resolved, String> {
    let count = decoded.len();
    let lastcombat = varns.get("lastcombat").copied();
    let mut modes = BTreeMap::new();
    let mut unsupported = 0;
    for (id, mode) in decoded {
        let id = u16::try_from(id).map_err(|_| format!("Hunt mode id {id} outside u16"))?;
        if mode
            .not_combat_self
            .is_some_and(|varn| Some(i32::from(varn)) != lastcombat)
        {
            unsupported += 1;
            continue;
        }
        modes.insert(id, mode);
    }
    let cache_types = types.values().filter(|t| t.hunt_mode.is_some()).count();
    Ok(Resolved {
        modes,
        defaults,
        decoded: count,
        unsupported,
        cache_types,
    })
}
impl Resolved {
    pub fn summary(&self) -> String {
        let attacking = self
            .modes
            .values()
            .filter(|m| m.player_hunt_supported())
            .count();
        format!(
            "NPC hunt: {} modes from cache ({} player attack, {} unsupported), {} types with a \
             cache hunt mode, {} types from ordinary_aggression",
            self.decoded,
            attacking,
            self.unsupported,
            self.cache_types,
            self.defaults.len()
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::hunt::Kind;
    #[test]
    fn foreign_npc_combat_clocks_are_unsupported() {
        let attack = Mode {
            kind: Kind::Player,
            find_new_mode: gielinor_world::hunt::OP_PLAYER2,
            not_combat_self: Some(1),
            ..Mode::default()
        };
        let other = Mode {
            not_combat_self: Some(7),
            ..attack.clone()
        };
        let varns = Symbols::from([("lastcombat".to_owned(), 1)]);
        let resolved = resolve(
            BTreeMap::from([(6, attack), (9, other), (22, Mode::default())]),
            &varns,
            &BTreeMap::new(),
            BTreeMap::from([(2856, 6)]),
        )
        .unwrap();
        assert_eq!(resolved.modes.keys().copied().collect::<Vec<_>>(), [6, 22]);
        assert_eq!(
            resolved.summary(),
            "NPC hunt: 3 modes from cache (1 player attack, 1 unsupported), 0 types with a cache \
             hunt mode, 1 types from ordinary_aggression"
        );
    }
}
