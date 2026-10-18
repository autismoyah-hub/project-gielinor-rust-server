//! Three uncharged one-handed axe families; charge, quest and felling mechanics are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::woodcutting::Axe;
pub struct SpecialAxe {
    pub id: i32,
    pub axe: Axe,
    pub rates: std::collections::BTreeMap<i32, (i32, i32)>,
}
const ROWS: [(&str, i32, &str, &str, u16, i32, bool); 3] = [
    (
        "dragon_axe",
        6739,
        "Dragon axe",
        "human_woodcutting_dragon_axe",
        2846,
        61,
        true,
    ),
    (
        "3a_axe",
        20011,
        "3rd age axe",
        "human_woodcutting_3a_axe",
        7264,
        61,
        true,
    ),
    (
        "trail_gilded_axe",
        23279,
        "Gilded axe",
        "human_woodcutting_gilded_axe",
        8303,
        41,
        false,
    ),
];
// Log identity -> dragon/third-age and gilded/rune comparative endpoints.
const RATES: [(i32, (i32, i32), (i32, i32)); 8] = [
    (1511, (240, 750), (224, 700)),
    (1521, (120, 375), (112, 350)),
    (1519, (60, 187), (56, 175)),
    (1517, (30, 93), (28, 87)),
    (1515, (15, 47), (14, 44)),
    (6333, (60, 190), (55, 164)),
    (6332, (34, 94), (29, 88)),
    (1513, (7, 22), (7, 21)),
];
pub fn special_woodcutting_axes(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<SpecialAxe>, String> {
    if stats.get("woodcutting") != Some(&8) {
        return Err("Missing Woodcutting stat".into());
    }
    let stat = defs.stats.get(&8).ok_or("Missing Woodcutting definition")?;
    if stat.name != "Woodcutting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99
    {
        return Err("Invalid Woodcutting stat".into());
    }
    let mut result = vec![];
    for (symbol, id, name, seq, animation, level, members) in ROWS {
        if objects.get(symbol) != Some(&id) || sequences.get(seq) != Some(&i32::from(animation)) {
            return Err(format!("Missing/aliased special axe {symbol}"));
        }
        let d = defs.items.get(&(id as u32)).ok_or("Missing special axe")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || d.params.get(&23) != Some(&Param::Integer(level))
            || d.params.get(&65494) != Some(&Param::Integer(i32::from(animation)))
        {
            return Err(format!("Invalid special axe {id}"));
        }
        result.push(SpecialAxe {
            id,
            axe: Axe {
                members,
                level,
                animation,
            },
            rates: RATES
                .into_iter()
                .map(|(log, dragon, gilded)| (log, if members { dragon } else { gilded }))
                .collect(),
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Inventory, Item, StackType},
        stats::Stats,
        woodcutting::{Config, Tree},
    };
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
        let mut o = Symbols::new();
        let mut s = Symbols::new();
        for (symbol, id, name, seq, animation, level, members) in ROWS {
            o.insert(symbol.into(), id);
            s.insert(seq.into(), i32::from(animation));
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if members {
                b.push(16);
            }
            b.push(0);
            let mut item = gielinor_cache::definitions::ItemType::decode(&b).unwrap();
            item.params.insert(23, Param::Integer(level));
            item.params
                .insert(65494, Param::Integer(i32::from(animation)));
            d.items.insert(id as u32, item);
        }
        d.items.insert(
            1519,
            gielinor_cache::definitions::ItemType::decode(b"\x02Willow logs\0\0").unwrap(),
        );
        (d, o, s, [("woodcutting".into(), 8)].into())
    }
    fn config(tools: &[SpecialAxe], log: i32) -> Config {
        let mut axes: std::collections::BTreeMap<_, _> =
            tools.iter().map(|t| (t.id, t.axe)).collect();
        axes.insert(
            1359,
            Axe {
                members: false,
                level: 41,
                animation: 867,
            },
        );
        let mut rates: std::collections::BTreeMap<_, _> =
            tools.iter().map(|t| (t.id, t.rates[&log])).collect();
        rates.insert(
            1359,
            tools.iter().find(|t| t.id == 23279).unwrap().rates[&log],
        );
        Config {
            skill: 8,
            axes,
            trees: [(
                10819,
                Tree {
                    members: false,
                    level: 30,
                    product: log,
                    product_name: "Willow logs".into(),
                    xp: 67.5,
                    stump: 9711,
                    deplete_chance: 0,
                    chopping_ticks: Some(50),
                    respawn: [14, 14],
                    rates,
                },
            )]
            .into(),
            full_sound: 0,
            fall_sound: 0,
        }
    }
    #[test]
    fn special_woodcutting_axes_strict_types_member_current_selection_and_source_rounded_tie() {
        let (mut d, o, s, n) = fixture();
        let tools = special_woodcutting_axes(&o, &s, &n, &d).unwrap();
        let c = config(&tools, 1521);
        c.validate().unwrap();
        let tree = &c.trees[&10819];
        let mut inv = Inventory::new(28, StackType::Normal, false);
        let mut worn = Inventory::new(14, StackType::Normal, false);
        inv.restore_slot(
            4,
            Some(Item {
                id: 23279,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            5,
            Some(Item {
                id: 6739,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            c.axe_for_tree_for_member(&inv, &worn, 61, tree, true)
                .unwrap()
                .0,
            6739
        );
        assert_eq!(
            c.axe_for_tree_for_member(&inv, &worn, 60, tree, true)
                .unwrap()
                .0,
            23279
        );
        assert_eq!(
            c.axe_for_tree_for_member(&inv, &worn, 61, tree, false)
                .unwrap()
                .0,
            23279
        );
        assert!(c
            .axe_for_tree_for_member(&inv, &worn, 40, tree, true)
            .is_none());
        worn.restore_slot(
            3,
            Some(Item {
                id: 20011,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            c.axe_for_tree_for_member(&inv, &worn, 61, tree, true)
                .unwrap()
                .0,
            20011
        );
        worn.restore_slot(3, None).unwrap();
        inv.restore_slot(4, None).unwrap();
        assert!(c.axe_for_tree(&inv, &worn, 61, tree).is_none());
        assert!(c.axe(&inv, &worn, 61).is_none());
        let magic = config(&tools, 1513);
        inv.restore_slot(
            4,
            Some(Item {
                id: 1359,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        // Source integer average: (7+21)/2 == (7+22)/2; preserve first-carried tie.
        assert_eq!(
            magic
                .axe_for_tree_for_member(&inv, &worn, 75, &magic.trees[&10819], true)
                .unwrap()
                .0,
            1359
        );
        d.items
            .get_mut(&6739)
            .unwrap()
            .params
            .insert(23, Param::Integer(60));
        assert!(special_woodcutting_axes(&o, &s, &n, &d).is_err());
    }
    #[test]
    fn special_woodcutting_axes_actual_reached_carried_tool_live_membership_and_no_attack_gate() {
        use gielinor_world::{
            loc_collision::{self, LocCollision},
            locations::Location,
            simulation::{Action, PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (d, o, s, n) = fixture();
        let tools = special_woodcutting_axes(&o, &s, &n, &d).unwrap();
        struct Draw(usize);
        impl gielinor_world::npc::RandomSource for Draw {
            fn below(&mut self, _: u32) -> u32 {
                self.0 += 1;
                0
            }
        }
        for scenario in 0..6 {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            w.object_types = d.inventory_catalog();
            if scenario == 4 {
                w.object_types.dummy.insert(1519);
            }
            w.woodcutting = Some(config(&tools, 1519));
            let c = CoordGrid::new(3201, 3200, 0).unwrap();
            let loc = Location::new(c, 10819, 10, 0).unwrap();
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
            loc_collision::toggle(&mut w.collision, c, 10, 0, ty, true).unwrap();
            let mut stats = Stats::default();
            stats.set_current(8, if scenario == 3 { 60 } else { 61 });
            stats.set_current(0, 0);
            let mut data = PlayerData::new(
                1,
                "Axe user".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            data.xp_rate_hundreds = 125;
            data.inventory
                .restore_slot(
                    5,
                    Some(Item {
                        id: 6739,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            if scenario != 2 {
                data.inventory
                    .restore_slot(
                        4,
                        Some(Item {
                            id: 23279,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, scenario != 1 && scenario != 2).unwrap();
            w.drain_events();
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::OperateLocation {
                    coords: c,
                    id: 10819,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut rng = Draw(0);
            let mut used = vec![];
            let mut animations = vec![];
            for _ in 0..20 {
                w.tick_with_random(&mut rng).unwrap();
                for e in w.drain_events() {
                    match e {
                        WorldEvent::Woodcutting { trace, .. } if trace.result == "success" => {
                            assert_eq!(trace.fine_xp_delta, 843);
                            used.push(trace.tool.unwrap());
                            if scenario == 5 {
                                w.set_membership(a, false).unwrap();
                            }
                        }
                        WorldEvent::PlayerCombatAnimation { sequence, .. } => {
                            animations.push(sequence)
                        }
                        _ => {}
                    }
                }
                if !used.is_empty() && (scenario != 5 || used.len() == 2) {
                    break;
                }
            }
            let now = w.data(a).unwrap();
            assert_eq!(now.stats.current(0), 0);
            assert_eq!(now.inventory.get(5).unwrap().id, 6739);
            assert_eq!(
                now.stats.fine_xp(8) - before.stats.fine_xp(8),
                843 * used.len() as i32
            );
            match scenario {
                0 => {
                    assert_eq!(used, vec![6739]);
                    assert!(animations.contains(&2846));
                }
                1 | 3 => {
                    assert_eq!(used, vec![23279]);
                    assert!(animations.contains(&8303));
                }
                5 => assert_eq!(used, vec![6739, 23279]),
                _ => {
                    assert!(used.is_empty());
                    assert_eq!(now.inventory, before.inventory);
                    assert_eq!(now.stats, before.stats);
                    if scenario == 2 {
                        assert_eq!(rng.0, 0);
                    }
                }
            }
        }
    }
}
