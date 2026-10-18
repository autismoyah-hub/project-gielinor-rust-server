//! Members' three-stage limestone quarry; canonical full-node restoration uses existing timers.
use super::{Definitions, Symbols};
use gielinor_world::mining::Rock;
pub fn limestone_mining(
    objects: &Symbols,
    locations: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, Rock>, String> {
    if objects.get("limestone") != Some(&3211) || stats.get("mining") != Some(&14) {
        return Err("Missing/aliased limestone or Mining".into());
    }
    let stat = defs.stats.get(&14).ok_or("Missing Mining stat")?;
    if stat.name != "Mining" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Mining stat".into());
    }
    let item = defs.items.get(&3211).ok_or("Missing limestone")?;
    if item.name != "Limestone"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
    {
        return Err("Invalid limestone".into());
    }
    for (symbol, id, name) in [
        ("limestone_rock1", 11382, "Limestone rock"),
        ("limestone_rock2", 11383, "Limestone rock"),
        ("limestone_rock3", 11384, "Limestone rock"),
        ("limestone_rock_noore", 11385, "Pile of rock"),
    ] {
        if locations.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased limestone stage {symbol}"));
        }
        let d = defs
            .locations
            .get(&(id as u32))
            .ok_or("Missing limestone stage")?;
        if d.name != name
            || d.ops[0].as_deref() != Some("Mine")
            || d.collision.width != 1
            || d.collision.length != 1
            || !d.multi.is_empty()
            || d.multi_default != -1
            || d.multi_varbit != -1
            || d.multi_varp != -1
        {
            return Err(format!("Invalid limestone stage {id}"));
        }
    }
    Ok([(11382, 11383, 39), (11383, 11384, 19), (11384, 11385, 9)]
        .into_iter()
        .map(|(id, depleted, respawn)| {
            (
                id,
                Rock {
                    members: true,
                    restore: Some(11382),
                    level: 10,
                    product: 3211,
                    product_name: "Limestone".into(),
                    xp: 26.5,
                    depleted,
                    respawn,
                    rates: (95, 350),
                },
            )
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::mining::{Config, Pickaxe};
    #[test]
    fn limestone_mining_validates_members_three_stages_and_rejects_cycles() {
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
        defs.items.insert(
            3211,
            gielinor_cache::definitions::ItemType::decode(b"\x02Limestone\0\x10\0").unwrap(),
        );
        let obj = [("limestone".into(), 3211)].into();
        let names = [("mining".into(), 14)].into();
        let mut locs = Symbols::new();
        for (symbol, id, name) in [
            ("limestone_rock1", 11382, "Limestone rock"),
            ("limestone_rock2", 11383, "Limestone rock"),
            ("limestone_rock3", 11384, "Limestone rock"),
            ("limestone_rock_noore", 11385, "Pile of rock"),
        ] {
            locs.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(b"\0\x1eMine\0\0");
            defs.locations.insert(
                id as u32,
                gielinor_cache::locations::LocationType::decode(&b).unwrap(),
            );
        }
        let rocks = limestone_mining(&obj, &locs, &names, &defs).unwrap();
        assert_eq!(rocks.len(), 3);
        assert_eq!((rocks[&11382].depleted, rocks[&11382].respawn), (11383, 39));
        assert_eq!((rocks[&11384].depleted, rocks[&11384].respawn), (11385, 9));
        assert!(rocks
            .values()
            .all(|r| r.members && r.restore == Some(11382)));
        let mut config = Config {
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
            rocks,
            full_sound: 0,
        };
        config.validate().unwrap();
        config.rocks.get_mut(&11384).unwrap().depleted = 11382;
        assert!(config.validate().is_err());
        config.rocks.get_mut(&11384).unwrap().depleted = 11385;
        config.rocks.get_mut(&11383).unwrap().members = false;
        assert!(config.validate().is_err());
        defs.items.get_mut(&3211).unwrap().members = false;
        assert!(limestone_mining(&obj, &locs, &names, &defs).is_err());
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
    #[derive(Default)]
    struct Success {
        draws: usize,
    }
    impl gielinor_world::npc::RandomSource for Success {
        fn below(&mut self, _: u32) -> u32 {
            self.draws += 1;
            0
        }
    }
    #[test]
    fn limestone_mining_actual_stages_member_gate_partial_restore_and_stale_timers() {
        for case in 0..7 {
            let expected = match case {
                1 | 5 => 1,
                2 => 2,
                3 => 3,
                _ => 0,
            };
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            let coords = CoordGrid::new(3201, 3200, 0).unwrap();
            let root = Location::new(coords, 11382, 10, 0).unwrap();
            let ty = LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            for id in 11382..=11385 {
                w.location_types.insert(id, ty);
            }
            w.locations.load(root);
            gielinor_world::loc_collision::toggle(&mut w.collision, coords, 10, 0, ty, true)
                .unwrap();
            let rocks = [(11382, 11383, 39), (11383, 11384, 19), (11384, 11385, 9)]
                .into_iter()
                .map(|(id, depleted, respawn)| {
                    (
                        id,
                        Rock {
                            members: true,
                            restore: Some(11382),
                            level: 10,
                            product: 3211,
                            product_name: "Limestone".into(),
                            xp: 26.5,
                            depleted,
                            respawn,
                            rates: (95, 350),
                        },
                    )
                })
                .collect();
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
                rocks,
                full_sound: 0,
            });
            let mut stats = Stats::default();
            stats.set_base(14, 10);
            stats.set_current(14, if case == 4 { 9 } else { 10 });
            let mut data = PlayerData::new(
                1,
                "Quarry miner".into(),
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
            if case == 6 {
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
            w.set_membership(a, case != 0).unwrap();
            w.drain_events();
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::OperateLocation {
                    coords,
                    id: 11382,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut random = Success::default();
            let mut stages = Vec::new();
            let mut last_success = None;
            let mut restored = None;
            for _ in 0..90 {
                w.tick_with_random(&mut random).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Mining { trace, .. } = e {
                        if trace.result == "success" {
                            assert_eq!(trace.fine_xp_delta, 331);
                            stages.push(trace.location.id);
                            last_success = Some((trace.tick, trace.respawn_delay.unwrap()));
                            if (case == 1 && stages.len() == 1) || (case == 2 && stages.len() == 2)
                            {
                                w.set_membership(a, false).unwrap();
                            }
                            if case == 5 && stages.len() == 1 {
                                w.object_types.dummy.insert(3211);
                            }
                        }
                    }
                }
                if !stages.is_empty() && restored.is_none() {
                    if let Some(rev) = w.locations.revision(root) {
                        restored = Some((w.clock.0 - 1, rev));
                    }
                }
            }
            assert_eq!(stages, &[11382, 11383, 11384][..expected]);
            let d = w.data(a).unwrap();
            assert_eq!(
                d.stats.fine_xp(14) - before.stats.fine_xp(14),
                331 * expected as i32
            );
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == 3211)
                    .count(),
                expected
            );
            assert_eq!(d.inventory.get(3).unwrap().id, 1265);
            if expected > 0 {
                let (t, revision) = restored.unwrap();
                let (last, duration) = last_success.unwrap();
                assert_eq!(t - last, duration);
                assert_eq!(
                    duration,
                    match expected {
                        1 => 39,
                        2 => 19,
                        _ => 9,
                    }
                );
                assert_eq!(
                    w.locations.revision(root),
                    Some(revision),
                    "Old intermediate timers must not replace the restored node again"
                );
            } else {
                assert!(restored.is_none());
                assert_eq!(random.draws, 0);
                assert_eq!(d.inventory, before.inventory);
                assert_eq!(d.stats, before.stats);
            }
        }
    }
}
