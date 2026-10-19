//! The mill's direct climbs and middle menu retain their distinct source delays.
use super::{Definitions, Symbols};
use gielinor_world::location_content::{Climb, Handler};
use std::collections::BTreeMap;
const ROWS: [(&str, u32, [Option<&str>; 5]); 3] = [
    (
        "qip_cook_ladder",
        12964,
        [Some("Climb-up"), None, None, None, None],
    ),
    (
        "qip_cook_ladder_middle",
        12965,
        [
            Some("Climb"),
            Some("Climb-up"),
            Some("Climb-down"),
            None,
            None,
        ],
    ),
    (
        "qip_cook_ladder_top",
        12966,
        [Some("Climb-down"), None, None, None, None],
    ),
];
pub fn windmill_ladders(
    locations: &Symbols,
    sequences: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if sequences.get("human_reachforladder") != Some(&828) {
        return Err("Unattested mill ladder sequence".into());
    }
    for (symbol, id, ops) in ROWS {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Ladder"
                    || d.collision.width != 1
                    || d.collision.length != 1
                    || d.ops.iter().map(|v| v.as_deref()).collect::<Vec<_>>() != ops
                    || !d.multi.is_empty()
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
                    || d.multi_default != -1
            })
        {
            return Err(format!("Unattested mill ladder {symbol}"));
        }
    }
    let mut out: BTreeMap<_, _> = [(12964, 1, 1), (12965, 2, 1), (12965, 3, -1), (12966, 1, -1)]
        .into_iter()
        .map(|(id, op, delta_level)| {
            (
                (id, op),
                Handler::Climb(Climb {
                    delta_z: 0,
                    delta_level,
                    animation: 828,
                    delay: 1,
                }),
            )
        })
        .collect();
    out.insert((12965, 1), Handler::WindmillChoice);
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windmill_ladder_choices_bind_exact_five_operations_and_keep_all_direct_animations828() {
        let mut l = Symbols::new();
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (symbol, id, ops) in ROWS {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Ladder".into();
            t.ops = ops.map(|v| v.map(str::to_owned));
            d.locations.insert(id, t);
        }
        let s = [("human_reachforladder".into(), 828)].into();
        let h = windmill_ladders(&l, &s, &d).unwrap();
        assert_eq!(h.len(), 5);
        assert!(matches!(h[&(12965, 1)], Handler::WindmillChoice));
        assert!(h[&(12965, 1)].arrival_delay());
        for (key, r) in &h {
            if let Handler::Climb(r) = r {
                assert_eq!((r.animation, r.delay, r.delta_z), (828, 1, 0));
                assert_eq!(
                    r.delta_level,
                    if key.0 == 12966 || key.1 == 3 { -1 } else { 1 }
                );
            }
        }
        d.locations.get_mut(&12965).unwrap().ops[2] = None;
        assert!(windmill_ladders(&l, &s, &d).is_err());
    }
}
