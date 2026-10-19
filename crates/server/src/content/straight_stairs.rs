//! Six explicit source flights with native geometry; special and spiral stairs are separate.
use super::{Definitions, Symbols};
use gielinor_world::location_content::{Handler, Stairs};
use std::collections::BTreeMap;
const ROWS: [(&str, u32, u8, u8, i32, i32, i32); 6] = [
    ("poor_stairs", 18991, 2, 3, 1, 3, 1),
    ("poor_stairstop", 18992, 2, 2, 1, -2, -1),
    ("stairs", 15645, 2, 3, 1, 3, 1),
    ("stairstop", 15648, 2, 2, 1, -2, -1),
    ("narrowstairs_wooden_bottom", 16670, 1, 3, 0, -1, 1),
    ("narrowstairs_wooden_top", 16669, 1, 2, 0, 3, -1),
];
pub fn straight_stairs(
    locations: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    for (symbol, id, width, length, _, _, dl) in ROWS {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Staircase"
                    || d.ops[0].as_deref() != Some(if dl > 0 { "Climb-up" } else { "Climb-down" })
                    || d.ops[1..].iter().any(Option::is_some)
                    || d.collision.width != width
                    || d.collision.length != length
                    || !d.multi.is_empty()
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
                    || d.multi_default != -1
            })
        {
            return Err(format!("Unattested straight staircase {symbol}"));
        }
    }
    Ok(ROWS
        .into_iter()
        .map(|(_, id, _, _, dx, dz, delta_level)| {
            (
                (id, 1),
                Handler::Stairs(Stairs {
                    dx,
                    dz,
                    delta_level,
                }),
            )
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn straight_stair_bindings_keep_distinct_top_bottom_geometry_and_narrow_reverse_offsets() {
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
        for (symbol, id, width, length, _, _, dl) in ROWS {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Staircase".into();
            t.ops[0] = Some(if dl > 0 { "Climb-up" } else { "Climb-down" }.into());
            t.collision.width = width;
            t.collision.length = length;
            d.locations.insert(id, t);
        }
        let h = straight_stairs(&l, &d).unwrap();
        assert_eq!(h.len(), 6);
        for (_, id, _, _, dx, dz, dl) in ROWS {
            let Handler::Stairs(r) = h[&(id, 1)] else {
                panic!("stairs")
            };
            assert_eq!((r.dx, r.dz, r.delta_level), (dx, dz, dl));
        }
        d.locations.get_mut(&16669).unwrap().collision.length = 3;
        assert!(straight_stairs(&l, &d).is_err());
    }
}
