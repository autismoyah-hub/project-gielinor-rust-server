//! Ordinary variants only; require complete equality of live source parameters except stump.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Tree;
const ROWS: [(&str, u32, u32, &str, u32, &str); 5] = [
    (
        "willow_tree2",
        10829,
        10819,
        "willow_tree2or3or4_stump",
        9471,
        "Tree Stump",
    ),
    (
        "willow_tree3",
        10831,
        10819,
        "willow_tree2or3or4_stump",
        9471,
        "Tree Stump",
    ),
    (
        "willow_tree4",
        10833,
        10819,
        "willow_tree2or3or4_stump",
        9471,
        "Tree Stump",
    ),
    (
        "mapletree_update",
        40754,
        10832,
        "mapletree_update_stump",
        40755,
        "Maple tree",
    ),
    (
        "yewtree_update",
        40756,
        10822,
        "yewtree_update_stump",
        40757,
        "Tree stump",
    ),
];
pub fn timed_tree_variants(
    locations: &Symbols,
    defs: &Definitions,
    primary: &std::collections::BTreeMap<u32, Tree>,
) -> Result<std::collections::BTreeMap<u32, Tree>, String> {
    let mut result = std::collections::BTreeMap::new();
    for (symbol, id, base_id, stump_symbol, stump_id, stump_name) in ROWS {
        if locations.get(symbol) != Some(&(id as i32))
            || locations.get(stump_symbol) != Some(&(stump_id as i32))
        {
            return Err(format!("Missing/aliased ordinary tree variant {symbol}"));
        }
        let base = defs
            .locations
            .get(&base_id)
            .ok_or("Missing primary tree definition")?;
        let prototype = primary
            .get(&base_id)
            .ok_or("Missing primary tree behavior")?;
        let live = defs.locations.get(&id).ok_or("Missing tree variant")?;
        let stump = defs
            .locations
            .get(&stump_id)
            .ok_or("Missing tree variant stump")?;
        for d in [live, stump] {
            if d.collision.width != base.collision.width
                || d.collision.length != base.collision.length
                || !d.multi.is_empty()
                || d.multi_default != -1
                || d.multi_varbit != -1
                || d.multi_varp != -1
            {
                return Err(format!("Invalid static tree variant footprint {id}"));
            }
        }
        if live.name != base.name
            || live.content_group != 32
            || live.ops[0].as_deref() != Some("Chop down")
            || stump.name != stump_name
            || !stump.ops.iter().all(|op| {
                op.as_deref()
                    .is_none_or(|s| s.eq_ignore_ascii_case("hidden"))
            })
        {
            return Err(format!("Invalid ordinary tree variant/stump {id}"));
        }
        if live.params.get(&65533) != Some(&Param::Integer(stump_id as i32))
            || base.params.get(&65533) != Some(&Param::Integer(prototype.stump as i32))
        {
            return Err(format!("Invalid tree variant stump parameter {id}"));
        }
        let mut expected = base.params.clone();
        expected.remove(&65533);
        let mut actual = live.params.clone();
        actual.remove(&65533);
        if actual != expected {
            return Err(format!("Different ordinary tree variant parameters {id}"));
        }
        let mut profile = prototype.clone();
        profile.stump = stump_id;
        if result.insert(id, profile).is_some() {
            return Err("Duplicate ordinary tree variant".into());
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timed_tree_variants_clone_only_matching_full_parameters_and_correct_stump_names() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut symbols = Symbols::new();
        let mut primary = std::collections::BTreeMap::new();
        for (id, name, width, stump, level, logs, xp, active, respawn, cache_respawn, curve) in [
            (
                10819,
                "Willow tree",
                2,
                9711,
                30,
                1519,
                675,
                50,
                14,
                15,
                65530,
            ),
            (
                10832,
                "Maple tree",
                2,
                9712,
                45,
                1517,
                1000,
                100,
                59,
                60,
                65528,
            ),
            (
                10822, "Yew tree", 3, 9714, 60, 1515, 1750, 190, 99, 100, 65525,
            ),
        ] {
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(&[0, 14, width, 15, width, 0]);
            let mut d = gielinor_cache::locations::LocationType::decode(&b).unwrap();
            d.ops[0] = Some("Chop down".into());
            d.content_group = 32;
            for (k, v) in [
                (23, level),
                (65492, logs),
                (65493, xp),
                (65489, active),
                (65488, cache_respawn),
                (65533, stump),
                (65485, curve),
            ] {
                d.params.insert(k, Param::Integer(v));
            }
            defs.locations.insert(id, d);
            primary.insert(
                id,
                Tree {
                    level,
                    product: logs,
                    product_name: "Logs".into(),
                    xp: f64::from(xp) / 10.0,
                    stump: stump as u32,
                    deplete_chance: 0,
                    chopping_ticks: Some(active),
                    respawn: [respawn, respawn],
                    rates: [(1349, (24, 75))].into(),
                },
            );
        }
        for (symbol, id, base_id, stump_symbol, stump_id, stump_name) in ROWS {
            symbols.insert(symbol.into(), id as i32);
            symbols.insert(stump_symbol.into(), stump_id as i32);
            let mut live = defs.locations[&base_id].clone();
            live.params.insert(65533, Param::Integer(stump_id as i32));
            let mut stump = live.clone();
            stump.name = stump_name.into();
            stump.ops = [None, None, None, None, None];
            stump.params.clear();
            defs.locations.insert(id, live);
            defs.locations.insert(stump_id, stump);
        }
        let profiles = timed_tree_variants(&symbols, &defs, &primary).unwrap();
        assert_eq!(profiles.len(), 5);
        assert_eq!(profiles[&10831].stump, 9471);
        assert_eq!(profiles[&40754].stump, 40755);
        assert_eq!(profiles[&40756].respawn, [99, 99]);
        assert_eq!(profiles[&10829].rates, primary[&10819].rates);
        defs.locations
            .get_mut(&10831)
            .unwrap()
            .params
            .insert(12345, Param::Integer(1));
        assert!(timed_tree_variants(&symbols, &defs, &primary).is_err());
        defs.locations
            .get_mut(&10831)
            .unwrap()
            .params
            .remove(&12345);
        defs.locations.get_mut(&40755).unwrap().ops[0] = Some("Chop down".into());
        assert!(timed_tree_variants(&symbols, &defs, &primary).is_err());
    }
}
