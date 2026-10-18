//! Independent public World completion checks; controlled recipe chance is explicit.
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
struct Random {
    calls: usize,
}
impl RandomSource for Random {
    fn below(&mut self, n: u32) -> u32 {
        assert_eq!(n, 256);
        self.calls += 1;
        0
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
fn admit(w: &mut World, a: ActorId, loc: Location, slot: usize, r: &mut Random) {
    let before = w.data(a).unwrap().clone();
    w.submit(
        a,
        Action::UseLocation {
            coords: loc.coords,
            id: loc.id,
            slot: slot as i32,
            object: 2132,
            control: false,
        },
    )
    .unwrap();
    w.tick_with_random(r).unwrap();
    assert_eq!(w.data(a).unwrap().inventory, before.inventory);
    assert_eq!(w.data(a).unwrap().stats, before.stats);
    assert_eq!(r.calls, 0);
    assert!(
        w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::PlayerCombatAnimation {
                sequence: 896 | 897,
                ..
            }
        )),
        "did not reach actual cooking admission"
    );
}
#[test]
fn two_capes_fire_range_drained_levels_full_bag_and_fractional_rates() {
    let mut cases = 0;
    for cape in [9801, 9802] {
        for surface in [26185, 26181] {
            for slot in [0, 13, 27] {
                for rate in [100, 125, 500] {
                    for current in [1, 50, 99] {
                        let (mut w, a, loc) = setup(cape, surface, slot, rate, current);
                        let mut r = Random { calls: 0 };
                        let before = w.data(a).unwrap().clone();
                        admit(&mut w, a, loc, slot, &mut r);
                        let mut transition = None;
                        for _ in 0..5 {
                            w.tick_with_random(&mut r).unwrap();
                            for e in w.drain_events() {
                                if let WorldEvent::CookingMade { trace, .. } = e {
                                    assert!(transition.is_none());
                                    transition = Some(trace);
                                }
                            }
                        }
                        let t = transition.expect("missing authoritative cook result");
                        assert!(!t.outcome.burned);
                        assert_eq!(t.outcome.output, 2142);
                        assert_eq!(t.committed_at - t.job.queued_at, 2);
                        assert_eq!(r.calls, 0);
                        let mut inv = before.inventory;
                        inv.restore_slot(
                            slot,
                            Some(Item {
                                id: 2142,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                        let mut stats = before.stats;
                        stats.set_fine_xp(7, 130344311 + 300 * rate / 100).unwrap();
                        assert_eq!(w.data(a).unwrap().inventory, inv);
                        assert_eq!(w.data(a).unwrap().stats, stats);
                        assert_eq!(w.data(a).unwrap().equipment, before.equipment);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 108);
    println!("PG_COOK_CAPE108 bothcapes fire/range selected0/13/27 rates100/125/500 current1/50/99 fullbag sameexactXP/gear zeroRNG strictselected2tick completion PASS;controlled0chance recipe,not actualfoodcurve metadata");
}
#[test]
fn live_equipment_member_refusal_and_atomic_guards() {
    let mut cases = 0;
    for surface in [26185, 26181] {
        for slot in [0, 13, 27] {
            for mode in 0..20 {
                let (mut w, a, loc) =
                    setup(if mode == 1 { -1 } else { 9801 }, surface, slot, 125, 1);
                let mut r = Random { calls: 0 };
                admit(&mut w, a, loc, slot, &mut r);
                match mode {
                    0 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(1, None)
                        .unwrap(),
                    1 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 9802,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap(),
                    2 => {
                        w.set_membership(a, false).unwrap();
                    }
                    3 => {
                        w.data_mut(a)
                            .unwrap()
                            .equipment
                            .restore_slot(1, None)
                            .unwrap();
                        w.data_mut(a)
                            .unwrap()
                            .equipment
                            .restore_slot(
                                0,
                                Some(Item {
                                    id: 9801,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    4 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 9801,
                                count: 2,
                                vars: 0,
                            }),
                        )
                        .unwrap(),
                    5 => w
                        .data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 9801,
                                count: 1,
                                vars: 7,
                            }),
                        )
                        .unwrap(),
                    6 => {
                        w.object_types.dummy.insert(9801);
                    }
                    7 => {
                        w.object_types.stackable.insert(9801);
                    }
                    8 => w.data_mut(a).unwrap().equipment.placeholders = true,
                    9 => w.data_mut(a).unwrap().stats.set_current(7, 0),
                    10 => w
                        .data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(slot, None)
                        .unwrap(),
                    11 => w
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
                    12 => w
                        .data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 2132,
                                count: 2,
                                vars: 0,
                            }),
                        )
                        .unwrap(),
                    13 => w
                        .data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 2132,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap(),
                    14 => {
                        w.object_types.dummy.insert(2142);
                    }
                    15 => {
                        w.object_types.dummy.insert(2146);
                    }
                    16 => w.data_mut(a).unwrap().xp_rate_hundreds = 0,
                    17 => {
                        w.set_modal(a, true).unwrap();
                    }
                    18 => {
                        w.locations.remove(loc);
                    }
                    _ => {
                        w.close_dialogs(a).unwrap();
                    }
                }
                let before = w.data(a).unwrap().clone();
                for _ in 0..5 {
                    w.tick_with_random(&mut r).unwrap();
                }
                let committed = mode < 9;
                let success = mode == 1;
                let mut inv = before.inventory.clone();
                let mut stats = before.stats.clone();
                if committed {
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: if success { 2142 } else { 2146 },
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                    if success {
                        stats.set_fine_xp(7, 130344686).unwrap();
                    }
                }
                assert_eq!(
                    w.data(a).unwrap().inventory,
                    inv,
                    "surface{surface}slot{slot}mode{mode}"
                );
                assert_eq!(w.data(a).unwrap().stats, stats, "mode{mode}");
                assert_eq!(w.data(a).unwrap().equipment, before.equipment);
                assert_eq!(r.calls, usize::from(committed && !success), "mode{mode}");
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 120);
    println!("PG_COOK_CAPE_GUARDS120 livecape equip/remove/member/wrongslot/count/vars/catalog versusnormalburn;lowlevel/stale/count/vars/bothoutputpolicies/rate/modal/location/Close refusebeforeRNG wholeimages PASS");
}
