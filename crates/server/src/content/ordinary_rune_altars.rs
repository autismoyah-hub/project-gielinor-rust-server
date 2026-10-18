//! Ordinary six-rune altar profiles use independent target and Wiki facts.
use super::{Definitions, Symbols};
use gielinor_world::runecrafting::{Config, OutfitBinding, Recipe};
const ROWS: [(&str, u32, &str, i32, &str, u8, u16, u8); 6] = [
    ("air_altar", 34760, "airrune", 556, "Air rune", 1, 50, 11),
    ("mind_altar", 34761, "mindrune", 558, "Mind rune", 2, 55, 14),
    (
        "water_altar",
        34762,
        "waterrune",
        555,
        "Water rune",
        5,
        60,
        19,
    ),
    (
        "earth_altar",
        34763,
        "earthrune",
        557,
        "Earth rune",
        9,
        65,
        26,
    ),
    (
        "fire_altar",
        34764,
        "firerune",
        554,
        "Fire rune",
        14,
        70,
        35,
    ),
    (
        "body_altar",
        34765,
        "bodyrune",
        559,
        "Body rune",
        20,
        75,
        46,
    ),
];
const HELD_GUARDS: [(&str, i32); 5] = [
    ("gotr_guardian_essence", 26879),
    ("scar_extract_warped", 28593),
    ("scar_extract_twisted", 28595),
    ("scar_extract_mangled", 28597),
    ("scar_extract_scarred", 28599),
];
const WORN_GUARDS: [(&str, i32); 14] = [
    ("abyssal_lantern", 26822),
    ("abyssal_lantern_normal", 26824),
    ("abyssal_lantern_normal_blue", 26826),
    ("abyssal_lantern_normal_red", 26828),
    ("abyssal_lantern_normal_white", 26830),
    ("abyssal_lantern_normal_purple", 26832),
    ("abyssal_lantern_normal_green", 26834),
    ("abyssal_lantern_oak", 26836),
    ("abyssal_lantern_willow", 26838),
    ("abyssal_lantern_maple", 26840),
    ("abyssal_lantern_yew", 26842),
    ("abyssal_lantern_blisterwood", 26844),
    ("abyssal_lantern_magic", 26846),
    ("abyssal_lantern_redwood", 26848),
];
const OUTFIT: [(&str, i32, &str, u8, usize, i8); 13] = [
    ("hat_of_the_eye", 26850, "Hat of the eye", 0, 0, 8),
    ("hat_of_the_eye_red", 26858, "Hat of the eye (red)", 0, 0, 8),
    (
        "hat_of_the_eye_green",
        26864,
        "Hat of the eye (green)",
        0,
        0,
        8,
    ),
    (
        "hat_of_the_eye_blue",
        26870,
        "Hat of the eye (blue)",
        0,
        0,
        8,
    ),
    ("robe_top_of_the_eye", 26852, "Robe top of the eye", 1, 4, 6),
    (
        "robe_top_of_the_eye_red",
        26860,
        "Robe top of the eye (red)",
        1,
        4,
        6,
    ),
    (
        "robe_top_of_the_eye_green",
        26866,
        "Robe top of the eye (green)",
        1,
        4,
        6,
    ),
    (
        "robe_top_of_the_eye_blue",
        26872,
        "Robe top of the eye (blue)",
        1,
        4,
        6,
    ),
    (
        "robe_bottom_of_the_eye",
        26854,
        "Robe bottoms of the eye",
        2,
        7,
        -1,
    ),
    (
        "robe_bottom_of_the_eye_red",
        26862,
        "Robe bottoms of the eye (red)",
        2,
        7,
        -1,
    ),
    (
        "robe_bottom_of_the_eye_green",
        26868,
        "Robe bottoms of the eye (green)",
        2,
        7,
        -1,
    ),
    (
        "robe_bottom_of_the_eye_blue",
        26874,
        "Robe bottoms of the eye (blue)",
        2,
        7,
        -1,
    ),
    ("boots_of_the_eye", 26856, "Boots of the eye", 3, 10, -1),
];
pub fn ordinary_rune_altars(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if stats.get("runecrafting") != Some(&20)
        || sequences.get("human_runecraft") != Some(&791)
        || spots.get("runecrafting") != Some(&186)
    {
        return Err("Missing Rune altar skill/effect aliases".into());
    }
    let stat = defs.stats.get(&20).ok_or("Missing Runecrafting stat")?;
    if stat.name != "Runecrafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99
    {
        return Err("Invalid Runecrafting stat".into());
    }
    let check = |symbol: &str, id: i32, name: &str, stack: bool| -> Result<(), String> {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Rune altar item")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || d.members
            || d.stackable != stack
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid Rune altar item {id}"));
        }
        Ok(())
    };
    check("blankrune", 1436, "Rune essence", false)?;
    check("blankrune_high", 7936, "Pure essence", false)?;
    let daeyalt = defs.items.get(&24704).ok_or("Missing Daeyalt essence")?;
    if objects.get("blankrune_daeyalt") != Some(&24704)
        || daeyalt.name != "Daeyalt essence"
        || !daeyalt.members
        || daeyalt.stackable
        || daeyalt.dummy == 1
        || daeyalt.cert.template != 0
        || daeyalt.placeholder.template != 0
        || daeyalt.transform.template != 0
    {
        return Err("Invalid member Daeyalt essence".into());
    }

    let mut altars = std::collections::BTreeMap::new();
    for &(symbol, id, item_symbol, output, name, level, xp_tenths, interval) in &ROWS {
        let d = defs
            .locations
            .get(&id)
            .ok_or("Missing Rune altar location")?;
        if locations.get(symbol) != Some(&(id as i32))
            || d.name != "Altar"
            || d.collision.width != 3
            || d.collision.length != 3
            || d.ops[0].as_deref() != Some("Craft-rune")
            || !d.multi.is_empty()
            || d.multi_varbit != -1
            || d.multi_varp != -1
            || d.multi_default != -1
        {
            return Err(format!("Invalid ordinary Rune altar {id}"));
        }
        check(item_symbol, output, name, true)?;
        altars.insert(
            id,
            Recipe {
                output,
                level,
                xp_tenths,
                interval,
                name,
            },
        );
    }
    for &(symbol, id) in HELD_GUARDS.iter().chain(WORN_GUARDS.iter()) {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing unsupported Rune modifier binding")?;
        if objects.get(symbol) != Some(&id)
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid unsupported Rune modifier {id}"));
        }
    }
    let mut outfit = std::collections::BTreeMap::new();
    for &(symbol, id, name, group, slot, secondary) in &OUTFIT {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Raiments item")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || !d.members
            || d.stackable
            || d.wear != [slot as i8, secondary, -1]
            || d.inventory_ops[1].as_deref() != Some("Wear")
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid Raiments item {id}"));
        }
        outfit.insert(id, OutfitBinding { group, slot });
    }
    Ok(Config {
        altars,
        outfit,
        blocked_held: HELD_GUARDS.iter().map(|v| v.1).collect(),
        blocked_worn: WORN_GUARDS.iter().map(|v| v.1).collect(),
        animation: 791,
        spot: 186,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rune_altars_exact_six_static_profiles_and_late_bad_output_do_not_publish_partial_config() {
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
        let mut locations = Symbols::new();
        for (symbol, id, name, stack) in [
            ("blankrune", 1436, "Rune essence", false),
            ("blankrune_high", 7936, "Pure essence", false),
        ]
        .into_iter()
        .chain(ROWS.iter().map(|v| (v.2, v.3, v.4, true)))
        {
            let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            item.name = name.into();
            item.stackable = stack;
            d.items.insert(id as u32, item);
            objects.insert(symbol.into(), id);
        }
        let mut daeyalt = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
        daeyalt.name = "Daeyalt essence".into();
        daeyalt.members = true;
        d.items.insert(24704, daeyalt);
        objects.insert("blankrune_daeyalt".into(), 24704);
        for &(symbol, id) in HELD_GUARDS.iter().chain(WORN_GUARDS.iter()) {
            d.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(b"\0").unwrap(),
            );
            objects.insert(symbol.into(), id);
        }
        for &(symbol, id, name, _, slot, secondary) in &OUTFIT {
            let mut item = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            item.name = name.into();
            item.members = true;
            item.wear = [slot as i8, secondary, -1];
            item.inventory_ops[1] = Some("Wear".into());
            d.items.insert(id as u32, item);
            objects.insert(symbol.into(), id);
        }
        for &(symbol, id, ..) in &ROWS {
            let mut raw = b"\x02Altar\0\x0e\x03\x0f\x03\x1eCraft-rune\0\0".to_vec();
            let ty = gielinor_cache::locations::LocationType::decode(&raw).unwrap();
            raw.clear();
            d.locations.insert(id, ty);
            locations.insert(symbol.into(), id as i32);
        }
        d.stats.insert(
            20,
            gielinor_cache::definitions::StatType::decode(b"\x01Runecrafting\0\0").unwrap(),
        );
        let seq = [("human_runecraft".into(), 791)].into();
        let spots = [("runecrafting".into(), 186)].into();
        let stats = [("runecrafting".into(), 20)].into();
        let c = ordinary_rune_altars(&objects, &locations, &seq, &spots, &stats, &d).unwrap();
        assert_eq!(c.altars.len(), 6);
        assert_eq!(c.altars[&34763].level, 9);
        assert_eq!(c.blocked_held.len(), 5);
        assert!(!c.blocked_held.contains(&24704));
        assert_eq!(c.blocked_worn.len(), 14);
        assert_eq!(c.outfit.len(), 13);
        assert_eq!(c.outfit[&26874].slot, 7);
        assert_eq!(
            c.altars.values().map(|v| v.xp_tenths).collect::<Vec<_>>(),
            vec![50, 55, 60, 65, 70, 75]
        );
        d.items.get_mut(&26856).unwrap().wear[0] = 3;
        assert!(ordinary_rune_altars(&objects, &locations, &seq, &spots, &stats, &d).is_err());
        d.items.get_mut(&26856).unwrap().wear[0] = 10;
        d.items.get_mut(&559).unwrap().dummy = 1;
        assert!(ordinary_rune_altars(&objects, &locations, &seq, &spots, &stats, &d).is_err());
    }
}
