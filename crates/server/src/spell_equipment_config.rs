//! Cache symbols resolved from BaseObjs and the existing spell equipment checks.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::{spell_equipment::Catalog, spell_modifiers::Spell};
use std::collections::{BTreeMap, BTreeSet};
pub fn resolve(
    objects: &Symbols,
    params: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<Catalog, String> {
    let obj = |name: &str| {
        objects
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing spell equipment symbol {name}"))
    };
    let set = |names: &[&str]| -> Result<BTreeSet<i32>, String> {
        names.iter().map(|name| obj(name)).collect()
    };
    let mut c = Catalog::default();
    let mut add = |slot: usize, attribute: Spell, names: &[&str]| -> Result<(), String> {
        for name in names {
            *c.items.entry((slot, obj(name)?)).or_default() |= 1 << attribute as u8;
        }
        Ok(())
    };
    add(3, Spell::SlayerStaffE, &["slayer_staff_enchanted"])?;
    add(9, Spell::ChaosGauntlets, &["gauntlets_of_chaos"])?;
    add(
        3,
        Spell::SmokeStaff,
        &[
            "smoke_battlestaff",
            "mystic_smoke_battlestaff",
            "twinflame_staff",
        ],
    )?;
    add(2, Spell::AmuletOfAvarice, &["wild_cave_amulet"])?;
    add(2, Spell::SalveAmuletEi, &["nzone_salve_amulet_e"])?;
    add(2, Spell::SalveAmuletI, &["nzone_salve_amulet"])?;
    add(3, Spell::DragonHunterWand, &["dragonhunter_wand"])?;
    add(3, Spell::DragonHunterLance, &["dragonhunter_lance"])?;
    add(
        3,
        Spell::DragonHunterCrossbow,
        &[
            "dragonhunter_xbow",
            "dragonhunter_xbow_vorkath",
            "dragonhunter_xbow_kbd",
        ],
    )?;
    add(
        3,
        Spell::RevenantWeapon,
        &[
            "wild_cave_accursed_charged",
            "wild_cave_accursed_charged_recol",
            "wild_cave_sceptre_charged",
            "wild_cave_sceptre_charged_recol",
        ],
    )?;
    add(3, Spell::PurgingStaff, &["purging_staff"])?;
    add(5, Spell::WaterTome, &["tome_of_water"])?;
    add(5, Spell::EarthTome, &["tome_of_earth"])?;
    add(5, Spell::FireTome, &["tome_of_fire"])?;
    add(12, Spell::EfaritaysAid, &["vampyre_ring"])?;
    drop(add);
    c.damned = set(&["damned_amulet"])?;
    c.brimstone = set(&["brimstone_ring"])?;
    c.void_helm = set(&[
        "game_pest_mage_helm",
        "game_pest_mage_helm_trouver",
        "league_3_void_mage_helm",
        "league_3_void_mage_helm_trouver",
    ])?;
    c.void_gloves = set(&[
        "pest_void_knight_gloves",
        "pest_void_knight_gloves_trouver",
        "league_3_void_knight_gloves",
        "league_3_void_knight_gloves_trouver",
    ])?;
    c.void_top = set(&[
        "pest_void_knight_top",
        "pest_void_knight_top_trouver",
        "league_3_void_knight_top",
        "league_3_void_knight_top_trouver",
        "elite_void_knight_top",
        "elite_void_knight_top_trouver",
        "league_3_void_knight_top_elite",
        "league_3_void_knight_top_elite_trouver",
    ])?;
    c.void_robe = set(&[
        "pest_void_knight_robes",
        "pest_void_knight_robes_trouver",
        "league_3_void_knight_robes",
        "league_3_void_knight_robes_trouver",
        "elite_void_knight_robes",
        "elite_void_knight_robes_trouver",
        "league_3_void_knight_robes_elite",
        "league_3_void_knight_robes_elite_trouver",
    ])?;
    c.ahrim[0] = set(&[
        "barrows_ahrim_head_100",
        "barrows_ahrim_head_75",
        "barrows_ahrim_head_50",
        "barrows_ahrim_head_25",
    ])?;
    c.ahrim[1] = set(&[
        "barrows_ahrim_body_100",
        "barrows_ahrim_body_75",
        "barrows_ahrim_body_50",
        "barrows_ahrim_body_25",
    ])?;
    c.ahrim[2] = set(&[
        "barrows_ahrim_legs_100",
        "barrows_ahrim_legs_75",
        "barrows_ahrim_legs_50",
        "barrows_ahrim_legs_25",
    ])?;
    c.ahrim[3] = set(&[
        "barrows_ahrim_weapon_100",
        "barrows_ahrim_weapon_75",
        "barrows_ahrim_weapon_50",
        "barrows_ahrim_weapon_25",
    ])?;
    let mut spell = |attribute: Spell, names: &[&str]| -> Result<(), String> {
        for name in names {
            *c.spells.entry(obj(name)?).or_default() |= 1 << attribute as u8;
        }
        Ok(())
    };
    spell(
        Spell::WindSpell,
        &[
            "01_wind_strike",
            "17_wind_bolt",
            "41_wind_blast",
            "62_wind_wave",
            "81_wind_surge",
        ],
    )?;
    spell(
        Spell::WaterSpell,
        &[
            "05_water_strike",
            "23_water_bolt",
            "47_water_blast",
            "65_water_wave",
            "85_water_surge",
        ],
    )?;
    spell(
        Spell::EarthSpell,
        &[
            "09_earth_strike",
            "29_earth_bolt",
            "53_earth_blast",
            "70_earth_wave",
            "90_earth_surge",
        ],
    )?;
    spell(
        Spell::FireSpell,
        &[
            "13_fire_strike",
            "35_fire_bolt",
            "59_fire_blast",
            "75_fire_wave",
            "95_fire_surge",
        ],
    )?;
    spell(Spell::BindSpell, &["20_bind", "50_snare", "79_entangle"])?;
    spell(Spell::MagicDart, &["50_magic_dart"])?;
    spell(
        Spell::BoltSpell,
        &[
            "17_wind_bolt",
            "23_water_bolt",
            "29_earth_bolt",
            "35_fire_bolt",
        ],
    )?;
    spell(
        Spell::Demonbane,
        &[
            "br_mithril_scimitar",
            "br_willow_bow",
            "br_adamant_scimitar",
        ],
    )?;
    drop(spell);
    c.god_spells = set(&[
        "60_claws_of_guthix",
        "60_flames_of_zamorak",
        "60_saradomin_strike",
    ])?;
    let masks = ["blackmask_imbued", "slayer_helm_imbued"].map(|name| {
        params
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing spell mask parameter {name}"))
    });
    let masks = [masks[0].clone()? as u32, masks[1].clone()? as u32];
    for (&id, item) in &defs.items {
        for &param in &masks {
            let value = match item.params.get(&param) {
                Some(Param::Integer(v)) => *v,
                Some(_) => return Err("Noninteger spell mask parameter".into()),
                None => parameter_types
                    .get(&param)
                    .and_then(|p| p.default_int)
                    .ok_or("Missing spell mask default")?,
            };
            if value != 0 {
                *c.items.entry((0, id as i32)).or_default() |= 1 << Spell::BlackMaskI as u8;
                break;
            }
        }
    }
    Ok(c)
}
