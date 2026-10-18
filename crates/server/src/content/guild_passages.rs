//! Target-bound Crafting and Cooks' Guild entry outfits and temporary door stages.
use super::{Definitions, Symbols};
use gielinor_world::guild_passages::{Gate, Kind, Outfit};
const OUTFITS: [(&str, i32, &str, usize, i8, i8, bool, i32, bool, bool); 49] = [
    (
        "brown_apron",
        1757,
        "Brown apron",
        4,
        -1,
        -1,
        false,
        -1,
        true,
        false,
    ),
    (
        "golden_apron",
        20208,
        "Golden apron",
        4,
        -1,
        -1,
        false,
        -1,
        true,
        false,
    ),
    (
        "skillcape_crafting",
        9780,
        "Crafting cape",
        1,
        -1,
        -1,
        true,
        34,
        true,
        false,
    ),
    (
        "skillcape_crafting_trimmed",
        9781,
        "Crafting cape(t)",
        1,
        -1,
        -1,
        true,
        34,
        true,
        false,
    ),
    (
        "chefs_hat",
        1949,
        "Chef's hat",
        0,
        -1,
        -1,
        false,
        -1,
        false,
        true,
    ),
    (
        "chefs_hat_gold",
        20205,
        "Golden chef's hat",
        0,
        -1,
        -1,
        false,
        -1,
        false,
        true,
    ),
    (
        "skillcape_cooking",
        9801,
        "Cooking cape",
        1,
        -1,
        -1,
        true,
        34,
        false,
        true,
    ),
    (
        "skillcape_cooking_trimmed",
        9802,
        "Cooking cape(t)",
        1,
        -1,
        -1,
        true,
        34,
        false,
        true,
    ),
    (
        "skillcape_cooking_hood",
        9803,
        "Cooking hood",
        0,
        8,
        -1,
        true,
        35,
        false,
        true,
    ),
    (
        "varrock_armour_hard",
        13106,
        "Varrock armour 3",
        4,
        6,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "varrock_armour_elite",
        13107,
        "Varrock armour 4",
        4,
        6,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max",
        13280,
        "Max cape",
        1,
        -1,
        -1,
        true,
        36,
        true,
        true,
    ),
    (
        "skillcape_max_hood",
        13281,
        "Max hood",
        0,
        8,
        -1,
        true,
        37,
        false,
        true,
    ),
    (
        "skillcape_max_firecape",
        13329,
        "Fire max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_firecape",
        13330,
        "Fire max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_saradomin",
        13331,
        "Saradomin max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_saradomin",
        13332,
        "Saradomin max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_zamorak",
        13333,
        "Zamorak max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_zamorak",
        13334,
        "Zamorak max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_guthix",
        13335,
        "Guthix max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_guthix",
        13336,
        "Guthix max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_anma",
        13337,
        "Accumulator max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_anma",
        13338,
        "Accumulator max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_ardy",
        20760,
        "Ardougne max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_ardy",
        20764,
        "Ardougne max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_hood_infernalcape",
        21282,
        "Infernal max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_infernalcape",
        21285,
        "Infernal max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_saradomin2",
        21776,
        "Imbued saradomin max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_saradomin2",
        21778,
        "Imbued saradomin max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_zamorak2",
        21780,
        "Imbued zamorak max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_zamorak2",
        21782,
        "Imbued zamorak max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_guthix2",
        21784,
        "Imbued guthix max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_guthix2",
        21786,
        "Imbued guthix max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_assembler",
        21898,
        "Assembler max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_assembler",
        21900,
        "Assembler max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_infernalcape_trouver",
        24133,
        "Infernal max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_firecape_trouver",
        24134,
        "Fire max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_assembler_trouver",
        24135,
        "Assembler max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_saradomin2_trouver",
        24232,
        "Imbued saradomin max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_zamorak2_trouver",
        24233,
        "Imbued zamorak max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_guthix2_trouver",
        24234,
        "Imbued guthix max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_mythical",
        24855,
        "Mythical max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_mythical",
        24857,
        "Mythical max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_assembler_masori",
        27363,
        "Masori assembler max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_assembler_masori_trouver",
        27365,
        "Masori assembler max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_assembler_masori",
        27366,
        "Masori assembler max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_dizanas",
        28902,
        "Dizana's max cape",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
    (
        "skillcape_max_hood_dizanas",
        28904,
        "Dizana's max hood",
        0,
        8,
        -1,
        true,
        -1,
        false,
        true,
    ),
    (
        "skillcape_max_dizanas_trouver",
        28906,
        "Dizana's max cape (l)",
        1,
        -1,
        -1,
        true,
        -1,
        true,
        true,
    ),
];
pub fn guild_passages(
    objects: &Symbols,
    locations: &Symbols,
    stats: &Symbols,
    content: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, Gate>, String> {
    if content.get("max_cape") != Some(&36) || content.get("max_hood") != Some(&37) {
        return Err("Missing max outfit content aliases".into());
    }
    for (symbol, id, name) in [("crafting", 12, "Crafting"), ("cooking", 7, "Cooking")] {
        let d = defs.stats.get(&id).ok_or("Missing guild skill")?;
        if stats.get(symbol) != Some(&(id as i32))
            || d.name != name
            || d.unreleased
            || d.min_level != 1
            || d.max_level != 99
        {
            return Err("Invalid guild skill".into());
        }
    }
    for (symbol, id, name, op) in [
        ("craftingguilddoor", 14910, Some("Guild Door"), Some("Open")),
        ("inactivepoordoor", 1539, None, None),
        ("chefdoor", 24958, Some("Door"), Some("Open")),
        ("chefdoor_open", 24959, None, None),
    ] {
        let d = defs.locations.get(&id).ok_or("Missing guild door")?;
        if locations.get(symbol) != Some(&(id as i32))
            || name.is_some_and(|v| d.name != v)
            || d.ops[0].as_deref() != op
            || d.collision.width != 1
            || d.collision.length != 1
            || !d.multi.is_empty()
            || d.multi_default != -1
            || d.multi_varp != -1
            || d.multi_varbit != -1
        {
            return Err(format!("Invalid guild door {id}"));
        }
    }
    let mut crafting = Vec::new();
    let mut cooking = Vec::new();
    for &(symbol, id, name, slot, secondary, tertiary, members, group, craft, cook) in &OUTFITS {
        let d = defs.items.get(&(id as u32)).ok_or("Missing guild outfit")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || d.members != members
            || d.stackable
            || d.wear != [slot as i8, secondary, tertiary]
            || d.content_group != group
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid guild outfit {id}"));
        }
        let outfit = Outfit { id, slot, members };
        if craft {
            crafting.push(outfit);
        }
        if cook {
            cooking.push(outfit);
        }
    }
    Ok([
        (
            14910,
            Gate {
                kind: Kind::Crafting,
                opened: 1539,
                outfits: crafting,
            },
        ),
        (
            24958,
            Gate {
                kind: Kind::Cooking,
                opened: 24959,
                outfits: cooking,
            },
        ),
    ]
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guild_passages_all_target_wardrobes_and_late_bad_stage_or_gear_refuse_partial_profile() {
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
        for &(sym, id, name, slot, second, third, members, group, _, _) in &OUTFITS {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.wear = [slot as i8, second, third];
            t.members = members;
            t.content_group = group;
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        for (sym, id, name, op) in [
            ("craftingguilddoor", 14910, "Guild Door", true),
            ("inactivepoordoor", 1539, "null", false),
            ("chefdoor", 24958, "Door", true),
            ("chefdoor_open", 24959, "null", false),
        ] {
            let mut raw = vec![2];
            raw.extend(name.as_bytes());
            raw.push(0);
            if op {
                raw.push(30);
                raw.extend(b"Open\0");
            }
            raw.push(0);
            d.locations.insert(
                id,
                gielinor_cache::locations::LocationType::decode(&raw).unwrap(),
            );
            locations.insert(sym.into(), id as i32);
        }
        for (id, name) in [(12, "Crafting"), (7, "Cooking")] {
            let mut raw = vec![1];
            raw.extend(name.as_bytes());
            raw.extend([0, 0]);
            d.stats.insert(
                id,
                gielinor_cache::definitions::StatType::decode(&raw).unwrap(),
            );
        }
        let stats = [("crafting".into(), 12), ("cooking".into(), 7)].into();
        let content = [("max_cape".into(), 36), ("max_hood".into(), 37)].into();
        let gates = guild_passages(&objects, &locations, &stats, &content, &d).unwrap();
        assert_eq!(gates[&14910].outfits.len(), 27);
        assert_eq!(gates[&24958].outfits.len(), 45);
        for (&id, g) in &gates {
            let door = gielinor_world::locations::Location::new(
                gielinor_world::CoordGrid::new(3201, 3200, 0).unwrap(),
                id,
                0,
                0,
            )
            .unwrap();
            let source = door
                .coords
                .translate(0, if g.kind == Kind::Crafting { 1 } else { -1 }, 0)
                .unwrap();
            let skill = if g.kind == Kind::Crafting { 12 } else { 7 };
            let level = if g.kind == Kind::Crafting { 40 } else { 32 };
            let mut st = gielinor_world::stats::Stats::default();
            st.set_base(skill, level - 1);
            st.set_current(skill, level);
            for o in &g.outfits {
                let mut worn = gielinor_world::inventory::Inventory::new(
                    15,
                    gielinor_world::inventory::StackType::Normal,
                    false,
                );
                let item = gielinor_world::inventory::Item {
                    id: o.id,
                    count: 1,
                    vars: 0,
                };
                worn.restore_slot(o.slot, Some(item)).unwrap();
                assert!(
                    g.prepare(door, source, &st, &worn, &Default::default(), true)
                        .is_ok(),
                    "{id}/{}",
                    o.id
                );
                assert_eq!(
                    g.prepare(door, source, &st, &worn, &Default::default(), false)
                        .is_ok(),
                    !o.members
                );
                worn.restore_slot(
                    o.slot,
                    Some(gielinor_world::inventory::Item { vars: 9, ..item }),
                )
                .unwrap();
                assert!(g
                    .prepare(door, source, &st, &worn, &Default::default(), true)
                    .is_err());
            }
        }
        d.items.get_mut(&28906).unwrap().wear[0] = 0;
        assert!(guild_passages(&objects, &locations, &stats, &content, &d).is_err());
        d.items.get_mut(&28906).unwrap().wear[0] = 1;
        d.locations.get_mut(&24959).unwrap().multi_default = 1539;
        assert!(guild_passages(&objects, &locations, &stats, &content, &d).is_err());
    }
}
