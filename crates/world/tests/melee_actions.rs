use gielinor_world::{
    autocast, autocast_select,
    bank_actions::Binding,
    combat_stance, combat_wear,
    hit_display::Builder,
    magic, melee,
    npc::{BlockWalk, Mode, Npc, RandomSource, Restrict},
    npc_combat::Definition,
    npc_interaction::Handler,
    npc_options,
    simulation::{Action, ActorId, NpcId, PlayerData, World, WorldEvent},
    stats::Stats,
    variables::{Transmit, Varbit, Varp},
    CoordGrid,
};
use std::collections::BTreeMap;
struct Zero;
struct RangedDraws(Vec<u32>);
impl RandomSource for RangedDraws {
    fn below(&mut self, bound: u32) -> u32 {
        self.0.push(bound);
        if bound == 5 {
            4
        } else {
            0
        }
    }
}
fn setup_ranged() -> (World, ActorId, NpcId) {
    use gielinor_world::{inventory::Item, projectile, ranged, ranged_ammo, ranged_weapon};
    let (mut world, actor, target) = setup();
    let common = world.melee.as_ref().unwrap();
    world.ranged = Some(ranged::Config {
        equipment: Default::default(),
        npcs: [(
            1,
            ranged::NpcDefence {
                bonuses: [0; 3],
                magic_attack: 0,
            },
        )]
        .into(),
        prayers: [common.invocation; 5],
        stat: 4,
    });
    let item = ranged_weapon::Item {
        category: 1,
        weapon: ranged_ammo::Weapon {
            kind: ranged_ammo::Kind::Bow,
            required: None,
            level: 1,
            bone: 0,
        },
        charged: false,
        thrown: false,
        projectile: Some(1),
        travel: Some(1000),
        launch: None,
        animation: Some(426),
        sound: Some(101),
        recovery: None,
        metallic: false,
    };
    world.ranged_weapons = Some(ranged_weapon::Catalog {
        items: [(100, item), (101, item)].into(),
        categories: ranged_ammo::Categories {
            arrows: 1,
            dragon_arrow: 2,
            bolts: 3,
            javelins: 4,
        },
        javelin: 4,
        ballista_spot: 30,
        styles: [(
            1,
            projectile::Style {
                start_height: 40,
                end_height: 30,
                delay: 20,
                angle: 15,
                length_adjustment: 0,
                progress: 11,
                step_multiplier: 5,
            },
        )]
        .into(),
    });
    world.attack_profiles.weapons.insert(
        100,
        gielinor_world::attack_profile::Weapon {
            category: 0,
            range: 7,
            speed: 4,
        },
    );
    world.attack_profiles.types.insert(0, 5);
    world.attack_profiles.styles.insert(0, 5);
    let data = world.data_mut(actor).unwrap();
    data.stats.set_base(4, 50);
    data.stats.set_current(4, 50);
    for (slot, id, count) in [(3, 100, 1), (13, 101, 5)] {
        data.equipment
            .restore_slot(
                slot,
                Some(Item {
                    id,
                    count,
                    vars: 17,
                }),
            )
            .unwrap();
    }
    (world, actor, target)
}
#[test]
fn ranged_shot_consumes_before_roll_and_delays_hit_and_defend_animation() {
    let (mut w, actor, target) = setup_ranged();
    let attack = gielinor_world::attack_profile::Attack::Ranged {
        kind: 5,
        style: Some(5),
    };
    let mut rng = RangedDraws(vec![]);
    assert!(w
        .attack_npc_ranged(actor, target, attack, &mut rng)
        .unwrap());
    assert_eq!(&rng.0[..2], &[5, 10000]);
    assert_eq!(rng.0.len(), 3);
    let item = w.data(actor).unwrap().equipment.get(13).unwrap();
    assert_eq!((item.count, item.vars), (4, 17));
    assert!(w.data(actor).unwrap().stats.xp(4) > 0);
    assert_eq!(w.npc(target).unwrap().stats[3], 50);
    let events = w.drain_events();
    let position = |predicate: fn(&WorldEvent) -> bool| events.iter().position(predicate).unwrap();
    assert!(
        position(|e| matches!(e, WorldEvent::Projectile(_)))
            < position(|e| matches!(e, WorldEvent::AmmoConsumed { .. }))
    );
    assert!(
        position(|e| matches!(e, WorldEvent::AmmoConsumed { .. }))
            < position(|e| matches!(e, WorldEvent::CombatXp { .. }))
    );
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::NpcDelayedAnimation {
            sequence: 424,
            delay: 40,
            ..
        }
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::PlayerSpotAnimation {
            spot: 65535,
            height: 96,
            slot: 1,
            ..
        }
    )));
    w.tick_with_random(&mut rng).unwrap();
    assert_eq!(w.npc(target).unwrap().stats[3], 50);
    w.tick_with_random(&mut rng).unwrap();
    assert_eq!(w.npc(target).unwrap().stats[3], 49);
}
#[test]
fn ranged_ammo_failure_keeps_cooldown_without_drawing_or_launching() {
    let (mut w, actor, target) = setup_ranged();
    w.data_mut(actor)
        .unwrap()
        .equipment
        .restore_slot(13, None)
        .unwrap();
    let mut rng = RangedDraws(vec![]);
    assert!(!w
        .attack_npc_ranged(
            actor,
            target,
            gielinor_world::attack_profile::Attack::Ranged {
                kind: 5,
                style: Some(5)
            },
            &mut rng
        )
        .unwrap());
    assert!(w.is_attack_delayed(actor).unwrap());
    assert!(rng.0.is_empty());
    let events = w.drain_events();
    assert!(events.iter().any(|e| matches!(e,WorldEvent::CombatFeedback { events,.. }
        if events.contains(&autocast::Event::Message("There is no ammo left in your quiver.".into())))));
    assert!(!events.iter().any(|e| matches!(
        e,
        WorldEvent::Projectile(_) | WorldEvent::AmmoConsumed { .. }
    )));
}
#[test]
fn last_thrown_weapon_is_removed_before_formula_and_reports_last_one() {
    let (mut w, actor, target) = setup_ranged();
    let item = w
        .ranged_weapons
        .as_mut()
        .unwrap()
        .items
        .get_mut(&100)
        .unwrap();
    item.thrown = true;
    item.weapon.kind = gielinor_world::ranged_ammo::Kind::Other;
    let mut bonus = gielinor_world::bonuses::ItemBonuses::default();
    bonus.values[4] = 100;
    bonus.values[11] = 100;
    w.melee.as_mut().unwrap().bonuses.insert(100, bonus);
    let mut rng = RangedDraws(vec![]);
    w.attack_npc_ranged(
        actor,
        target,
        gielinor_world::attack_profile::Attack::Ranged {
            kind: 5,
            style: Some(5),
        },
        &mut rng,
    )
    .unwrap();
    assert!(w.data(actor).unwrap().equipment.get(3).is_none());
    assert_eq!(w.data(actor).unwrap().equipment.get(13).unwrap().count, 5);
    // Original max-hit computation reads worn bonuses after the last weapon is gone.
    assert_eq!(w.data(actor).unwrap().variables.get(17), 6);
    assert_eq!(rng.0, [5, 10000, 6]);
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e,WorldEvent::CombatFeedback { events,.. }
        if events.contains(&autocast::Event::Message("That was your last one!".into())))));
}
impl RandomSource for Zero {
    fn below(&mut self, max: u32) -> u32 {
        assert!(max > 0);
        0
    }
}
fn setup() -> (World, ActorId, NpcId) {
    let mut w = World::default();
    for x in (3192..3224).step_by(8) {
        for z in (3192..3224).step_by(8) {
            w.collision.allocate(x, z, 0);
        }
    }
    let v = |id| Varp {
        id,
        permanent: false,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let b = |id| Binding {
        base: v(id),
        bit: Varbit {
            base: id,
            lsb: 0,
            msb: 7,
        },
    };
    let empty = b(10);
    let saved = autocast::Saved {
        spell: b(11),
        defensive: b(12),
    };
    w.combat_wear = Some(combat_wear::Config {
        autocast: autocast::Config {
            enabled: b(1),
            spell: b(2),
            defensive: b(3),
            staff: saved,
            bladed: saved,
            weapons: Default::default(),
            spells: Default::default(),
            autocast_items: Default::default(),
            restricted: Default::default(),
        },
        runes: magic::Config {
            catalog: Default::default(),
            fake: [empty; 2],
            fountain: empty,
            blighted: empty,
            regular_pouches: Default::default(),
            divine_pouches: Default::default(),
            pouch_types: [empty; 4],
            pouch_counts: [(0, empty); 4],
            lms_pouch: 0,
            packs: Default::default(),
            names: Default::default(),
        },
        level: b(13),
        decimal: b(14),
        combat_stats: [0, 1, 2, 3, 4, 6, 5],
    });
    w.autocast_selection = Some(autocast_select::Config {
        book: b(4),
        magic_stat: 6,
        spells: Default::default(),
    });
    w.combat_stance = Some(combat_stance::Config {
        stance: v(0),
        melee: v(5),
        retaliate: v(6),
        saved: Default::default(),
        styles: BTreeMap::from([(0, 0x4322)]),
        weapons: Default::default(),
    });
    w.attack_profiles.types.insert(0, 3);
    w.attack_profiles.styles.insert(0, 2);
    w.melee = Some(melee::Config {
        npc_defend: BTreeMap::from([(1, 424)]),
        npc_defend_sounds: BTreeMap::from([(1, 155)]),
        npc_marks: BTreeMap::from([(
            1,
            Builder {
                target: 16,
                source: 16,
                public: Some(17),
                zero_lit: Some(12),
                zero_tint: Some(13),
                max_lit: Some(43),
                target_threshold: i32::MAX,
                source_threshold: i32::MAX,
                delay: 0,
            },
        )]),
        hit_threshold: b(15),
        equipment: Default::default(),
        bonuses: Default::default(),
        stance: v(0),
        attack_prayers: [empty; 5],
        strength_prayers: [empty; 5],
        souls: v(16),
        invocation: empty,
        max_hit: v(17),
        stats: [0, 2, 3],
        unarmed_fx: [(422, 2566); 4],
        weapon_fx: Default::default(),
    });
    w.configure_npc_combat(BTreeMap::from([(
        1,
        Definition {
            xp_multiplier: 1000,
            defence: [0; 3],
            attributes: 0,
            attack_rate: 4,
            retreat: 0,
            aggression_range: 8,
        },
    )]));
    w.npc_options.types.insert(
        1,
        npc_options::Definition {
            operations: [None, Some("Attack".into()), None, None, None],
            ..Default::default()
        },
    );
    w.npc_handlers.insert((1, 2), Handler::Combat);
    let coords = CoordGrid::new(3200, 3200, 0).unwrap();
    let mut stats = Stats::default();
    for skill in [0, 2, 3] {
        stats.set_base(skill, 20);
        stats.set_current(skill, 20);
    }
    w.queue_login(1, PlayerData::new(1, "Fighter".into(), coords, stats))
        .unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    let mut npc = Npc::new(
        1,
        CoordGrid::new(3204, 3200, 0).unwrap(),
        1,
        Restrict::NoMove,
        BlockWalk::Npc,
    )
    .unwrap();
    npc.mode = Mode::None;
    npc.stats[3] = 50;
    npc.base_stats[3] = 50;
    npc.regen_rate = 0;
    let target = w.spawn_npc(npc).unwrap();
    w.drain_events();
    (w, actor, target)
}
fn enable_retaliation(w: &mut World, target: NpcId) {
    let melee = w.melee.as_ref().unwrap();
    w.npc_melee = Some(gielinor_world::npc_melee::Config {
        player_mark: melee.npc_marks[&1],
        definitions: BTreeMap::from([(
            1,
            gielinor_world::npc_melee::Definition {
                retreat_range: 7,
                give_chase: true,
                attack_type: 3,
                attack_bonus: 0,
                strength_bonus: 0,
                animation: 100,
                sound: Some(101),
            },
        )]),
        prayers: [melee.hit_threshold; 11],
        torag: Default::default(),
        damned: -1,
        elysian: -1,
        justiciar: [-1; 3],
        bulwarks: Default::default(),
        passive_delay: melee.souls,
        defence_stat: 1,
        health_stat: 3,
        unarmed_defend: 424,
        item_defend: Default::default(),
    });
    w.npc_mut(target).unwrap().stats[0] = 30;
    w.npc_mut(target).unwrap().stats[2] = 30;
}
struct LandMaximum;
#[test]
fn npc_operation_dispatches_ranged_at_distance_and_repeats_on_weapon_cooldown() {
    let (mut w, actor, target) = setup_ranged();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 2,
            control: false,
        },
    )
    .unwrap();
    let mut rng = RangedDraws(vec![]);
    let mut launches = vec![];
    let mut impacts = vec![];
    for cycle in 0..11 {
        w.tick_with_random(&mut rng).unwrap();
        for event in w.drain_events() {
            match event {
                WorldEvent::Projectile(flight) => {
                    assert_eq!(flight.duration.server_delay, 2);
                    launches.push(cycle);
                }
                WorldEvent::Hit { impact, .. } => {
                    assert_eq!(impact.hit.style, gielinor_world::hits::Style::Ranged);
                    assert_eq!(impact.applied, 1);
                    impacts.push(cycle);
                }
                WorldEvent::ActionRejected { error, .. } => panic!("Ranged rejection: {error:?}"),
                _ => (),
            }
        }
    }
    assert_eq!(launches.len(), 3);
    assert!(launches.windows(2).all(|pair| pair[1] - pair[0] == 4));
    assert_eq!(
        impacts,
        launches
            .iter()
            .map(|cycle| cycle + 2)
            .filter(|cycle| *cycle < 11)
            .collect::<Vec<_>>()
    );
    assert_eq!(w.observe(actor).unwrap().coords.x(), 3200);
    assert_eq!(w.data(actor).unwrap().equipment.get(13).unwrap().count, 2);
}
#[test]
fn npc_stops_targeting_a_pending_logout_before_the_save_is_ready() {
    let (mut w, actor, target) = setup();
    enable_retaliation(&mut w, target);
    w.npc_mut(target).unwrap().attacking_player = Some(actor);
    w.delay_player(actor, 10).unwrap();
    w.request_logout(actor).unwrap();
    assert!(!w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::Save { .. })));
    w.tick_with_random(&mut LandMaximum).unwrap();
    assert!(w.npc(target).unwrap().attacking_player.is_none());
    assert!(w.data(actor).is_ok());
    assert!(!w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::NpcCombatAnimation { .. } | WorldEvent::Save { .. }
    )));
}
impl RandomSource for LandMaximum {
    fn below(&mut self, bound: u32) -> u32 {
        if bound == 10000 {
            0
        } else {
            bound - 1
        }
    }
}
#[test]
fn npc_retaliates_after_half_speed_flinch_and_repeats_at_attack_rate() {
    let (mut w, actor, target) = setup();
    enable_retaliation(&mut w, target);
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 2,
            control: false,
        },
    )
    .unwrap();
    let mut player_attacks = vec![];
    let mut npc_attacks = vec![];
    let mut hits = vec![];
    for cycle in 0..13 {
        w.tick_with_random(&mut LandMaximum).unwrap();
        for event in w.drain_events() {
            match event {
                WorldEvent::CombatAnimation { .. } => player_attacks.push(cycle),
                WorldEvent::NpcCombatAnimation { sequence: 100, .. } => npc_attacks.push(cycle),
                WorldEvent::Hit {
                    target: gielinor_world::hits::Target::Player(id),
                    impact,
                } => {
                    assert_eq!(id, actor);
                    assert_eq!(impact.applied, 4);
                    hits.push(cycle);
                }
                WorldEvent::NpcCombatError { error, .. }
                | WorldEvent::ActionRejected { error, .. } => panic!("Combat error {error:?}"),
                _ => (),
            }
        }
    }
    assert_eq!(npc_attacks[0] - player_attacks[0], 3);
    assert!(npc_attacks.windows(2).all(|a| a[1] - a[0] == 4));
    assert_eq!(
        hits, npc_attacks,
        "NPC hits process later in the same world tick"
    );
    assert_eq!(
        w.data(actor).unwrap().stats.current(3) as u8,
        20 - hits.len() as u8 * 4
    );
}
#[test]
fn npc_attack_routes_repeats_after_delay_and_applies_hit_one_tick_later() {
    let (mut w, actor, target) = setup();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 2,
            control: false,
        },
    )
    .unwrap();
    let mut attacks = vec![];
    let mut impacts = vec![];
    for cycle in 0..13 {
        w.tick_with_random(&mut Zero).unwrap();
        for event in w.drain_events() {
            match event {
                WorldEvent::CombatAnimation {
                    sequence, sound, ..
                } => {
                    assert_eq!((sequence, sound), (422, 2566));
                    attacks.push(cycle);
                }
                WorldEvent::Hit { impact, .. } => {
                    assert_eq!(impact.applied, 1);
                    assert!(impact.hit.display.is_some());
                    impacts.push(cycle);
                }
                WorldEvent::ActionRejected { error, .. } => {
                    panic!("Unexpected rejection {error:?}")
                }
                _ => (),
            }
        }
    }
    assert!(attacks.len() >= 3);
    assert!(attacks.windows(2).all(|a| a[1] - a[0] == 4));
    assert_eq!(
        impacts,
        attacks
            .iter()
            .map(|c| c + 1)
            .filter(|c| *c < 13)
            .collect::<Vec<_>>()
    );
    assert_eq!(w.observe(actor).unwrap().coords.x(), 3203);
    assert_eq!(
        w.data(actor).unwrap().stats.fine_xp(0),
        attacks.len() as i32 * 40
    );
    assert!(w.data(actor).unwrap().variables.get(17) > 0);
    w.submit(
        actor,
        Action::Walk {
            destination: CoordGrid::new(3200, 3200, 0).unwrap(),
            run: false,
        },
    )
    .unwrap();
    for _ in 0..6 {
        w.tick_with_random(&mut Zero).unwrap();
        assert!(!w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::CombatAnimation { .. })));
    }
}

#[test]
fn flat_armour_changes_queued_hit_after_xp_and_emits_defend_animation() {
    let (mut w, actor, target) = setup();
    w.npc_mut(target).unwrap().flat_armour = 1;
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 2,
            control: false,
        },
    )
    .unwrap();
    let mut defended = false;
    let mut impact = None;
    for _ in 0..7 {
        w.tick_with_random(&mut Zero).unwrap();
        for event in w.drain_events() {
            match event {
                WorldEvent::NpcCombatAnimation {
                    actor: id,
                    sequence,
                } => {
                    assert_eq!(id, target);
                    assert_eq!(sequence, 424);
                    defended = true;
                }
                WorldEvent::Hit { impact: value, .. } => {
                    impact = Some(value);
                }
                _ => (),
            }
        }
        if impact.is_some() {
            break;
        }
    }
    assert!(defended);
    let impact = impact.expect("queued impact");
    assert_eq!(impact.applied, 0);
    assert_eq!(impact.hit.display.unwrap().source, 12);
    assert_eq!(w.data(actor).unwrap().stats.fine_xp(0), 40);
    assert_eq!(w.npc(target).unwrap().flat_armour, 0);
    assert!(w.npc(target).unwrap().hero_points.entries().is_empty());
    assert_eq!(w.npc(target).unwrap().aggressive_player, Some(actor));
}

#[test]
fn npc_single_combat_owner_blocks_other_player_until_multiway() {
    let (mut w, first, target) = setup();
    w.npc_mut(target).unwrap().aggressive_player = Some(first);
    w.npc_mut(target).unwrap().last_combat = w.clock.0;
    w.queue_login(
        2,
        PlayerData::new(
            2,
            "Second".into(),
            CoordGrid::new(3203, 3200, 0).unwrap(),
            Stats::default(),
        ),
    )
    .unwrap();
    w.tick().unwrap();
    let second = w.actors().find(|a| *a != first).unwrap();
    w.drain_events();
    w.submit(
        second,
        Action::OperateNpc {
            target,
            operation: 2,
            control: false,
        },
    )
    .unwrap();
    w.tick_with_random(&mut Zero).unwrap();
    let events = w.drain_events();
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::NpcInteraction {
            event: gielinor_world::npc_interaction::Event::Message(
                "Someone else is fighting that."
            ),
            ..
        }
    )));
    assert!(!events
        .iter()
        .any(|e| matches!(e, WorldEvent::CombatAnimation { .. })));
    w.multiway_area = Some(7);
    w.areas
        .register_coord(w.observe(second).unwrap().coords, [7])
        .unwrap();
    w.tick_with_random(&mut Zero).unwrap();
    assert!(w
        .drain_events()
        .iter()
        .any(|e| matches!(e,WorldEvent::CombatAnimation{actor,..}if *actor==second)));
    assert_eq!(w.npc(target).unwrap().aggressive_player, Some(second));
}

#[test]
fn player_auto_retaliation_respects_toggle_and_flinch_delay() {
    for disabled in [false, true] {
        let (mut w, actor, target) = setup();
        enable_retaliation(&mut w, target);
        let config = w.combat_stance.as_ref().unwrap();
        let retaliation = config.retaliate;
        w.data_mut(actor)
            .unwrap()
            .variables
            .set(retaliation, i32::from(disabled), false);
        let npc = w.npc_mut(target).unwrap();
        npc.attacking_player = Some(actor);
        npc.movement.coords = CoordGrid::new(3201, 3200, 0).unwrap();
        let mut first_hit = None;
        let mut first_reply = None;
        for cycle in 0..7 {
            w.tick_with_random(&mut Zero).unwrap();
            for event in w.drain_events() {
                match event {
                    WorldEvent::Hit {
                        target: gielinor_world::hits::Target::Player(_),
                        ..
                    } => {
                        first_hit.get_or_insert(cycle);
                    }
                    WorldEvent::CombatAnimation { .. } => {
                        first_reply.get_or_insert(cycle);
                    }
                    _ => (),
                }
            }
        }
        assert_eq!(first_hit, Some(0));
        assert_eq!(first_reply, if disabled { None } else { Some(2) });
    }
}

#[test]
fn retreat_uses_spawn_bounds_and_attack_leash_cancels() {
    let (mut w, actor, target) = setup();
    enable_retaliation(&mut w, target);
    w.npc_combat.get_mut(&1).unwrap().retreat = 50;
    let clock = w.clock.0;
    let npc = w.npc_mut(target).unwrap();
    npc.restrict = Restrict::Normal;
    npc.aggressive_player = Some(actor);
    npc.retaliate_due.push_back(clock);
    let start = npc.movement.coords;
    w.tick_with_random(&mut Zero).unwrap();
    let npc = w.npc(target).unwrap();
    assert_eq!(npc.escape_player, Some(actor));
    assert_eq!(npc.movement.coords, start.translate(1, -1, 0).unwrap());
    assert!(!w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::NpcCombatAnimation { sequence: 100, .. })));
    for _ in 0..12 {
        w.tick_with_random(&mut Zero).unwrap();
    }
    let npc = w.npc(target).unwrap();
    assert!((npc.movement.coords.x() - npc.spawn.x()).abs() <= 7);
    assert!((npc.movement.coords.z() - npc.spawn.z()).abs() <= 7);
    let npc = w.npc_mut(target).unwrap();
    npc.escape_player = None;
    npc.attacking_player = Some(actor);
    npc.spawn = CoordGrid::new(3220, 3220, 0).unwrap();
    w.tick_with_random(&mut Zero).unwrap();
    assert!(w.npc(target).unwrap().attacking_player.is_none());
}
#[test]
fn spell_attempt_consumes_before_cast_xp_and_rejects_stale_or_logging_out_actor() {
    use gielinor_world::{
        inventory::Item,
        runes::Requirement,
        spell_cast::{Definition, ResultKindCast},
    };
    let (mut world, actor, _) = setup();
    world.object_types.stackable.insert(500);
    world
        .data_mut(actor)
        .unwrap()
        .inventory
        .restore_slot(
            0,
            Some(Item {
                id: 500,
                count: 2,
                vars: 0,
            }),
        )
        .unwrap();
    let spell = Definition {
        spell: autocast::Spell {
            item: 600,
            requirements: vec![Requirement {
                item: 500,
                count: 1,
                worn_slot: None,
            }],
        },
        book: Some(0),
        level: 1,
        cast_xp: 5.5,
    };
    let result = world.attempt_spell_cast(actor, &spell, 501, 1.0).unwrap();
    assert!(result.success());
    assert_eq!(
        world.data(actor).unwrap().inventory.get(0).unwrap().count,
        1
    );
    assert_eq!(world.data(actor).unwrap().stats.fine_xp(6), 55);
    assert!(world
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 6, .. })));
    let wrong_book = Definition {
        book: Some(1),
        ..spell
    };
    let result = world
        .attempt_spell_cast(actor, &wrong_book, 501, 1.0)
        .unwrap();
    assert_eq!(result.kind, ResultKindCast::IncorrectSpellbook);
    assert_eq!(result.message, None);
    assert_eq!(
        world.data(actor).unwrap().inventory.get(0).unwrap().count,
        1
    );
    assert_eq!(world.data(actor).unwrap().stats.fine_xp(6), 55);
    assert!(!world
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
    let stale = ActorId {
        generation: actor.generation + 1,
        ..actor
    };
    assert!(world
        .attempt_spell_cast(stale, &wrong_book, 501, 1.0)
        .is_err());
    world.request_logout(actor).unwrap();
    world.tick().unwrap();
    assert!(world
        .attempt_spell_cast(actor, &wrong_book, 501, 1.0)
        .is_err());
}
