//! Primary ordinary willow/maple/yew only; farming and regional variants are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Tree;
struct Row {
    symbol: &'static str,
    id: u32,
    name: &'static str,
    width: u8,
    stump_symbol: &'static str,
    stump: u32,
    item_symbol: &'static str,
    item: i32,
    item_name: &'static str,
    level: i32,
    xp_tenths: i32,
    active: i32,
    cache_respawn: i32,
    respawn: i32,
    rates: [(i32, i32); 7],
}
const ROWS: [Row; 3] = [
    Row {
        symbol: "willowtree",
        id: 10819,
        name: "Willow tree",
        width: 2,
        stump_symbol: "willow_tree_stump_new",
        stump: 9711,
        item_symbol: "willow_logs",
        item: 1519,
        item_name: "Willow logs",
        level: 30,
        xp_tenths: 675,
        active: 50,
        cache_respawn: 15,
        respawn: 14,
        rates: [
            (16, 50),
            (24, 75),
            (32, 100),
            (36, 112),
            (40, 125),
            (48, 150),
            (56, 175),
        ],
    },
    Row {
        symbol: "mapletree",
        id: 10832,
        name: "Maple tree",
        width: 2,
        stump_symbol: "maple_tree_stump_new",
        stump: 9712,
        item_symbol: "maple_logs",
        item: 1517,
        item_name: "Maple logs",
        level: 45,
        xp_tenths: 1000,
        active: 100,
        cache_respawn: 60,
        respawn: 59,
        rates: [
            (8, 25),
            (12, 37),
            (16, 50),
            (18, 56),
            (20, 62),
            (24, 75),
            (28, 87),
        ],
    },
    Row {
        symbol: "yewtree",
        id: 10822,
        name: "Yew tree",
        width: 3,
        stump_symbol: "yew_tree_stump_new",
        stump: 9714,
        item_symbol: "yew_logs",
        item: 1515,
        item_name: "Yew logs",
        level: 60,
        xp_tenths: 1750,
        active: 190,
        cache_respawn: 100,
        respawn: 99,
        rates: [
            (4, 12),
            (6, 19),
            (8, 25),
            (9, 28),
            (10, 31),
            (12, 37),
            (14, 44),
        ],
    },
];
pub fn timed_trees(
    objects: &Symbols,
    locations: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, Tree>, String> {
    if stats.get("woodcutting") != Some(&8) {
        return Err("Missing/aliased Woodcutting stat".into());
    }
    let stat = defs.stats.get(&8).ok_or("Missing Woodcutting stat")?;
    if stat.name != "Woodcutting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99
    {
        return Err("Invalid Woodcutting stat".into());
    }
    let mut trees = std::collections::BTreeMap::new();
    for row in &ROWS {
        if objects.get(row.item_symbol) != Some(&row.item)
            || locations.get(row.symbol) != Some(&(row.id as i32))
            || locations.get(row.stump_symbol) != Some(&(row.stump as i32))
        {
            return Err(format!("Missing/aliased ordinary tree {}", row.symbol));
        }
        let item = defs
            .items
            .get(&(row.item as u32))
            .ok_or("Missing ordinary logs")?;
        if item.name != row.item_name
            || item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid ordinary logs {}", row.item));
        }
        for (id, name, live) in [(row.id, row.name, true), (row.stump, "Tree Stump", false)] {
            let loc = defs
                .locations
                .get(&id)
                .ok_or("Missing ordinary tree/stump")?;
            if loc.name != name
                || loc.collision.width != row.width
                || loc.collision.length != row.width
                || !loc.multi.is_empty()
                || loc.multi_default != -1
                || loc.multi_varbit != -1
                || loc.multi_varp != -1
                || if live {
                    loc.ops[0].as_deref() != Some("Chop down") || loc.content_group != 32
                } else {
                    loc.ops[0].is_some()
                }
            {
                return Err(format!("Invalid ordinary tree/stump {id}"));
            }
            if live {
                for (key, value) in [
                    (23, row.level),
                    (65492, row.item),
                    (65493, row.xp_tenths),
                    (65533, row.stump as i32),
                    (65489, row.active),
                    (65488, row.cache_respawn),
                ] {
                    if loc.params.get(&key) != Some(&Param::Integer(value)) {
                        return Err(format!("Unattested ordinary tree parameter {id}:{key}"));
                    }
                }
            }
        }
        trees.insert(
            row.id,
            Tree {
                level: row.level,
                product: row.item,
                product_name: row.item_name.into(),
                xp: f64::from(row.xp_tenths) / 10.0,
                stump: row.stump,
                deplete_chance: 0,
                chopping_ticks: Some(row.active),
                respawn: [row.respawn, row.respawn],
                rates: [1351, 1349, 1353, 1361, 1355, 1357, 1359]
                    .into_iter()
                    .zip(row.rates)
                    .collect(),
            },
        );
    }
    Ok(trees)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timed_trees_attest_three_static_families_and_preserve_cache_wiki_respawn_difference() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            8,
            gielinor_cache::definitions::StatType::decode(b"\x01Woodcutting\0\0").unwrap(),
        );
        let mut obj = Symbols::new();
        let mut locs = Symbols::new();
        for row in &ROWS {
            obj.insert(row.item_symbol.into(), row.item);
            let mut b = vec![2];
            b.extend_from_slice(row.item_name.as_bytes());
            b.extend_from_slice(&[0, 0]);
            defs.items.insert(
                row.item as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
            for (symbol, id, name, live) in [
                (row.symbol, row.id, row.name, true),
                (row.stump_symbol, row.stump, "Tree Stump", false),
            ] {
                locs.insert(symbol.into(), id as i32);
                let mut b = vec![2];
                b.extend_from_slice(name.as_bytes());
                b.extend_from_slice(&[0, 14, row.width, 15, row.width]);
                if live {
                    b.extend_from_slice(b"\x1eChop down\0");
                }
                b.push(0);
                let mut d = gielinor_cache::locations::LocationType::decode(&b).unwrap();
                if live {
                    d.content_group = 32;
                    for (key, v) in [
                        (23, row.level),
                        (65492, row.item),
                        (65493, row.xp_tenths),
                        (65533, row.stump as i32),
                        (65489, row.active),
                        (65488, row.cache_respawn),
                    ] {
                        d.params.insert(key, Param::Integer(v));
                    }
                }
                defs.locations.insert(id, d);
            }
        }
        let names = [("woodcutting".into(), 8)].into();
        let trees = timed_trees(&obj, &locs, &names, &defs).unwrap();
        assert_eq!(trees.len(), 3);
        assert_eq!(
            (
                trees[&10819].chopping_ticks,
                trees[&10819].respawn,
                trees[&10819].xp
            ),
            (Some(50), [14, 14], 67.5)
        );
        assert_eq!(
            (trees[&10832].chopping_ticks, trees[&10832].respawn),
            (Some(100), [59, 59])
        );
        assert_eq!(
            (trees[&10822].chopping_ticks, trees[&10822].respawn),
            (Some(190), [99, 99])
        );
        defs.locations
            .get_mut(&10819)
            .unwrap()
            .params
            .insert(65533, Param::Integer(1));
        assert!(timed_trees(&obj, &locs, &names, &defs).is_err());
    }
}

#[cfg(test)]
mod dispatch_tests {
    use gielinor_world::{
        inventory::Item,
        loc_collision::LocCollision,
        locations::Location,
        simulation::{Action, PlayerData, World, WorldEvent},
        stats::Stats,
        woodcutting::{Axe, Config, Tree},
        CoordGrid,
    };
    struct Success;
    impl gielinor_world::npc::RandomSource for Success {
        fn below(&mut self, _: u32) -> u32 {
            0
        }
    }
    #[test]
    fn timed_trees_actual_willow_depletes_after_shared_active_threshold_and_restores_in14() {
        for refusal in 0..3 {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            let coords = CoordGrid::new(3201, 3200, 0).unwrap();
            let loc = Location::new(coords, 10819, 10, 0).unwrap();
            let stump = Location::new(coords, 9711, 10, 0).unwrap();
            let ty = LocCollision {
                width: 2,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(10819, ty);
            w.location_types.insert(9711, ty);
            w.locations.load(loc);
            gielinor_world::loc_collision::toggle(&mut w.collision, coords, 10, 0, ty, true)
                .unwrap();
            w.woodcutting = Some(Config {
                skill: 8,
                axes: [(
                    1349,
                    Axe {
                        level: 1,
                        animation: 877,
                    },
                )]
                .into(),
                trees: [(
                    10819,
                    Tree {
                        level: 30,
                        product: 1519,
                        product_name: "Willow logs".into(),
                        xp: 67.5,
                        stump: 9711,
                        deplete_chance: 0,
                        chopping_ticks: Some(50),
                        respawn: [14, 14],
                        rates: [(1349, (24, 75))].into(),
                    },
                )]
                .into(),
                full_sound: 0,
                fall_sound: 0,
            });
            let mut stats = Stats::default();
            stats.set_base(8, 30);
            stats.set_current(8, if refusal == 1 { 29 } else { 30 });
            let mut data = PlayerData::new(
                1,
                "Willow cutter".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            data.xp_rate_hundreds = 125;
            data.inventory
                .restore_slot(
                    3,
                    Some(Item {
                        id: 1349,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            if refusal == 2 {
                for slot in 0..28 {
                    if slot != 3 {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 1511,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                }
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.drain_events();
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::OperateLocation {
                    coords,
                    id: 10819,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut rewards = 0;
            let mut felled = None;
            let mut restored = None;
            for _ in 0..85 {
                w.tick_with_random(&mut Success).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Woodcutting { trace, .. } = e {
                        if trace.result == "success" {
                            rewards += 1;
                            assert_eq!(trace.fine_xp_delta, 843);
                            if trace.respawn_delay == Some(14) {
                                assert!(trace.active_ticks.unwrap() >= 50);
                                felled = Some(trace.tick);
                                assert!(w.locations.revision(stump).is_some());
                            }
                        }
                    }
                }
                if felled.is_some() && restored.is_none() && w.locations.revision(loc).is_some() {
                    restored = Some(w.clock.0 - 1);
                }
            }
            let d = w.data(a).unwrap();
            if refusal == 0 {
                assert!(rewards > 1);
                assert_eq!(restored.unwrap() - felled.unwrap(), 14);
                assert_eq!(d.stats.fine_xp(8) - before.stats.fine_xp(8), 843 * rewards);
                assert_eq!(
                    d.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 1519)
                        .count(),
                    rewards as usize
                );
                assert_eq!(d.inventory.get(3).unwrap().id, 1349);
            } else {
                assert_eq!(rewards, 0);
                assert!(felled.is_none() && restored.is_none());
                assert_eq!(d.inventory, before.inventory);
                assert_eq!(d.stats, before.stats);
            }
        }
    }
}
