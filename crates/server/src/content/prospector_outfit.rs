//! Normal and golden Prospector only; Varrock-armour and other XP channels are separate.
use super::{Definitions, Symbols};
use gielinor_world::gathering_outfits::{Outfit, Piece};
const ITEMS: [(&str, i32, &str, i8, i8); 8] = [
    ("motherlode_reward_hat", 12013, "Prospector helmet", 0, -1),
    (
        "motherlode_reward_hat_gold",
        25549,
        "Golden prospector helmet",
        0,
        -1,
    ),
    ("motherlode_reward_top", 12014, "Prospector jacket", 4, 6),
    (
        "motherlode_reward_top_gold",
        25551,
        "Golden prospector jacket",
        4,
        6,
    ),
    ("motherlode_reward_legs", 12015, "Prospector legs", 7, -1),
    (
        "motherlode_reward_legs_gold",
        25553,
        "Golden prospector legs",
        7,
        -1,
    ),
    ("motherlode_reward_boots", 12016, "Prospector boots", 10, -1),
    (
        "motherlode_reward_boots_gold",
        25555,
        "Golden prospector boots",
        10,
        -1,
    ),
];
pub fn prospector_outfit(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Outfit, String> {
    if stats.get("mining") != Some(&14) {
        return Err("Missing/aliased Mining stat".into());
    }
    let stat = defs.stats.get(&14).ok_or("Missing Mining stat")?;
    if stat.name != "Mining" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Mining stat".into());
    }
    for (symbol, id, name, primary, secondary) in ITEMS {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased Prospector piece {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Prospector piece")?;
        if d.name != name
            || !d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || d.wear != [primary, secondary, -1]
        {
            return Err(format!("Invalid Prospector piece {id}"));
        }
    }
    let outfit = Outfit {
        members: true,
        full_set_basis_points: 50,
        pieces: vec![
            Piece {
                slot: 0,
                item_ids: vec![12013, 25549],
                basis_points: 40,
            },
            Piece {
                slot: 4,
                item_ids: vec![12014, 25551],
                basis_points: 80,
            },
            Piece {
                slot: 7,
                item_ids: vec![12015, 25553],
                basis_points: 60,
            },
            Piece {
                slot: 10,
                item_ids: vec![12016, 25555],
                basis_points: 20,
            },
        ],
    };
    outfit.validate().map_err(str::to_string)?;
    Ok(outfit)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prospector_outfit_requires_eight_exact_member_types_and_wear_slots() {
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
        let mut objects = Symbols::new();
        for (symbol, id, name, primary, secondary) in ITEMS {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(&[0, 16, 13, primary as u8]);
            if secondary >= 0 {
                b.extend_from_slice(&[14, secondary as u8]);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        let names = [("mining".into(), 14)].into();
        let outfit = prospector_outfit(&objects, &names, &defs).unwrap();
        assert_eq!(outfit.pieces.len(), 4);
        assert_eq!(outfit.full_set_basis_points, 50);
        assert_eq!(
            outfit.pieces.iter().map(|p| p.basis_points).sum::<u16>(),
            200
        );
        defs.items.get_mut(&25551).unwrap().wear[0] = 5;
        assert!(prospector_outfit(&objects, &names, &defs).is_err());
    }
}

#[cfg(test)]
mod dispatch_tests {
    use super::*;
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
    fn fixture() -> (Definitions, Outfit) {
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
        let mut objects = Symbols::new();
        for (symbol, id, name, primary, secondary) in ITEMS {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(&[0, 16, 13, primary as u8]);
            if secondary >= 0 {
                b.extend_from_slice(&[14, secondary as u8]);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        let outfit = prospector_outfit(&objects, &[("mining".into(), 14)].into(), &defs).unwrap();
        (defs, outfit)
    }
    fn allocate(w: &mut World, loc: Location, width: u8) {
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let ty = LocCollision {
            width,
            length: width,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(loc.id, ty);
        w.locations.load(loc);
        gielinor_world::loc_collision::toggle(
            &mut w.collision,
            loc.coords,
            loc.shape,
            loc.angle,
            ty,
            true,
        )
        .unwrap();
    }
    #[test]
    fn prospector_actual_mining_normal_golden_mixed_carried_member_live_wear_and_atomic_credit() {
        let (defs, outfit) = fixture();
        for case in 0..10 {
            let mut w = World::default();
            let coords = CoordGrid::new(3201, 3200, 0).unwrap();
            let loc = Location::new(coords, 11364, 10, 0).unwrap();
            allocate(&mut w, loc, 1);
            w.location_types.insert(11391, w.location_types[&11364]);
            w.object_types = defs.inventory_catalog();
            if case == 9 {
                w.object_types.dummy.insert(440);
            }
            w.gathering_outfits.insert(14, outfit.clone());
            w.mining = Some(Config {
                skill: 14,
                picks: [(
                    1265,
                    Pickaxe {
                        members: false,
                        fast_one_in: None,
                        level: 1,
                        animation: 625,
                        interval: 8,
                    },
                )]
                .into(),
                rocks: [(
                    11364,
                    Rock {
                        members: false,
                        restore: None,
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
            let mut stats = Stats::default();
            stats.set_base(14, 15);
            stats.set_current(14, 15);
            let mut data = PlayerData::new(
                1,
                "Prospector miner".into(),
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
            let ids = if case == 2 {
                [25549, 25551, 25553, 25555]
            } else if matches!(case, 3 | 8 | 9) {
                [12013, 25551, 12015, 25555]
            } else {
                [12013, 12014, 12015, 12016]
            };
            for (index, (slot, id)) in [0, 4, 7, 10].into_iter().zip(ids).enumerate() {
                if case == 0 || (case == 7 && index >= 2) {
                    continue;
                }
                let item = Some(Item {
                    id,
                    count: 1,
                    vars: if case == 8 { 37 } else { 0 },
                });
                if case == 4 {
                    data.inventory.restore_slot(6 + index, item).unwrap();
                } else {
                    data.equipment.restore_slot(slot, item).unwrap();
                }
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, case != 5).unwrap();
            w.drain_events();
            let before = w.data(a).unwrap().inventory.clone();
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
            w.tick_with_random(&mut Success).unwrap();
            if case == 6 {
                w.data_mut(a)
                    .unwrap()
                    .equipment
                    .restore_slot(4, None)
                    .unwrap();
            }
            let worn = w.data(a).unwrap().equipment.clone();
            let expected = match case {
                1 | 2 | 3 | 8 => 448,
                6 | 7 => 442,
                9 => 0,
                _ => 437,
            };
            let mut successes = 0;
            for _ in 0..22 {
                w.tick_with_random(&mut Success).unwrap();
                for e in w.drain_events() {
                    if let WorldEvent::Mining { trace, .. } = e {
                        if trace.result == "success" {
                            successes += 1;
                            assert_eq!(trace.fine_xp_delta, expected);
                        }
                    }
                }
            }
            let d = w.data(a).unwrap();
            assert_eq!(d.stats.fine_xp(14), expected);
            assert_eq!(d.equipment, worn);
            assert_eq!(d.inventory.get(3).unwrap().id, 1265);
            assert_eq!(successes, if case == 9 { 0 } else { 1 });
            if case == 9 {
                assert_eq!(d.inventory, before);
            } else {
                assert_eq!(
                    d.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 440)
                        .count(),
                    1
                );
            }
        }
    }
    #[test]
    fn prospector_actual_oak_does_not_apply_mining_outfit_to_woodcutting() {
        use gielinor_world::woodcutting::{Axe, Config as Woods, Tree};
        let (defs, outfit) = fixture();
        let mut w = World::default();
        let coords = CoordGrid::new(3201, 3200, 0).unwrap();
        let loc = Location::new(coords, 10820, 10, 0).unwrap();
        allocate(&mut w, loc, 2);
        w.object_types = defs.inventory_catalog();
        w.gathering_outfits.insert(14, outfit);
        w.woodcutting = Some(Woods {
            skill: 8,
            axes: [(
                1351,
                Axe {
                    level: 1,
                    animation: 879,
                },
            )]
            .into(),
            trees: [(
                10820,
                Tree {
                    level: 15,
                    product: 1521,
                    product_name: "Oak logs".into(),
                    xp: 37.5,
                    stump: 9710,
                    deplete_chance: 0,
                    chopping_ticks: Some(45),
                    respawn: [14, 14],
                    rates: [(1351, (32, 100))].into(),
                },
            )]
            .into(),
            full_sound: 0,
            fall_sound: 0,
        });
        let mut stats = Stats::default();
        stats.set_base(8, 15);
        stats.set_current(8, 15);
        let mut data = PlayerData::new(
            1,
            "Prospector woodcutter".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        data.inventory
            .restore_slot(
                3,
                Some(Item {
                    id: 1351,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        for (slot, id) in [(0, 12013), (4, 25551), (7, 12015), (10, 25555)] {
            data.equipment
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 37,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        w.drain_events();
        w.submit(
            a,
            Action::OperateLocation {
                coords,
                id: 10820,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        let mut made = false;
        for _ in 0..15 {
            w.tick_with_random(&mut Success).unwrap();
            for e in w.drain_events() {
                if let WorldEvent::Woodcutting { trace, .. } = e {
                    if trace.result == "success" {
                        assert_eq!(trace.fine_xp_delta, 468);
                        made = true;
                    }
                }
            }
            if made {
                break;
            }
        }
        assert!(made);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(14), 0);
    }
}
