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

#[test]
fn observe_actual_life_boundaries_before_and_during_cape_cooking() {
    use gielinor_world::hits::{Hit, Style, Target};
    let mut cases = 0;
    let mut violations = 0;
    for cape in [9801, 9802] {
        for surface in [26185, 26181] {
            for slot in [0, 13, 27] {
                for phase in 0..6 {
                    let (mut w, a, loc) = setup(cape, surface, slot, 125, 1);
                    let mut r = Random { calls: 0 };
                    let c = w.cooking.as_mut().unwrap();
                    c.recipes.get_mut(&2132).unwrap().ticks = 4;
                    c.surfaces
                        .get_mut(&26181)
                        .unwrap()
                        .recipes
                        .get_mut(&2132)
                        .unwrap()
                        .ticks = 4;
                    let predead = phase < 3;
                    if predead {
                        if phase == 0 {
                            w.data_mut(a).unwrap().stats.set_current(3, 0);
                        } else {
                            w.queue_hit(
                                Target::Player(a),
                                Hit {
                                    display: None,
                                    source: None,
                                    style: Style::Typeless,
                                    damage: 20,
                                },
                                1,
                            )
                            .unwrap();
                            w.tick_with_random(&mut r).unwrap();
                            assert!(w
                                .drain_events()
                                .iter()
                                .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
                            if phase == 2 {
                                w.data_mut(a).unwrap().stats.set_current(3, 10);
                            }
                        }
                    }
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
                    w.tick_with_random(&mut r).unwrap();
                    assert_eq!(
                        w.data(a).unwrap().stats.fine_xp(7),
                        130344311,
                        "admission must not already credit"
                    );
                    if !predead {
                        if phase == 3 {
                            w.data_mut(a).unwrap().stats.set_current(3, 0);
                        } else {
                            w.queue_hit(
                                Target::Player(a),
                                Hit {
                                    display: None,
                                    source: None,
                                    style: Style::Typeless,
                                    damage: 20,
                                },
                                1,
                            )
                            .unwrap();
                            w.tick_with_random(&mut r).unwrap();
                            assert!(w
                                .drain_events()
                                .iter()
                                .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
                            assert_eq!(
                                w.data(a).unwrap().stats.fine_xp(7),
                                130344311,
                                "four-tick barrier must precede completion"
                            );
                            if phase == 5 {
                                w.data_mut(a).unwrap().stats.set_current(3, 10);
                            }
                        }
                    }
                    let before = w.data(a).unwrap().clone();
                    for _ in 0..6 {
                        w.tick_with_random(&mut r).unwrap();
                    }
                    let p = w.data(a).unwrap();
                    let delta = p.stats.fine_xp(7) - before.stats.fine_xp(7);
                    let cooked = p.inventory.get(slot).is_some_and(|v| v.id == 2142);
                    let bad =
                        p.inventory != before.inventory || p.stats != before.stats || r.calls != 0;
                    cases += 1;
                    violations += usize::from(bad);
                    println!("PG_COOK_CAPE_LIFE cape={cape} surface={surface} slot={slot} phase={phase} hp={} fineXPdelta={delta} cooked={cooked} calls={} violation={bad}",p.stats.current(3),r.calls);
                }
            }
        }
    }
    assert_eq!(cases, 72);
    println!("PG_COOK_CAPE_LIFE_SUMMARY cases={cases} violations={violations};diagnostic completion is not acceptancePASS when violations>0");
}
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
