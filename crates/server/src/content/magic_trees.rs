//! Primary normal magic tree only; Farming, region access and extra drops are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Tree;
pub fn magic_trees(
    objects: &Symbols,
    locations: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, Tree>, String> {
    if stats.get("woodcutting") != Some(&8) {
        return Err("Missing Woodcutting stat".into());
    }
    let stat = defs.stats.get(&8).ok_or("Missing Woodcutting definition")?;
    if stat.name != "Woodcutting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99
    {
        return Err("Invalid Woodcutting stat".into());
    }
    if objects.get("magic_logs") != Some(&1513)
        || locations.get("magictree") != Some(&10834)
        || locations.get("magic_tree_stump_new") != Some(&9713)
    {
        return Err("Missing/aliased magic tree definitions".into());
    }
    let item = defs.items.get(&1513).ok_or("Missing Magic logs")?;
    if item.name != "Magic logs"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
    {
        return Err("Invalid Magic logs".into());
    }
    for (id, name, live) in [(10834, "Magic tree", true), (9713, "Tree Stump", false)] {
        let loc = defs.locations.get(&id).ok_or("Missing magic tree/stump")?;
        if loc.name != name
            || loc.collision.width != 2
            || loc.collision.length != 2
            || !loc.multi.is_empty()
            || loc.multi_default != -1
            || loc.multi_varbit != -1
            || loc.multi_varp != -1
            || loc.ops[0].as_deref() != if live { Some("Chop down") } else { None }
        {
            return Err(format!("Invalid magic tree/stump {id}"));
        }
        if live {
            for (k, v) in [
                (23, 75),
                (65492, 1513),
                (65493, 2500),
                (65533, 9713),
                (65489, 390),
                (65488, 200),
                (65485, 65524),
            ] {
                if loc.params.get(&k) != Some(&Param::Integer(v)) {
                    return Err(format!("Unattested magic tree parameter {k}"));
                }
            }
        }
    }
    Ok([(
        10834,
        Tree {
            members: true,
            level: 75,
            product: 1513,
            product_name: "Magic logs".into(),
            xp: 250.0,
            stump: 9713,
            deplete_chance: 0,
            chopping_ticks: Some(390),
            respawn: [199, 199],
            rates: [1351, 1349, 1353, 1361, 1355, 1357, 1359]
                .into_iter()
                .zip([(2, 6), (3, 9), (4, 12), (5, 13), (5, 15), (6, 18), (7, 21)])
                .collect(),
        },
    )]
    .into())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Definitions, Symbols, Symbols, Symbols) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        d.stats.insert(
            8,
            gielinor_cache::definitions::StatType::decode(b"\x01Woodcutting\0\0").unwrap(),
        );
        d.items.insert(
            1513,
            gielinor_cache::definitions::ItemType::decode(b"\x02Magic logs\0\x10\0").unwrap(),
        );
        for (id, name, live) in [(10834, "Magic tree", true), (9713, "Tree Stump", false)] {
            let mut loc = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
            loc.name = name.into();
            loc.collision.width = 2;
            loc.collision.length = 2;
            if live {
                loc.ops[0] = Some("Chop down".into());
                for (k, v) in [
                    (23, 75),
                    (65492, 1513),
                    (65493, 2500),
                    (65533, 9713),
                    (65489, 390),
                    (65488, 200),
                    (65485, 65524),
                ] {
                    loc.params.insert(k, Param::Integer(v));
                }
            }
            d.locations.insert(id, loc);
        }
        (
            d,
            [("magic_logs".into(), 1513)].into(),
            [
                ("magictree".into(), 10834),
                ("magic_tree_stump_new".into(), 9713),
            ]
            .into(),
            [("woodcutting".into(), 8)].into(),
        )
    }
    #[test]
    fn magic_trees_strict_primary_and_actual_long_lifetime_without_inventory_shortcut() {
        use gielinor_world::{
            inventory::Item,
            loc_collision::{self, LocCollision},
            locations::Location,
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            woodcutting::{Axe, Config},
            CoordGrid,
        };
        let (mut defs, o, l, n) = fixture();
        let trees = magic_trees(&o, &l, &n, &defs).unwrap();
        assert_eq!(trees[&10834].rates.len(), 7);
        for refusal in 0..6 {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            w.object_types = defs.inventory_catalog();
            if refusal == 4 {
                w.object_types.dummy.insert(1513);
            }
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
                trees: trees.clone(),
                full_sound: 0,
                fall_sound: 0,
            });
            let c = CoordGrid::new(3201, 3200, 0).unwrap();
            let live = Location::new(c, 10834, 10, 0).unwrap();
            let stump = Location::new(c, 9713, 10, 0).unwrap();
            let ty = LocCollision {
                width: 2,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(10834, ty);
            w.location_types.insert(9713, ty);
            w.locations.load(live);
            loc_collision::toggle(&mut w.collision, c, 10, 0, ty, true).unwrap();
            let mut stats = Stats::default();
            stats.set_current(8, if refusal == 2 { 74 } else { 75 });
            let mut data = PlayerData::new(
                1,
                "Magic cutter".into(),
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
            struct Draw {
                now: i32,
                rolls: usize,
            }
            impl gielinor_world::npc::RandomSource for Draw {
                fn below(&mut self, b: u32) -> u32 {
                    if b != 256 {
                        return 0;
                    }
                    self.rolls += 1;
                    if self.rolls == 1 || self.now >= 400 {
                        0
                    } else {
                        255
                    }
                }
            }
            let mut rng = Draw { now: 0, rolls: 0 };
            w.submit(
                a,
                Action::OperateLocation {
                    coords: c,
                    id: 10834,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut count = 0;
            let mut felled = None;
            let mut restored = None;
            for _ in 0..if refusal == 0 { 620 } else { 20 } {
                rng.now = w.clock.0;
                w.tick_with_random(&mut rng).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Woodcutting { trace, .. } = e {
                        if trace.result == "success" {
                            count += 1;
                            assert_eq!(trace.fine_xp_delta, 3125);
                            if refusal == 5 {
                                w.set_membership(a, false).unwrap();
                            }
                            if trace.respawn_delay.is_some() {
                                assert!(trace.active_ticks.unwrap() >= 390);
                                assert_eq!(trace.respawn_delay, Some(199));
                                felled = Some(trace.tick);
                                assert!(w.locations.revision(stump).is_some());
                            }
                        }
                    }
                }
                if felled.is_some() && restored.is_none() && w.locations.revision(live).is_some() {
                    restored = Some(w.clock.0 - 1);
                }
            }
            let d = w.data(a).unwrap();
            assert_eq!(d.stats.fine_xp(8) - before.stats.fine_xp(8), 3125 * count);
            assert_eq!(d.inventory.get(3).unwrap().id, 1349);
            if refusal == 0 {
                assert_eq!(count, 2);
                assert_eq!(restored.unwrap() - felled.unwrap(), 199);
            } else if refusal == 5 {
                assert_eq!(count, 1);
                assert!(felled.is_none());
            } else {
                assert_eq!(count, 0);
                assert_eq!(d.inventory, before.inventory);
                assert_eq!(d.stats, before.stats);
                if refusal != 4 {
                    assert_eq!(rng.rolls, 0);
                }
            }
        }
        defs.locations
            .get_mut(&10834)
            .unwrap()
            .params
            .insert(65489, Param::Integer(400));
        assert!(magic_trees(&o, &l, &n, &defs).is_err());
    }
}
