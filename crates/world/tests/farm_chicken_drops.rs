use gielinor_world::{
    hits::{Hit, Style, Target},
    npc::{BlockWalk, Mode, Npc, RandomSource, Restrict},
    npc_death::{Chicken, Config, Definition, DropPolicy, DropResult},
    simulation::{NpcId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};

use std::collections::BTreeMap;

const FARM: [u16; 7] = [2804, 2805, 2806, 10494, 10495, 10496, 10497];

fn tile() -> CoordGrid {
    CoordGrid::new(3200, 3200, 0).unwrap()
}

struct Draw {
    value: u32,
    calls: usize,
}

impl RandomSource for Draw {
    fn below(&mut self, b: u32) -> u32 {
        assert_eq!(b, 128);
        self.calls += 1;
        self.value
    }
}

fn setup(raw: u16, member: bool, respawns: bool) -> (World, NpcId) {
    let mut w = World::default();
    w.collision.allocate(3200, 3200, 0);
    w.object_types.stackable.insert(314);
    w.npc_death = Some(Config {
        definitions: [(
            raw,
            Definition {
                animation: 5389,
                animation_ticks: 4,
                sound: Some(356),
            },
        )]
        .into(),
        bones: 526,
        chickens: [(
            raw,
            Chicken {
                raw: 2138,
                feather: 314,
            },
        )]
        .into(),
    });
    let mut stats = Stats::default();
    stats.set_base(3, 10);
    stats.set_current(3, 10);
    let mut p = PlayerData::new(1, "Farm chicken drop fixture".into(), tile(), stats);
    p.character_id = 400;
    w.queue_login(1, p).unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    w.set_membership(actor, member).unwrap();
    w.set_observer_id(actor, 900).unwrap();
    let mut npc = Npc::new(raw, tile(), 1, Restrict::NoMove, BlockWalk::Npc).unwrap();
    npc.mode = Mode::None;
    npc.stats[3] = 3;
    npc.base_stats[3] = 3;
    npc.regen_rate = 0;
    npc.respawns = respawns;
    npc.respawn_rate = 4;
    npc.hero_points.credit(400, 3);
    let id = w.spawn_npc(npc).unwrap();
    w.queue_hit(
        Target::Npc(id),
        Hit {
            source: Some(Target::Player(actor)),
            style: Style::Typeless,
            damage: 3,
            display: None,
        },
        1,
    )
    .unwrap();
    w.drain_events();
    (w, id)
}

fn advance(
    w: &mut World,
    random: &mut Draw,
    n: usize,
) -> Vec<gielinor_world::npc_death::DropTrace> {
    let mut traces = vec![];
    for _ in 0..n {
        w.tick_with_random(random).unwrap();
        traces.extend(w.drain_events().into_iter().filter_map(|e| {
            if let WorldEvent::NpcDrop(t) = e {
                Some(t)
            } else {
                None
            }
        }));
    }
    traces
}

#[test]

fn farm_all_128_weighted_draws_and_member_boundaries_use_existing_private_atomic_kernel() {
    for raw in FARM {
        let mut weights = BTreeMap::new();
        for member in [false, true] {
            let draws = if member {
                vec![0, 63, 64, 95, 96, 127]
            } else {
                (0..128).collect()
            };
            for draw in draws {
                let (mut w, id) = setup(raw, member, false);
                let a = w.actors().next().unwrap();
                let before = w.data(a).unwrap().stats.clone();
                let mut rng = Draw {
                    value: draw,
                    calls: 0,
                };
                let t = advance(&mut w, &mut rng, 5);
                assert_eq!(t.len(), 1);
                let t = &t[0];
                assert_eq!(
                    (t.npc, t.npc_type, t.policy, t.result, t.draw, t.owner),
                    (
                        id,
                        raw,
                        DropPolicy::Chicken,
                        DropResult::Committed,
                        Some(draw),
                        Some(a)
                    )
                );
                assert_eq!(rng.calls, 1);
                let stock = w
                    .objects
                    .at(tile())
                    .map(|h| {
                        let o = h.snapshot();
                        assert_eq!((o.owner, o.receiver), (Some(900), Some(900)));
                        assert!(!o.visible_to(Some(901)));
                        (o.item, o.count)
                    })
                    .collect::<BTreeMap<_, _>>();
                assert_eq!(stock[&526], 1);
                assert_eq!(stock[&2138], 1);
                let feather = stock.get(&314).copied().unwrap_or(0);
                assert!(matches!(feather, 0 | 5 | 15));
                if !member {
                    *weights.entry(feather).or_insert(0) += 1;
                }
                assert_eq!(stock.len(), if feather == 0 { 2 } else { 3 });
                assert_eq!(
                    t.products.iter().copied().collect::<BTreeMap<_, _>>(),
                    stock
                );
                assert_eq!(w.data(a).unwrap().stats, before);
                assert!(w.npc(id).is_none());
                assert!(advance(&mut w, &mut rng, 2).is_empty());
                assert_eq!(rng.calls, 1);
            }
        }
        assert_eq!(weights, [(0, 32), (5, 64), (15, 32)].into());
    }
}

#[test]

fn farm_four_tick_family_death_waits_and_refuses_offline_winner_or_invalid_product_without_partial_batch(
) {
    for raw in FARM {
        for case in 0..3 {
            let (mut w, id) = setup(raw, false, false);
            if case == 1 {
                w.npc_mut(id).unwrap().hero_points.credit(999, 100);
            }
            if case == 2 {
                w.object_types.dummy.insert(2138);
            }
            let mut rng = Draw {
                value: 64,
                calls: 0,
            };
            for _ in 0..4 {
                assert!(advance(&mut w, &mut rng, 1).is_empty());
                assert_eq!(rng.calls, 0);
                assert_eq!(w.objects.count(), 0);
            }
            let t = advance(&mut w, &mut rng, 1);
            assert_eq!(t.len(), 1);
            assert_eq!(
                t[0].result,
                match case {
                    0 => DropResult::Committed,
                    1 => DropResult::NoEligibleOwner,
                    _ => DropResult::Rejected,
                }
            );
            assert_eq!(rng.calls, usize::from(case != 1));
            assert_eq!(w.objects.count(), if case == 0 { 3 } else { 0 });
            assert!(advance(&mut w, &mut rng, 5).is_empty());
            assert_eq!(rng.calls, usize::from(case != 1));
        }
    }
}

#[test]

fn farm_respawn_replaces_generation_and_uses_a_new_draw_for_the_second_life() {
    for raw in FARM {
        let (mut w, old) = setup(raw, false, true);
        let mut rng = Draw {
            value: 95,
            calls: 0,
        };
        let mut traces = vec![];
        let mut fresh = None;
        for _ in 0..9 {
            w.tick_with_random(&mut rng).unwrap();
            for e in w.drain_events() {
                match e {
                    WorldEvent::NpcDrop(t) => traces.push(t),
                    WorldEvent::SynchronizeNpcs { npcs, .. } => {
                        fresh = fresh.or_else(|| {
                            npcs.into_iter()
                                .find(|n| n.actor.slot == old.slot && n.actor != old)
                                .map(|n| n.actor)
                        })
                    }
                    _ => (),
                }
            }
        }
        let fresh = fresh.expect("Farm chicken did not respawn with a fresh generation");
        assert_eq!(traces.len(), 1);
        assert_eq!(rng.calls, 1);
        assert!(w.npc(old).is_none());
        let a = w.actors().next().unwrap();
        let hit = Hit {
            source: Some(Target::Player(a)),
            style: Style::Typeless,
            damage: 3,
            display: None,
        };
        assert!(w.queue_hit(Target::Npc(old), hit, 1).is_err());
        w.npc_mut(fresh).unwrap().hero_points.credit(400, 3);
        w.queue_hit(Target::Npc(fresh), hit, 1).unwrap();
        rng.value = 96;
        let t = advance(&mut w, &mut rng, 5);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].npc, fresh);
        assert_eq!(t[0].products, [(526, 1), (2138, 1)]);
        assert_eq!(rng.calls, 2);
        let mut stock = BTreeMap::new();
        for h in w.objects.at(tile()) {
            let o = h.snapshot();
            *stock.entry(o.item).or_insert(0) += o.count;
        }
        assert_eq!(stock, [(526, 2), (2138, 2), (314, 15)].into());
    }
}
