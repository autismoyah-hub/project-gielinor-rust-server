use gielinor_world::{
    attack_profile::Attack,
    npc::{BlockWalk, Npc, Restrict},
    npc_combat::Definition,
    simulation::{ActionError, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
#[test]
fn authoritative_awards_cap_damage_track_spawn_multiplier_and_reject_stale_target() {
    let mut world = World::default();
    let coords = CoordGrid::new(3200, 3200, 0).unwrap();
    world.collision.allocate(3200, 3200, 0);
    world
        .queue_login(
            1,
            PlayerData::new(1, "Fighter".into(), coords, Stats::default()),
        )
        .unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    let mut npc = Npc::new(1, coords, 1, Restrict::NoMove, BlockWalk::None).unwrap();
    npc.stats[3] = 3;
    let target = world.spawn_npc(npc.clone()).unwrap();
    world.configure_npc_combat(
        [(
            1,
            Definition {
                xp_multiplier: 1250,
                defence: [0; 3],
                attributes: 0,
                attack_rate: 4,
                retreat: 0,
                aggression_range: 8,
            },
        )]
        .into(),
    );
    assert_eq!(world.npc(target).unwrap().combat_xp_multiplier, 1250);
    let later = world.spawn_npc(npc).unwrap();
    assert_eq!(world.npc(later).unwrap().combat_xp_multiplier, 1250);
    let attack = Attack::Melee {
        kind: Some(3),
        style: Some(2),
        stance: 0,
    };
    world
        .give_npc_combat_xp(actor, target, attack, 99, 1.0)
        .unwrap();
    assert_eq!(world.data(actor).unwrap().stats.fine_xp(0), 150);
    assert_eq!(world.data(actor).unwrap().stats.fine_xp(3), 49);
    assert_eq!(
        world.npc(target).unwrap().stats[3],
        3,
        "award must not apply queued damage"
    );
    assert!(world
        .drain_events()
        .iter()
        .any(|e| matches!(e,WorldEvent::CombatXp{awards,..} if awards.len()==2)));
    world.despawn_npc(target).unwrap();
    assert_eq!(
        world.give_npc_combat_xp(actor, target, attack, 99, 1.0),
        Err(ActionError::StaleActor)
    );
    assert_eq!(world.data(actor).unwrap().stats.fine_xp(0), 150);
}
#[test]
fn attack_delay_expires_on_exact_cycle_and_zero_awards_remain_observable() {
    let mut world = World::default();
    let coords = CoordGrid::new(3200, 3200, 0).unwrap();
    world.collision.allocate(3200, 3200, 0);
    world
        .queue_login(
            1,
            PlayerData::new(1, "Fighter".into(), coords, Stats::default()),
        )
        .unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.set_next_attack_delay(actor, 2).unwrap();
    assert!(world.is_attack_delayed(actor).unwrap());
    world.tick().unwrap();
    assert!(world.is_attack_delayed(actor).unwrap());
    world.tick().unwrap();
    assert!(!world.is_attack_delayed(actor).unwrap());
    let target = world
        .spawn_npc(Npc::new(1, coords, 1, Restrict::NoMove, BlockWalk::None).unwrap())
        .unwrap();
    world
        .give_npc_combat_xp(actor, target, Attack::Spell { defensive: false }, 0, 1.0)
        .unwrap();
    assert!(world.drain_events().iter().any(|e|matches!(e,WorldEvent::CombatXp{awards,..} if awards.len()==2 && awards.iter().all(|(_,a)|a.added_xp==0))));
}
