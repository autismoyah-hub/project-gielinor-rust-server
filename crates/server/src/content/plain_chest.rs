//! The two source plain-chest types only; key, quest, clue and thieving chests are separate.
use super::{Definitions, Symbols};
use gielinor_world::location_content::{Handler, Swap};
use std::collections::BTreeMap;
pub fn plain_chest(
    locations: &Symbols,
    sequences: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if sequences.get("human_openchest") != Some(&536) {
        return Err("Unattested chest opening sequence".into());
    }
    for (symbol, id, name, ops) in [
        (
            "chestclosed",
            375,
            "Closed chest",
            [Some("Open"), None, None, None, None],
        ),
        (
            "chestopen",
            378,
            "Open chest",
            [None, Some("Search"), Some("Shut"), None, None],
        ),
    ] {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != name
                    || d.ops.iter().map(|s| s.as_deref()).collect::<Vec<_>>() != ops
                    || d.collision.width != 1
                    || d.collision.length != 1
                    || !d.multi.is_empty()
                    || d.multi_default != -1
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested plain chest {symbol}"));
        }
    }
    Ok([
        (
            (375, 1),
            Handler::TimedSwap(Swap {
                next: 378,
                animation: Some(536),
                sound: Some(52),
                duration: 500,
                arrival_delay: true,
            }),
        ),
        ((378, 2), Handler::Message("You find nothing of interest.")),
        (
            (378, 3),
            Handler::TimedSwap(Swap {
                next: 375,
                animation: Some(536),
                sound: None,
                duration: 500,
                arrival_delay: true,
            }),
        ),
    ]
    .into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plain_chest_exact_open_search_shut_bindings_animation_sound_and_late_wrong_variant_refusal()
    {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let l = [("chestclosed".into(), 375), ("chestopen".into(), 378)].into();
        let a = [("human_openchest".into(), 536)].into();
        for (id, name, ops) in [
            (375, "Closed chest", [Some("Open"), None, None, None, None]),
            (
                378,
                "Open chest",
                [None, Some("Search"), Some("Shut"), None, None],
            ),
        ] {
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = name.into();
            t.ops = ops.map(|v| v.map(str::to_owned));
            d.locations.insert(id, t);
        }
        let h = plain_chest(&l, &a, &d).unwrap();
        assert_eq!(h.len(), 3);
        let Handler::TimedSwap(open) = h[&(375, 1)] else {
            panic!("open")
        };
        let Handler::TimedSwap(shut) = h[&(378, 3)] else {
            panic!("shut")
        };
        assert_eq!(
            (open.next, open.animation, open.sound, open.duration),
            (378, Some(536), Some(52), 500)
        );
        assert_eq!((shut.next, shut.sound, shut.duration), (375, None, 500));
        assert!(matches!(
            h[&(378, 2)],
            Handler::Message("You find nothing of interest.")
        ));
        d.locations.get_mut(&378).unwrap().ops[0] = Some("Loot".into());
        assert!(plain_chest(&l, &a, &d).is_err());
    }
    #[test]
    fn plain_chest_source500_tick_policy_runs_real_world_restoration() {
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
        for id in [375, 378] {
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
            (375, 1),
            Handler::TimedSwap(Swap {
                next: 378,
                animation: Some(536),
                sound: Some(52),
                duration: 500,
                arrival_delay: true,
            }),
        );
        let l = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 375, 10, 0).unwrap();
        w.locations.load(l);
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Plainchest".into(),
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
                id: 375,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(l.coords, 10).unwrap().id, 378);
        for _ in 0..499 {
            w.tick().unwrap();
            assert_eq!(w.locations.find_shape(l.coords, 10).unwrap().id, 378);
        }
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(l.coords, 10), Some(l));
    }
}
