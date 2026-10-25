//! OpenRune ordinary fishing (FishingConfig/FishingTable) bound to revision-239.1 identities.
//! 239.1 has no fishing content groups, so every spot is an attested NPC with checked options.
use super::{Definitions, Symbols};
use gielinor_world::fishing::{Binding, Catch, Config, Method, Tool};
use std::collections::BTreeMap;

// OpenRune FishingSpotDef ids.
const NET_BAIT: u8 = 0;
const LURE_BAIT: u8 = 1;
const CAGE_HARPOON: u8 = 2;
const BIG_NET_HARPOON: u8 = 3;
const SLIMY_EEL: u8 = 5;
const CAVE_EEL: u8 = 6;
const LAVA_EEL: u8 = 7;
const ANGLERFISH: u8 = 8;
const DARK_CRAB: u8 = 9;
// OpenRune FishingMethod ids.
const NET: u8 = 0;
const BAIT: u8 = 1;
const LURE: u8 = 2;
const CAGE: u8 = 3;
const HARPOON: u8 = 4;
const BIG_NET: u8 = 5;
const OILY_ROD: u8 = 6;
const SANDWORM: u8 = 7;
const DARK_CRAB_CAGE: u8 = 8;

/// (one-based operation, method, accepted 239.1 option texts) per spot kind. Cave eel
/// spots offer "Small Net" (frog spawn, unsupported) first; OpenRune binds Bait to op1,
/// the cache and Wiki place it on op3.
fn options(kind: u8) -> &'static [(u8, u8, &'static [&'static str])] {
    match kind {
        NET_BAIT => &[(1, NET, &["Small Net", "Net"]), (3, BAIT, &["Bait"])],
        LURE_BAIT => &[(1, LURE, &["Lure"]), (3, BAIT, &["Bait"])],
        CAGE_HARPOON => &[(1, CAGE, &["Cage"]), (3, HARPOON, &["Harpoon"])],
        BIG_NET_HARPOON => &[(1, BIG_NET, &["Big Net", "Net"]), (3, HARPOON, &["Harpoon"])],
        SLIMY_EEL => &[(1, BAIT, &["Bait"])],
        CAVE_EEL => &[(3, BAIT, &["Bait"])],
        LAVA_EEL => &[(1, OILY_ROD, &["Bait"])],
        ANGLERFISH => &[(1, SANDWORM, &["Bait"])],
        DARK_CRAB => &[(1, DARK_CRAB_CAGE, &["Cage"])],
        _ => &[],
    }
}

/// OpenRune npcs.toml spot groups whose names exist in 239.1.
const SPOTS: &[(&str, u16, u8)] = &[
    ("0_20_46_saltfish", 14041, NET_BAIT),
    ("0_20_47_saltfish", 14524, NET_BAIT),
    ("0_21_46_saltfish", 14040, NET_BAIT),
    ("0_23_53_saltfish", 7459, NET_BAIT),
    ("0_24_51_saltfish", 14038, NET_BAIT),
    ("0_24_53_saltfish", 7462, NET_BAIT),
    ("0_26_54_saltfish", 7467, NET_BAIT),
    ("0_27_59_saltfish", 7469, NET_BAIT),
    ("0_28_56_saltfish", 7155, NET_BAIT),
    ("0_39_44_saltfish", 7947, NET_BAIT),
    ("0_39_55_saltfish", 1544, NET_BAIT),
    ("0_41_57_saltfish", 3913, NET_BAIT),
    ("0_43_51_saltfish", 1514, NET_BAIT),
    ("0_44_52_saltfish", 1517, NET_BAIT),
    ("0_44_53_saltfish", 1518, NET_BAIT),
    ("0_45_49_saltfish", 1521, NET_BAIT),
    ("0_46_49_saltfish", 1523, NET_BAIT),
    ("0_47_57_saltfish", 1524, NET_BAIT),
    ("0_48_50_saltfish", 1525, NET_BAIT),
    ("0_50_49_saltfish", 1530, NET_BAIT),
    ("0_51_49_saltfish", 1528, NET_BAIT),
    ("0_19_48_freshfish", 14521, LURE_BAIT),
    ("0_19_53_freshfish", 14525, LURE_BAIT),
    ("0_19_57_freshfish", 8524, LURE_BAIT),
    ("0_20_152_freshfish", 14522, LURE_BAIT),
    ("0_20_52_freshfish", 14526, LURE_BAIT),
    ("0_21_51_freshfish", 14527, LURE_BAIT),
    ("0_22_52_freshfish", 14528, LURE_BAIT),
    ("0_24_49_freshfish", 12774, LURE_BAIT),
    ("0_24_55_freshfish", 7463, LURE_BAIT),
    ("0_25_50_freshfish", 14036, LURE_BAIT),
    ("0_25_55_freshfish", 7464, LURE_BAIT),
    ("0_26_56_freshfish", 7468, LURE_BAIT),
    ("0_26_57_freshfish", 394, LURE_BAIT),
    ("0_34_50_freshfish", 3417, LURE_BAIT),
    ("0_35_50_freshfish", 3418, LURE_BAIT),
    ("0_37_53_freshfish", 1506, LURE_BAIT),
    ("0_38_49_freshfish", 1507, LURE_BAIT),
    ("0_39_53_freshfish", 1508, LURE_BAIT),
    ("0_40_52_freshfish", 1509, LURE_BAIT),
    ("0_41_73_freshfish", 1512, LURE_BAIT),
    ("0_42_55_freshfish", 1513, LURE_BAIT),
    ("0_44_46_freshfish", 1515, LURE_BAIT),
    ("0_44_52_freshfish", 1516, LURE_BAIT),
    ("0_48_53_freshfish", 1526, LURE_BAIT),
    ("0_52_149_freshfish", 1529, LURE_BAIT),
    ("0_21_44_rarefish", 14039, CAGE_HARPOON),
    ("0_23_53_rarefish", 7460, CAGE_HARPOON),
    ("0_24_46_rarefish", 12777, CAGE_HARPOON),
    ("0_26_54_rarefish", 7465, CAGE_HARPOON),
    ("0_27_59_rarefish", 7470, CAGE_HARPOON),
    ("0_28_56_rarefish", 7199, CAGE_HARPOON),
    ("0_33_52_rarefish", 9173, CAGE_HARPOON),
    ("0_34_53_rarefish", 9174, CAGE_HARPOON),
    ("0_35_46_rarefish", 10515, CAGE_HARPOON),
    ("0_37_59_lobster", 5820, CAGE_HARPOON),
    ("0_38_45_rarefish", 7946, CAGE_HARPOON),
    ("0_40_53_rarefish", 1510, CAGE_HARPOON),
    ("0_40_60_rarefish", 3657, CAGE_HARPOON),
    ("0_41_57_rarefish", 3914, CAGE_HARPOON),
    ("0_44_53_rarefish", 1519, CAGE_HARPOON),
    ("0_45_49_rarefish", 1522, CAGE_HARPOON),
    ("52_59_rarefish", 2146, CAGE_HARPOON),
    ("0_18_57_memberfish", 8525, BIG_NET_HARPOON),
    ("0_18_58_memberfish", 8527, BIG_NET_HARPOON),
    ("0_19_49_memberfish", 14523, BIG_NET_HARPOON),
    ("0_19_58_memberfish", 8526, BIG_NET_HARPOON),
    ("0_23_53_memberfish", 7461, BIG_NET_HARPOON),
    ("0_24_46_memberfish", 12775, BIG_NET_HARPOON),
    ("0_24_51_memberfish", 14037, BIG_NET_HARPOON),
    ("0_26_54_memberfish", 7466, BIG_NET_HARPOON),
    ("0_27_46_memberfish", 12776, BIG_NET_HARPOON),
    ("0_28_56_memberfish", 7200, BIG_NET_HARPOON),
    ("0_33_51_memberfish", 3419, BIG_NET_HARPOON),
    ("0_33_52_memberfish", 9171, BIG_NET_HARPOON),
    ("0_34_53_memberfish", 9172, BIG_NET_HARPOON),
    ("0_35_44_memberfish", 10514, BIG_NET_HARPOON),
    ("0_37_59_tuna", 5821, BIG_NET_HARPOON),
    ("0_40_53_memberfish", 1511, BIG_NET_HARPOON),
    ("0_41_57_memberfish", 3915, BIG_NET_HARPOON),
    ("0_42_42_memberfish", 5234, BIG_NET_HARPOON),
    ("0_43_42_memberfish", 5233, BIG_NET_HARPOON),
    ("0_44_53_memberfish", 1520, BIG_NET_HARPOON),
    ("0_54_49_memberfish", 4476, BIG_NET_HARPOON),
    ("0_55_49_memberfish", 4477, BIG_NET_HARPOON),
    ("0_53_51_slimey_eel", 2655, SLIMY_EEL),
    ("0_53_53_slimey_eel", 2654, SLIMY_EEL),
    ("0_54_53_slimey_eel", 2653, SLIMY_EEL),
    ("0_49_149_cavefish", 1497, CAVE_EEL),
    ("0_45_152_lavafish", 4928, LAVA_EEL),
    ("0_47_59_lavafish", 6784, LAVA_EEL),
    ("piscariliusfish", 6825, ANGLERFISH),
    ("49_61_crabs", 1536, DARK_CRAB),
    ("52_59_crabs", 1535, DARK_CRAB),
];

/// OpenRune FishingTable rows for the supported spot kinds:
/// (spot, method, obj symbol, id, cache name, level, xp tenths, low, high, row bait).
#[allow(clippy::type_complexity)]
const CATCHES: &[(u8, u8, &str, i32, &str, i32, i32, i32, i32, Option<&str>)] = &[
    (NET_BAIT, NET, "raw_shrimp", 317, "Raw shrimps", 1, 100, 48, 256, None),
    (NET_BAIT, NET, "raw_anchovies", 321, "Raw anchovies", 15, 400, 24, 128, None),
    (NET_BAIT, BAIT, "raw_sardine", 327, "Raw sardine", 5, 200, 32, 192, None),
    (NET_BAIT, BAIT, "raw_herring", 345, "Raw herring", 10, 300, 24, 128, None),
    (LURE_BAIT, LURE, "raw_trout", 335, "Raw trout", 20, 500, 32, 192, None),
    (LURE_BAIT, LURE, "raw_salmon", 331, "Raw salmon", 30, 700, 16, 96, None),
    (LURE_BAIT, LURE, "hunting_raw_fish_special", 10138, "Raw rainbow fish", 38, 800, 8, 64, Some("hunting_stripy_bird_feather")),
    (LURE_BAIT, BAIT, "raw_pike", 349, "Raw pike", 25, 600, 16, 96, None),
    (CAGE_HARPOON, CAGE, "raw_lobster", 377, "Raw lobster", 40, 900, 6, 95, None),
    (CAGE_HARPOON, HARPOON, "raw_tuna", 359, "Raw tuna", 35, 800, 8, 64, None),
    (CAGE_HARPOON, HARPOON, "raw_swordfish", 371, "Raw swordfish", 50, 1000, 4, 48, None),
    (BIG_NET_HARPOON, BIG_NET, "raw_mackerel", 353, "Raw mackerel", 16, 200, 5, 65, None),
    (BIG_NET_HARPOON, BIG_NET, "raw_cod", 341, "Raw cod", 23, 450, 4, 55, None),
    (BIG_NET_HARPOON, BIG_NET, "raw_bass", 363, "Raw bass", 46, 1000, 3, 40, None),
    (BIG_NET_HARPOON, HARPOON, "raw_shark", 383, "Raw shark", 76, 1100, 3, 40, None),
    (SLIMY_EEL, BAIT, "mort_slimey_eel", 3379, "Raw slimy eel", 28, 650, 10, 80, None),
    (CAVE_EEL, BAIT, "raw_cave_eel", 5001, "Raw cave eel", 38, 800, 10, 80, None),
    (LAVA_EEL, OILY_ROD, "raw_lava_eel", 2148, "Raw lava eel", 53, 300, 16, 96, None),
    (ANGLERFISH, SANDWORM, "raw_anglerfish", 13439, "Raw anglerfish", 82, 1200, 3, 36, None),
    (DARK_CRAB, DARK_CRAB_CAGE, "raw_dark_crab", 11934, "Raw dark crab", 85, 1300, 3, 40, None),
];

pub fn ordinary_fishing(
    npcs: &Symbols,
    objects: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    let attest = |symbols: &Symbols, name: &str, id: i32| -> Result<i32, String> {
        if symbols.get(name) != Some(&id) {
            return Err(format!("Missing/aliased fishing binding {name}"));
        }
        Ok(id)
    };
    let short = |symbols: &Symbols, name: &str, id: i32| -> Result<u16, String> {
        u16::try_from(attest(symbols, name, id)?).map_err(|_| "Invalid fishing binding".into())
    };
    // Item identity plus the cache's own name and membership flag.
    let item = |name: &str, id: i32, label: &str| -> Result<bool, String> {
        attest(objects, name, id)?;
        let t = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing fishing item {id}"))?;
        if t.name != label || t.cert.template != 0 || t.placeholder.template != 0 {
            return Err(format!("Unattested fishing item {id}"));
        }
        Ok(t.members)
    };
    let skill = short(stats, "fishing", 10)? as u8;
    let stat = defs
        .stats
        .get(&(skill as u32))
        .ok_or("Missing Fishing stat")?;
    if stat.name != "Fishing" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Fishing stat".into());
    }
    let tool = |name: &str, id: i32, label: &'static str, level: i32, bonus: i32, animation: Option<u16>| {
        Ok::<_, String>(Tool {
            id,
            level,
            members: item(name, id, label)?,
            bonus,
            animation,
            name: match label {
                "Dragon harpoon" => "dragon harpoon",
                _ => "tool",
            },
        })
    };
    let bait = |name: &str, id: i32, label: &str| -> Result<Option<i32>, String> {
        item(name, id, label)?;
        Ok(Some(id))
    };
    let net = tool("net", 303, "Small fishing net", 1, 0, None)?;
    let rod = tool("fishing_rod", 307, "Fishing rod", 1, 0, None)?;
    let pearl = tool("fishingrod_pearl", 22846, "Pearl fishing rod", 1, 0, None)?;
    let fly = tool("fly_fishing_rod", 309, "Fly fishing rod", 1, 0, None)?;
    let pearl_fly = tool("fishingrod_pearl_fly", 22844, "Pearl fly fishing rod", 1, 0, None)?;
    let pot = tool("lobster_pot", 301, "Lobster pot", 1, 0, None)?;
    let harpoons = vec![
        tool("harpoon", 311, "Harpoon", 1, 0, None)?,
        tool(
            "hunting_barbed_harpoon",
            10129,
            "Barb-tail harpoon",
            1,
            0,
            Some(short(sequences, "human_harpoon_barbed", 5108)?),
        )?,
        // OpenRune DRAGON_HARPOON_LEVEL / DRAGON_HARPOON_BONUS.
        tool(
            "dragon_harpoon",
            21028,
            "Dragon harpoon",
            61,
            20,
            Some(short(sequences, "human_harpoon_dragon", 7401)?),
        )?,
    ];
    let big_net = tool("big_net", 305, "Big fishing net", 1, 0, None)?;
    let oily = vec![
        tool("oily_fishing_rod", 1585, "Oily fishing rod", 1, 0, None)?,
        tool("fishingrod_pearl_oily", 23122, "Oily pearl fishing rod", 1, 0, None)?,
    ];
    let fishing_bait = bait("fishing_bait", 313, "Fishing bait")?;
    let smallnet = short(sequences, "human_smallnet", 621)?;
    let casting = short(sequences, "human_fishing_casting", 622)?;
    let caging = short(sequences, "human_lobster", 619)?;
    let harpooning = short(sequences, "human_harpoon", 618)?;
    let largenet = short(sequences, "human_largenet", 620)?;
    const LINE: &[&str] = &["You cast out your line...", "You attempt to catch a fish."];
    let method = |tools: Vec<Tool>, bait: Option<i32>, animation: u16, missing_tool: &'static str, cast: &'static [&'static str]| Method {
        tools,
        bait,
        animation,
        missing_tool,
        cast,
    };
    let methods: BTreeMap<u8, Method> = [
        (NET, method(vec![net], None, smallnet, "You need a small fishing net to fish here.", &["You cast out your net..."])),
        (BAIT, method(vec![rod, pearl], fishing_bait, casting, "You need a fishing rod to fish here.", LINE)),
        (LURE, method(vec![fly, pearl_fly], bait("feather", 314, "Feather")?, casting, "You need a fly fishing rod to lure these fish.", LINE)),
        (CAGE, method(vec![pot], None, caging, "You need a lobster pot to catch these fish.", &[])),
        (HARPOON, method(harpoons, None, harpooning, "You need a harpoon to catch these fish.", &[])),
        (BIG_NET, method(vec![big_net], None, largenet, "You need a big fishing net to catch these fish.", &[])),
        (OILY_ROD, method(oily, fishing_bait, casting, "You need an oily fishing rod to fish here.", LINE)),
        (SANDWORM, method(vec![rod, pearl], bait("piscarilius_sandworms", 13431, "Sandworms")?, casting, "You need a fishing rod to fish here.", LINE)),
        (DARK_CRAB_CAGE, method(vec![pot], bait("wilderness_fishing_bait", 11940, "Dark fishing bait")?, caging, "You need a lobster pot to catch these fish.", &[])),
    ]
    .into();
    let mut catches = Vec::new();
    for &(spot, method, symbol, id, label, level, xp_tenths, low, high, row_bait) in CATCHES {
        let members = item(symbol, id, label)?;
        let bait = match row_bait {
            Some(name) => bait(name, 10087, "Stripy feather")?,
            None => None,
        };
        // Wiki (27 November 2024 onward): the lower-case item name, "You catch some raw
        // shrimps." Lure catches (raw trout/salmon) have no closing period in game.
        let fish = label.to_lowercase();
        let article = if method == NET {
            "some"
        } else if fish.starts_with(['a', 'e', 'i', 'o', 'u']) {
            "an"
        } else {
            "a"
        };
        let period = if method == LURE { "" } else { "." };
        catches.push((
            spot,
            Catch {
                method,
                product: id,
                level,
                xp_tenths,
                rates: (low, high),
                members,
                bait,
                message: format!("You catch {article} {fish}{period}"),
            },
        ));
    }
    let mut spots = BTreeMap::new();
    for &(name, id, kind) in SPOTS {
        let id = short(npcs, name, i32::from(id))?;
        let d = defs
            .npcs
            .get(&u32::from(id))
            .ok_or_else(|| format!("Missing fishing spot {id}"))?;
        if !matches!(d.name.as_str(), "Fishing spot" | "Rod Fishing spot")
            || d.size != 1
            || !d.multi.is_empty()
            || d.multi_default.is_some()
        {
            return Err(format!("Unattested fishing spot {id}"));
        }
        for &(operation, method, texts) in options(kind) {
            let text = d.ops[usize::from(operation - 1)].as_deref();
            if !text.is_some_and(|t| texts.contains(&t)) {
                return Err(format!("Unattested fishing spot option {id}/{operation}"));
            }
            if spots
                .insert((id, operation), Binding { spot: kind, method })
                .is_some()
            {
                return Err(format!("Duplicate fishing spot {id}"));
            }
        }
    }
    let config = Config {
        skill,
        full_sound: short(sounds, "pillory_wrong", 2277)?,
        methods,
        catches,
        spots,
    };
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn named_item(name: &str, members: bool) -> gielinor_cache::definitions::ItemType {
        let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        t.name = name.into();
        t.members = members;
        t
    }
    fn fixture() -> (Symbols, Symbols, Symbols, Symbols, Symbols, Definitions) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        d.stats.insert(
            10,
            gielinor_cache::definitions::StatType::decode(b"\x01Fishing\0\0").unwrap(),
        );
        let mut objects = Symbols::new();
        for (symbol, id, name, members) in [
            ("net", 303, "Small fishing net", false),
            ("fishing_rod", 307, "Fishing rod", false),
            ("fishingrod_pearl", 22846, "Pearl fishing rod", false),
            ("fly_fishing_rod", 309, "Fly fishing rod", false),
            ("fishingrod_pearl_fly", 22844, "Pearl fly fishing rod", false),
            ("lobster_pot", 301, "Lobster pot", false),
            ("harpoon", 311, "Harpoon", false),
            ("hunting_barbed_harpoon", 10129, "Barb-tail harpoon", true),
            ("dragon_harpoon", 21028, "Dragon harpoon", true),
            ("big_net", 305, "Big fishing net", true),
            ("oily_fishing_rod", 1585, "Oily fishing rod", true),
            ("fishingrod_pearl_oily", 23122, "Oily pearl fishing rod", true),
            ("fishing_bait", 313, "Fishing bait", false),
            ("feather", 314, "Feather", false),
            ("hunting_stripy_bird_feather", 10087, "Stripy feather", true),
            ("piscarilius_sandworms", 13431, "Sandworms", true),
            ("wilderness_fishing_bait", 11940, "Dark fishing bait", true),
        ] {
            objects.insert(symbol.into(), id);
            d.items.insert(id as u32, named_item(name, members));
        }
        for &(_, _, symbol, id, name, ..) in CATCHES {
            objects.insert(symbol.into(), id);
            d.items.insert(id as u32, named_item(name, false));
        }
        let mut npcs = Symbols::new();
        for &(name, id, kind) in SPOTS {
            npcs.insert(name.into(), i32::from(id));
            let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            t.name = "Fishing spot".into();
            for &(op, _, texts) in options(kind) {
                t.ops[usize::from(op - 1)] = Some(texts[0].into());
            }
            d.npcs.insert(u32::from(id), t);
        }
        let seq = [
            ("human_smallnet", 621),
            ("human_fishing_casting", 622),
            ("human_lobster", 619),
            ("human_harpoon", 618),
            ("human_largenet", 620),
            ("human_harpoon_barbed", 5108),
            ("human_harpoon_dragon", 7401),
        ]
        .map(|(n, i)| (n.to_string(), i))
        .into();
        (
            npcs,
            objects,
            seq,
            [("pillory_wrong".into(), 2277)].into(),
            [("fishing".into(), 10)].into(),
            d,
        )
    }
    #[test]
    fn binds_every_supported_spot_option_and_catch() {
        let (n, o, s, a, st, d) = fixture();
        let c = ordinary_fishing(&n, &o, &s, &a, &st, &d).unwrap();
        let options: usize = SPOTS.iter().map(|&(_, _, k)| options(k).len()).sum();
        assert_eq!(c.spots.len(), options);
        assert_eq!(c.catches.len(), CATCHES.len());
        assert_eq!(c.spots[&(1530, 1)], Binding { spot: NET_BAIT, method: NET });
        assert_eq!(c.spots[&(1530, 3)], Binding { spot: NET_BAIT, method: BAIT });
        // Cave eel Bait is the third option in 239.1.
        assert!(c.spots.contains_key(&(1497, 3)) && !c.spots.contains_key(&(1497, 1)));
        let shrimp = c.catches.iter().find(|(_, r)| r.product == 317).unwrap();
        assert_eq!(shrimp.1.message, "You catch some raw shrimps.");
        let angler = c.catches.iter().find(|(_, r)| r.product == 13439).unwrap();
        assert_eq!(angler.1.message, "You catch a raw anglerfish.");
        let trout = c.catches.iter().find(|(_, r)| r.product == 335).unwrap();
        assert_eq!(trout.1.message, "You catch a raw trout");
        let rainbow = c.catches.iter().find(|(_, r)| r.product == 10138).unwrap();
        assert_eq!(rainbow.1.bait, Some(10087));
        let dragon = c.methods[&HARPOON].tools.iter().find(|t| t.id == 21028).unwrap();
        assert_eq!((dragon.level, dragon.bonus, dragon.members), (61, 20, true));
    }
    #[test]
    fn drifted_identities_and_options_refuse() {
        for mode in 0..6 {
            let (mut n, mut o, s, a, st, mut d) = fixture();
            match mode {
                0 => {
                    n.remove("0_50_49_saltfish");
                }
                1 => d.npcs.get_mut(&1530).unwrap().ops[2] = Some("Harpoon".into()),
                2 => d.npcs.get_mut(&1530).unwrap().name = "Man".into(),
                3 => {
                    o.insert("raw_shrimp".into(), 318);
                }
                4 => d.items.get_mut(&317).unwrap().name = "Shrimps".into(),
                _ => {
                    d.npcs.remove(&1497);
                }
            }
            assert!(ordinary_fishing(&n, &o, &s, &a, &st, &d).is_err(), "mode{mode}");
        }
    }
}
