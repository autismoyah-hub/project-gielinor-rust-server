//! The native source drawer pair; quest, thieving and loot variants are separate.
use super::{Definitions, Symbols};
use gielinor_world::location_content::{Handler, Swap};
use std::collections::BTreeMap;
pub fn drawers(
    locations: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    // Native239's sparse symbols omit this alias. Donor gamevals.toml pins64;
    // the raw native asset is attested without changing cache or symbol files.
    if sequences.get("human_openchest") != Some(&536)
        || sounds.get("drawer_open").is_some_and(|&id| id != 64)
    {
        return Err("Unattested drawer presentation".into());
    }
    for (symbol, id, ops) in [
        ("drawers2", 350, [Some("Open"), None, None, None, None]),
        (
            "drawers2open",
            351,
            [None, Some("Search"), Some("Shut"), None, None],
        ),
    ] {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Drawers"
                    || d.ops.iter().map(|v| v.as_deref()).collect::<Vec<_>>() != ops
                    || d.collision.width != 1
                    || d.collision.length != 1
                    || !d.multi.is_empty()
                    || d.multi_default != -1
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested drawer type {symbol}"));
        }
    }
    Ok([
        (
            (350, 1),
            Handler::DelayedSwap {
                rule: Swap {
                    next: 351,
                    animation: Some(536),
                    sound: Some(64),
                    duration: 500,
                    arrival_delay: true,
                },
                delay: 1,
            },
        ),
        (
            (351, 2),
            Handler::ArrivedMessage("You search the drawers but find nothing."),
        ),
        (
            (351, 3),
            Handler::DelayedSwap {
                rule: Swap {
                    next: 350,
                    animation: Some(536),
                    sound: None,
                    duration: 500,
                    arrival_delay: true,
                },
                delay: 1,
            },
        ),
    ]
    .into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn drawers_bind_native_source_pair_one_tick_warmup_and_attested_missing_alias_without_other_variants(
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
        let l = [("drawers2".into(), 350), ("drawers2open".into(), 351)].into();
        let a = [("human_openchest".into(), 536)].into();
        let mut s = Symbols::new();
        for (id, ops) in [
            (350, [Some("Open"), None, None, None, None]),
            (351, [None, Some("Search"), Some("Shut"), None, None]),
        ] {
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Drawers".into();
            t.ops = ops.map(|v| v.map(str::to_owned));
            d.locations.insert(id, t);
        }
        let h = drawers(&l, &a, &s, &d).unwrap();
        assert_eq!(h.len(), 3);
        let Handler::DelayedSwap { rule, delay } = h[&(350, 1)] else {
            panic!("warmup")
        };
        assert_eq!(
            (rule.next, rule.sound, rule.duration, delay),
            (351, Some(64), 500, 1)
        );
        assert!(h[&(351, 2)].arrival_delay());
        for id in [3190, 3191, 5618, 5619, 6875, 25766, 25767] {
            assert!(!h.keys().any(|&(node, _)| node == id));
        }
        s.insert("drawer_open".into(), 65);
        assert!(drawers(&l, &a, &s, &d).is_err());
    }
    #[test]
    fn drawers_actual_native_config_one_tick_change_and_five_hundred_tick_restore() {
        use gielinor_world::{
            loc_collision::LocCollision,
            locations::Location,
            simulation::{Action, PlayerData, World},
            stats::Stats,
            CoordGrid,
        };
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        for id in [350, 351] {
            w.location_types.insert(
                id,
                LocCollision {
                    width: 1,
                    length: 1,
                    block_walk: 2,
                    block_range: true,
                    break_route_finding: false,
                },
            );
        }
        w.location_handlers.insert(
            (350, 1),
            Handler::DelayedSwap {
                rule: Swap {
                    next: 351,
                    animation: Some(536),
                    sound: Some(64),
                    duration: 500,
                    arrival_delay: true,
                },
                delay: 1,
            },
        );
        let l = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 350, 10, 0).unwrap();
        w.locations.load(l);
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Drawers".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.submit(
            a,
            Action::OperateLocation {
                coords: l.coords,
                id: 350,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(l.coords, 10), Some(l));
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(l.coords, 10).unwrap().id, 351);
        for _ in 0..499 {
            w.tick().unwrap();
            assert_eq!(w.locations.find_shape(l.coords, 10).unwrap().id, 351);
        }
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(l.coords, 10), Some(l));
    }
}
