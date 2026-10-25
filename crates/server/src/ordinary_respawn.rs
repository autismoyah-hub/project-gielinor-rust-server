//! Issue37 Part A: five effective respawn rates, without changing packed cache
//! bytes or any combat stat. The symbolic cache anchors are revision239.
//! Archived TOML explicitly supplies chicken/cow/giant-rat rates; goblin3028
//! and guard3254 instead use the cited Wiki values (not neighbouring variants).
use gielinor_cache::{npcs::NpcType, symbols::Symbols};
use std::collections::BTreeMap;

pub struct Row {
    pub symbol: &'static str,
    pub id: u16,
    pub name: &'static str,
    pub ticks: i32,
    pub wiki_page: &'static str,
    pub wiki_revision: u64,
}

pub const ROWS: &[Row] = &[
    Row { symbol: "chicken", id: 1173, name: "Chicken", ticks: 25, wiki_page: "Chicken", wiki_revision: 15283952 },
    Row { symbol: "cow", id: 2790, name: "Cow", ticks: 45, wiki_page: "Cow", wiki_revision: 15364588 },
    Row { symbol: "goblin", id: 3028, name: "Goblin", ticks: 35, wiki_page: "Goblin", wiki_revision: 15374683 },
    Row { symbol: "giantrat", id: 2856, name: "Giant rat", ticks: 30, wiki_page: "Giant rat", wiki_revision: 15260179 },
    Row { symbol: "guard1", id: 3254, name: "Guard", ticks: 50, wiki_page: "Guard", wiki_revision: 15321511 },
];

pub fn resolve(names: &Symbols, types: &BTreeMap<u32, NpcType>) -> Result<BTreeMap<u16, i32>, String> {
    let mut rates = BTreeMap::new();
    for row in ROWS {
        if names.get(row.symbol).copied() != Some(i32::from(row.id)) {
            return Err(format!("ordinary_respawn: symbol {} must resolve to {}", row.symbol, row.id));
        }
        let ty = types.get(&u32::from(row.id))
            .ok_or_else(|| format!("ordinary_respawn: missing NPC type {}", row.id))?;
        if ty.name != row.name {
            return Err(format!("ordinary_respawn: NPC {} name {:?}, expected {}", row.id, ty.name, row.name));
        }
        if rates.insert(row.id, row.ticks).is_some() || row.ticks <= 0 {
            return Err("ordinary_respawn: invalid or duplicate rate".into());
        }
    }
    Ok(rates)
}
