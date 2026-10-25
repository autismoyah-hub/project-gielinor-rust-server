use super::*;
use crate::{
    inventory::Item,
    loc_collision::LocCollision,
    mining::{Config, Essence, Gems, Pickaxe, Rock, PRE_ROLL_TOTAL},
    stats::Stats,
};

const CLAY_ROCK: u32 = 11362;
const ESSENCE_ROCK: u32 = 34773;
const DEPLETED: u32 = 11391;
const CLAY: i32 = 434;
const RUNE: i32 = 1436;
const PURE: i32 = 7936;
const FILLER: i32 = 1931;

/// Every success roll is drawn below 256; any other bound is recorded so tests can pin it.
struct Draws(Vec<u32>);
impl RandomSource for Draws {
    fn below(&mut self, bound: u32) -> u32 {
        self.0.push(bound);
        0
    }
}

/// A miner next to `rock` with Mining `level`, a bronze pickaxe and `free` empty slots.
fn fixture(rock: u32, level: i8, member: bool, free: usize) -> (World, ActorId, Location) {
    let mut w = World::default();
    for x in (3192..3216).step_by(8) {
        for z in (3192..3216).step_by(8) {
            w.collision.allocate(x, z, 0);
        }
    }
    let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), rock, 10, 0).unwrap();
    let ty = LocCollision { width: 1, length: 1, block_walk: 2, block_range: true, break_route_finding: false };
    for id in [CLAY_ROCK, ESSENCE_ROCK, DEPLETED] {
        w.location_types.insert(id, ty);
    }
    w.locations.load(loc);
    crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true).unwrap();
    let clay = Rock {
        members: false,
        restore: None,
        level: 1,
        product: CLAY,
        product_name: "Clay".into(),
        xp: 5.0,
        depleted: DEPLETED,
        respawn: 2,
        rates: (128, 400),
    };
    let essence = Rock {
        members: false,
        restore: None,
        level: 1,
        product: RUNE,
        product_name: "Rune essence".into(),
        xp: 5.0,
        depleted: ESSENCE_ROCK,
        respawn: 0,
        rates: (256, 256),
    };
    let config = Config {
        skill: 14,
        picks: [(1265, Pickaxe { level: 1, animation: 625, interval: 2, members: false, fast_one_in: None })].into(),
        rocks: [(CLAY_ROCK, clay), (ESSENCE_ROCK, essence)].into(),
        full_sound: 2277,
    };
    config.validate().unwrap();
    let rule = Essence { rocks: [ESSENCE_ROCK].into(), pure: PURE, pure_name: "Pure essence".into(), pure_level: 30 };
    rule.validate(&config.rocks).unwrap();
    w.mining = Some(config);
    w.mining_essence = Some(rule);
    let mut stats = Stats::default();
    stats.set_base(14, level);
    stats.set_current(14, level);
    let mut p = PlayerData::new(1, "Clay and essence miner".into(), CoordGrid::new(3200, 3200, 0).unwrap(), stats);
    p.xp_rate_hundreds = 500;
    p.inventory.restore_slot(0, Some(Item { id: 1265, count: 1, vars: 0 })).unwrap();
    for slot in 1..28 - free {
        p.inventory.restore_slot(slot, Some(Item { id: FILLER, count: 1, vars: 0 })).unwrap();
    }
    w.queue_login(1, p).unwrap();
    w.tick_with_random(&mut Draws(vec![])).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, member).unwrap();
    w.drain_events();
    (w, a, loc)
}
fn mine(w: &mut World, a: ActorId, loc: Location) {
    w.submit(a, Action::OperateLocation { coords: loc.coords, id: loc.id, operation: 1, control: false }).unwrap();
}
/// Run `ticks` ticks, returning the mining traces, the drawn bounds and the messages.
fn run(w: &mut World, ticks: usize) -> (Vec<crate::gathering::Trace>, Vec<u32>, Vec<String>) {
    let mut random = Draws(vec![]);
    let (mut traces, mut messages) = (vec![], vec![]);
    for _ in 0..ticks {
        w.tick_with_random(&mut random).unwrap();
        for e in w.drain_events() {
            match e {
                WorldEvent::Mining { trace, .. } => traces.push(trace),
                WorldEvent::GameMessage { text, .. } => messages.push(text),
                _ => {}
            }
        }
    }
    (traces, random.0, messages)
}
fn count(w: &World, a: ActorId, item: i32) -> usize {
    w.data(a).unwrap().inventory.slots().iter().flatten().filter(|i| i.id == item).count()
}

#[test]
fn clay_at_level_one_awards_five_xp_times_five_depletes_and_respawns() {
    let (mut w, a, loc) = fixture(CLAY_ROCK, 1, false, 10);
    let xp = w.data(a).unwrap().stats.fine_xp(14);
    mine(&mut w, a, loc);
    // Schedule, then the attempt: the success roll and the depletion roll; no gem pre-roll.
    let (traces, bounds, messages) = run(&mut w, 2);
    let success = traces.iter().find(|t| t.result == "success").expect("clay mined");
    assert_eq!((success.product, success.product_delta, success.respawn_delay), (CLAY, 1, Some(2)));
    assert_eq!(bounds, [256, 255]);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(14) - xp, 250, "5 XP at x5, in tenths");
    assert_eq!(count(&w, a, CLAY), 1);
    assert!(messages.iter().any(|m| m == "You manage to mine some clay."));
    assert!(w.locations.find_type(loc.coords, DEPLETED).is_some(), "clay depleted to rocks2");
    assert!(w.players.get(a.slot as usize).unwrap().location_interaction.is_none(), "mining stops on depletion");
    run(&mut w, 2);
    assert!(w.locations.find_type(loc.coords, CLAY_ROCK).is_some(), "clay respawned after 2 ticks");
}

#[test]
fn full_pack_message_uses_the_miners_essence_product_without_awarding_items_or_xp() {
    for (member, level, product, name) in [
        (true, 30, PURE, "pure essence"),
        (true, 29, RUNE, "rune essence"),
        (false, 30, RUNE, "rune essence"),
        (false, 29, RUNE, "rune essence"),
    ] {
        let (mut w, a, loc) = fixture(ESSENCE_ROCK, level, member, 0);
        let inventory = w.data(a).unwrap().inventory.slots().to_vec();
        let xp = w.data(a).unwrap().stats.fine_xp(14);
        mine(&mut w, a, loc);
        let (traces, bounds, messages) = run(&mut w, 3);
        assert_eq!(messages, [format!("Your inventory is too full to hold any more {name}.")], "member={member}, level={level}");
        assert_eq!(traces.len(), 1, "capacity refusal stops the action");
        let trace = &traces[0];
        assert_eq!((trace.result, trace.product, trace.product_delta, trace.fine_xp_delta), ("capacity", product, 0, 0));
        assert!(bounds.is_empty(), "full inventory draws no success/depletion rolls");
        assert_eq!(w.data(a).unwrap().inventory.slots(), inventory);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(14), xp);
        assert!(w.locations.find_type(loc.coords, ESSENCE_ROCK).is_some());
        assert!(w.players.get(a.slot as usize).unwrap().location_interaction.is_none());
    }
}

#[test]
fn essence_never_depletes_and_keeps_mining_until_the_inventory_is_full() {
    let (mut w, a, loc) = fixture(ESSENCE_ROCK, 1, false, 3);
    mine(&mut w, a, loc);
    let (traces, bounds, messages) = run(&mut w, 12);
    let successes: Vec<_> = traces.iter().filter(|t| t.result == "success").collect();
    assert_eq!(successes.len(), 3, "one essence per free slot");
    for t in &successes {
        assert_eq!((t.product, t.product_delta, t.respawn_delay, t.depletion_roll), (RUNE, 1, None, None));
    }
    // One success roll per attempt and no depletion roll: the rock never depletes.
    assert_eq!(bounds, [256, 256, 256]);
    assert!(w.locations.find_type(loc.coords, ESSENCE_ROCK).is_some());
    assert!(w.locations.find_type(loc.coords, DEPLETED).is_none());
    assert_eq!(count(&w, a, RUNE), 3);
    // Attempts are a pickaxe interval (2 ticks) apart.
    let ticks: Vec<i32> = successes.iter().map(|t| t.tick).collect();
    assert_eq!((ticks[1] - ticks[0], ticks[2] - ticks[1]), (2, 2));
    assert_eq!(traces.last().unwrap().result, "capacity");
    assert!(messages.iter().any(|m| m == "Your inventory is too full to hold any more rune essence."));
    assert!(w.players.get(a.slot as usize).unwrap().location_interaction.is_none(), "stops at full inventory");
}

#[test]
fn pure_essence_needs_membership_and_mining_thirty() {
    for (member, level, product) in [(false, 29, RUNE), (false, 30, RUNE), (true, 29, RUNE), (true, 30, PURE)] {
        let (mut w, a, loc) = fixture(ESSENCE_ROCK, level, member, 1);
        let xp = w.data(a).unwrap().stats.fine_xp(14);
        mine(&mut w, a, loc);
        let (traces, _, messages) = run(&mut w, 2);
        let t = traces.iter().find(|t| t.result == "success").expect("essence mined");
        assert_eq!(t.product, product, "member={member} level={level}");
        assert_eq!((count(&w, a, product), count(&w, a, if product == RUNE { PURE } else { RUNE })), (1, 0));
        assert_eq!(w.data(a).unwrap().stats.fine_xp(14) - xp, 250, "both give 5 XP");
        let name = if product == PURE { "pure essence" } else { "rune essence" };
        assert!(messages.iter().any(|m| *m == format!("You manage to mine some {name}.")));
    }
}

#[test]
fn clay_draws_the_gem_pre_roll_first_and_essence_does_not() {
    // Clay is pre-roll eligible; the infinite essence rock is not (OpenRune rollGemPreTable).
    let gems = || Gems {
        pre_roll_rocks: [CLAY_ROCK].into(),
        pre_roll_gems: vec![(32, 1623), (16, 1621), (8, 1619), (2, 1617)],
        gem_rock_table: vec![(128, 1623)],
        names: [(1617, "Uncut diamond"), (1619, "Uncut ruby"), (1621, "Uncut emerald"), (1623, "Uncut sapphire")]
            .map(|(id, name)| (id, name.to_owned()))
            .into(),
        ..Default::default()
    };
    for (rock, bounds) in [(CLAY_ROCK, vec![PRE_ROLL_TOTAL, 256, 255]), (ESSENCE_ROCK, vec![256])] {
        let (mut w, a, loc) = fixture(rock, 1, false, 5);
        let g = gems();
        g.validate(&w.mining.as_ref().unwrap().rocks).unwrap();
        w.mining_gems = Some(g);
        mine(&mut w, a, loc);
        // Draw 0 of the pre-roll is a nothing row (SkipOre), so scripted draws are not all 0.
        struct Keep(Vec<u32>);
        impl RandomSource for Keep {
            fn below(&mut self, bound: u32) -> u32 {
                self.0.push(bound);
                if bound == PRE_ROLL_TOTAL { PRE_ROLL_TOTAL - 1 } else { 0 }
            }
        }
        let mut random = Keep(vec![]);
        for _ in 0..2 {
            w.tick_with_random(&mut random).unwrap();
        }
        assert_eq!(random.0, bounds, "rock {rock}");
        assert_eq!(count(&w, a, if rock == CLAY_ROCK { CLAY } else { RUNE }), 1);
    }
}

#[test]
fn infinite_rocks_must_deplete_into_themselves() {
    let mut rock = Rock {
        members: false,
        restore: None,
        level: 1,
        product: RUNE,
        product_name: "Rune essence".into(),
        xp: 5.0,
        depleted: DEPLETED,
        respawn: 0,
        rates: (256, 256),
    };
    let config = |rock: &Rock| Config {
        skill: 14,
        picks: [(1265, Pickaxe { level: 1, animation: 625, interval: 2, members: false, fast_one_in: None })].into(),
        rocks: [(ESSENCE_ROCK, rock.clone())].into(),
        full_sound: 2277,
    };
    assert!(config(&rock).validate().is_err(), "infinite rock with a different depleted loc");
    rock.depleted = ESSENCE_ROCK;
    assert!(config(&rock).validate().is_ok());
    rock.respawn = 2;
    assert!(config(&rock).validate().is_err(), "a depleting rock cannot deplete into itself");
    rock.respawn = -1;
    assert!(config(&rock).validate().is_err());
    rock.respawn = 0;
    let rocks = config(&rock).rocks;
    let rule = |pure| Essence { rocks: [ESSENCE_ROCK].into(), pure, pure_name: "Pure essence".into(), pure_level: 30 };
    assert!(rule(PURE).validate(&rocks).is_ok());
    assert!(rule(RUNE).validate(&rocks).is_err(), "pure must differ from the rock's product");
}
