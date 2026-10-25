//! Intentional adaptation (#38): hunt modes for ordinary-world NPC types that have none.
//!
//! No 239.1 reference source marks an ordinary-world NPC aggressive: the cache assigns a
//! hunt mode to one type (the duck's NPC hunt), OpenRune `npcs.toml` only to bosses and
//! God Wars minions, and the rsmod enricher to none. These rows follow the OSRS Wiki
//! infobox `aggressive = Yes`. A cache `hunt_mode` always wins: the world only fills
//! types whose cache type has no hunt mode.
//!
//! All rows use the cache mode `constant_melee`: it applies the Wiki's combat-level rule
//! (`check_nottoostrong`) and keeps hunting after a find, so the NPC attacks again once
//! combat ends. `aggressive_*` ignore level, and `cowardly` stops hunting after one find.
use gielinor_cache::{npcs::NpcType, symbols::Symbols};
use std::collections::BTreeMap;

pub struct Row {
    /// rev-239 `npc.sym` name; must resolve to `id`.
    pub npc: &'static str,
    pub id: u16,
    /// `hunt.sym` name of the cache hunt mode to use.
    pub hunt: &'static str,
    pub wiki_page: &'static str,
    pub wiki_revision: u64,
}

const GIANT_RAT: (&str, u64) = ("Giant rat", 15260179);
const MUGGER: (&str, u64) = ("Mugger", 15350646);

const fn row(npc: &'static str, id: u16, wiki: (&'static str, u64)) -> Row {
    Row {
        npc,
        id,
        hunt: "constant_melee",
        wiki_page: wiki.0,
        wiki_revision: wiki.1,
    }
}

/// Giant rat (level 3, wiki ids 2856-2861) and Mugger (level 6, wiki id 513).
pub const ROWS: &[Row] = &[
    row("giantrat", 2856, GIANT_RAT),
    row("giantrat2", 2857, GIANT_RAT),
    row("giantrat3", 2858, GIANT_RAT),
    row("giantrat_grey", 2859, GIANT_RAT),
    row("giantrat_grey2", 2860, GIANT_RAT),
    row("giantrat_grey3", 2861, GIANT_RAT),
    row("mugger", 513, MUGGER),
];

/// NPC type to hunt mode id, for rows whose cache type has no hunt mode.
pub fn resolve(
    npc_names: &Symbols,
    hunt_names: &Symbols,
    types: &BTreeMap<u32, NpcType>,
) -> Result<BTreeMap<u16, u16>, String> {
    let mut result = BTreeMap::new();
    for row in ROWS {
        let symbol = npc_names.get(row.npc).copied();
        if symbol != Some(row.id as i32) {
            return Err(format!(
                "ordinary_aggression: npc {} is {symbol:?} in rev-239 symbols, expected {}",
                row.npc, row.id
            ));
        }
        let ty = types
            .get(&u32::from(row.id))
            .ok_or_else(|| format!("ordinary_aggression: no cache type {}", row.id))?;
        let hunt = hunt_names
            .get(row.hunt)
            .copied()
            .and_then(|id| u16::try_from(id).ok())
            .ok_or_else(|| format!("ordinary_aggression: missing hunt mode {}", row.hunt))?;
        if ty.hunt_mode.is_none() {
            result.insert(row.id, hunt);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rows_cite_wiki_and_are_unique() {
        let mut ids = std::collections::BTreeSet::new();
        for row in ROWS {
            assert!(ids.insert(row.id), "duplicate {}", row.id);
            assert!(row.wiki_revision > 0 && !row.wiki_page.is_empty());
        }
    }
    #[test]
    fn symbols_must_match_and_cache_modes_win() {
        let mut npcs = Symbols::new();
        let mut hunts = Symbols::new();
        hunts.insert("constant_melee".into(), 1);
        let mut types = BTreeMap::new();
        for row in ROWS {
            npcs.insert(row.npc.into(), row.id as i32);
            types.insert(u32::from(row.id), NpcType::decode(&[0]).unwrap());
        }
        types.get_mut(&513).unwrap().hunt_mode = Some(3);
        let table = resolve(&npcs, &hunts, &types).unwrap();
        assert_eq!(table.len(), ROWS.len() - 1);
        assert_eq!(table[&2856], 1);
        assert!(!table.contains_key(&513), "cache hunt_mode wins");
        npcs.insert("mugger".into(), 514);
        assert!(resolve(&npcs, &hunts, &types).is_err());
    }
}
