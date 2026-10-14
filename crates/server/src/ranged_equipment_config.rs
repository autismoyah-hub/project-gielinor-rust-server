//! Canonical names from BaseObjs, EquipmentChecks and CombatRangedAttributeCollector.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::{ranged_equipment::Catalog, ranged_modifiers::Ranged};
use std::collections::{BTreeMap, BTreeSet};
pub fn resolve(
    objects: &Symbols,
    params: &Symbols,
    categories: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<Catalog, String> {
    let symbol = |map: &Symbols, name: &str| {
        map.get(name)
            .copied()
            .ok_or_else(|| format!("Missing ranged equipment symbol {name}"))
    };
    let obj = |name: &str| symbol(objects, name);
    let set = |names: &[&str]| -> Result<BTreeSet<i32>, String> {
        names.iter().map(|name| obj(name)).collect()
    };
    let mut c = Catalog::default();
    let mut add = |slot: usize, attribute: Ranged, names: &[&str]| -> Result<(), String> {
        for name in names {
            *c.items.entry((slot, obj(name)?)).or_default() |= 1 << attribute as u8;
        }
        Ok(())
    };
    add(2, Ranged::AmuletOfAvarice, &["wild_cave_amulet"])?;
    add(2, Ranged::SalveAmuletEi, &["nzone_salve_amulet_e"])?;
    add(2, Ranged::SalveAmuletI, &["nzone_salve_amulet"])?;
    add(
        3,
        Ranged::CrystalBow,
        &[
            "crystal_bow",
            "bow_of_faerdhinen",
            "bow_of_faerdhinen_infinite",
            "bow_of_faerdhinen_infinite_ithell",
            "bow_of_faerdhinen_infinite_iorwerth",
            "bow_of_faerdhinen_infinite_trahaearn",
            "bow_of_faerdhinen_infinite_cadarn",
            "bow_of_faerdhinen_infinite_crwys",
            "bow_of_faerdhinen_infinite_meilyr",
            "bow_of_faerdhinen_infinite_amlodd",
        ],
    )?;
    add(
        0,
        Ranged::CrystalHelm,
        &[
            "crystal_helmet_hefin",
            "crystal_helmet_ithell",
            "crystal_helmet_iorwerth",
            "crystal_helmet_trahaearn",
            "crystal_helmet_cadarn",
            "crystal_helmet_crwys",
            "crystal_helmet",
            "crystal_helmet_amlodd",
        ],
    )?;
    add(
        4,
        Ranged::CrystalBody,
        &[
            "crystal_chestplate_hefin",
            "crystal_chestplate_ithell",
            "crystal_chestplate_iorwerth",
            "crystal_chestplate_trahaearn",
            "crystal_chestplate_cadarn",
            "crystal_chestplate_crwys",
            "crystal_chestplate",
            "crystal_chestplate_amlodd",
        ],
    )?;
    add(
        7,
        Ranged::CrystalLegs,
        &[
            "crystal_platelegs_hefin",
            "crystal_platelegs_ithell",
            "crystal_platelegs_iorwerth",
            "crystal_platelegs_trahaearn",
            "crystal_platelegs_cadarn",
            "crystal_platelegs_crwys",
            "crystal_platelegs",
            "crystal_platelegs_amlodd",
        ],
    )?;
    add(3, Ranged::TwistedBow, &["twisted_bow"])?;
    add(
        3,
        Ranged::RevenantWeapon,
        &["wild_cave_bow_charged", "wild_cave_webweaver_charged"],
    )?;
    add(
        3,
        Ranged::DragonHunterCrossbow,
        &[
            "dragonhunter_xbow",
            "dragonhunter_xbow_vorkath",
            "dragonhunter_xbow_kbd",
        ],
    )?;
    add(3, Ranged::ScorchingBow, &["scorching_bow"])?;
    add(3, Ranged::RatBoneWeapon, &["rat_bone_bow"])?;
    c.void_helm = set(&[
        "game_pest_archer_helm",
        "game_pest_archer_helm_trouver",
        "league_3_void_range_helm",
        "league_3_void_range_helm_trouver",
    ])?;
    c.void_gloves = set(&[
        "pest_void_knight_gloves",
        "pest_void_knight_gloves_trouver",
        "league_3_void_knight_gloves",
        "league_3_void_knight_gloves_trouver",
    ])?;
    c.regular_top = set(&[
        "pest_void_knight_top",
        "pest_void_knight_top_trouver",
        "league_3_void_knight_top",
        "league_3_void_knight_top_trouver",
    ])?;
    c.regular_robe = set(&[
        "pest_void_knight_robes",
        "pest_void_knight_robes_trouver",
        "league_3_void_knight_robes",
        "league_3_void_knight_robes_trouver",
    ])?;
    c.elite_top = set(&[
        "elite_void_knight_top",
        "elite_void_knight_top_trouver",
        "league_3_void_knight_top_elite",
        "league_3_void_knight_top_elite_trouver",
    ])?;
    c.elite_robe = set(&[
        "elite_void_knight_robes",
        "elite_void_knight_robes_trouver",
        "league_3_void_knight_robes_elite",
        "league_3_void_knight_robes_elite_trouver",
    ])?;
    let chinchompa = symbol(categories, "chinchompa")?;
    let masks = [
        symbol(params, "blackmask_imbued")? as u32,
        symbol(params, "slayer_helm_imbued")? as u32,
    ];
    for (&id, item) in &defs.items {
        if item.category == chinchompa {
            c.chinchompas.insert(id as i32);
        }
        for &param in &masks {
            let value = match item.params.get(&param) {
                Some(Param::Integer(v)) => *v,
                Some(_) => return Err(format!("Invalid ranged mask parameter {param}")),
                None => parameter_types
                    .get(&param)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing ranged mask default {param}"))?,
            };
            if value != 0 {
                *c.items.entry((0, id as i32)).or_default() |= 1 << Ranged::BlackMaskI as u8;
                break;
            }
        }
    }
    Ok(c)
}
