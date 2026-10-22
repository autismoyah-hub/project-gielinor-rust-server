use gielinor_world::{
    hits::{Hit, Style, Target},
    npc::{BlockWalk, Mode, Npc, RandomSource, Restrict},
    npc_death::{Chicken, Config, Definition, DropPolicy, DropResult, DropTrace, GuaranteedExtras},
    npc_loot::Profile,
    simulation::{NpcId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn tile() -> CoordGrid {
    CoordGrid::new(3200, 3200, 0).unwrap()
}
struct NoDraw;
impl RandomSource for NoDraw {
    fn below(&mut self, b: u32) -> u32 {
        panic!("No normal spider loot may draw RNG: {b}")
    }
}
fn setup(raw: u16, respawns: bool) -> (World, NpcId) {
    let mut w = World::default();
    w.collision.allocate(3200, 3200, 0);
    w.npc_death = Some(Config {
        definitions: [(
            raw,
            Definition {
                animation: 6251,
                animation_ticks: 3,
                sound: Some(3608),
            },
        )]
        .into(),
        bones: 526,
        chickens: Default::default(),
    });
    w.npc_loot_profiles
        .insert(raw, Profile::new(DropPolicy::Spider, vec![], None).unwrap());
    let mut p = PlayerData::new(
        1,
        "Spider no-remains fixture".into(),
        tile(),
        Stats::default(),
    );
    p.character_id = 400;
    w.queue_login(1, p).unwrap();
    w.tick().unwrap();
    let a = w.actors().next().unwrap();
    w.set_observer_id(a, 900).unwrap();
    let hp = if raw == 2478 { 22 } else { 2 };
    let mut n = Npc::new(raw, tile(), 1, Restrict::NoMove, BlockWalk::Npc).unwrap();
    n.mode = Mode::None;
    n.stats[3] = hp;
    n.base_stats[3] = hp;
    n.respawns = respawns;
    n.respawn_rate = 4;
    n.hero_points.credit(400, hp);
    n.aggressive_player = Some(a);
    let id = w.spawn_npc(n).unwrap();
    w.queue_hit(
        Target::Npc(id),
        Hit {
            source: Some(Target::Player(a)),
            style: Style::Typeless,
            damage: hp,
            display: None,
        },
        1,
    )
    .unwrap();
    w.drain_events();
    (w, id)
}
fn advance(w: &mut World, n: usize) -> Vec<WorldEvent> {
    let mut out = vec![];
    for _ in 0..n {
        w.tick_with_random(&mut NoDraw).unwrap();
        out.extend(w.drain_events());
    }
    out
}
fn traces(events: &[WorldEvent]) -> Vec<&DropTrace> {
    events
        .iter()
        .filter_map(|e| {
            if let WorldEvent::NpcDrop(t) = e {
                Some(t)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn source_spiders_complete_empty_loot_without_bones_rng_extra_xp_or_replay() {
    for raw in [2478, 3019, 4561] {
        let (mut w, id) = setup(raw, false);
        let a = w.actors().next().unwrap();
        let before = w.data(a).unwrap().stats.clone();
        let early = advance(&mut w, 3);
        assert!(traces(&early).is_empty());
        assert_eq!(w.objects.count(), 0);
        assert_eq!(
            early
                .iter()
                .filter(
                    |e| matches!(e,WorldEvent::NpcCombatAnimation{actor,sequence:6251}if *actor==id)
                )
                .count(),
            1
        );
        assert_eq!(
            early
                .iter()
                .filter(|e| matches!(e,WorldEvent::CombatSound{actor,sound:3608}if *actor==a))
                .count(),
            1
        );
        let events = advance(&mut w, 1);
        let t = traces(&events);
        assert_eq!(t.len(), 1);
        let t = t[0];
        assert_eq!(
            (t.policy, t.npc, t.owner, t.draw, t.result),
            (DropPolicy::Spider, id, Some(a), None, DropResult::Committed)
        );
        assert!(t.products.is_empty() && t.handles.is_empty());
        assert!(t.json().contains("selected_spider"));
        assert_eq!(w.objects.count(), 0);
        assert_eq!(w.data(a).unwrap().stats, before);
        assert!(w.npc(id).is_none());
        assert!(traces(&advance(&mut w, 6)).is_empty());
    }
}
#[test]
fn no_loot_profiles_distinguish_owner_refusals_conflicts_and_irrelevant_bone_catalog() {
    for raw in [2478, 3019, 4561] {
        for case in 0..6 {
            let (mut w, id) = setup(raw, false);
            match case {
                0 => w.npc_mut(id).unwrap().hero_points.credit(999, 100000),
                1 => {
                    w.npc_no_death_drops.insert(raw);
                }
                2 => {
                    w.npc_guaranteed_extras.insert(
                        raw,
                        GuaranteedExtras {
                            products: vec![(948, 1)],
                            policy: DropPolicy::Bear,
                        },
                    );
                }
                3 => {
                    w.npc_death.as_mut().unwrap().chickens.insert(
                        raw,
                        Chicken {
                            raw: 2138,
                            feather: 314,
                        },
                    );
                }
                4 => {
                    w.object_types.dummy.insert(526);
                }
                _ => w.npc_mut(id).unwrap().hero_points.clear(),
            };
            let events = advance(&mut w, 4);
            let t = traces(&events);
            assert_eq!(t.len(), 1);
            assert_eq!(
                t[0].result,
                if matches!(case, 2 | 3) {
                    DropResult::Rejected
                } else if case == 4 {
                    DropResult::Committed
                } else {
                    DropResult::NoEligibleOwner
                }
            );
            assert!(t[0].products.is_empty() && t[0].handles.is_empty());
            assert_eq!(t[0].draw, None);
            assert_eq!(w.objects.count(), 0);
            assert!(traces(&advance(&mut w, 5)).is_empty());
        }
    }
}
#[test]
fn empty_death_batch_preserves_existing_ground_identity_and_original_deadlines() {
    let (mut w, _) = setup(3019, false);
    let created = w.clock.0;
    let object = gielinor_world::ground::Object {
        coords: tile(),
        item: 526,
        count: 1,
        scope: gielinor_world::ground::Scope::Private,
        created,
        receiver: Some(900),
        owner: Some(900),
    };
    let added = w
        .objects
        .add_batch(&[object], &w.object_types, w.clock, 200, 100)
        .unwrap();
    let h = added[0][0].clone();
    let events = advance(&mut w, 4);
    assert_eq!(traces(&events)[0].result, DropResult::Committed);
    assert_eq!(w.objects.count(), 1);
    assert_eq!(w.objects.at(tile()).next().unwrap().id(), h.id());
    assert_eq!(h.snapshot().count, 1);
    while w.clock.0 < created + 99 {
        advance(&mut w, 1);
    }
    assert!(!h.snapshot().visible_to(Some(901)));
    advance(&mut w, 1);
    assert!(h.snapshot().visible_to(Some(901)));
    while w.clock.0 < created + 199 {
        advance(&mut w, 1);
    }
    assert_eq!(w.objects.count(), 1);
    advance(&mut w, 1);
    assert_eq!(w.objects.count(), 0);
}
#[test]
fn new_spider_life_keeps_empty_profile_and_stale_generation_cannot_reward() {
    let (mut w, old) = setup(3019, true);
    let events = advance(&mut w, 8);
    assert_eq!(traces(&events).len(), 1);
    let fresh = events
        .iter()
        .find_map(|e| {
            if let WorldEvent::SynchronizeNpcs { npcs, .. } = e {
                npcs.iter()
                    .find(|n| n.actor.slot == old.slot && n.actor != old)
                    .map(|n| n.actor)
            } else {
                None
            }
        })
        .expect("Missing fresh spider generation");
    let a = w.actors().next().unwrap();
    let hit = Hit {
        source: Some(Target::Player(a)),
        style: Style::Typeless,
        damage: 2,
        display: None,
    };
    assert!(w.queue_hit(Target::Npc(old), hit, 1).is_err());
    w.npc_mut(fresh).unwrap().hero_points.credit(400, 2);
    w.queue_hit(Target::Npc(fresh), hit, 1).unwrap();
    let events = advance(&mut w, 4);
    let t = traces(&events);
    assert_eq!(t.len(), 1);
    assert_eq!((t[0].npc, t[0].result), (fresh, DropResult::Committed));
    assert!(t[0].products.is_empty() && t[0].handles.is_empty());
    assert_eq!(w.objects.count(), 0);
    assert!(traces(&advance(&mut w, 3)).is_empty());
}
#[test]
fn selected_empty_spider_profile_preserves_other_npc_generic_bone_fallback() {
    let (mut w, _) = setup(3019, false);
    advance(&mut w, 4);
    assert_eq!(w.objects.count(), 0);
    let other = CoordGrid::new(3201, 3200, 0).unwrap();
    w.npc_death.as_mut().unwrap().definitions.insert(
        1,
        Definition {
            animation: 836,
            animation_ticks: 2,
            sound: None,
        },
    );
    let a = w.actors().next().unwrap();
    let mut n = Npc::new(1, other, 1, Restrict::NoMove, BlockWalk::Npc).unwrap();
    n.mode = Mode::None;
    n.stats[3] = 1;
    n.base_stats[3] = 1;
    n.respawns = false;
    n.hero_points.credit(400, 1);
    let id = w.spawn_npc(n).unwrap();
    w.queue_hit(
        Target::Npc(id),
        Hit {
            source: Some(Target::Player(a)),
            style: Style::Typeless,
            damage: 1,
            display: None,
        },
        1,
    )
    .unwrap();
    advance(&mut w, 3);
    assert_eq!(w.objects.at(tile()).count(), 0);
    let stock = w
        .objects
        .at(other)
        .map(|h| {
            let v = h.snapshot();
            assert_eq!((v.owner, v.receiver), (Some(900), Some(900)));
            (v.item, v.count)
        })
        .collect::<Vec<_>>();
    assert_eq!(stock, [(526, 1)]);
}
