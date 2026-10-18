//! Independent public World completion checks; controlled recipe chance is explicit.
use gielinor_world::cooking::{GauntletProfile, Gauntlets};
use gielinor_world::{
    cooking::{Config, NoBurnCape, Recipe, Surface},
    inventory::Item,
    loc_collision::{self, LocCollision},
    locations::Location,
    npc::RandomSource,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};

fn fixture(surface: u32, slot: usize, mode: usize) -> (World, ActorId, Location) {
    let cape = if mode == 3 || mode == 19 {
        9801
    } else if mode == 4 {
        9802
    } else {
        -1
    };
    let level = if [1, 2, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14].contains(&mode) {
        64
    } else {
        40
    };
    let (mut w, a, loc) = setup(cape, surface, slot, 125, level);
    let row = Recipe {
        raw: 377,
        cooked: 379,
        burnt: 381,
        level: 40,
        stop_burn: Some(99),
        low: 0,
        high: 0,
        xp: 120,
        fine_xp_override: None,
        ticks: 2,
    };
    let c = w.cooking.as_mut().unwrap();
    c.recipes = [(377, row)].into();
    c.surfaces.get_mut(&26181).unwrap().recipes = [(377, row)].into();
    c.gauntlets = Some(Gauntlets {
        item: 775,
        surfaces: [26185, 26181].into(),
        profiles: [(
            377,
            GauntletProfile {
                cooked: 379,
                burnt: 381,
                level: 40,
                stop_burn: 64,
                low: 55,
                high: 368,
            },
        )]
        .into(),
    });
    w.held_use_types.insert(
        377,
        gielinor_world::held_use::Type {
            id: 377,
            content: 0,
        },
    );
    for s in [0, 13, 27] {
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(
                s,
                Some(Item {
                    id: 377,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    if mode != 2 && mode != 6 {
        w.data_mut(a)
            .unwrap()
            .equipment
            .restore_slot(
                9,
                Some(Item {
                    id: 775,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    (w, a, loc)
}
#[test]
fn actual_world_live_tool_cape_precedence_and_late_atomicity() {
    let mut cases = 0;
    for surface in [26185, 26181] {
        for slot in [0, 13, 27] {
            for mode in 0..27 {
                let (mut w, a, loc) = fixture(surface, slot, mode);
                let mut random = Random { calls: 0 };
                let initial = w.data(a).unwrap().clone();
                w.submit(
                    a,
                    Action::UseLocation {
                        coords: loc.coords,
                        id: loc.id,
                        slot: slot as i32,
                        object: 377,
                        control: false,
                    },
                )
                .unwrap();
                w.tick_with_random(&mut random).unwrap();
                assert_eq!(w.data(a).unwrap().inventory, initial.inventory);
                assert_eq!(w.data(a).unwrap().stats, initial.stats);
                assert_eq!(random.calls, 0);
                assert!(w
                    .drain_events()
                    .iter()
                    .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { .. })));
                match mode {
                    5 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(9, None)
                        .unwrap(),
                    6 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            9,
                            Some(Item {
                                id: 775,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap(),
                    7 => {
                        w.set_membership(a, false).unwrap();
                    }
                    8 => {
                        w.data_mut(a)
                            .unwrap()
                            .equipment
                            .restore_slot(9, None)
                            .unwrap();
                        w.data_mut(a)
                            .unwrap()
                            .equipment
                            .restore_slot(
                                8,
                                Some(Item {
                                    id: 775,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    9 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            9,
                            Some(Item {
                                id: 775,
                                count: 1,
                                vars: 7,
                            }),
                        )
                        .unwrap(),
                    10 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            9,
                            Some(Item {
                                id: 775,
                                count: 2,
                                vars: 0,
                            }),
                        )
                        .unwrap(),
                    11 => {
                        w.object_types.dummy.insert(775);
                    }
                    12 => {
                        w.cooking
                            .as_mut()
                            .unwrap()
                            .gauntlets
                            .as_mut()
                            .unwrap()
                            .surfaces
                            .remove(&surface);
                    }
                    13 => {
                        w.cooking
                            .as_mut()
                            .unwrap()
                            .gauntlets
                            .as_mut()
                            .unwrap()
                            .profiles
                            .remove(&377);
                    }
                    14 => {
                        w.cooking
                            .as_mut()
                            .unwrap()
                            .gauntlets
                            .as_mut()
                            .unwrap()
                            .profiles
                            .get_mut(&377)
                            .unwrap()
                            .cooked = 315
                    }
                    15 => {
                        let c = w.cooking.as_mut().unwrap();
                        c.recipes.get_mut(&377).unwrap().low = -1;
                        c.surfaces
                            .get_mut(&26181)
                            .unwrap()
                            .recipes
                            .get_mut(&377)
                            .unwrap()
                            .low = -1;
                    }
                    16 => {
                        w.object_types.dummy.insert(379);
                    }
                    17 => {
                        w.object_types.dummy.insert(381);
                    }
                    18 => w.data_mut(a).unwrap().stats.set_current(7, 39),
                    19 => w.cooking.as_mut().unwrap().no_burn_cape = None,
                    20 => {
                        w.submit(
                            a,
                            Action::Walk {
                                destination: CoordGrid::new(3224, 3225, 0).unwrap(),
                                run: false,
                            },
                        )
                        .unwrap();
                    }
                    21 => {
                        w.close_dialogs(a).unwrap();
                    }
                    22 => {
                        w.request_logout(a).unwrap();
                    }
                    23 => {
                        w.set_modal_open(a, true).unwrap();
                    }
                    24 => w
                        .data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(slot, None)
                        .unwrap(),
                    25 => w
                        .data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 317,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap(),
                    26 => w
                        .data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 377,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap(),
                    _ => {}
                }
                let before = w.data(a).unwrap().clone();
                for _ in 0..5 {
                    w.tick_with_random(&mut random).unwrap();
                }
                let success = [1, 3, 4, 6].contains(&mode);
                let commit = mode < 15 || mode == 19;
                let mut inv = before.inventory.clone();
                let mut stats = before.stats.clone();
                if commit {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: if success { 379 } else { 381 },
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                    if success {
                        stats.set_fine_xp(7, 130345811).unwrap();
                    }
                }
                assert_eq!(
                    w.data(a).unwrap().inventory,
                    inv,
                    "surface{surface} slot{slot} mode{mode}"
                );
                assert_eq!(w.data(a).unwrap().stats, stats, "mode{mode}");
                assert_eq!(w.data(a).unwrap().equipment, before.equipment);
                assert_eq!(random.calls, usize::from(commit && !success), "mode{mode}");
                if mode == 20 {
                    assert_ne!(w.data(a).unwrap().coords, before.coords);
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 162);
    println!("PG_GAUNTLET_WORLD162 fire-range three selectedslots liveequip/remove/member/count/vars/slot/catalog/profile/currentlevel,capeprecedence,invalidoriginal/bothoutputs andWalk/Close/logout/modal/stale controls preservewholeimages andRNG policy PASS");
}
struct Random {
    calls: usize,
}
impl RandomSource for Random {
    fn below(&mut self, n: u32) -> u32 {
        assert_eq!(n, 256);
        self.calls += 1;
        255
    }
}
fn setup(
    cape: i32,
    surface: u32,
    slot: usize,
    rate: i32,
    current: i8,
) -> (World, ActorId, Location) {
    let mut w = World::default();
    for x in [3216, 3224, 3232] {
        for z in [3216, 3224, 3232] {
            w.collision.allocate(x, z, 0);
        }
    }
    let recipe = Recipe {
        raw: 2132,
        cooked: 2142,
        burnt: 2146,
        level: 1,
        stop_burn: Some(99),
        low: 0,
        high: 0,
        xp: 30,
        fine_xp_override: None,
        ticks: 2,
    };
    w.cooking = Some(Config {
        gauntlets: None,
        no_burn_cape: Some(NoBurnCape {
            capes: [9801, 9802],
            excluded_raw: 2337,
        }),
        member_inputs: Default::default(),
        surfaces: [(
            26181,
            Surface {
                animation: 896,
                recipes: [(2132, recipe)].into(),
            },
        )]
        .into(),
        fire: 26185,
        animation: 897,
        skill: 7,
        max_level: 99,
        recipes: [(2132, recipe)].into(),
        tutorial: None,
    });
    let loc = Location::new(CoordGrid::new(3225, 3224, 0).unwrap(), surface, 10, 0).unwrap();
    let ty = LocCollision {
        width: 1,
        length: 1,
        block_walk: 2,
        block_range: false,
        break_route_finding: false,
    };
    w.location_types.insert(surface, ty);
    w.locations.load(loc);
    loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true).unwrap();
    w.held_use_types.insert(
        2132,
        gielinor_world::held_use::Type {
            id: 2132,
            content: 0,
        },
    );
    let mut stats = Stats::default();
    stats.set_base(3, 10);
    stats.set_current(3, 10);
    stats.set_base(7, 99);
    stats.set_current(7, current);
    stats.set_fine_xp(7, 130344311).unwrap();
    let mut p = PlayerData::new(
        1,
        "Independent Cooking cape observer".into(),
        CoordGrid::new(3224, 3224, 0).unwrap(),
        stats,
    );
    p.xp_rate_hundreds = rate;
    for s in 0..28 {
        p.inventory
            .restore_slot(
                s,
                Some(Item {
                    id: if [0, 13, 27].contains(&s) {
                        2132
                    } else if s == 26 {
                        995
                    } else {
                        1931
                    },
                    count: if s == 26 { 777 } else { 1 },
                    vars: if s == 5 { 9 } else { 0 },
                }),
            )
            .unwrap();
    }
    if cape >= 0 {
        p.equipment
            .restore_slot(
                1,
                Some(Item {
                    id: cape,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    p.equipment
        .restore_slot(
            4,
            Some(Item {
                id: 1129,
                count: 1,
                vars: 9,
            }),
        )
        .unwrap();
    w.object_types.stackable.insert(995);
    w.queue_login(1, p).unwrap();
    w.tick_with_random(&mut Random { calls: 0 }).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, true).unwrap();
    w.drain_events();
    assert_eq!(w.data(a).unwrap().inventory.get(slot).unwrap().id, 2132);
    (w, a, loc)
}
