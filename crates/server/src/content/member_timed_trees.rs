//! Primary and Isle of Souls teak/mahogany; Prifddinas, Farming and quest variants remain separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Tree;
#[derive(Clone, Copy)]
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
    xp: i32,
    active: i32,
    cache_respawn: i32,
    respawn: i32,
    rate_enum: i32,
    rates: [(i32, i32); 7],
}
const PRIMARY_ROWS: [Row; 2] = [
    Row {
        symbol: "teaktree",
        id: 9036,
        name: "Teak tree",
        width: 1,
        stump_symbol: "teak_stump",
        stump: 9037,
        item_symbol: "teak_logs",
        item: 6333,
        item_name: "Teak logs",
        level: 35,
        xp: 850,
        active: 50,
        cache_respawn: 16,
        respawn: 15,
        rate_enum: 65529,
        rates: [
            (15, 46),
            (23, 70),
            (31, 93),
            (35, 102),
            (39, 117),
            (47, 140),
            (55, 164),
        ],
    },
    Row {
        symbol: "mahoganytree",
        id: 9034,
        name: "Mahogany tree",
        width: 3,
        stump_symbol: "mahogany_stump",
        stump: 9035,
        item_symbol: "mahogany_logs",
        item: 6332,
        item_name: "Mahogany logs",
        level: 50,
        xp: 1250,
        active: 100,
        cache_respawn: 15,
        respawn: 14,
        rate_enum: 65526,
        rates: [
            (8, 25),
            (12, 38),
            (16, 50),
            (18, 54),
            (20, 63),
            (25, 75),
            (29, 88),
        ],
    },
];
const ROWS: [Row; 4] = [
    PRIMARY_ROWS[0],
    PRIMARY_ROWS[1],
    Row {
        symbol: "teaktree_update",
        id: 40758,
        stump_symbol: "teaktree_update_stump",
        stump: 40759,
        ..PRIMARY_ROWS[0]
    },
    Row {
        symbol: "mahoganytree_update",
        id: 40760,
        stump_symbol: "mahoganytree_update_stump",
        stump: 40761,
        ..PRIMARY_ROWS[1]
    },
];
pub fn member_timed_trees(
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
    let mut trees = std::collections::BTreeMap::new();
    for r in ROWS {
        if objects.get(r.item_symbol) != Some(&r.item)
            || locations.get(r.symbol) != Some(&(r.id as i32))
            || locations.get(r.stump_symbol) != Some(&(r.stump as i32))
        {
            return Err(format!("Missing/aliased member tree {}", r.symbol));
        }
        let item = defs
            .items
            .get(&(r.item as u32))
            .ok_or("Missing member logs")?;
        if item.name != r.item_name
            || !item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err("Invalid member logs".into());
        }
        for (id, name, live) in [(r.id, r.name, true), (r.stump, "Tree stump", false)] {
            let loc = defs.locations.get(&id).ok_or("Missing member tree/stump")?;
            if loc.name != name
                || loc.collision.width != r.width
                || loc.collision.length != r.width
                || !loc.multi.is_empty()
                || loc.multi_default != -1
                || loc.multi_varbit != -1
                || loc.multi_varp != -1
                || loc.ops[0].as_deref() != if live { Some("Chop down") } else { None }
            {
                return Err(format!("Invalid member tree/stump {id}"));
            }
            if live {
                for (k, v) in [
                    (23, r.level),
                    (65492, r.item),
                    (65493, r.xp),
                    (65533, r.stump as i32),
                    (65489, r.active),
                    (65488, r.cache_respawn),
                    (65485, r.rate_enum),
                ] {
                    if loc.params.get(&k) != Some(&Param::Integer(v)) {
                        return Err(format!("Unattested member tree parameter {id}/{k}"));
                    }
                }
            }
        }
        trees.insert(
            r.id,
            Tree {
                members: true,
                level: r.level,
                product: r.item,
                product_name: r.item_name.into(),
                xp: f64::from(r.xp) / 10.0,
                stump: r.stump,
                deplete_chance: 0,
                chopping_ticks: Some(r.active),
                respawn: [r.respawn, r.respawn],
                rates: [1351, 1349, 1353, 1361, 1355, 1357, 1359]
                    .into_iter()
                    .zip(r.rates)
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
        let mut obj = Symbols::new();
        let mut locs = Symbols::new();
        for r in ROWS {
            obj.insert(r.item_symbol.into(), r.item);
            locs.insert(r.symbol.into(), r.id as i32);
            locs.insert(r.stump_symbol.into(), r.stump as i32);
            let mut b = vec![2];
            b.extend_from_slice(r.item_name.as_bytes());
            b.extend_from_slice(b"\0\x10\0");
            d.items.insert(
                r.item as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
            for (id, name, live) in [(r.id, r.name, true), (r.stump, "Tree stump", false)] {
                let mut ty = gielinor_cache::locations::LocationType::decode(b"\0").unwrap();
                ty.name = name.into();
                ty.collision.width = r.width;
                ty.collision.length = r.width;
                if live {
                    ty.ops[0] = Some("Chop down".into());
                    for (k, v) in [
                        (23, r.level),
                        (65492, r.item),
                        (65493, r.xp),
                        (65533, r.stump as i32),
                        (65489, r.active),
                        (65488, r.cache_respawn),
                        (65485, r.rate_enum),
                    ] {
                        ty.params.insert(k, Param::Integer(v));
                    }
                }
                d.locations.insert(id, ty);
            }
        }
        (d, obj, locs, [("woodcutting".into(), 8)].into())
    }
    #[test]
    fn member_timed_trees_strict_primary_types_params_and_seven_axe_curves() {
        let (mut d, o, l, n) = fixture();
        let trees = member_timed_trees(&o, &l, &n, &d).unwrap();
        assert_eq!(trees.len(), 4);
        for r in ROWS {
            let t = &trees[&r.id];
            assert!(t.members);
            assert_eq!(
                (t.level, t.product, t.xp, t.chopping_ticks, t.respawn),
                (
                    r.level,
                    r.item,
                    f64::from(r.xp) / 10.0,
                    Some(r.active),
                    [r.respawn, r.respawn]
                )
            );
            assert_eq!(t.rates.len(), 7);
            assert_eq!(t.rates[&1359], r.rates[6]);
        }
        d.locations
            .get_mut(&9036)
            .unwrap()
            .params
            .insert(65485, Param::Integer(65526));
        assert!(member_timed_trees(&o, &l, &n, &d).is_err());
        d.locations
            .get_mut(&9036)
            .unwrap()
            .params
            .insert(65485, Param::Integer(65529));
        d.items.get_mut(&6333).unwrap().members = false;
        assert!(member_timed_trees(&o, &l, &n, &d).is_err());
    }
    #[test]
    fn member_timed_trees_actual_reached_member_gate_live_loss_and_exact_restoration() {
        use gielinor_world::{
            inventory::Item,
            loc_collision::{self, LocCollision},
            locations::Location,
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            woodcutting::{Axe, Config},
            CoordGrid,
        };
        struct Success(usize);
        impl gielinor_world::npc::RandomSource for Success {
            fn below(&mut self, _: u32) -> u32 {
                self.0 += 1;
                0
            }
        }
        let (defs, o, l, n) = fixture();
        let trees = member_timed_trees(&o, &l, &n, &defs).unwrap();
        for r in ROWS {
            for refusal in 0..7 {
                let mut w = World::default();
                for x in (3192..3224).step_by(8) {
                    for z in (3192..3224).step_by(8) {
                        w.collision.allocate(x, z, 0);
                    }
                }
                w.object_types = defs.inventory_catalog();
                if refusal == 4 {
                    w.object_types.dummy.insert(r.item);
                }
                let c = CoordGrid::new(3201, 3200, 0).unwrap();
                let live = Location::new(c, r.id, 10, 0).unwrap();
                let stump = Location::new(c, r.stump, 10, 0).unwrap();
                let ty = LocCollision {
                    width: r.width,
                    length: r.width,
                    block_walk: 2,
                    block_range: true,
                    break_route_finding: false,
                };
                w.location_types.insert(r.id, ty);
                w.location_types.insert(r.stump, ty);
                w.locations.load(live);
                loc_collision::toggle(&mut w.collision, c, 10, 0, ty, true).unwrap();
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
                    trees: [(r.id, trees[&r.id].clone())].into(),
                    full_sound: 0,
                    fall_sound: 0,
                });
                let mut stats = Stats::default();
                stats.set_current(8, if refusal == 2 { r.level - 1 } else { r.level } as i8);
                let mut data = PlayerData::new(
                    1,
                    "Hardwood cutter".into(),
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
                w.submit(
                    a,
                    Action::OperateLocation {
                        coords: c,
                        id: r.id,
                        operation: 1,
                        control: false,
                    },
                )
                .unwrap();
                let mut rng = Success(0);
                let mut rewards = 0;
                let mut felled = None;
                let mut restored = None;
                for _ in 0..150 {
                    w.tick_with_random(&mut rng).unwrap();
                    for e in w.drain_events() {
                        if let WorldEvent::Woodcutting { trace, .. } = e {
                            if trace.result == "success" {
                                rewards += 1;
                                assert_eq!(
                                    trace.fine_xp_delta,
                                    if r.item == 6333 { 1062 } else { 1562 }
                                );
                                if refusal == 5 {
                                    w.set_membership(a, false).unwrap();
                                }
                                if refusal == 6 {
                                    w.data_mut(a)
                                        .unwrap()
                                        .stats
                                        .set_current(8, (r.level - 1) as i8);
                                }
                                if trace.respawn_delay.is_some() {
                                    assert!(trace.active_ticks.unwrap() >= r.active);
                                    assert_eq!(trace.respawn_delay, Some(r.respawn));
                                    felled = Some(trace.tick);
                                    assert!(w.locations.revision(stump).is_some());
                                }
                            }
                        }
                    }
                    if felled.is_some()
                        && restored.is_none()
                        && w.locations.revision(live).is_some()
                    {
                        restored = Some(w.clock.0 - 1);
                    }
                }
                let d = w.data(a).unwrap();
                assert_eq!(d.inventory.get(3).unwrap().id, 1349);
                assert_eq!(
                    d.stats.fine_xp(8) - before.stats.fine_xp(8),
                    rewards * if r.item == 6333 { 1062 } else { 1562 }
                );
                if refusal == 0 {
                    assert!(rewards > 1);
                    assert_eq!(restored.unwrap() - felled.unwrap(), r.respawn);
                } else if refusal >= 5 {
                    assert_eq!(rewards, 1);
                    assert!(felled.is_none());
                } else {
                    assert_eq!(rewards, 0);
                    assert_eq!(d.inventory, before.inventory);
                    assert_eq!(d.stats, before.stats);
                    assert!(felled.is_none());
                    if refusal != 4 {
                        assert_eq!(rng.0, 0);
                    }
                }
            }
        }
    }
}
