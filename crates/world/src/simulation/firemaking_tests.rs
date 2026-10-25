//! World tests for ordinary firemaking (issue #40 acceptance criteria).
use super::*;
use crate::{
    firemaking::ATTEMPT_TICKS,
    ground::Scope,
    held_use::{Handler, Request, Scope as UseScope, Type},
    inventory::{Insert, Item},
    loc_collision::LocCollision,
    locations::Location,
};
const TINDERBOX: i32 = 590;
const LOGS: i32 = 1511;
const YEW: i32 = 1515;
const TEAK: i32 = 6333;
const FIRE: u32 = 26185;
const ASHES: u16 = 592;
/// Draws from a fixed script, then repeats its last value.
struct Draws(Vec<u32>);
impl RandomSource for Draws {
    fn below(&mut self, bound: u32) -> u32 {
        let value = if self.0.len() > 1 {
            self.0.remove(0)
        } else {
            self.0[0]
        };
        value.min(bound - 1)
    }
}
fn setup(level: i8, items: &[i32]) -> (World, ActorId) {
    let mut w = World::default();
    for x in (3184..3224).step_by(8) {
        for z in (3184..3224).step_by(8) {
            w.collision.allocate(x, z, 0);
        }
    }
    let config = crate::firemaking::tests::config();
    for &log in config.logs.keys() {
        w.held_use_handlers
            .register(UseScope::Types, TINDERBOX, log, Handler::Firemaking { log })
            .unwrap();
    }
    for id in config.logs.keys().copied().chain([TINDERBOX]) {
        w.held_use_types.insert(id, Type { id, content: -1 });
    }
    w.firemaking = Some(config);
    w.location_types.insert(
        FIRE,
        LocCollision {
            width: 1,
            length: 1,
            block_walk: 1,
            block_range: false,
            break_route_finding: false,
        },
    );
    w.pickup_effects = Some(crate::ground_transfer::PickupEffects {
        sound: 1,
        animation: 1,
    });
    let mut stats = crate::stats::Stats::default();
    stats.set_base(11, level);
    stats.set_current(11, level);
    let mut p = PlayerData::new(
        1,
        "Firemaker".into(),
        CoordGrid::new(3200, 3200, 0).unwrap(),
        stats,
    );
    p.xp_rate_hundreds = 500;
    for &id in items {
        p.inventory
            .insert(&w.object_types, Insert::new(id, 1))
            .unwrap();
    }
    w.queue_login(1, p).unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    w.set_observer_id(actor, 1).unwrap();
    w.drain_events();
    (w, actor)
}
fn use_items(w: &mut World, actor: ActorId, selected: (i32, i32), target: (i32, i32)) {
    let r = Request {
        selected_slot: selected.0,
        selected_id: selected.1,
        target_slot: target.0,
        target_id: target.1,
    };
    assert!(w.prepare_held_use(actor, r).unwrap().is_some());
    w.finish_held_use(actor, r).unwrap();
}
fn messages(w: &mut World) -> Vec<String> {
    w.drain_events()
        .into_iter()
        .filter_map(|e| match e {
            WorldEvent::GameMessage { text, .. } => Some(text),
            WorldEvent::EngineMessage { message, .. } => Some(message.into()),
            _ => None,
        })
        .collect()
}
fn ground(w: &World, coords: CoordGrid) -> Vec<u16> {
    w.objects.at(coords).map(|h| h.snapshot().item).collect()
}
fn drop_logs(w: &mut World, coords: CoordGrid) {
    w.objects
        .add(
            crate::ground::Object {
                coords,
                item: LOGS as u16,
                count: 1,
                scope: Scope::Temporary,
                created: w.clock.0,
                receiver: None,
                owner: None,
            },
            &w.object_types,
            w.clock,
            1000,
            i32::MAX,
        )
        .unwrap();
}
fn light(coords: CoordGrid) -> Action {
    Action::LightObject {
        coords,
        id: LOGS as u16,
        control: false,
    }
}
#[test]
fn tinderbox_and_logs_either_way_drop_light_award_x5_and_step_west() {
    for reverse in [false, true] {
        let (mut w, actor) = setup(1, &[TINDERBOX, LOGS]);
        let origin = w.data(actor).unwrap().coords;
        let mut random = Draws(vec![0]);
        if reverse {
            use_items(&mut w, actor, (1, LOGS), (0, TINDERBOX));
        } else {
            use_items(&mut w, actor, (0, TINDERBOX), (1, LOGS));
        }
        let start = w.clock.0;
        assert!(w.data(actor).unwrap().inventory.get(1).is_none());
        assert_eq!(ground(&w, origin), [LOGS as u16]);
        assert_eq!(messages(&mut w), [crate::firemaking::ATTEMPT]);
        // weakQueue(4): the attempt runs in the fourth cycle after the start.
        while w.clock.0 < start + ATTEMPT_TICKS - 1 {
            w.tick_with_random(&mut random).unwrap();
            assert!(w.locations.find_type(origin, FIRE).is_none());
        }
        w.tick_with_random(&mut random).unwrap();
        assert_eq!(w.clock.0, start + ATTEMPT_TICKS);
        let fire = w.locations.find_type(origin, FIRE).expect("fire lit");
        assert_eq!((fire.shape, fire.angle), (10, 0));
        assert!(ground(&w, origin).is_empty());
        // Firemaking rev 15367339: logs 40 XP, x5 on this server.
        assert_eq!(w.data(actor).unwrap().stats.fine_xp(11), 2000);
        assert!(messages(&mut w).contains(&crate::firemaking::CATCHES.to_string()));
        w.tick_with_random(&mut random).unwrap();
        assert_eq!(
            w.data(actor).unwrap().coords,
            origin.translate(-1, 0, 0).unwrap()
        );
        assert_eq!(
            w.data(actor).unwrap().inventory.get(0).unwrap().id,
            TINDERBOX
        );
    }
}
#[test]
fn wiki_levels_and_fine_xp_for_bound_logs() {
    for (log, level, fine) in [(LOGS, 1, 2000), (1521, 15, 3000), (YEW, 60, 10125)] {
        let (mut w, actor) = setup(level, &[TINDERBOX, log]);
        let mut random = Draws(vec![0]);
        use_items(&mut w, actor, (0, TINDERBOX), (1, log));
        for _ in 0..6 {
            w.tick_with_random(&mut random).unwrap();
        }
        assert_eq!(w.data(actor).unwrap().stats.fine_xp(11), fine, "log {log}");
    }
}
#[test]
fn level_tile_and_membership_refusals_keep_the_inventory() {
    let cases: [(i8, i32, bool, bool, &str); 4] = [
        (
            59,
            YEW,
            false,
            false,
            "You need a Firemaking level of 60 to burn yew logs.",
        ),
        (60, YEW, true, false, crate::firemaking::BLOCKED),
        (99, TEAK, false, false, "Nothing interesting happens."),
        (99, TEAK, false, true, crate::firemaking::ATTEMPT),
    ];
    for (level, log, blocked, member, expected) in cases {
        let (mut w, actor) = setup(level, &[TINDERBOX, log]);
        w.players.get_mut(actor.slot as usize).unwrap().member = member;
        let origin = w.data(actor).unwrap().coords;
        if blocked {
            w.locations.load(Location::new(origin, 7, 10, 0).unwrap());
        }
        let before = w.data(actor).unwrap().inventory.slots().to_vec();
        use_items(&mut w, actor, (0, TINDERBOX), (1, log));
        assert_eq!(messages(&mut w), [expected], "log {log} level {level}");
        if expected != crate::firemaking::ATTEMPT {
            assert_eq!(w.data(actor).unwrap().inventory.slots(), before.as_slice());
            assert!(ground(&w, origin).is_empty());
            for _ in 0..6 {
                w.tick().unwrap();
            }
            assert!(w.locations.find_type(origin, FIRE).is_none());
            assert_eq!(w.data(actor).unwrap().stats.fine_xp(11), 0);
        }
    }
}
#[test]
fn failed_roll_retries_every_four_ticks_and_walking_leaves_the_logs() {
    let (mut w, actor) = setup(1, &[TINDERBOX, LOGS]);
    let origin = w.data(actor).unwrap().coords;
    let mut random = Draws(vec![255, 0]);
    use_items(&mut w, actor, (0, TINDERBOX), (1, LOGS));
    let start = w.clock.0;
    while w.locations.find_type(origin, FIRE).is_none() {
        w.tick_with_random(&mut random).unwrap();
        assert!(w.clock.0 <= start + 2 * ATTEMPT_TICKS);
    }
    assert_eq!(w.clock.0, start + 2 * ATTEMPT_TICKS);

    let (mut w, actor) = setup(1, &[TINDERBOX, LOGS]);
    use_items(&mut w, actor, (0, TINDERBOX), (1, LOGS));
    w.submit(
        actor,
        Action::Walk {
            destination: origin.translate(0, 3, 0).unwrap(),
            run: false,
        },
    )
    .unwrap();
    let mut random = Draws(vec![0]);
    for _ in 0..10 {
        w.tick_with_random(&mut random).unwrap();
    }
    assert!(w.locations.find_type(origin, FIRE).is_none());
    assert_eq!(ground(&w, origin), [LOGS as u16]);
    assert_eq!(w.data(actor).unwrap().stats.fine_xp(11), 0);
}
#[test]
fn light_ground_logs_then_fire_burns_out_to_ashes() {
    let (mut w, actor) = setup(1, &[TINDERBOX]);
    let tile = CoordGrid::new(3202, 3200, 0).unwrap();
    drop_logs(&mut w, tile);
    let mut random = Draws(vec![0]);
    w.submit(actor, light(tile)).unwrap();
    let mut lit = None;
    for _ in 0..12 {
        w.tick_with_random(&mut random).unwrap();
        if w.locations.find_type(tile, FIRE).is_some() {
            lit = Some(w.clock.0);
            break;
        }
    }
    let lit = lit.expect("ground logs lit");
    assert!(ground(&w, tile).is_empty());
    assert_eq!(w.data(actor).unwrap().stats.fine_xp(11), 2000);
    // The lowest draw gives the shortest Wiki lifetime, 100 ticks (60 s). The fire
    // was placed during PlayerMain of the cycle before `lit`; the location timer
    // removes it in WorldPostTick once the clock reaches placement + 100.
    let placed = lit - 1;
    while w.locations.find_type(tile, FIRE).is_some() {
        w.tick_with_random(&mut random).unwrap();
        assert!(w.clock.0 <= placed + 100);
    }
    assert_eq!(w.clock.0, placed + 100);
    assert_eq!(ground(&w, tile), [ASHES]);
}
#[test]
fn missing_tinderbox_on_ground_logs_is_refused() {
    let (mut w, actor) = setup(1, &[]);
    let tile = w.data(actor).unwrap().coords;
    drop_logs(&mut w, tile);
    w.submit(actor, light(tile)).unwrap();
    let mut seen = vec![];
    for _ in 0..8 {
        w.tick().unwrap();
        seen.extend(messages(&mut w));
    }
    assert_eq!(seen, [crate::firemaking::NO_SOURCE]);
    assert_eq!(ground(&w, tile), [LOGS as u16]);
}
#[test]
fn food_cooks_on_a_lit_fire() {
    let (mut w, actor) = setup(1, &[TINDERBOX, LOGS, 317]);
    w.data_mut(actor).unwrap().stats.set_current(7, 34);
    w.held_use_types.insert(317, Type { id: 317, content: -1 });
    w.cooking = Some(crate::cooking::Config {
        gauntlets: None,
        no_burn_cape: None,
        member_inputs: Default::default(),
        surfaces: Default::default(),
        fire: FIRE,
        animation: 897,
        skill: 7,
        max_level: 99,
        tutorial: None,
        recipes: [(
            317,
            crate::cooking::Recipe {
                fine_xp_override: None,
                raw: 317,
                cooked: 315,
                burnt: 7954,
                level: 1,
                stop_burn: Some(34),
                low: 128,
                high: 512,
                xp: 30,
                ticks: 4,
            },
        )]
        .into(),
    });
    let origin = w.data(actor).unwrap().coords;
    let mut random = Draws(vec![0]);
    use_items(&mut w, actor, (0, TINDERBOX), (1, LOGS));
    for _ in 0..8 {
        w.tick_with_random(&mut random).unwrap();
    }
    assert!(w.locations.find_type(origin, FIRE).is_some());
    assert_ne!(w.data(actor).unwrap().coords, origin);
    w.submit(
        actor,
        Action::UseLocation {
            coords: origin,
            id: FIRE,
            slot: 2,
            object: 317,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut random).unwrap();
    }
    assert_eq!(
        w.data(actor).unwrap().inventory.get(2),
        Some(Item {
            id: 315,
            count: 1,
            vars: 0
        })
    );
}
