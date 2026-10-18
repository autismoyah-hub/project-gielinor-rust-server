//! Lead target definitions exist without symbol aliases; only pinned direct IDs are allowed.
use super::{Definitions, Symbols};
use gielinor_world::mining::Rock;
pub fn lead_mining(
    objects: &Symbols,
    locations: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, Rock>, String> {
    if objects.get("lead_ore").is_some_and(|id| *id != 31716) || stats.get("mining") != Some(&14) {
        return Err("Aliased lead ore or missing Mining".into());
    }
    let stat = defs.stats.get(&14).ok_or("Missing Mining stat")?;
    if stat.name != "Mining" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Mining stat".into());
    }
    let item = defs
        .items
        .get(&31716)
        .ok_or("Missing direct lead ore definition")?;
    if item.name != "Lead ore"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
    {
        return Err("Invalid direct lead ore definition".into());
    }
    for (symbol, id, name) in [
        ("leadrock1", 58917, "Lead rocks"),
        ("leadrock1_empty", 58918, "Rocks"),
    ] {
        if locations.get(symbol).is_some_and(|v| *v != id) {
            return Err(format!("Aliased lead rock {symbol}"));
        }
        let d = defs
            .locations
            .get(&(id as u32))
            .ok_or("Missing direct lead rock definition")?;
        if d.name != name
            || d.ops[0].as_deref() != Some("Mine")
            || d.collision.width != 1
            || d.collision.length != 1
            || !d.multi.is_empty()
            || d.multi_default != -1
            || d.multi_varbit != -1
            || d.multi_varp != -1
        {
            return Err(format!("Invalid direct lead rock {id}"));
        }
    }
    Ok([(
        58917,
        Rock {
            members: true,
            restore: None,
            level: 25,
            product: 31716,
            product_name: "Lead ore".into(),
            xp: 40.5,
            depleted: 58918,
            respawn: 10,
            rates: (112, 278),
        },
    )]
    .into())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Definitions, Symbols) {
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
            31716,
            gielinor_cache::definitions::ItemType::decode(b"\x02Lead ore\0\x10\0").unwrap(),
        );
        for (id, name) in [(58917, "Lead rocks"), (58918, "Rocks")] {
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(b"\0\x1eMine\0\0");
            defs.locations.insert(
                id,
                gielinor_cache::locations::LocationType::decode(&b).unwrap(),
            );
        }
        (defs, [("mining".into(), 14)].into())
    }
    #[test]
    fn lead_mining_accepts_only_attested_direct_ids_and_rejects_conflicting_aliases() {
        let (mut defs, stats) = fixture();
        let empty = Symbols::new();
        let rocks = lead_mining(&empty, &empty, &stats, &defs).unwrap();
        let r = &rocks[&58917];
        assert_eq!(
            (r.level, r.product, r.xp, r.respawn, r.rates),
            (25, 31716, 40.5, 10, (112, 278))
        );
        assert!(r.members && r.restore.is_none());
        let wrong_obj = [("lead_ore".into(), 440)].into();
        assert!(lead_mining(&wrong_obj, &empty, &stats, &defs).is_err());
        let wrong_loc = [("leadrock1".into(), 11364)].into();
        assert!(lead_mining(&empty, &wrong_loc, &stats, &defs).is_err());
        defs.items.get_mut(&31716).unwrap().placeholder.template = 799;
        assert!(lead_mining(&empty, &empty, &stats, &defs).is_err());
    }
    #[test]
    fn lead_mining_actual_reached_member_level_and_credit_guards_restore_plain_node() {
        use gielinor_world::{
            inventory::Item,
            loc_collision::LocCollision,
            locations::Location,
            mining::{Config, Pickaxe},
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
        let (defs, names) = fixture();
        for refusal in 0..5 {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            let coords = CoordGrid::new(3201, 3200, 0).unwrap();
            let loc = Location::new(coords, 58917, 10, 0).unwrap();
            let depleted = Location::new(coords, 58918, 10, 0).unwrap();
            let ty = LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(58917, ty);
            w.location_types.insert(58918, ty);
            w.locations.load(loc);
            gielinor_world::loc_collision::toggle(&mut w.collision, coords, 10, 0, ty, true)
                .unwrap();
            w.object_types = defs.inventory_catalog();
            if refusal == 4 {
                w.object_types.dummy.insert(31716);
            }
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
                rocks: lead_mining(&Symbols::new(), &Symbols::new(), &names, &defs).unwrap(),
                full_sound: 0,
            });
            let mut stats = Stats::default();
            stats.set_base(14, 1);
            stats.set_current(14, if refusal == 2 { 24 } else { 25 });
            let mut data = PlayerData::new(
                1,
                "Lead miner".into(),
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
            if refusal == 3 {
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
            w.set_membership(a, refusal != 1).unwrap();
            w.drain_events();
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::OperateLocation {
                    coords,
                    id: 58917,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut random = Success::default();
            let mut mined = None;
            let mut restored = None;
            for _ in 0..30 {
                w.tick_with_random(&mut random).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Mining { trace, .. } = e {
                        if trace.result == "success" {
                            assert!(mined.is_none());
                            assert_eq!(trace.fine_xp_delta, 506);
                            assert_eq!(trace.respawn_delay, Some(10));
                            mined = Some(trace.tick);
                            assert!(w.locations.revision(depleted).is_some());
                        }
                    }
                }
                if mined.is_some() && restored.is_none() && w.locations.revision(loc).is_some() {
                    restored = Some(w.clock.0 - 1);
                }
            }
            let d = w.data(a).unwrap();
            if refusal == 0 {
                assert_eq!(restored.unwrap() - mined.unwrap(), 10);
                assert_eq!(d.stats.fine_xp(14) - before.stats.fine_xp(14), 506);
                assert_eq!(
                    d.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 31716)
                        .count(),
                    1
                );
                assert_eq!(d.inventory.get(3).unwrap().id, 1265);
            } else {
                assert!(mined.is_none() && restored.is_none());
                assert_eq!(d.inventory, before.inventory);
                assert_eq!(d.stats, before.stats);
                if refusal != 4 {
                    assert_eq!(random.draws, 0);
                }
            }
        }
    }
}
