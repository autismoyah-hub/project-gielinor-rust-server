//! Primary Arctic pine; log splitting, shields, pyres and regional access are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Tree;
pub fn arctic_pine(
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
    if objects.get("arctic_pine_log") != Some(&10810)
        || locations.get("arctic_pine") != Some(&3037)
        || locations.get("arctic_pine_tree_stump") != Some(&21274)
    {
        return Err("Missing/aliased Arctic pine definitions".into());
    }
    let item = defs.items.get(&10810).ok_or("Missing Arctic pine logs")?;
    if item.name != "Arctic pine logs"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
    {
        return Err("Invalid Arctic pine logs".into());
    }
    for (id, name, live) in [
        (3037, "Arctic pine tree", true),
        (21274, "Tree Stump", false),
    ] {
        let loc = defs.locations.get(&id).ok_or("Missing Arctic pine/stump")?;
        if loc.name != name
            || loc.collision.width != 2
            || loc.collision.length != 2
            || !loc.multi.is_empty()
            || loc.multi_default != -1
            || loc.multi_varbit != -1
            || loc.multi_varp != -1
            || loc.ops[0].as_deref() != if live { Some("Cut down") } else { None }
        {
            return Err(format!("Invalid Arctic pine/stump {id}"));
        }
        if live {
            for (k, v) in [
                (23, 54),
                (65492, 10810),
                (65493, 400),
                (65533, 21274),
                (65489, 140),
                (65488, 15),
                (65485, 65527),
            ] {
                if loc.params.get(&k) != Some(&Param::Integer(v)) {
                    return Err(format!("Unattested Arctic pine parameter {k}"));
                }
            }
        }
    }
    // Wiki curves intentionally differ from the protected target enum 65527.
    Ok([(
        3037,
        Tree {
            members: true,
            level: 54,
            product: 10810,
            product_name: "Arctic pine logs".into(),
            xp: 40.0,
            stump: 21274,
            deplete_chance: 0,
            chopping_ticks: Some(140),
            respawn: [14, 14],
            rates: [1351, 1349, 1353, 1361, 1355, 1357, 1359]
                .into_iter()
                .zip([
                    (8, 25),
                    (12, 37),
                    (16, 50),
                    (18, 56),
                    (20, 62),
                    (24, 75),
                    (28, 87),
                ])
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
            10810,
            gielinor_cache::definitions::ItemType::decode(b"\x02Arctic pine logs\0\x10\0").unwrap(),
        );
        for (id, name, live) in [
            (3037, "Arctic pine tree", true),
            (21274, "Tree Stump", false),
        ] {
            let mut t = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
            t.name = name.into();
            t.collision.width = 2;
            t.collision.length = 2;
            if live {
                t.ops[0] = Some("Cut down".into());
                for (k, v) in [
                    (23, 54),
                    (65492, 10810),
                    (65493, 400),
                    (65533, 21274),
                    (65489, 140),
                    (65488, 15),
                    (65485, 65527),
                ] {
                    t.params.insert(k, Param::Integer(v));
                }
            }
            d.locations.insert(id, t);
        }
        (
            d,
            [("arctic_pine_log".into(), 10810)].into(),
            [
                ("arctic_pine".into(), 3037),
                ("arctic_pine_tree_stump".into(), 21274),
            ]
            .into(),
            [("woodcutting".into(), 8)].into(),
        )
    }
    #[test]
    fn arctic_pine_actual_cut_down_member_lifetime_wiki_rates_and_restoration() {
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
        let trees = arctic_pine(&o, &l, &n, &defs).unwrap();
        assert_eq!(trees[&3037].rates[&1359], (28, 87));
        for refusal in 0..6 {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            w.object_types = defs.inventory_catalog();
            if refusal == 4 {
                w.object_types.dummy.insert(10810);
            }
            w.woodcutting = Some(Config {
                skill: 8,
                axes: [(
                    1359,
                    Axe {
                        members: false,
                        level: 41,
                        animation: 867,
                    },
                )]
                .into(),
                trees: trees.clone(),
                full_sound: 0,
                fall_sound: 0,
            });
            let c = CoordGrid::new(3201, 3200, 0).unwrap();
            let live = Location::new(c, 3037, 10, 0).unwrap();
            let stump = Location::new(c, 21274, 10, 0).unwrap();
            let ty = LocCollision {
                width: 2,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(3037, ty);
            w.location_types.insert(21274, ty);
            w.locations.load(live);
            loc_collision::toggle(&mut w.collision, c, 10, 0, ty, true).unwrap();
            let mut stats = Stats::default();
            stats.set_current(8, if refusal == 2 { 53 } else { 54 });
            stats.set_current(0, 0);
            let mut data = PlayerData::new(
                1,
                "Pine cutter".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            data.xp_rate_hundreds = 125;
            data.inventory
                .restore_slot(
                    3,
                    Some(Item {
                        id: 1359,
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
                    coords: c,
                    id: 3037,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
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
                    if self.rolls == 1 || self.now >= 160 {
                        0
                    } else {
                        255
                    }
                }
            }
            let mut rng = Draw { now: 0, rolls: 0 };
            let mut count = 0;
            let mut felled = None;
            let mut restored = None;
            for _ in 0..if refusal == 0 { 200 } else { 20 } {
                rng.now = w.clock.0;
                w.tick_with_random(&mut rng).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Woodcutting { trace, .. } = e {
                        if trace.result == "success" {
                            count += 1;
                            assert_eq!(trace.fine_xp_delta, 500);
                            if refusal == 5 {
                                w.set_membership(a, false).unwrap();
                            }
                            if trace.respawn_delay.is_some() {
                                assert!(trace.active_ticks.unwrap() >= 140);
                                assert_eq!(trace.respawn_delay, Some(14));
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
            assert_eq!(d.stats.fine_xp(8) - before.stats.fine_xp(8), 500 * count);
            assert_eq!(d.stats.current(0), 0);
            assert_eq!(d.inventory.get(3).unwrap().id, 1359);
            if refusal == 0 {
                assert_eq!(count, 2);
                assert_eq!(restored.unwrap() - felled.unwrap(), 14);
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
        defs.locations.get_mut(&3037).unwrap().ops[0] = Some("Chop down".into());
        assert!(arctic_pine(&o, &l, &n, &defs).is_err());
    }
}
