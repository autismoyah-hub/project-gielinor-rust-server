//! Canonical cache names resolved from BaseObjs and EquipmentChecks.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::{melee_equipment::Catalog, melee_modifiers::Melee};
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
            .ok_or_else(|| format!("Missing melee equipment symbol {name}"))
    };
    let obj = |name: &str| symbol(objects, name);
    let set = |names: &[&str]| -> Result<BTreeSet<i32>, String> {
        names.iter().map(|name| obj(name)).collect()
    };
    let mut catalog = Catalog::default();
    let mut add = |slot: usize, attr: Melee, names: &[&str]| -> Result<(), String> {
        for name in names {
            *catalog.items.entry((slot, obj(name)?)).or_default() |= 1u64 << attr as u8;
        }
        Ok(())
    };
    add(2, Melee::AmuletOfAvarice, &["wild_cave_amulet"])?;
    add(
        2,
        Melee::SalveAmuletE,
        &["lotr_crystalshard_necklace_upgrade", "nzone_salve_amulet_e"],
    )?;
    add(
        2,
        Melee::SalveAmulet,
        &["crystalshard_necklace", "nzone_salve_amulet"],
    )?;
    add(
        2,
        Melee::BerserkerNeck,
        &["jewl_beserker_necklace", "jewl_beserker_necklace_ornament"],
    )?;
    add(3, Melee::Arclight, &["arclight"])?;
    add(3, Melee::BurningClaws, &["bone_claws"])?;
    add(
        3,
        Melee::TzHaarWeapon,
        &[
            "tzhaar_knife",
            "tzhaar_splitsword",
            "tzhaar_mace",
            "tzhaar_maul",
            "tzhaar_maul_t",
        ],
    )?;
    add(3, Melee::DragonHunterLance, &["dragonhunter_lance"])?;
    add(3, Melee::DragonHunterWand, &["dragonhunter_wand"])?;
    add(3, Melee::KerisBreachPartisan, &["keris_partisan_breach"])?;
    add(3, Melee::KerisSunPartisan, &["keris_partisan_sun"])?;
    add(
        3,
        Melee::KerisWeapon,
        &[
            "contact_keris",
            "contact_keris_p",
            "contact_keris_p+",
            "contact_keris_p++",
            "keris_partisan",
            "keris_partisan_corruption",
        ],
    )?;
    add(
        3,
        Melee::BarroniteMaceWeapon,
        &["barronite_mace", "barronite_mace_trouver"],
    )?;
    add(
        3,
        Melee::RevenantWeapon,
        &["wild_cave_chainmace_charged", "wild_cave_ursine_charged"],
    )?;
    add(3, Melee::Silverlight, &["agrith_silverlight_dyed"])?;
    add(
        3,
        Melee::LeafBladed,
        &[
            "leafbladed_sword",
            "slayer_leafbladed_spear",
            "leafbladed_battleaxe",
        ],
    )?;
    add(3, Melee::ColossalBlade, &["giants_foundry_colossal_blade"])?;
    add(3, Melee::RatBoneWeapon, &["rat_bone_mace"])?;
    add(3, Melee::InquisitorWeapon, &["inquisitors_mace"])?;
    add(
        3,
        Melee::OsmumtensFang,
        &["osmumtens_fang", "osmumtens_fang_ornament"],
    )?;
    add(3, Melee::Gadderhammer, &["gadderanks_warhammer"])?;
    add(0, Melee::InquisitorHelm, &["inquisitors_helm"])?;
    add(4, Melee::InquisitorTop, &["inquisitors_body"])?;
    add(7, Melee::InquisitorBottom, &["inquisitors_skirt"])?;
    catalog.obsidian = [
        set(&["obsidian_helmet"])?,
        set(&["obsidian_platebody"])?,
        set(&["obsidian_platelegs"])?,
    ];
    catalog.dharok = [
        set(&[
            "barrows_dharok_head_100",
            "barrows_dharok_head_75",
            "barrows_dharok_head_50",
            "barrows_dharok_head_25",
        ])?,
        set(&[
            "barrows_dharok_body_100",
            "barrows_dharok_body_75",
            "barrows_dharok_body_50",
            "barrows_dharok_body_25",
        ])?,
        set(&[
            "barrows_dharok_legs_100",
            "barrows_dharok_legs_75",
            "barrows_dharok_legs_50",
            "barrows_dharok_legs_25",
        ])?,
        set(&[
            "barrows_dharok_weapon_100",
            "barrows_dharok_weapon_75",
            "barrows_dharok_weapon_50",
            "barrows_dharok_weapon_25",
        ])?,
    ];
    catalog.void = [
        set(&[
            "game_pest_melee_helm",
            "game_pest_melee_helm_trouver",
            "league_3_void_melee_helm",
            "league_3_void_melee_helm_trouver",
        ])?,
        set(&[
            "pest_void_knight_top",
            "pest_void_knight_top_trouver",
            "league_3_void_knight_top",
            "league_3_void_knight_top_trouver",
            "elite_void_knight_top",
            "elite_void_knight_top_trouver",
            "league_3_void_knight_top_elite",
            "league_3_void_knight_top_elite_trouver",
        ])?,
        set(&[
            "pest_void_knight_robes",
            "pest_void_knight_robes_trouver",
            "league_3_void_knight_robes",
            "league_3_void_knight_robes_trouver",
            "elite_void_knight_robes",
            "elite_void_knight_robes_trouver",
            "league_3_void_knight_robes_elite",
            "league_3_void_knight_robes_elite_trouver",
        ])?,
        set(&[
            "pest_void_knight_gloves",
            "pest_void_knight_gloves_trouver",
            "league_3_void_knight_gloves",
            "league_3_void_knight_gloves_trouver",
        ])?,
    ];
    catalog.soulreaper = set(&["soulreaper"])?;
    catalog.corpbane = set(&["osmumtens_fang", "osmumtens_fang_ornament"])?;
    let halberd = symbol(categories, "halberd")?;
    let spear = symbol(categories, "spear")?;
    let masks = [
        "blackmask",
        "slayer_helm",
        "blackmask_imbued",
        "slayer_helm_imbued",
    ]
    .map(|name| symbol(params, name).map(|v| v as u32))
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    for (&id, item) in &defs.items {
        if item.category == halberd || item.category == spear {
            catalog.corpbane.insert(id as i32);
        }
        for &param in &masks {
            let value = match item.params.get(&param) {
                Some(Param::Integer(v)) => *v,
                Some(_) => return Err(format!("Invalid black-mask parameter {param}")),
                None => parameter_types
                    .get(&param)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing black-mask default {param}"))?,
            };
            if value != 0 {
                *catalog.items.entry((0, id as i32)).or_default() |= 1u64 << Melee::BlackMask as u8;
                break;
            }
        }
    }
    Ok(catalog)
}
