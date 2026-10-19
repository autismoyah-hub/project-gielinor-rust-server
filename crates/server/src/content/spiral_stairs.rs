//! Direct spiral operations only; the middle-flight first-option menu is a separate unit.
use super::{Definitions, Symbols};
use gielinor_world::location_content::{Handler, Spiral};
use std::collections::{BTreeMap, BTreeSet};
const NODES: [(&str, u32, i32, u8, [Option<&str>; 5]); 5] = [
    (
        "spiralstairs",
        16671,
        11,
        2,
        [Some("Climb-up"), None, None, None, None],
    ),
    (
        "spiralstairsmiddle",
        16672,
        12,
        2,
        [
            Some("Climb"),
            Some("Climb-up"),
            Some("Climb-down"),
            None,
            None,
        ],
    ),
    (
        "spiralstairstop",
        16673,
        10,
        1,
        [Some("Climb-down"), None, None, None, None],
    ),
    (
        "spiralstairsbottom_3",
        56230,
        11,
        2,
        [Some("Climb-up"), Some("Top-floor"), None, None, None],
    ),
    (
        "spiralstairstop_3",
        56231,
        10,
        1,
        [Some("Climb-down"), Some("Bottom-floor"), None, None, None],
    ),
];
pub fn spiral_stairs(
    locations: &Symbols,
    contents: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    for (name, id) in [
        ("spiralstaircase_down", 10),
        ("spiralstaircase_up", 11),
        ("spiralstaircase_option", 12),
    ] {
        if contents.get(name) != Some(&id) {
            return Err(format!("Unattested spiral content {name}"));
        }
    }
    let actual: BTreeSet<_> = defs
        .locations
        .iter()
        .filter(|(_, d)| [10, 11, 12].contains(&d.content_group))
        .map(|(&id, _)| id)
        .collect();
    if actual != NODES.iter().map(|v| v.1).collect() {
        return Err("Unattested native spiral staircase set".into());
    }
    for (symbol, id, group, size, ops) in NODES {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Staircase"
                    || d.content_group != group
                    || d.collision.width != size
                    || d.collision.length != size
                    || d.ops.iter().map(|v| v.as_deref()).collect::<Vec<_>>() != ops
                    || !d.multi.is_empty()
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
                    || d.multi_default != -1
            })
        {
            return Err(format!("Unattested native spiral staircase {symbol}"));
        }
    }
    Ok([
        (16671, 1, 1, false),
        (16673, 1, -1, false),
        (16672, 2, 1, true),
        (16672, 3, -1, true),
        (56230, 1, 1, false),
        (56230, 2, 2, false),
        (56231, 1, -1, false),
        (56231, 2, -2, false),
    ]
    .into_iter()
    .map(|(id, op, delta_level, arrival_delay)| {
        (
            (id, op),
            Handler::Spiral(Spiral {
                delta_level,
                arrival_delay,
            }),
        )
    })
    .collect())
}
/// The source middle-flight menu uses the same strict native attestation.
pub fn spiral_choices(
    locations: &Symbols,
    contents: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    spiral_stairs(locations, contents, defs)?;
    Ok([((16672, 1), Handler::SpiralChoice)].into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spiral_stair_direct_bindings_keep_native_footprints_and_lumbridge_two_floor_overrides() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut l = Symbols::new();
        for (symbol, id, group, size, ops) in NODES {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Staircase".into();
            t.content_group = group;
            t.collision.width = size;
            t.collision.length = size;
            t.ops = ops.map(|v| v.map(str::to_owned));
            d.locations.insert(id, t);
        }
        let c = [
            ("spiralstaircase_down".into(), 10),
            ("spiralstaircase_up".into(), 11),
            ("spiralstaircase_option".into(), 12),
        ]
        .into();
        let h = spiral_stairs(&l, &c, &d).unwrap();
        assert_eq!(h.len(), 8);
        assert!(!h.contains_key(&(16672, 1)));
        for (id, delta) in [(56230, 2), (56231, -2)] {
            let Handler::Spiral(r) = h[&(id, 2)] else {
                panic!("two floors")
            };
            assert_eq!(r.delta_level, delta);
            assert!(!r.arrival_delay);
        }
        assert!(h[&(16672, 2)].arrival_delay());
        d.locations.get_mut(&56231).unwrap().collision.width = 2;
        assert!(spiral_stairs(&l, &c, &d).is_err());
    }
}
