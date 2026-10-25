//! Exact rev-239 Goblin 3028 correction. Goblin Wiki revision 15374683 maps
//! this ID to the level-2 version with 5 HP; packed cache HP remains 3.
//! This layer changes only effective HP, before explicit Kronos overrides.
use gielinor_cache::{npcs::NpcType, symbols::Symbols};
use std::collections::BTreeMap;

pub const ID: u16 = 3028;
pub const HITPOINTS: u16 = 5;
pub const WIKI_REVISION: u64 = 15374683;

pub fn resolve(names: &Symbols, types: &BTreeMap<u32, NpcType>) -> Result<BTreeMap<u16, u16>, String> {
    if names.get("goblin").copied() != Some(i32::from(ID)) {
        return Err("ordinary_hitpoints: goblin must resolve to 3028".into());
    }
    let ty = types.get(&u32::from(ID))
        .ok_or("ordinary_hitpoints: missing NPC type 3028")?;
    if ty.name != "Goblin" || ty.combat_level != 2 {
        return Err(format!("ordinary_hitpoints: NPC 3028 expected Goblin combat 2, got {:?} combat {}", ty.name, ty.combat_level));
    }
    Ok([(ID, HITPOINTS)].into())
}
