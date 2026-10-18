//! Ordinary use of uncharged dragon cosmetics, third-age and gilded picks only.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::mining::Pickaxe;
const ROWS: [(&str, i32, &str, &str, u16, i32, bool, Option<u32>); 5] = [
    (
        "dragon_pickaxe",
        11920,
        "Dragon pickaxe",
        "human_mining_dragon_pickaxe",
        7139,
        61,
        true,
        Some(6),
    ),
    (
        "dragon_pickaxe_pretty",
        12797,
        "Dragon pickaxe",
        "human_mining_dragon_pickaxe_pretty",
        642,
        61,
        true,
        Some(6),
    ),
    (
        "zalcano_pickaxe",
        23677,
        "Dragon pickaxe (or)",
        "human_mining_zalcano_pickaxe",
        8346,
        61,
        true,
        Some(6),
    ),
    (
        "3a_pickaxe",
        20014,
        "3rd age pickaxe",
        "human_mining_3a_pickaxe",
        7283,
        61,
        true,
        Some(6),
    ),
    (
        "trail_gilded_pickaxe",
        23276,
        "Gilded pickaxe",
        "human_mining_gilded_pickaxe",
        8313,
        41,
        false,
        None,
    ),
];
pub fn special_pickaxes(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<(i32, Pickaxe)>, String> {
    if stats.get("mining") != Some(&14) {
        return Err("Missing Mining stat".into());
    }
    let stat = defs
        .stats
        .get(&14)
        .ok_or("Missing Mining stat definition")?;
    if stat.name != "Mining" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Mining stat".into());
    }
    let mut picks = vec![];
    for (symbol, id, name, seq, animation, level, members, fast_one_in) in ROWS {
        if objects.get(symbol) != Some(&id) || sequences.get(seq) != Some(&i32::from(animation)) {
            return Err(format!("Missing/aliased special pickaxe {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing special pickaxe definition")?;
        if d.name != name
            || d.members != members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || d.params.get(&23) != Some(&Param::Integer(level))
            || d.params.get(&65494) != Some(&Param::Integer(i32::from(animation)))
            || d.params.get(&65383) != Some(&Param::Integer(3))
            || d.params.get(&65382) != Some(&Param::Integer(fast_one_in.unwrap_or(0) as i32))
        {
            return Err(format!("Invalid special pickaxe {id}"));
        }
        picks.push((
            id,
            Pickaxe {
                level,
                animation,
                interval: 3,
                members,
                fast_one_in,
            },
        ));
    }
    Ok(picks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Inventory, Item, StackType},
        mining::{Config, Rock},
        stats::Stats,
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
            14,
            gielinor_cache::definitions::StatType::decode(b"\x01Mining\0\0").unwrap(),
        );
        let mut objs = Symbols::new();
        let mut seqs = Symbols::new();
        for (symbol, id, name, seq, animation, level, members, fast) in ROWS {
            objs.insert(symbol.into(), id);
            seqs.insert(seq.into(), i32::from(animation));
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if members {
                b.push(16);
            }
            b.push(0);
            let mut ty = gielinor_cache::definitions::ItemType::decode(&b).unwrap();
            for (k, v) in [
                (23, level),
                (65494, i32::from(animation)),
                (65383, 3),
                (65382, fast.unwrap_or(0) as i32),
            ] {
                ty.params.insert(k, Param::Integer(v));
            }
            d.items.insert(id as u32, ty);
        }
        (d, objs, seqs, [("mining".into(), 14)].into())
    }
    fn rock() -> Rock {
        Rock {
            members: false,
            restore: None,
            level: 1,
            product: 440,
            product_name: "Iron ore".into(),
            xp: 35.0,
            depleted: 11365,
            respawn: 9,
            rates: (255, 255),
        }
    }
    #[test]
    fn special_pickaxes_exact_params_member_current_eligibility_and_cadence() {
        let (mut d, o, q, n) = fixture();
        let picks = special_pickaxes(&o, &q, &n, &d).unwrap();
        let config = Config {
            skill: 14,
            picks: picks.clone().into_iter().collect(),
            rocks: [(11364, rock())].into(),
            full_sound: 0,
        };
        config.validate().unwrap();
        let worn = Inventory::new(14, StackType::Normal, false);
        struct Draw {
            value: u32,
            calls: usize,
        }
        impl gielinor_world::npc::RandomSource for Draw {
            fn below(&mut self, n: u32) -> u32 {
                assert_eq!(n, 6);
                self.calls += 1;
                self.value
            }
        }
        for (id, pick) in picks {
            let mut inv = Inventory::new(28, StackType::Normal, false);
            inv.restore_slot(
                4,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            assert!(config
                .pick_for_member(&inv, &worn, pick.level - 1, true)
                .is_none());
            assert_eq!(
                config
                    .pick_for_member(&inv, &worn, pick.level, true)
                    .unwrap()
                    .0,
                id
            );
            assert_eq!(
                config
                    .pick_for_member(&inv, &worn, pick.level, false)
                    .is_some(),
                !pick.members
            );
            assert_eq!(
                config.pick(&inv, &worn, pick.level).is_some(),
                !pick.members
            );
            for value in 0..6 {
                let mut draw = Draw { value, calls: 0 };
                let delay = pick.action_delay(&mut draw);
                assert_eq!(
                    delay,
                    if pick.fast_one_in.is_some() && value == 0 {
                        2
                    } else {
                        3
                    }
                );
                assert_eq!(draw.calls, usize::from(pick.fast_one_in.is_some()));
            }
        }
        let wrong = [("dragon_pickaxe".into(), 12797)].into();
        assert!(special_pickaxes(&wrong, &q, &n, &d).is_err());
        d.items
            .get_mut(&11920)
            .unwrap()
            .params
            .insert(65383, Param::Integer(2));
        assert!(special_pickaxes(&o, &q, &n, &d).is_err());
        let mut bad = config.clone();
        bad.picks.get_mut(&11920).unwrap().fast_one_in = Some(0);
        assert!(bad.validate().is_err());
    }
    #[test]
    fn special_pickaxes_actual_reached_carried_dragon_uses_no_attack_gate_and_fast_roll() {
        use gielinor_world::{
            loc_collision::{self, LocCollision},
            locations::Location,
            simulation::{Action, PlayerData, World, WorldEvent},
            CoordGrid,
        };
        let (mut defs, o, q, n) = fixture();
        defs.items.insert(
            440,
            gielinor_cache::definitions::ItemType::decode(b"\x02Iron ore\0\0").unwrap(),
        );
        for fast in [0, 1] {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            w.object_types = defs.inventory_catalog();
            w.mining = Some(Config {
                skill: 14,
                picks: special_pickaxes(&o, &q, &n, &defs)
                    .unwrap()
                    .into_iter()
                    .collect(),
                rocks: [(11364, rock())].into(),
                full_sound: 0,
            });
            let c = CoordGrid::new(3201, 3200, 0).unwrap();
            let loc = Location::new(c, 11364, 10, 0).unwrap();
            let ty = LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(11364, ty);
            w.location_types.insert(11365, ty);
            w.locations.load(loc);
            loc_collision::toggle(&mut w.collision, c, 10, 0, ty, true).unwrap();
            let mut stats = Stats::default();
            stats.set_current(14, 61);
            stats.set_current(0, 0);
            let mut data = PlayerData::new(
                1,
                "Dragon miner".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            );
            data.xp_rate_hundreds = 125;
            data.inventory
                .restore_slot(
                    4,
                    Some(Item {
                        id: 11920,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, true).unwrap();
            w.drain_events();
            struct Draw(std::collections::VecDeque<(u32, u32)>);
            impl gielinor_world::npc::RandomSource for Draw {
                fn below(&mut self, bound: u32) -> u32 {
                    let (b, v) = self.0.pop_front().expect("No replay draw");
                    assert_eq!(bound, b);
                    v
                }
            }
            let mut draw = Draw([(6, fast), (256, 0), (255, 0)].into());
            w.submit(
                a,
                Action::OperateLocation {
                    coords: c,
                    id: 11364,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            let mut scheduled = false;
            let mut success = false;
            for _ in 0..8 {
                w.tick_with_random(&mut draw).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Mining { trace, .. } = e {
                        if trace.result == "scheduled" {
                            assert_eq!(trace.tool, Some(11920));
                            assert_eq!(
                                trace.action_delay - trace.tick,
                                if fast == 0 { 1 } else { 2 }
                            );
                            scheduled = true;
                        }
                        if trace.result == "success" {
                            assert_eq!(trace.fine_xp_delta, 437);
                            success = true;
                        }
                    }
                }
            }
            assert!(scheduled && success && draw.0.is_empty());
            assert_eq!(w.data(a).unwrap().inventory.get(4).unwrap().id, 11920);
            assert_eq!(w.data(a).unwrap().stats.current(0), 0);
        }
    }
}
