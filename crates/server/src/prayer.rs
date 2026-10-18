//! Resolve the Prayer content group and XP parameters from the packed reference cache.
use crate::startup::Names;
use gielinor_cache::{definitions::Param, game::Definitions, symbols::Symbols};
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

/// Five demonic Scatter rows; augment the existing Prayer transaction catalog.
/// Normal packed bone XP and restoration metadata are never overwritten.
pub fn configure_ashes(
    objects: &Symbols,
    sequences: &Symbols,
    defs: &Definitions,
    config: &mut Config,
) -> io::Result<BTreeMap<i32, u16>> {
    let mut bones = config.bones.clone();
    let animation = u16::try_from(
        *sequences
            .get("farming_ingredient_sprinkle")
            .ok_or_else(|| io::Error::other("Missing scatter sequence"))?,
    )
    .map_err(io::Error::other)?;
    if config.skill != 5
        || animation != 2295
        || defs.stats.get(&5).is_none_or(|d| {
            d.name != "Prayer" || d.unreleased || d.min_level != 1 || d.max_level != 99
        })
    {
        return Err(io::Error::other("Unattested ash Prayer/sequence binding"));
    }
    let mut ashes = BTreeMap::new();
    for (symbol, id, name, members, xp) in [
        ("fiendish_ashes", 25766, "Fiendish ashes", false, 10.0),
        ("vile_ashes", 25769, "Vile ashes", false, 25.0),
        ("malicious_ashes", 25772, "Malicious ashes", true, 65.0),
        ("abyssal_ashes", 25775, "Abyssal ashes", true, 85.0),
        ("infernal_ashes", 25778, "Infernal ashes", true, 110.0),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(io::Error::other("Aliased ash item"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| io::Error::other("Missing ash definition"))?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || d.inventory_ops[0].as_deref() != Some("Scatter")
            || config.restore.get(&id).is_some_and(|&value| value != 0)
            || bones.get(&id).is_some_and(|&value| value != xp)
        {
            return Err(io::Error::other(
                "Unattested ordinary Scatter profile or conflicting packed policy",
            ));
        }
        bones.insert(id, xp);
        ashes.insert(id, animation);
    }
    config.bones = bones;
    Ok(ashes)
}

#[cfg(test)]
mod ash_tests {
    use super::*;
    use gielinor_world::{
        inventory::{Catalog, Item},
        simulation::PlayerData,
        stats::Stats,
        CoordGrid,
    };
    #[test]
    fn ash_profiles_bind_exact_scatter_types_rates_and_stage_without_overwriting_bones() {
        let mut objects = Symbols::new();
        let sequences = Symbols::from([("farming_ingredient_sprinkle".into(), 2295)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut prayer = gielinor_cache::definitions::StatType::decode(&[0]).unwrap();
        prayer.name = "Prayer".into();
        defs.stats.insert(5, prayer);
        for (symbol, id, name, member) in [
            ("fiendish_ashes", 25766, "Fiendish ashes", false),
            ("vile_ashes", 25769, "Vile ashes", false),
            ("malicious_ashes", 25772, "Malicious ashes", true),
            ("abyssal_ashes", 25775, "Abyssal ashes", true),
            ("infernal_ashes", 25778, "Infernal ashes", true),
        ] {
            objects.insert(symbol.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            d.name = name.into();
            d.members = member;
            d.inventory_ops[0] = Some("Scatter".into());
            defs.items.insert(id as u32, d);
        }
        let baseline = Config {
            skill: 5,
            animation: 827,
            bones: [(526, 4.5)].into(),
            necklace: Some(22111),
            restore: [(526, 1)].into(),
            worn_xp_percent: Default::default(),
            chaos: Some([2947, 3817, 2958, 3825]),
        };
        let mut config = baseline.clone();
        let ashes = configure_ashes(&objects, &sequences, &defs, &mut config).unwrap();
        assert_eq!(ashes.len(), 5);
        assert_eq!(config.bones.len(), 6);
        assert_eq!(config.bones[&526], 4.5);
        assert_eq!(config.restore, baseline.restore);
        for (id, xp, fine) in [
            (25766, 10.0, 125),
            (25769, 25.0, 312),
            (25772, 65.0, 812),
            (25775, 85.0, 1062),
            (25778, 110.0, 1375),
        ] {
            assert_eq!(config.bones[&id], xp);
            assert_eq!(ashes[&id], 2295);
            let mut p = PlayerData::new(
                1,
                "Ash".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            );
            p.xp_rate_hundreds = 125;
            for slot in 0..28 {
                p.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            let pending = config
                .prepare_selected(&p, &Catalog::default(), id, 27)
                .unwrap();
            config
                .finish_selected(&mut p, &Catalog::default(), pending, 27)
                .unwrap()
                .unwrap();
            assert_eq!(p.stats.fine_xp(5), fine);
            assert_eq!(p.inventory.get(0).unwrap().id, id);
            assert!(p.inventory.get(27).is_none());
        }
        let mut config = baseline.clone();
        defs.items.get_mut(&25778).unwrap().cert.template = 799;
        assert!(configure_ashes(&objects, &sequences, &defs, &mut config).is_err());
        assert_eq!(config.bones, baseline.bones);
        defs.items.get_mut(&25778).unwrap().cert.template = 0;
        config.bones.insert(25778, 99.0);
        let before = config.bones.clone();
        assert!(configure_ashes(&objects, &sequences, &defs, &mut config).is_err());
        assert_eq!(config.bones, before);
    }
}
