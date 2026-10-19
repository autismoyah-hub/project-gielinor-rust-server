//! Native stile and broken-fence crossings; no Agility reward or invented gate.
use super::{Definitions, Symbols};
use gielinor_world::location_content::{Crossing, Handler};
use std::collections::BTreeMap;
const ROWS: [(&str, u32, &str, u8, u8, Crossing); 3] = [
    ("fullstyle", 993, "Stile", 1, 2, Crossing::Stile),
    (
        "qip_sheep_shearer_fullstyle",
        12982,
        "Stile",
        1,
        2,
        Crossing::Stile,
    ),
    ("gertrudefence", 2618, "Broken fence", 1, 1, Crossing::Fence),
];
pub fn fence_crossings(
    locations: &Symbols,
    sequences: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if sequences.get("human_walk_style") != Some(&839) {
        return Err("Unattested crossing sequence".into());
    }
    for (symbol, id, name, width, length, _) in ROWS {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != name
                    || d.collision.width != width
                    || d.collision.length != length
                    || d.ops[0].as_deref() != Some("Climb-over")
                    || d.ops[1..].iter().any(Option::is_some)
                    || !d.multi.is_empty()
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
                    || d.multi_default != -1
            })
        {
            return Err(format!("Unattested native crossing {symbol}"));
        }
    }
    Ok(ROWS
        .into_iter()
        .map(|(_, id, _, _, _, rule)| ((id, 1), Handler::Crossing(rule)))
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fence_crossings_exact_three_native_nodes_keep_stile_and_fence_timing_distinct() {
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
        for (symbol, id, name, width, length, _) in ROWS {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = name.into();
            t.collision.width = width;
            t.collision.length = length;
            t.ops[0] = Some("Climb-over".into());
            d.locations.insert(id, t);
        }
        let s = [("human_walk_style".into(), 839)].into();
        let h = fence_crossings(&l, &s, &d).unwrap();
        assert_eq!(h.len(), 3);
        let Handler::Crossing(stile) = h[&(993, 1)] else {
            panic!("stile")
        };
        let Handler::Crossing(fence) = h[&(2618, 1)] else {
            panic!("fence")
        };
        assert_eq!(stile.protection_ticks(), 0);
        assert_eq!(fence.protection_ticks(), 3);
        assert!(!h[&(993, 1)].arrival_delay());
        assert!(h[&(2618, 1)].arrival_delay());
        d.locations.get_mut(&12982).unwrap().collision.length = 1;
        assert!(fence_crossings(&l, &s, &d).is_err());
    }
}
