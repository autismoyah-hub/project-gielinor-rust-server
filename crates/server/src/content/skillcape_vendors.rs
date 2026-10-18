//! Target-bound guild cape sellers; ordinary NPC dialogue and inventory remain shared.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::skillcape_vendor::{Config, Kind};
const ITEMS: [(&str, i32, &str, Option<u8>); 7] = [
    ("coins", 995, "Coins", None),
    ("skillcape_crafting", 9780, "Crafting cape", Some(12)),
    (
        "skillcape_crafting_trimmed",
        9781,
        "Crafting cape(t)",
        Some(12),
    ),
    ("skillcape_crafting_hood", 9782, "Crafting hood", Some(12)),
    ("skillcape_cooking", 9801, "Cooking cape", Some(7)),
    (
        "skillcape_cooking_trimmed",
        9802,
        "Cooking cape(t)",
        Some(7),
    ),
    ("skillcape_cooking_hood", 9803, "Cooking hood", Some(7)),
];
pub fn skillcape_vendors(
    objects: &Symbols,
    npcs: &Symbols,
    stats: &Symbols,
    params: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u16, Config>, String> {
    if params.get("statreq1_skill") != Some(&434) || params.get("statreq1_level") != Some(&436) {
        return Err("Missing vendor stat requirement aliases".into());
    }
    for (symbol, id, name) in [("crafting", 12, "Crafting"), ("cooking", 7, "Cooking")] {
        let d = defs.stats.get(&id).ok_or("Missing vendor skill")?;
        if stats.get(symbol) != Some(&(id as i32))
            || d.name != name
            || d.unreleased
            || d.min_level != 1
            || d.max_level != 99
        {
            return Err("Invalid vendor skill".into());
        }
    }
    for &(symbol, id, name, skill) in &ITEMS {
        let d = defs.items.get(&(id as u32)).ok_or("Missing vendor item")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || d.members != skill.is_some()
            || d.stackable != skill.is_none()
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid vendor item {id}"));
        }
        if let Some(skill) = skill {
            let hood = [9782, 9803].contains(&id);
            let wear = if hood { [0, 8, -1] } else { [1, -1, -1] };
            if d.wear != wear
                || d.inventory_ops[1].as_deref() != Some("Wear")
                || d.params.get(&434) != Some(&Param::Integer(skill as i32))
                || d.params.get(&436) != Some(&Param::Integer(99))
            {
                return Err(format!("Invalid vendor wear/requirements {id}"));
            }
        }
    }
    let active_stats = defs
        .stats
        .iter()
        .filter(|(id, d)| **id < 24 && !d.unreleased && d.max_level == 99)
        .fold(0u32, |mask, (&id, _)| mask | (1u32 << id));
    if active_stats & (1 << 7) == 0 || active_stats & (1 << 12) == 0 {
        return Err("Missing active vendor skills".into());
    }
    let mut result = std::collections::BTreeMap::new();
    for (symbol, id, name, kind, skill, capes, hood) in [
        (
            "master_crafter",
            5810,
            "Master Crafter",
            Kind::Crafting,
            12,
            [9780, 9781],
            9782,
        ),
        (
            "head_chef",
            2658,
            "Head chef",
            Kind::Cooking,
            7,
            [9801, 9802],
            9803,
        ),
    ] {
        let d = defs.npcs.get(&(id as u32)).ok_or("Missing cape vendor")?;
        if npcs.get(symbol) != Some(&(id as i32))
            || d.name != name
            || d.size != 1
            || d.ops[0].as_deref() != Some("Talk-to")
            || !d.multi.is_empty()
            || d.multi_default.is_some()
            || d.multi_varbit.is_some()
            || d.multi_varp.is_some()
        {
            return Err(format!("Invalid cape vendor {id}"));
        }
        result.insert(
            id,
            Config {
                kind,
                npc: id,
                skill,
                coins: 995,
                capes,
                hood,
                active_stats,
            },
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vendor_dialogue_profiles_bind_real_sellers_and_released_skill_trim_mask_without_partial_config(
    ) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut objects = Symbols::new();
        for &(sym, id, name, skill) in &ITEMS {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = skill.is_some();
            t.stackable = skill.is_none();
            if let Some(skill) = skill {
                t.wear = if [9782, 9803].contains(&id) {
                    [0, 8, -1]
                } else {
                    [1, -1, -1]
                };
                t.inventory_ops[1] = Some("Wear".into());
                t.params.insert(434, Param::Integer(skill as i32));
                t.params.insert(436, Param::Integer(99));
            }
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        let mut npcs = Symbols::new();
        for (sym, id, name) in [
            ("master_crafter", 5810, "Master Crafter"),
            ("head_chef", 2658, "Head chef"),
        ] {
            let mut raw = vec![2];
            raw.extend(name.bytes());
            raw.extend(b"\0\x1eTalk-to\0\0");
            d.npcs
                .insert(id, gielinor_cache::npcs::NpcType::decode(&raw).unwrap());
            npcs.insert(sym.into(), id as i32);
        }
        for (id, name) in [
            (0, "Attack"),
            (7, "Cooking"),
            (12, "Crafting"),
            (23, "Sailing"),
        ] {
            let mut ty = gielinor_cache::definitions::StatType::decode(b"\0").unwrap();
            ty.name = name.into();
            ty.unreleased = id == 23;
            d.stats.insert(id, ty);
        }
        let stats = [("crafting".into(), 12), ("cooking".into(), 7)].into();
        let params = [
            ("statreq1_skill".into(), 434),
            ("statreq1_level".into(), 436),
        ]
        .into();
        let c = skillcape_vendors(&objects, &npcs, &stats, &params, &d).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[&5810].kind, Kind::Crafting);
        assert_eq!(c[&2658].capes, [9801, 9802]);
        assert_eq!(c[&5810].active_stats, 1 | (1 << 7) | (1 << 12));
        npcs.insert("head_chef".into(), 4626);
        assert!(skillcape_vendors(&objects, &npcs, &stats, &params, &d).is_err());
        npcs.insert("head_chef".into(), 2658);
        d.items.get_mut(&9803).unwrap().transform.template = 1;
        assert!(skillcape_vendors(&objects, &npcs, &stats, &params, &d).is_err());
    }
}
