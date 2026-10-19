//! Exact native single-direction links; choice ladders and quest routing are separate.
use super::{Definitions, Symbols};
use gielinor_cache::{definitions::Param, parameters::ParameterType};
use gielinor_world::location_content::{Climb, Handler};
use std::collections::{BTreeMap, BTreeSet};
const NODES: [(&str, u32, &str, i32, &str, i32, i32, u16); 4] = [
    (
        "qip_cook_trapdoor_open",
        14880,
        "Trapdoor",
        22,
        "Climb-down",
        0,
        6400,
        827,
    ),
    ("laddertop", 16679, "Ladder", 7, "Climb-down", -1, 0, 827),
    ("ladder", 16683, "Ladder", 8, "Climb-up", 1, 0, 828),
    (
        "ladder_from_cellar",
        17385,
        "Ladder",
        23,
        "Climb-up",
        0,
        -6400,
        828,
    ),
];
pub fn ladder_climbs(
    locations: &Symbols,
    contents: &Symbols,
    param_names: &Symbols,
    parameters: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    for (name, id) in [
        ("ladder_down", 7),
        ("ladder_up", 8),
        ("dungeonladder_down", 22),
        ("dungeonladder_up", 23),
    ] {
        if contents.get(name) != Some(&id) {
            return Err(format!("Unattested ladder content {name}"));
        }
    }
    if param_names.get("climb_anim") != Some(&65531)
        || parameters
            .get(&65531)
            .is_none_or(|p| p.literal != Some(6) || p.default_int != Some(828) || p.transmit)
    {
        return Err("Unattested native climb animation parameter".into());
    }
    let actual: BTreeSet<_> = defs
        .locations
        .iter()
        .filter(|(_, d)| [7, 8, 22, 23].contains(&d.content_group))
        .map(|(&id, _)| id)
        .collect();
    if actual != NODES.iter().map(|v| v.1).collect() {
        return Err("Unattested native single-direction ladder set".into());
    }
    let mut h = BTreeMap::new();
    for (symbol, id, name, group, op, delta_level, delta_z, animation) in NODES {
        let d = defs.locations.get(&id).ok_or("Missing native ladder")?;
        let seq = match d.params.get(&65531) {
            None => 828,
            Some(Param::Integer(v)) => *v,
            _ => return Err("Invalid ladder animation parameter type".into()),
        };
        if locations.get(symbol) != Some(&(id as i32))
            || d.name != name
            || d.content_group != group
            || d.ops[0].as_deref() != Some(op)
            || d.ops[1..].iter().any(Option::is_some)
            || d.collision.width != 1
            || d.collision.length != 1
            || !d.multi.is_empty()
            || d.multi_default != -1
            || d.multi_varbit != -1
            || d.multi_varp != -1
            || d.params.keys().any(|&k| k != 65531)
            || seq != animation as i32
        {
            return Err(format!("Unattested native ladder {symbol}"));
        }
        h.insert(
            (id, 1),
            Handler::Climb(Climb {
                delta_z,
                delta_level,
                animation,
                delay: 1,
            }),
        );
    }
    Ok(h)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ladder_native_four_mappings_default_animation_and_strict_parameter_refusal() {
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
        for (symbol, id, name, group, op, _, _, animation) in NODES {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = name.into();
            t.content_group = group;
            t.ops[0] = Some(op.into());
            if animation == 827 {
                t.params.insert(65531, Param::Integer(827));
            }
            d.locations.insert(id, t);
        }
        let c = [
            ("ladder_down".into(), 7),
            ("ladder_up".into(), 8),
            ("dungeonladder_down".into(), 22),
            ("dungeonladder_up".into(), 23),
        ]
        .into();
        let p = [("climb_anim".into(), 65531)].into();
        let params = [(
            65531,
            ParameterType::decode(&[1, 65, 2, 0, 0, 3, 60, 200, 0]).unwrap(),
        )]
        .into();
        let h = ladder_climbs(&l, &c, &p, &params, &d).unwrap();
        assert_eq!(h.len(), 4);
        for (_, id, _, _, _, dl, dz, anim) in NODES {
            let Handler::Climb(r) = h[&(id, 1)] else {
                panic!("climb")
            };
            assert_eq!(
                (r.delta_level, r.delta_z, r.animation, r.delay),
                (dl, dz, anim, 1)
            );
        }
        d.locations
            .get_mut(&17385)
            .unwrap()
            .params
            .insert(12345, Param::Integer(2));
        assert!(ladder_climbs(&l, &c, &p, &params, &d).is_err());
    }
}
