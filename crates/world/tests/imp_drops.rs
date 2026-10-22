use gielinor_world::{
    hits::{Hit, Style, Target},
    npc::{BlockWalk, Npc, RandomSource, Restrict},
    npc_death::{Chicken, Config, Definition, DropPolicy, DropResult},
    npc_loot::{Entry, Product, Profile, Weighted},
    simulation::{NpcId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn tile() -> CoordGrid {
    CoordGrid::new(3200, 3200, 0).unwrap()
}
struct Draw {
    value: u32,
    calls: usize,
}
impl RandomSource for Draw {
    fn below(&mut self, bound: u32) -> u32 {
        assert_eq!(bound, 128);
        assert!(self.value < bound);
        self.calls += 1;
        self.value
    }
}
fn setup() -> (World, NpcId) {
    let mut w = World::default();
    w.collision.allocate(3200, 3200, 0);
    w.npc_death = Some(Config {
        definitions: [(
            5007,
            Definition {
                animation: 172,
                animation_ticks: 3,
                sound: Some(535),
            },
        )]
        .into(),
        bones: 526,
        chickens: Default::default(),
    });
    w.npc_loot_profiles.insert(
        5007,
        Profile::new(
            DropPolicy::Imp,
            vec![Product {
                item: 25766,
                count: 1,
            }],
            Some(
                Weighted::new(
                    128,
                    vec![
                        Entry {
                            weight: 64,
                            product: Some(Product {
                                item: 995,
                                count: 1,
                            }),
                        },
                        Entry {
                            weight: 64,
                            product: Some(Product {
                                item: 592,
                                count: 1,
                            }),
                        },
                    ],
                )
                .unwrap(),
            ),
        )
        .unwrap(),
    );
    w.object_types.stackable.insert(995);
    let mut p = PlayerData::new(1, "Imp drop fixture".into(), tile(), Stats::default());
    p.character_id = 400;
    w.queue_login(1, p).unwrap();
    w.tick().unwrap();
    let a = w.actors().next().unwrap();
    w.set_observer_id(a, 900).unwrap();
    let mut n = Npc::new(5007, tile(), 1, Restrict::NoMove, BlockWalk::Npc).unwrap();
    n.stats[3] = 8;
    n.base_stats[3] = 8;
    n.respawns = false;
    n.hero_points.credit(400, 8);
    let id = w.spawn_npc(n).unwrap();
    w.queue_hit(
        Target::Npc(id),
        Hit {
            source: Some(Target::Player(a)),
            style: Style::Typeless,
            damage: 8,
            display: None,
        },
        1,
    )
    .unwrap();
    w.drain_events();
    (w, id)
}
fn advance(w: &mut World, n: usize, r: &mut Draw) -> Vec<gielinor_world::npc_death::DropTrace> {
    let mut out = vec![];
    for _ in 0..n {
        w.tick_with_random(r).unwrap();
        out.extend(w.drain_events().into_iter().filter_map(|e| {
            if let WorldEvent::NpcDrop(t) = e {
                Some(t)
            } else {
                None
            }
        }));
    }
    out
}
#[test]
fn weighted_private_batch_uses_one_draw_without_bones_extra_xp_or_replay() {
    for draw in 0..128 {
        let (mut w, id) = setup();
        let a = w.actors().next().unwrap();
        let stats = w.data(a).unwrap().stats.clone();
        let mut r = Draw {
            value: draw,
            calls: 0,
        };
        assert!(advance(&mut w, 3, &mut r).is_empty());
        assert_eq!(r.calls, 0);
        let t = advance(&mut w, 1, &mut r);
        assert_eq!(t.len(), 1);
        let t = &t[0];
        assert_eq!(t.npc, id);
        assert_eq!(t.owner, Some(a));
        assert_eq!(t.policy, DropPolicy::Imp);
        assert_eq!(t.draw, Some(draw));
        assert_eq!(t.result, DropResult::Committed);
        assert_eq!(
            t.products,
            [(25766, 1), (if draw < 64 { 995 } else { 592 }, 1)]
        );
        assert_eq!(w.objects.count(), 2);
        for h in w.objects.at(tile()) {
            let v = h.snapshot();
            assert_eq!((v.owner, v.receiver), (Some(900), Some(900)));
            assert!(!v.visible_to(Some(901)));
            assert_ne!(v.item, 526);
        }
        assert_eq!(w.data(a).unwrap().stats, stats);
        assert_eq!(r.calls, 1);
        assert!(advance(&mut w, 8, &mut r).is_empty());
        assert_eq!(r.calls, 1);
    }
}
#[test]
fn invalid_catalog_conflicts_suppression_and_offline_winner_never_roll_or_reward() {
    for case in 0..6 {
        let (mut w, id) = setup();
        match case {
            0 => {
                w.object_types.dummy.insert(25766);
            }
            1 => {
                w.object_types.dummy.insert(592);
            }
            2 => {
                w.npc_death.as_mut().unwrap().chickens.insert(
                    5007,
                    Chicken {
                        raw: 2138,
                        feather: 314,
                    },
                );
            }
            3 => {
                w.npc_no_death_drops.insert(5007);
            }
            4 => w.npc_mut(id).unwrap().hero_points.credit(999, 100),
            _ => {
                w.npc_guaranteed_extras.insert(
                    5007,
                    gielinor_world::npc_death::GuaranteedExtras {
                        products: vec![(1739, 1)],
                        policy: DropPolicy::Cow,
                    },
                );
            }
        }
        let mut r = Draw { value: 0, calls: 0 };
        let t = advance(&mut w, 4, &mut r);
        assert_eq!(t.len(), 1);
        assert_eq!(
            t[0].result,
            if matches!(case, 3 | 4) {
                DropResult::NoEligibleOwner
            } else {
                DropResult::Rejected
            }
        );
        assert!(t[0].products.is_empty());
        assert_eq!(r.calls, 0);
        assert_eq!(w.objects.count(), 0);
        assert!(advance(&mut w, 5, &mut r).is_empty());
    }
}
#[test]
fn weighted_profiles_retain_shared_reveal_and_expiry_deadlines() {
    let (mut w, _) = setup();
    let mut r = Draw {
        value: 64,
        calls: 0,
    };
    let t = advance(&mut w, 4, &mut r);
    let created = t[0].tick;
    let h = w.objects.at(tile()).cloned().collect::<Vec<_>>();
    while w.clock.0 < created + 99 {
        advance(&mut w, 1, &mut r);
    }
    assert!(h.iter().all(|h| !h.snapshot().visible_to(Some(901))));
    advance(&mut w, 1, &mut r);
    assert!(h.iter().all(|h| h.snapshot().visible_to(Some(901))));
    while w.clock.0 < created + 199 {
        advance(&mut w, 1, &mut r);
    }
    assert_eq!(w.objects.count(), 2);
    advance(&mut w, 1, &mut r);
    assert_eq!(w.objects.count(), 0);
    assert_eq!(r.calls, 1);
}

#[test]
fn respawn_uses_new_identity_and_one_fresh_weighted_reward() {
    let (mut w, old) = setup();
    let npc = w.npc_mut(old).unwrap();
    npc.respawns = true;
    npc.respawn_rate = 4;
    let mut r = Draw { value: 0, calls: 0 };
    let mut fresh = None;
    let mut drops = vec![];
    for _ in 0..8 {
        w.tick_with_random(&mut r).unwrap();
        for e in w.drain_events() {
            match e {
                WorldEvent::NpcDrop(t) => drops.push(t),
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
    let fresh = fresh.expect("New imp generation missing");
    assert_eq!(drops.len(), 1);
    assert_eq!(r.calls, 1);
    let a = w.actors().next().unwrap();
    let hit = Hit {
        source: Some(Target::Player(a)),
        style: Style::Typeless,
        damage: 8,
        display: None,
    };
    assert!(w.queue_hit(Target::Npc(old), hit, 1).is_err());
    w.npc_mut(fresh).unwrap().hero_points.credit(400, 8);
    w.queue_hit(Target::Npc(fresh), hit, 1).unwrap();
    r.value = 127;
    let t = advance(&mut w, 4, &mut r);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].npc, fresh);
    assert_eq!(t[0].products, [(25766, 1), (592, 1)]);
    assert_eq!(r.calls, 2);
    assert!(advance(&mut w, 3, &mut r).is_empty());
    assert_eq!(
        w.objects
            .at(tile())
            .map(|h| h.snapshot().count)
            .sum::<i32>(),
        4
    );
}
