//! Random gems while mining and gem rocks (OpenRune `Mining.kt` rollGemPreTable,
//! `MiningGemDropTables.kt`, `MiningEquipment.kt` wearingChargedGlory, the
//! `mining_gemrock` row of `tables/skills/Mining.kt`).
//!
//! Pre-roll: /32768, or /8216 with a charged glory. The Wiki "Drop table" page
//! (rev 15263067, Mining gem drop table) gives the same rates: 1/256, or about 1/64.2
//! with a glory. The Mining page (rev 15374799) says 1/86 with a glory, which
//! contradicts it. The gem message follows OpenRune; the in-game text is unresolved.
use super::{Definitions, Symbols};
use gielinor_world::mining::{Gems, Rock};
use std::collections::{BTreeMap, BTreeSet};

/// Pre-roll gems after the 70 nothing rows (Drop table, Mining gem drop table).
pub const PRE_ROLL_GEMS: [(u32, &str, i32); 4] = [
    (32, "uncut_sapphire", 1623),
    (16, "uncut_emerald", 1621),
    (8, "uncut_ruby", 1619),
    (2, "uncut_diamond", 1617),
];
/// Gem rocks (Wiki rev 15361559), /128.
pub const GEM_ROCK_TABLE: [(u32, &str, i32); 7] = [
    (60, "uncut_opal", 1625),
    (30, "uncut_jade", 1627),
    (15, "uncut_red_topaz", 1629),
    (9, "uncut_sapphire", 1623),
    (5, "uncut_emerald", 1621),
    (5, "uncut_ruby", 1619),
    (4, "uncut_diamond", 1617),
];
pub const GEM_ROCKS: [(&str, u32); 2] = [("gemrock1", 11380), ("gemrock", 11381)];
pub const GEM_ROCK_EMPTY: (&str, u32) = ("rocks1", 11390);
pub const CHARGED_GLORY: [(&str, i32); 13] = [
    ("amulet_of_glory_1", 1706),
    ("amulet_of_glory_2", 1708),
    ("amulet_of_glory_3", 1710),
    ("amulet_of_glory_4", 1712),
    ("amulet_of_glory_5", 11976),
    ("amulet_of_glory_6", 11978),
    ("amulet_of_glory_inf", 19707),
    ("trail_amulet_of_glory_1", 10360),
    ("trail_amulet_of_glory_2", 10358),
    ("trail_amulet_of_glory_3", 10356),
    ("trail_amulet_of_glory_4", 10354),
    ("trail_amulet_of_glory_5", 11966),
    ("trail_amulet_of_glory_6", 11964),
];

fn symbol(group: &Symbols, kind: &str, name: &str, id: i64) -> Result<(), String> {
    let found = group.get(name).copied();
    if found.map(i64::from) != Some(id) {
        return Err(format!(
            "mining gems: {kind} {name} is {found:?} in rev-239 symbols, expected {id}"
        ));
    }
    Ok(())
}

/// Gem rock entries for the mining config, and the gem settings. `pre_roll_rocks` are
/// the OpenRune-eligible rock ids already configured (ordinary, lead, limestone, clay).
pub fn mining_gems(
    obj: &Symbols,
    loc: &Symbols,
    defs: &Definitions,
    pre_roll_rocks: BTreeSet<u32>,
) -> Result<(BTreeMap<u32, Rock>, Gems), String> {
    let mut names = BTreeMap::new();
    for &(_, name, id) in PRE_ROLL_GEMS.iter().chain(&GEM_ROCK_TABLE) {
        symbol(obj, "obj", name, id.into())?;
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("mining gems: missing item {id}"))?;
        if item.stackable || item.cert.template != 0 {
            return Err(format!("mining gems: {id} is not an ordinary uncut gem"));
        }
        names.insert(id, item.name.clone());
    }
    let mut charged_glory = BTreeSet::new();
    for &(name, id) in &CHARGED_GLORY {
        symbol(obj, "obj", name, id.into())?;
        charged_glory.insert(id);
    }
    let (empty_name, empty) = GEM_ROCK_EMPTY;
    symbol(loc, "loc", empty_name, empty.into())?;
    if !defs.locations.contains_key(&empty) {
        return Err("mining gems: missing empty gem rock".into());
    }
    let mut rocks = BTreeMap::new();
    for &(name, id) in &GEM_ROCKS {
        symbol(loc, "loc", name, id.into())?;
        let ty = defs
            .locations
            .get(&id)
            .ok_or_else(|| format!("mining gems: missing gem rock {id}"))?;
        if !ty.ops[0].as_deref().is_some_and(|op| op.eq_ignore_ascii_case("Mine")) {
            return Err(format!("mining gems: gem rock {id} has no Mine op"));
        }
        rocks.insert(
            id,
            Rock {
                members: true,
                restore: None,
                level: 40,
                // Replaced by the gem-rock table on each success; the capacity
                // message reads "Your inventory is too full to hold any more gems."
                product: GEM_ROCK_TABLE[0].2,
                product_name: "gems".into(),
                xp: 65.0,
                depleted: empty,
                respawn: 99,
                rates: (28, 70),
            },
        );
    }
    let gems = Gems {
        pre_roll_rocks,
        pre_roll_gems: PRE_ROLL_GEMS.iter().map(|&(w, _, id)| (w, id)).collect(),
        gem_rocks: rocks.keys().copied().collect(),
        gem_rock_table: GEM_ROCK_TABLE.iter().map(|&(w, _, id)| (w, id)).collect(),
        names,
        charged_glory,
    };
    Ok((rocks, gems))
}
