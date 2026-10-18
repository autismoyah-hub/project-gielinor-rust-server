//! Base cutting for the two ordinary redwood section types; guild modifiers/access are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Tree;
const ROWS: [(&str, u32, &str, u32); 2] = [
    ("redwoodtree_l", 29668, "redwoodtree_l_cut", 29669),
    ("redwoodtree_r", 29670, "redwoodtree_r_cut", 29671),
];
pub fn redwood_sections(
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
    if objects.get("redwood_logs") != Some(&19669) {
        return Err("Missing/aliased Redwood logs".into());
    }
    let item = defs.items.get(&19669).ok_or("Missing Redwood logs")?;
    if item.name != "Redwood logs"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
    {
        return Err("Invalid Redwood logs".into());
    }
    let mut trees = std::collections::BTreeMap::new();
    for (symbol, id, empty_symbol, empty) in ROWS {
        if locations.get(symbol) != Some(&(id as i32))
            || locations.get(empty_symbol) != Some(&(empty as i32))
        {
            return Err("Missing/aliased redwood section".into());
        }
        for (i, live) in [(id, true), (empty, false)] {
            let loc = defs
                .locations
                .get(&i)
                .ok_or("Missing redwood section definition")?;
            if loc.name != "Redwood tree"
                || loc.collision.width != 1
                || loc.collision.length != 2
                || !loc.multi.is_empty()
                || loc.multi_default != -1
                || loc.multi_varbit != -1
                || loc.multi_varp != -1
                || loc.ops[0].as_deref() != if live { Some("Cut") } else { None }
                || loc.params.get(&1545) != Some(&Param::Integer(1))
            {
                return Err(format!("Invalid redwood section {i}"));
            }
            if live {
                for (k, v) in [
                    (23, 90),
                    (65492, 19669),
                    (65493, 3800),
                    (65533, empty as i32),
                    (65489, 440),
                    (65488, 200),
                    (65485, 65523),
                ] {
                    if loc.params.get(&k) != Some(&Param::Integer(v)) {
                        return Err(format!("Unattested redwood section parameter {i}/{k}"));
                    }
                }
            }
        }
        trees.insert(
            id,
            Tree {
                members: true,
                level: 90,
                product: 19669,
                product_name: "Redwood logs".into(),
                xp: 380.0,
                stump: empty,
                deplete_chance: 0,
                chopping_ticks: Some(440),
                respawn: [199, 199],
                rates: [1351, 1349, 1353, 1361, 1355, 1357, 1359]
                    .into_iter()
                    .zip([(2, 6), (3, 9), (4, 12), (4, 14), (5, 15), (6, 18), (7, 21)])
                    .collect(),
            },
        );
    }
    Ok(trees)
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
            19669,
            gielinor_cache::definitions::ItemType::decode(b"\x02Redwood logs\0\x10\0").unwrap(),
        );
        let mut l = Symbols::new();
        for (symbol, id, empty_symbol, empty) in ROWS {
            l.insert(symbol.into(), id as i32);
            l.insert(empty_symbol.into(), empty as i32);
            for (i, live) in [(id, true), (empty, false)] {
                let mut t = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
                t.name = "Redwood tree".into();
                t.collision.width = 1;
                t.collision.length = 2;
                t.params.insert(1545, Param::Integer(1));
                if live {
                    t.ops[0] = Some("Cut".into());
                    for (k, v) in [
                        (23, 90),
                        (65492, 19669),
                        (65493, 3800),
                        (65533, empty as i32),
                        (65489, 440),
                        (65488, 200),
                        (65485, 65523),
                    ] {
                        t.params.insert(k, Param::Integer(v));
                    }
                }
                d.locations.insert(i, t);
            }
        }
        (
            d,
            [("redwood_logs".into(), 19669)].into(),
            l,
            [("woodcutting".into(), 8)].into(),
        )
    }
    #[test]
    fn redwood_sections_actual_cut_plane_rotation_member_lifetime_and_independent_restore() {
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
        let trees = redwood_sections(&o, &l, &n, &defs).unwrap();
        assert_eq!(trees.len(), 2);
        for (index, (_, id, _, empty)) in ROWS.into_iter().enumerate() {
            for refusal in 0..6 {
                let mut w = World::default();
                for x in (3192..3224).step_by(8) {
                    for z in (3192..3224).step_by(8) {
                        w.collision.allocate(x, z, 1);
                    }
                }
                w.object_types = defs.inventory_catalog();
                if refusal == 4 {
                    w.object_types.dummy.insert(19669);
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
                let c = CoordGrid::new(3201, 3200, 1).unwrap();
                let angle = index as u8;
                let live = Location::new(c, id, 10, angle).unwrap();
                let stump = Location::new(c, empty, 10, angle).unwrap();
                let ty = LocCollision {
                    width: 1,
                    length: 2,
                    block_walk: 2,
                    block_range: true,
                    break_route_finding: false,
                };
                for (_, i, _, e) in ROWS {
                    w.location_types.insert(i, ty);
                    w.location_types.insert(e, ty);
                }
                w.locations.load(live);
                loc_collision::toggle(&mut w.collision, c, 10, angle, ty, true).unwrap();
                let other = Location::new(
                    CoordGrid::new(3210, 3200, 1).unwrap(),
                    if id == 29668 { 29670 } else { 29668 },
                    10,
                    0,
                )
                .unwrap();
                w.locations.load(other);
                loc_collision::toggle(&mut w.collision, other.coords, 10, 0, ty, true).unwrap();
                let other_revision = w.locations.revision(other);
                let mut stats = Stats::default();
                stats.set_current(8, if refusal == 2 { 89 } else { 90 });
                stats.set_current(0, 0);
                let mut data = PlayerData::new(
                    1,
                    "Redwood cutter".into(),
                    CoordGrid::new(3200, 3200, 1).unwrap(),
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
                        id,
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
                        if self.rolls == 1 || self.now >= 460 {
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
                for _ in 0..if refusal == 0 { 700 } else { 20 } {
                    rng.now = w.clock.0;
                    w.tick_with_random(&mut rng).unwrap();
                    for e in w.drain_events() {
                        if let WorldEvent::Woodcutting { trace, .. } = e {
                            if trace.result == "success" {
                                count += 1;
                                assert_eq!(trace.fine_xp_delta, 4750);
                                if refusal == 5 {
                                    w.set_membership(a, false).unwrap();
                                }
                                if trace.respawn_delay.is_some() {
                                    assert!(trace.active_ticks.unwrap() >= 440);
                                    assert_eq!(trace.respawn_delay, Some(199));
                                    felled = Some(trace.tick);
                                    assert!(w.locations.revision(stump).is_some());
                                }
                            }
                        }
                    }
                    assert_eq!(w.locations.revision(other), other_revision);
                    if felled.is_some()
                        && restored.is_none()
                        && w.locations.revision(live).is_some()
                    {
                        restored = Some(w.clock.0 - 1);
                    }
                }
                let d = w.data(a).unwrap();
                assert_eq!(d.stats.fine_xp(8) - before.stats.fine_xp(8), 4750 * count);
                assert_eq!(d.stats.current(0), 0);
                assert_eq!(d.inventory.get(3).unwrap().id, 1359);
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
        }
        defs.locations.get_mut(&29668).unwrap().collision.length = 1;
        assert!(redwood_sections(&o, &l, &n, &defs).is_err());
    }
}
