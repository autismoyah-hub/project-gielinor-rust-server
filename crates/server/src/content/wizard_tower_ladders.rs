//! Wizard's Tower fixed stairwell endpoints through the shared protected climb.
use super::{Definitions, Symbols};
use gielinor_world::{
    location_content::{FixedClimb, Handler},
    CoordGrid,
};
use std::collections::BTreeMap;
const ROWS: [(&str, u32, &str, i32, i32, u16); 2] = [
    (
        "wizards_tower_laddertop",
        2147,
        "Climb-down",
        3104,
        9576,
        827,
    ),
    ("wizards_tower_ladder", 2148, "Climb-up", 3105, 3162, 828),
];
pub fn wizard_tower_ladders(
    locations: &Symbols,
    sequences: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    for (name, id) in [("human_pickupfloor", 827), ("human_reachforladder", 828)] {
        if sequences.get(name) != Some(&id) {
            return Err(format!("Unattested Tower ladder sequence {name}"));
        }
    }
    for (symbol, id, op, _, _, _) in ROWS {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Ladder"
                    || d.collision.width != 1
                    || d.collision.length != 1
                    || d.ops[0].as_deref() != Some(op)
                    || d.ops[1..].iter().any(Option::is_some)
                    || !d.multi.is_empty()
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
                    || d.multi_default != -1
            })
        {
            return Err(format!("Unattested Tower ladder {symbol}"));
        }
    }
    Ok(ROWS
        .into_iter()
        .map(|(_, id, _, x, z, animation)| {
            (
                (id, 1),
                Handler::FixedClimb(FixedClimb {
                    destination: CoordGrid::new(x, z, 0).expect("checked source endpoint"),
                    animation,
                    delay: 1,
                }),
            )
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wizard_tower_fixed_climb_native_bindings_preserve_distinct_x_z_endpoints_and_source_sequences(
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
        let mut l = Symbols::new();
        for (symbol, id, op, _, _, _) in ROWS {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Ladder".into();
            t.ops[0] = Some(op.into());
            d.locations.insert(id, t);
        }
        let s = [
            ("human_pickupfloor".into(), 827),
            ("human_reachforladder".into(), 828),
        ]
        .into();
        let h = wizard_tower_ladders(&l, &s, &d).unwrap();
        assert_eq!(h.len(), 2);
        for (_, id, _, x, z, animation) in ROWS {
            let Handler::FixedClimb(r) = h[&(id, 1)] else {
                panic!("fixed climb")
            };
            assert_eq!(r.destination, CoordGrid::new(x, z, 0).unwrap());
            assert_eq!((r.animation, r.delay), (animation, 1));
            assert!(h[&(id, 1)].arrival_delay());
        }
        d.locations.get_mut(&2148).unwrap().ops[1] = Some("Unexpected".into());
        assert!(wizard_tower_ladders(&l, &s, &d).is_err());
    }
}
