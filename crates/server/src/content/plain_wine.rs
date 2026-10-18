//! Actual239.1 plain-wine ingredients and effects; internal timer is not a cache ID.
use super::{Definitions, Symbols};
use gielinor_world::wine::{Config, TIMER};
const ITEMS: [(&str, i32, &str, bool); 6] = [
    ("grapes", 1987, "Grapes", false),
    ("jug_water", 1937, "Jug of water", false),
    ("jug_unfermented_wine", 1995, "Unfermented wine", false),
    ("jug_wine", 1993, "Jug of wine", false),
    ("jug_bad_wine", 1991, "Jug of bad wine", false),
    (
        "jug_unfermented_zamorak_wine",
        20752,
        "Zamorak's unfermented wine",
        true,
    ),
];
const ZAMORAK_ITEMS: [(&str, i32, &str, bool); 6] = [
    ("zamorak_grapes", 20749, "Zamorak's grapes", true),
    ("jug_water", 1937, "Jug of water", false),
    (
        "jug_unfermented_zamorak_wine",
        20752,
        "Zamorak's unfermented wine",
        true,
    ),
    ("wine_of_zamorak", 245, "Wine of zamorak", false),
    ("jug_bad_wine", 1991, "Jug of bad wine", false),
    ("jug_unfermented_wine", 1995, "Unfermented wine", false),
];
fn bind(
    config: Config,
    items: &[(&str, i32, &str, bool)],
    objects: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    timers: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if timers.values().any(|&id| {
        [
            i32::from(TIMER),
            i32::from(gielinor_world::wine::ZAMORAK_TIMER),
        ]
        .contains(&id)
    }) {
        return Err("Reserved internal wine timer collides with target timer".into());
    }
    if sequences.get("human_make_wine") != Some(&7529)
        || spots.get("cooking_make_wine_spotanim") != Some(&47)
    {
        return Err("Invalid wine effects".into());
    }
    if defs.stats.get(&7).is_none_or(|s| {
        s.name != "Cooking" || s.unreleased || s.min_level != 1 || s.max_level != 99
    }) {
        return Err("Invalid wine Cooking skill".into());
    }
    for &(symbol, id, name, members) in items {
        let d = defs.items.get(&(id as u32)).ok_or("Missing wine item")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid wine item {id}"));
        }
    }
    let raw = defs
        .items
        .get(&(config.unfermented as u32))
        .ok_or("Missing unfermented wine")?;
    let anchor = defs
        .items
        .get(&(config.placeholder as u32))
        .ok_or("Missing wine placeholder")?;
    if raw.placeholder.template != 0
        || raw.placeholder.link != config.placeholder
        || anchor.placeholder.template != 14401
        || anchor.placeholder.link != config.unfermented
    {
        return Err("Invalid reciprocal wine placeholder".into());
    }
    Ok(config)
}
pub fn plain_wine(
    objects: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    timers: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    bind(
        Config {
            grapes: 1987,
            water: 1937,
            unfermented: 1995,
            other_unfermented: 20752,
            wine: 1993,
            bad: 1991,
            placeholder: 19146,
            animation: 7529,
            spot: 47,
            minimum_level: 35,
            no_fail: 68,
            low: 48,
            high: 352,
            members: false,
            timer: TIMER,
            rolls: gielinor_world::wine::Rolls::FifteenBatches,
        },
        &ITEMS,
        objects,
        sequences,
        spots,
        timers,
        defs,
    )
}
pub fn zamorak_wine(
    objects: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    timers: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    bind(
        Config {
            grapes: 20749,
            water: 1937,
            unfermented: 20752,
            other_unfermented: 1995,
            wine: 245,
            bad: 1991,
            placeholder: 20753,
            animation: 7529,
            spot: 47,
            minimum_level: 65,
            no_fail: 98,
            low: -125,
            high: 260,
            members: true,
            timer: gielinor_world::wine::ZAMORAK_TIMER,
            rolls: gielinor_world::wine::Rolls::PerJug,
        },
        &ZAMORAK_ITEMS,
        objects,
        sequences,
        spots,
        timers,
        defs,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plain_wine_bindings_require_all_six_types_reciprocal_placeholder_and_private_timer_separation(
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
        for &(sym, id, name, members) in &ITEMS {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = members;
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        d.items.get_mut(&1995).unwrap().placeholder.link = 19146;
        let mut anchor = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
        anchor.placeholder.link = 1995;
        anchor.placeholder.template = 14401;
        d.items.insert(19146, anchor);
        let mut skill = gielinor_cache::definitions::StatType::decode(b"\0").unwrap();
        skill.name = "Cooking".into();
        d.stats.insert(7, skill);
        let sequences = [("human_make_wine".into(), 7529)].into();
        let spots = [("cooking_make_wine_spotanim".into(), 47)].into();
        let mut timers = Symbols::new();
        let c = plain_wine(&objects, &sequences, &spots, &timers, &d).unwrap();
        assert_eq!((c.recipe().ticks, c.recipe().xp_tenths), (3, 0));
        timers.insert("collision".into(), i32::from(TIMER));
        assert!(plain_wine(&objects, &sequences, &spots, &timers, &d).is_err());
        timers.clear();
        d.items.get_mut(&19146).unwrap().placeholder.link = 1993;
        assert!(plain_wine(&objects, &sequences, &spots, &timers, &d).is_err());
    }

    #[test]
    fn zamorak_wine_binds_member_mix_and_negative_curve_independently_of_f2p_product() {
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
        for &(sym, id, name, members) in ITEMS.iter().chain(ZAMORAK_ITEMS.iter()) {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = members;
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        for (raw, anchor) in [(1995, 19146), (20752, 20753)] {
            d.items.get_mut(&raw).unwrap().placeholder.link = anchor as i32;
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.placeholder.template = 14401;
            t.placeholder.link = raw as i32;
            d.items.insert(anchor, t);
        }
        let mut skill = gielinor_cache::definitions::StatType::decode(b"\0").unwrap();
        skill.name = "Cooking".into();
        d.stats.insert(7, skill);
        let seq = [("human_make_wine".into(), 7529)].into();
        let spots = [("cooking_make_wine_spotanim".into(), 47)].into();
        let timers = Symbols::new();
        let c = zamorak_wine(&objects, &seq, &spots, &timers, &d).unwrap();
        assert_eq!(
            (c.minimum_level, c.no_fail, c.low, c.high),
            (65, 98, -125, 260)
        );
        assert!(c.recipe().recipe.members);
        assert!(!d.items[&245].members);
        assert_eq!(c.rolls, gielinor_world::wine::Rolls::PerJug);
        assert_eq!(c.timer, 65534);
        assert!(plain_wine(&objects, &seq, &spots, &timers, &d).is_ok());
        objects.insert("wine_of_zamorak".into(), 1993);
        assert!(zamorak_wine(&objects, &seq, &spots, &timers, &d).is_err());
    }
}
