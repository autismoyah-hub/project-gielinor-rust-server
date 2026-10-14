use gielinor_world::{
    hits::{Hit, Style, Target},
    npc::{BlockWalk, Npc, Restrict},
    npc_death::{Config, Definition},
    simulation::{PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
use std::collections::BTreeMap;
fn coords() -> CoordGrid {
    CoordGrid::new(3200, 3200, 0).unwrap()
}
fn setup(respawns: bool) -> (World, gielinor_world::simulation::NpcId) {
    let mut world = World::default();
    world.collision.allocate(3200, 3200, 0);
    world.npc_death = Some(Config {
        bones: 526,
        definitions: BTreeMap::from([(
            1,
            Definition {
                animation: 2304,
                animation_ticks: 3,
                sound: Some(99),
            },
        )]),
    });
    let mut data = PlayerData::new(1, "Hero".into(), coords(), Stats::default());
    data.character_id = 400;
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.set_observer_id(actor, 900).unwrap();
    let mut npc = Npc::new(1, coords(), 1, Restrict::NoMove, BlockWalk::Npc).unwrap();
    npc.stats[3] = 10;
    npc.base_stats[3] = 10;
    npc.respawns = respawns;
    npc.respawn_rate = 4;
    npc.hero_points.credit(400, 10);
    npc.aggressive_player = Some(actor);
    npc.regen_rate = 1;
    let id = world.spawn_npc(npc).unwrap();
    world
        .queue_hit(
            Target::Npc(id),
            Hit {
                source: Some(Target::Player(actor)),
                style: Style::Melee,
                damage: 10,
                display: None,
            },
            1,
        )
        .unwrap();
    world.drain_events();
    (world, id)
}
#[test]
fn death_animation_private_bones_and_fresh_respawn_identity() {
    let (mut w, id) = setup(true);
    w.tick().unwrap();
    let events = w.drain_events();
    assert!(events
        .iter()
        .any(|e| matches!(e, WorldEvent::NpcCombatAnimation { sequence: 2304, .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, WorldEvent::CombatSound { sound: 99, .. })));
    assert_eq!(w.npc(id).unwrap().stats[3], 0);
    assert!(w.npc(id).unwrap().delayed);
    assert_eq!(w.objects.at(coords()).count(), 0);
    for _ in 0..2 {
        w.tick().unwrap();
        assert!(w.npc(id).unwrap().visible);
        assert_eq!(w.npc(id).unwrap().stats[3], 0);
    }
    w.tick().unwrap();
    assert!(!w.npc(id).unwrap().visible);
    let bones = w.objects.at(coords()).next().unwrap().snapshot();
    assert_eq!(
        (bones.item, bones.count, bones.receiver, bones.owner),
        (526, 1, Some(900), Some(900))
    );
    assert!(!bones.visible_to(Some(901)));
    for _ in 0..3 {
        w.tick().unwrap();
        assert!(!w.npc(id).unwrap().visible);
    }
    w.drain_events();
    w.tick().unwrap();
    assert!(
        w.npc(id).is_none(),
        "old generation must be invalid after respawn"
    );
    let fresh = w
        .drain_events()
        .into_iter()
        .find_map(|e| match e {
            WorldEvent::SynchronizeNpcs { npcs, .. } => npcs
                .into_iter()
                .find(|n| n.actor.slot == id.slot)
                .map(|n| n.actor),
            _ => None,
        })
        .unwrap();
    let npc = w.npc(fresh).unwrap();
    assert_eq!(npc.stats[3], 10);
    assert_eq!(npc.movement.coords, coords());
    assert!(npc.hero_points.entries().is_empty());
    assert!(npc.aggressive_player.is_none());
    assert_eq!(w.objects.at(coords()).count(), 1, "drops emitted only once");
}
#[test]
fn temporary_npc_is_removed_and_offline_top_hero_does_not_fall_back() {
    let (mut w, id) = setup(false);
    w.npc_mut(id).unwrap().hero_points.credit(999, 100);
    for _ in 0..4 {
        w.tick().unwrap();
    }
    assert!(w.npc(id).is_none());
    assert_eq!(w.objects.at(coords()).count(), 0);
}
#[test]
fn arrival_delay_preserves_previous_movement_cycle() {
    let (mut w, id) = setup(false);
    w.npc_mut(id).unwrap().movement.last_movement = w.clock.0 - 1;
    w.tick().unwrap();
    assert!(!w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::NpcCombatAnimation { sequence: 2304, .. })));
    w.tick().unwrap();
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::NpcCombatAnimation { sequence: 2304, .. })));
}

#[test]
fn runtime_binding_survives_cache_configuration_and_reapplies_on_respawn() {
    let (mut w, id) = setup(true);
    w.configure_npc_bindings(BTreeMap::from([(
        1,
        gielinor_world::npc_bind::Binding {
            stats: [7; 6],
            xp_multiplier: 1234,
            spawn_animation: Some(300),
            respawn_ticks: 2,
        },
    )]));
    w.configure_npc_combat(BTreeMap::from([(
        1,
        gielinor_world::npc_combat::Definition {
            xp_multiplier: 1000,
            defence: [0; 3],
            attributes: 0,
            attack_rate: 4,
            retreat: 0,
            aggression_range: 8,
        },
    )]));
    assert_eq!(w.npc(id).unwrap().combat_xp_multiplier, 1234);
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::NpcCombatAnimation { sequence: 300, .. })));
    for _ in 0..5 {
        w.tick().unwrap();
    }
    w.drain_events();
    w.tick().unwrap();
    assert!(w.npc(id).is_none());
    let events = w.drain_events();
    let fresh = events
        .iter()
        .find_map(|e| match e {
            WorldEvent::NpcCombatAnimation {
                actor,
                sequence: 300,
            } => Some(*actor),
            _ => None,
        })
        .unwrap();
    assert_eq!(w.npc(fresh).unwrap().stats, [7; 6]);
    assert_eq!(w.npc(fresh).unwrap().combat_xp_multiplier, 1234);
}

#[test]
fn negative_bound_respawn_removes_permanent_spawn() {
    let (mut w, id) = setup(true);
    w.npc_mut(id).unwrap().respawn_rate = -1;
    for _ in 0..4 {
        w.tick().unwrap();
    }
    assert!(w.npc(id).is_none());
}

#[test]
fn invalid_animation_duration_removes_npc_without_drops() {
    let (mut w, id) = setup(true);
    w.npc_death
        .as_mut()
        .unwrap()
        .definitions
        .get_mut(&1)
        .unwrap()
        .animation_ticks = 0;
    w.tick().unwrap();
    assert!(w.npc(id).is_none());
    assert_eq!(w.objects.at(coords()).count(), 0);
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::NpcCombatError { .. })));
}

fn tutorial_config(rat: u16, chicken: u16) -> gielinor_world::tutorial_combat::Config {
    use gielinor_world::variables::{Transmit, Varp};
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    gielinor_world::tutorial_combat::Config {
        progression: gielinor_world::tutorial_progression::Config {
            step: varp(5003),
            completed: varp(5004),
        },
        rat,
        chicken,
    }
}
#[test]
fn tutorial_rat_progress_runs_once_at_death_start_then_normal_drop_and_respawn_continue() {
    for (start, next) in [(35, 36), (36, 37)] {
        let (mut w, id) = setup(true);
        let actor = w.actors().next().unwrap();
        w.tutorial_combat = Some(tutorial_config(1, 2));
        w.data_mut(actor).unwrap().variables =
            gielinor_world::variables::Variables::restore([(5003, start)].into());
        // The custom death queue runs before deathWithDrops' arrival suspension.
        w.npc_mut(id).unwrap().movement.last_movement = w.clock.0 - 1;
        assert_eq!(w.data(actor).unwrap().variables.get(5003), start);
        w.tick().unwrap();
        let mut events = w.drain_events();
        assert_eq!(w.data(actor).unwrap().variables.get(5003), next);
        assert!(!events
            .iter()
            .any(|event| matches!(event, WorldEvent::NpcCombatAnimation { .. })));
        for _ in 0..4 {
            w.tick().unwrap();
            events.extend(w.drain_events());
        }
        assert_eq!(w.data(actor).unwrap().variables.get(5003), next);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    WorldEvent::LocationMessage {
                        message: "You have defeated the giant rat!",
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(w.objects.at(coords()).count(), 1);
        assert!(!w.npc(id).unwrap().visible);
        for _ in 0..4 {
            w.tick().unwrap();
        }
        assert!(
            w.npc(id).is_none(),
            "Normal respawn must replace the old generation"
        );
    }
}
#[test]
fn tutorial_death_does_not_credit_runner_up_when_winning_character_is_offline() {
    let (mut w, id) = setup(false);
    let actor = w.actors().next().unwrap();
    w.tutorial_combat = Some(tutorial_config(1, 2));
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, 35)].into());
    w.npc_mut(id).unwrap().hero_points.credit(999, 100);
    for _ in 0..4 {
        w.tick().unwrap();
    }
    assert_eq!(w.data(actor).unwrap().variables.get(5003), 35);
    assert!(!w
        .drain_events()
        .iter()
        .any(|event| matches!(event, WorldEvent::TutorialCombatProgress { .. })));
    assert!(w.npc(id).is_none());
}
#[test]
fn tutorial_chicken_death_emits_completion_once_and_preserves_ordinary_death_sequence() {
    use gielinor_world::{tutorial_progression::Effect as P, tutorial_steps::Step};
    let (mut w, id) = setup(false);
    let actor = w.actors().next().unwrap();
    w.tutorial_combat = Some(tutorial_config(2, 1));
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, Step::MagicRunes as i32)].into());
    let mut events = vec![];
    for _ in 0..4 {
        w.tick().unwrap();
        events.extend(w.drain_events());
    }
    assert_eq!(
        w.data(actor).unwrap().variables.get(5003),
        Step::LeaveTalk as i32
    );
    assert_eq!(w.data(actor).unwrap().variables.get(5004), 0);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event,
                WorldEvent::LocationMessage {
                    message: "Congratulations, you've completed a quest: Learning the Ropes",
                    ..
                }
            ))
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter_map(
                |event| if let WorldEvent::TutorialCombatProgress { effects, .. } = event {
                    Some(effects)
                } else {
                    None
                }
            )
            .flatten()
            .filter(|effect| matches!(
                effect,
                P::Focus {
                    step: Step::LeaveTalk,
                    npc: None
                }
            ))
            .count(),
        2
    );
    assert!(w.npc(id).is_none());
    assert_eq!(w.objects.at(coords()).count(), 1);
}
