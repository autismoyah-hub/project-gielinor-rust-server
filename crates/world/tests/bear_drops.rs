use gielinor_world::{
    hits::{Hit, Style, Target},
    npc::{BlockWalk, Npc, RandomSource, Restrict},
    npc_death::{Config, Definition, DropPolicy, DropResult},
    simulation::{NpcId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
use std::collections::BTreeMap;
fn tile() -> CoordGrid {
    CoordGrid::new(3200, 3200, 0).unwrap()
}
struct NoDraw;
impl RandomSource for NoDraw {
    fn below(&mut self, bound: u32) -> u32 {
        panic!("Guaranteed bear drops must not consume random rolls: {bound}")
    }
}
fn setup(raw: u16, respawns: bool) -> (World, NpcId) {
    let mut w = World::default();
    w.collision.allocate(3200, 3200, 0);
    w.npc_death = Some(Config {
        definitions: [(
            raw,
            Definition {
                animation: 4929,
                animation_ticks: 4,
                sound: None,
            },
        )]
        .into(),
        bones: 526,
        chickens: Default::default(),
    });
    w.npc_loot_profiles.insert(
        raw,
        gielinor_world::npc_loot::Profile::new(
            DropPolicy::Bear,
            vec![
                gielinor_world::npc_loot::Product {
                    item: 526,
                    count: 1,
                },
                gielinor_world::npc_loot::Product {
                    item: 948,
                    count: 1,
                },
                gielinor_world::npc_loot::Product {
                    item: 2136,
                    count: 1,
                },
            ],
            None,
        )
        .unwrap(),
    );
    let mut p = PlayerData::new(1, "Bear drop fixture".into(), tile(), Stats::default());
    p.character_id = 400;
    w.queue_login(1, p).unwrap();
    w.tick().unwrap();
    let a = w.actors().next().unwrap();
    w.set_observer_id(a, 900).unwrap();
    let mut n = Npc::new(raw, tile(), 1, Restrict::NoMove, BlockWalk::Npc).unwrap();
    n.stats[3] = 8;
    n.base_stats[3] = 8;
    n.respawns = respawns;
    n.respawn_rate = 4;
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
fn advance(w: &mut World, n: usize) -> Vec<gielinor_world::npc_death::DropTrace> {
    let mut out = vec![];
    for _ in 0..n {
        w.tick_with_random(&mut NoDraw).unwrap();
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
fn ordinary_adult_bear_profiles_commit_three_private_products_once_without_rng_or_xp() {
    for raw in [2838, 2839] {
        let (mut w, id) = setup(raw, false);
        let a = w.actors().next().unwrap();
        let stats = w.data(a).unwrap().stats.clone();
        let t = advance(&mut w, 5);
        assert_eq!(t.len(), 1);
        let t = &t[0];
        assert_eq!(t.policy, DropPolicy::Bear);
        assert!(t.json().contains("\"policy\":\"selected_bear\""));
        assert_eq!(t.npc, id);
        assert_eq!(t.owner, Some(a));
        assert_eq!(t.draw, None);
        assert_eq!(t.products, [(526, 1), (948, 1), (2136, 1)]);
        assert_eq!(t.result, DropResult::Committed);
        assert_eq!(t.handles.len(), 3);
        let stock = w
            .objects
            .at(tile())
            .map(|h| {
                let v = h.snapshot();
                assert_eq!((v.owner, v.receiver), (Some(900), Some(900)));
                assert!(!v.visible_to(Some(901)));
                (v.item, v.count)
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(stock, [(526, 1), (948, 1), (2136, 1)].into());
        assert_eq!(w.data(a).unwrap().stats, stats);
        assert!(w.npc(id).is_none());
        assert!(advance(&mut w, 5).is_empty());
        assert_eq!(w.objects.count(), 3);
    }
}
#[test]
fn bear_invalid_batches_are_atomic_and_offline_winner_or_suppression_cannot_fall_back() {
    for case in 0..5 {
        let (mut w, id) = setup(2838, false);
        match case {
            0 => {
                w.object_types.dummy.insert(948);
            }
            1 => {
                w.object_types.dummy.insert(2136);
            }
            2 => {
                w.object_types.dummy.insert(526);
            }
            3 => w.npc_mut(id).unwrap().hero_points.credit(999, 100),
            _ => {
                w.npc_no_death_drops.insert(2838);
            }
        }
        let t = advance(&mut w, 5);
        assert_eq!(t.len(), 1);
        assert_eq!(
            t[0].result,
            if case < 3 {
                DropResult::Rejected
            } else {
                DropResult::NoEligibleOwner
            }
        );
        assert!(t[0].handles.is_empty());
        assert_eq!(t[0].draw, None);
        assert_eq!(w.objects.count(), 0);
        if case < 3 {
            assert!(t[0].products.is_empty());
        } else {
            assert!(t[0].products.is_empty());
            assert_eq!(t[0].owner, None);
        }
        assert!(advance(&mut w, 6).is_empty());
        assert_eq!(w.objects.count(), 0);
    }
}
#[test]
fn bear_drop_batch_retains_shared_public_reveal_and_expiry_deadlines() {
    let (mut w, _) = setup(2839, false);
    let t = advance(&mut w, 5);
    let created = t[0].tick;
    let h = w.objects.at(tile()).cloned().collect::<Vec<_>>();
    assert_eq!(h.len(), 3);
    while w.clock.0 < created + 99 {
        advance(&mut w, 1);
    }
    assert!(h.iter().all(|h| !h.snapshot().visible_to(Some(901))));
    advance(&mut w, 1);
    assert!(h.iter().all(|h| h.snapshot().visible_to(Some(901))));
    while w.clock.0 < created + 199 {
        advance(&mut w, 1);
    }
    assert_eq!(w.objects.count(), 3);
    advance(&mut w, 1);
    assert_eq!(w.objects.count(), 0);
}
#[test]
fn bear_respawn_replaces_identity_and_second_life_earns_one_fresh_batch() {
    let (mut w, old) = setup(2839, true);
    let mut fresh = None;
    let mut drops = vec![];
    for _ in 0..9 {
        w.tick_with_random(&mut NoDraw).unwrap();
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
    let fresh = fresh.expect("Fresh NPC generation missing");
    assert!(w.npc(old).is_none());
    assert_eq!(drops.len(), 1);
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
    let t = advance(&mut w, 5);
    assert_eq!(t.len(), 1);
    assert_eq!(t[0].npc, fresh);
    assert_eq!(w.objects.count(), 6);
    assert!(advance(&mut w, 3).is_empty());
}
