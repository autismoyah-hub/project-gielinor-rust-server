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
pub fn plain_wine(
    objects: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    timers: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    if timers.values().any(|&id| id == i32::from(TIMER)) {
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
    for &(symbol, id, name, members) in &ITEMS {
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
            return Err(format!("Invalid plain wine item {id}"));
        }
    }
    let raw = defs.items.get(&1995).unwrap();
    let anchor = defs
        .items
        .get(&19146)
        .ok_or("Missing wine placeholder anchor")?;
    if raw.placeholder.template != 0
        || raw.placeholder.link != 19146
        || anchor.placeholder.template != 14401
        || anchor.placeholder.link != 1995
    {
        return Err("Invalid wine reciprocal bank placeholder".into());
    }
    Ok(Config {
        grapes: 1987,
        water: 1937,
        unfermented: 1995,
        zamorak_unfermented: 20752,
        wine: 1993,
        bad: 1991,
        placeholder: 19146,
        animation: 7529,
        spot: 47,
    })
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
}
