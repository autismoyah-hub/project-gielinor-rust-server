//! Base ore profiles only; special mines, modifier systems and side drops are separate.
use super::{Definitions, Symbols};
use gielinor_world::mining::Rock;
struct Row {
    stem: &'static str,
    name: &'static str,
    locations: &'static [u32],
    item_symbol: &'static str,
    item: i32,
    item_name: &'static str,
    level: i32,
    xp_tenths: u16,
    respawn: i32,
    low: i32,
    high: i32,
}
const ROWS: [Row; 9] = [
    Row {
        stem: "ironrock",
        name: "Iron rocks",
        locations: &[11364, 11365],
        item_symbol: "iron_ore",
        item: 440,
        item_name: "Iron ore",
        level: 15,
        xp_tenths: 350,
        respawn: 9,
        low: 96,
        high: 350,
    },
    Row {
        stem: "silverrock",
        name: "Silver rocks",
        locations: &[11368, 11369],
        item_symbol: "silver_ore",
        item: 442,
        item_name: "Silver ore",
        level: 20,
        xp_tenths: 400,
        respawn: 100,
        low: 25,
        high: 200,
    },
    Row {
        stem: "coalrock",
        name: "Coal rocks",
        locations: &[11366, 11367],
        item_symbol: "coal",
        item: 453,
        item_name: "Coal",
        level: 30,
        xp_tenths: 500,
        respawn: 50,
        low: 16,
        high: 100,
    },
    Row {
        stem: "goldrock",
        name: "Gold rocks",
        locations: &[11370, 11371],
        item_symbol: "gold_ore",
        item: 444,
        item_name: "Gold ore",
        level: 40,
        xp_tenths: 650,
        respawn: 100,
        low: 7,
        high: 75,
    },
    Row {
        stem: "mithrilrock",
        name: "Mithril rocks",
        locations: &[11372, 11373],
        item_symbol: "mithril_ore",
        item: 447,
        item_name: "Mithril ore",
        level: 55,
        xp_tenths: 800,
        respawn: 200,
        low: 4,
        high: 50,
    },
    Row {
        stem: "adamantiterock",
        name: "Adamantite rocks",
        locations: &[11374, 11375],
        item_symbol: "adamantite_ore",
        item: 449,
        item_name: "Adamantite ore",
        level: 70,
        xp_tenths: 950,
        respawn: 400,
        low: 2,
        high: 25,
    },
    Row {
        stem: "runiterock",
        name: "Runite rocks",
        locations: &[11376, 11377],
        item_symbol: "runite_ore",
        item: 451,
        item_name: "Runite ore",
        level: 85,
        xp_tenths: 1250,
        respawn: 1200,
        low: 1,
        high: 18,
    },
    Row {
        stem: "copperrock",
        name: "Copper rocks",
        locations: &[11161],
        item_symbol: "copper_ore",
        item: 436,
        item_name: "Copper ore",
        level: 1,
        xp_tenths: 175,
        respawn: 4,
        low: 100,
        high: 350,
    },
    Row {
        stem: "tinrock",
        name: "Tin rocks",
        locations: &[11361],
        item_symbol: "tin_ore",
        item: 438,
        item_name: "Tin ore",
        level: 1,
        xp_tenths: 175,
        respawn: 4,
        low: 100,
        high: 350,
    },
];
pub fn ordinary_mining_rocks(
    objects: &Symbols,
    locations: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, Rock>, String> {
    if stats.get("mining") != Some(&14) {
        return Err("Missing/aliased Mining stat".into());
    }
    let stat = defs.stats.get(&14).ok_or("Missing Mining stat")?;
    if stat.name != "Mining" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Mining stat".into());
    }
    if locations.get("rocks2") != Some(&11391) {
        return Err("Missing/aliased depleted rock".into());
    }
    let depleted = defs.locations.get(&11391).ok_or("Missing depleted rock")?;
    if depleted.name != "Rocks"
        || depleted.collision.width != 1
        || depleted.collision.length != 1
        || !depleted.multi.is_empty()
        || depleted.multi_default != -1
        || depleted.multi_varbit != -1
        || depleted.multi_varp != -1
    {
        return Err("Invalid depleted rock".into());
    }
    let mut rocks = std::collections::BTreeMap::new();
    for row in &ROWS {
        if objects.get(row.item_symbol) != Some(&row.item) {
            return Err(format!("Missing/aliased ore {}", row.item_symbol));
        }
        let item = defs.items.get(&(row.item as u32)).ok_or("Missing ore")?;
        if item.name != row.item_name
            || item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid ordinary ore {}", row.item));
        }
        for (index, &id) in row.locations.iter().enumerate() {
            let variant = if row.locations.len() == 1 {
                2
            } else {
                index + 1
            };
            let symbol = format!("{}{variant}", row.stem);
            if locations.get(&symbol) != Some(&(id as i32)) {
                return Err(format!("Missing/aliased mining rock {symbol}"));
            }
            let loc = defs.locations.get(&id).ok_or("Missing mining rock")?;
            if loc.name != row.name
                || loc.ops[0].as_deref() != Some("Mine")
                || loc.collision.width != 1
                || loc.collision.length != 1
                || !loc.multi.is_empty()
                || loc.multi_default != -1
                || loc.multi_varbit != -1
                || loc.multi_varp != -1
            {
                return Err(format!("Invalid mining rock {id}"));
            }
            if rocks
                .insert(
                    id,
                    Rock {
                        level: row.level,
                        product: row.item,
                        product_name: row.item_name.into(),
                        xp: f64::from(row.xp_tenths) / 10.0,
                        depleted: 11391,
                        respawn: row.respawn,
                        rates: (row.low, row.high),
                    },
                )
                .is_some()
            {
                return Err("Duplicate ordinary mining rock".into());
            }
        }
    }
    Ok(rocks)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_mining_sixteen_profiles_validate_base_levels_rewards_curves_and_static_targets() {
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
            14,
            gielinor_cache::definitions::StatType::decode(b"\x01Mining\0\0").unwrap(),
        );
        let mut obj = Symbols::new();
        let mut locs: Symbols = [("rocks2".into(), 11391)].into();
        defs.locations.insert(
            11391,
            gielinor_cache::locations::LocationType::decode(b"\x02Rocks\0\0").unwrap(),
        );
        for row in &ROWS {
            obj.insert(row.item_symbol.into(), row.item);
            let mut b = vec![2];
            b.extend_from_slice(row.item_name.as_bytes());
            b.extend_from_slice(&[0, 0]);
            defs.items.insert(
                row.item as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
            for (index, &id) in row.locations.iter().enumerate() {
                let variant = if row.locations.len() == 1 {
                    2
                } else {
                    index + 1
                };
                locs.insert(format!("{}{variant}", row.stem), id as i32);
                let mut b = vec![2];
                b.extend_from_slice(row.name.as_bytes());
                b.extend_from_slice(b"\0\x1eMine\0\0");
                defs.locations.insert(
                    id,
                    gielinor_cache::locations::LocationType::decode(&b).unwrap(),
                );
            }
        }
        let stats = [("mining".into(), 14)].into();
        let rocks = ordinary_mining_rocks(&obj, &locs, &stats, &defs).unwrap();
        assert_eq!(rocks.len(), 16);
        for row in &ROWS {
            for id in row.locations {
                let r = &rocks[id];
                assert_eq!(
                    (r.product, r.level, r.respawn, r.rates, r.depleted),
                    (row.item, row.level, row.respawn, (row.low, row.high), 11391)
                );
                assert_eq!(r.xp * 10.0, f64::from(row.xp_tenths));
            }
        }
        assert_eq!(
            (rocks[&11364].level, rocks[&11364].xp, rocks[&11364].respawn),
            (15, 35.0, 9)
        );
        assert_eq!(
            (rocks[&11377].level, rocks[&11377].xp, rocks[&11377].respawn),
            (85, 125.0, 1200)
        );
        defs.locations.get_mut(&11374).unwrap().ops[0] = Some("Prospect".into());
        assert!(ordinary_mining_rocks(&obj, &locs, &stats, &defs).is_err());
    }
}

#[cfg(test)]
mod dispatch_tests {
    use gielinor_world::{
        inventory::Item,
        loc_collision::LocCollision,
        locations::Location,
        mining::{Config, Pickaxe, Rock},
        simulation::{Action, PlayerData, World, WorldEvent},
        stats::Stats,
        CoordGrid,
    };
    struct Success;
    impl gielinor_world::npc::RandomSource for Success {
        fn below(&mut self, _: u32) -> u32 {
            0
        }
    }
    #[test]
    fn ordinary_mining_reached_iron_awards_once_depletes_and_full_bag_refuses() {
        for full in [false, true] {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            let coords = CoordGrid::new(3201, 3200, 0).unwrap();
            let loc = Location::new(coords, 11364, 10, 0).unwrap();
            let depleted = Location::new(coords, 11391, 10, 0).unwrap();
            let ty = LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(11364, ty);
            w.location_types.insert(11391, ty);
            w.locations.load(loc);
            gielinor_world::loc_collision::toggle(&mut w.collision, coords, 10, 0, ty, true)
                .unwrap();
            w.mining = Some(Config {
                skill: 14,
                picks: [(
                    1265,
                    Pickaxe {
                        level: 1,
                        animation: 625,
                        interval: 8,
                    },
                )]
                .into(),
                rocks: [(
                    11364,
                    Rock {
                        level: 15,
                        product: 440,
                        product_name: "Iron ore".into(),
                        xp: 35.0,
                        depleted: 11391,
                        respawn: 9,
                        rates: (96, 350),
                    },
                )]
                .into(),
                full_sound: 0,
            });
            assert!(w.has_location_handler(11364, 1));
            assert!(!w.has_location_handler(11364, 2));
            let mut stats = Stats::default();
            stats.set_base(14, 15);
            stats.set_current(14, 15);
            let mut data = PlayerData::new(
                1,
                "Iron miner".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            data.xp_rate_hundreds = 125;
            data.inventory
                .restore_slot(
                    3,
                    Some(Item {
                        id: 1265,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            if full {
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
                    id: 11364,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut success = false;
            let mut count = 0;
            for _ in 0..20 {
                w.tick_with_random(&mut Success).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Mining { ref trace, .. } = e {
                        if trace.result == "success" {
                            assert_eq!(trace.fine_xp_delta, 437);
                            assert_eq!(trace.respawn_delay, Some(9));
                            success = true;
                            assert!(w.locations.revision(depleted).is_some());
                        }
                    }
                    if matches!(e, WorldEvent::SkillAdvanced { skill: 14, .. }) {
                        count += 1;
                    }
                }
            }
            if full {
                assert!(!success);
                assert_eq!(count, 0);
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert_eq!(w.data(a).unwrap().stats, before.stats);
            } else {
                assert!(success);
                assert_eq!(count, 1);
                assert_eq!(
                    w.data(a).unwrap().stats.fine_xp(14) - before.stats.fine_xp(14),
                    437
                );
                assert_eq!(
                    w.data(a)
                        .unwrap()
                        .inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 440)
                        .count(),
                    1
                );
                assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().id, 1265);
            }
            assert!(w.locations.revision(loc).is_some());
        }
    }
}
