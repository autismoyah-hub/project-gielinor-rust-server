//! Resolve the Prayer content group and XP parameters from the packed reference cache.
use crate::startup::Names;
use gielinor_cache::{definitions::Param, game::Definitions};
use gielinor_world::prayer::Config;
use std::{collections::BTreeMap, io};
pub fn resolve(names: &Names, defs: &Definitions) -> io::Result<Config> {
    let skill = u8::try_from(names.id("stat", "prayer")?).map_err(io::Error::other)?;
    let group = names.id("content", "prayer_bones")? as i32;
    let xp = names.id("param", "skill_xp")?;
    let mod_stat = names.id("param", "xpmod_stat")?;
    let mod_percent = names.id("param", "xpmod_percent")?;
    let mut bones = BTreeMap::new();
    let mut worn_xp_percent = BTreeMap::new();
    for (&id, ty) in &defs.items {
        if ty.content_group == group {
            let Some(Param::Integer(fine)) = ty.params.get(&xp) else {
                return Err(io::Error::other(format!("Missing prayer XP for bone {id}")));
            };
            if *fine < 0 {
                return Err(io::Error::other("Negative bone XP"));
            }
            bones.insert(id as i32, *fine as f64 / 10.0);
        }
        if matches!(ty.params.get(&mod_stat), Some(Param::Integer(v)) if *v == skill as i32) {
            if let Some(Param::Integer(percent)) = ty.params.get(&mod_percent) {
                worn_xp_percent.insert(id as i32, *percent);
            }
        }
    }
    if bones.is_empty() {
        return Err(io::Error::other("No packed prayer bone definitions"));
    }
    // Current KronosPrayerCatalog/PrayerRules mapping, source checked against Bone.java.
    // Floor-owned XP above still comes from PrayerBones edits (bat bones5.3).
    let mut restore = BTreeMap::new();
    for (symbol, points) in [
        ("bones", 1),
        ("big_bones", 2),
        ("babydragon_bones", 3),
        ("dragon_bones", 4),
        ("dagannoth_king_bones", 4),
        ("dragon_bones_superior", 5),
    ] {
        restore.insert(names.id("obj", symbol)? as i32, points);
    }
    Ok(Config {
        skill,
        animation: names.short("seq", "human_pickupfloor")?,
        bones,
        necklace: Some(names.id("obj", "dragonbone_necklace")? as i32),
        restore,
        worn_xp_percent,
        chaos: Some([2947, 3817, 2958, 3825]),
    })
}
