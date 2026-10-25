use super::*;
use crate::{
    coal_bag::Config as Bag,
    inventory::Item,
    loc_collision::LocCollision,
    mining::{Config, Gems, Pickaxe, PreRoll, Rock, GEM_ROCK_TOTAL, PRE_ROLL_GLORY_TOTAL, PRE_ROLL_TOTAL},
    stats::Stats,
    variables::{Transmit, Varbit, Varp},
};

/// Scripted (bound, draw) pairs; every bound must match exactly.
struct Draws(VecDeque<(u32, u32)>);
impl RandomSource for Draws {
    fn below(&mut self, bound: u32) -> u32 {
        let (expected, draw) = self.0.pop_front().expect("unexpected mining RNG draw");
        assert_eq!(bound, expected);
        draw
    }
}

const ORE_ROCK: u32 = 1;
const GEM_ROCK: u32 = 3;
const GLORY: i32 = 1712;

fn gems() -> Gems {
    Gems {
        pre_roll_rocks: [ORE_ROCK].into(),
        pre_roll_gems: vec![(32, 1623), (16, 1621), (8, 1619), (2, 1617)],
        gem_rocks: [GEM_ROCK].into(),
        gem_rock_table: vec![(60, 1625), (30, 1627), (15, 1629), (9, 1623), (5, 1621), (5, 1619), (4, 1617)],
        names: [
            (1617, "Uncut diamond"),
            (1619, "Uncut ruby"),
            (1621, "Uncut emerald"),
            (1623, "Uncut sapphire"),
            (1625, "Uncut opal"),
            (1627, "Uncut jade"),
            (1629, "Uncut red topaz"),
        ]
        .map(|(id, name)| (id, name.to_owned()))
        .into(),
        charged_glory: [GLORY].into(),
    }
}

/// A miner next to `rock` (ore rock 1 of `product`, or gem rock 3), Mining 40.
fn fixture(rock: u32, product: i32, glory: bool, bag: bool) -> (World, ActorId, Location) {
    let mut w = World::default();
    for x in (3192..3216).step_by(8) {
        for z in (3192..3216).step_by(8) {
            w.collision.allocate(x, z, 0);
        }
    }
    let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), rock, 10, 0).unwrap();
    let ty = LocCollision { width: 1, length: 1, block_walk: 2, block_range: true, break_route_finding: false };
    for id in [1, 2, 3] {
        w.location_types.insert(id, ty);
    }
    w.locations.load(loc);
    crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true).unwrap();
    let ore = Rock {
        members: false,
        restore: None,
        level: 1,
        product,
        product_name: if product == 453 { "Coal" } else { "Iron ore" }.into(),
        xp: 50.0,
        depleted: 2,
        respawn: 12,
        rates: (256, 256),
    };
    let gem_rock = Rock {
        members: true,
        restore: None,
        level: 40,
        product: 1625,
        product_name: "gems".into(),
        xp: 65.0,
        depleted: 2,
        respawn: 99,
        rates: (28, 70),
    };
    let config = Config {
        skill: 14,
        picks: [(1265, Pickaxe { level: 1, animation: 625, interval: 2, members: false, fast_one_in: None })].into(),
        rocks: [(ORE_ROCK, ore), (GEM_ROCK, gem_rock)].into(),
        full_sound: 2277,
    };
    let gems = gems();
    gems.validate(&config.rocks).unwrap();
    w.mining = Some(config);
    w.mining_gems = Some(gems);
    let bag_config = Bag {
        closed: 12019,
        open: 24480,
        coal: 453,
        capes: [9795, 9796, 13280],
        base: Varp { id: 65531, permanent: true, transmit: Transmit::Never, bit_protect: true, client_code: -1 },
        stored: Varbit { base: 65531, lsb: 25, msb: 30 },
    };
    w.coal_bag = Some(bag_config);
    w.object_types.stackable.extend([12019, 24480]);
    let mut stats = Stats::default();
    stats.set_base(14, 40);
    stats.set_current(14, 40);
    let mut p = PlayerData::new(1, "Gem miner".into(), CoordGrid::new(3200, 3200, 0).unwrap(), stats);
    p.inventory.restore_slot(0, Some(Item { id: 1265, count: 1, vars: 0 })).unwrap();
    if bag {
        p.inventory.restore_slot(27, Some(Item { id: 24480, count: 1, vars: 0 })).unwrap();
        p.variables.set(bag_config.base, 0x81000001u32 as i32, false);
    }
    if glory {
        p.equipment.restore_slot(2, Some(Item { id: GLORY, count: 1, vars: 0 })).unwrap();
    }
    w.queue_login(1, p).unwrap();
    w.tick_with_random(&mut Draws(VecDeque::new())).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, true).unwrap();
    w.drain_events();
    (w, a, loc)
}

/// Submit Mine, then run the scheduling tick and the attempt tick with `draws`.
fn attempt(w: &mut World, a: ActorId, loc: Location, draws: &[(u32, u32)]) -> (crate::gathering::Trace, Vec<WorldEvent>) {
    w.submit(a, Action::OperateLocation { coords: loc.coords, id: loc.id, operation: 1, control: false }).unwrap();
    w.tick_with_random(&mut Draws(VecDeque::new())).unwrap();
    w.drain_events();
    let mut rng = Draws(draws.iter().copied().collect());
    w.tick_with_random(&mut rng).unwrap();
    assert!(rng.0.is_empty(), "every scripted draw was consumed");
    let events = w.drain_events();
    let trace = events
        .iter()
        .find_map(|e| match e {
            WorldEvent::Mining { trace, .. } => Some(trace.clone()),
            _ => None,
        })
        .expect("one mining attempt");
    (trace, events)
}
fn count(w: &World, a: ActorId, item: i32) -> usize {
    w.data(a).unwrap().inventory.slots().iter().flatten().filter(|i| i.id == item).count()
}
fn said(events: &[WorldEvent], text: &str) -> bool {
    events.iter().any(|e| matches!(e, WorldEvent::GameMessage { text: t, .. } if t == text))
}

#[test]
fn pre_roll_rows_follow_openrune_order_and_totals() {
    let g = gems();
    for (draw, expected) in [
        (0, PreRoll::SkipOre),
        (69, PreRoll::SkipOre),
        (70, PreRoll::Gem(1623)),
        (101, PreRoll::Gem(1623)),
        (102, PreRoll::Gem(1621)),
        (117, PreRoll::Gem(1621)),
        (118, PreRoll::Gem(1619)),
        (125, PreRoll::Gem(1619)),
        (126, PreRoll::Gem(1617)),
        (127, PreRoll::Gem(1617)),
        (128, PreRoll::KeepOre),
        (PRE_ROLL_GLORY_TOTAL - 1, PreRoll::KeepOre),
        (PRE_ROLL_TOTAL - 1, PreRoll::KeepOre),
    ] {
        assert_eq!(g.pre_roll(draw), expected, "draw {draw}");
    }
    // 128 non-keep rows: 1/256 of 32768, and 1/64.2 of the glory table's 8216.
    assert_eq!((PRE_ROLL_TOTAL / 128, PRE_ROLL_GLORY_TOTAL), (256, 8216));
    let mut gems_from_rock = std::collections::BTreeMap::new();
    for draw in 0..GEM_ROCK_TOTAL {
        *gems_from_rock.entry(g.gem_rock(draw).unwrap()).or_insert(0) += 1;
    }
    assert_eq!(
        gems_from_rock,
        [(1617, 4), (1619, 5), (1621, 5), (1623, 9), (1625, 60), (1627, 30), (1629, 15)].into()
    );
    assert_eq!(g.gem_rock(GEM_ROCK_TOTAL), None);
}

#[test]
fn a_gem_find_grants_the_gem_without_xp_or_depletion_and_skips_the_ore() {
    for (glory, total) in [(false, PRE_ROLL_TOTAL), (true, PRE_ROLL_GLORY_TOTAL)] {
        let (mut w, a, loc) = fixture(ORE_ROCK, 440, glory, false);
        let xp = w.data(a).unwrap().stats.fine_xp(14);
        // Only the pre-roll draws: no success or depletion roll on a gem find.
        let (trace, events) = attempt(&mut w, a, loc, &[(total, 102)]);
        assert_eq!((trace.result, trace.product, trace.product_delta), ("gem_found", 1621, 1));
        assert_eq!((count(&w, a, 1621), count(&w, a, 440)), (1, 0));
        assert_eq!(w.data(a).unwrap().stats.fine_xp(14), xp, "no experience");
        assert!(w.locations.find_type(loc.coords, ORE_ROCK).is_some(), "rock not depleted");
        assert!(said(&events, "You manage to mine some uncut emerald."));
        let p = w.players.get(a.slot as usize).unwrap();
        assert!(p.location_interaction.is_some(), "mining continues");
    }
}

#[test]
fn a_nothing_row_skips_the_ore_and_keep_ore_rolls_it() {
    let (mut w, a, loc) = fixture(ORE_ROCK, 440, false, false);
    let (trace, _) = attempt(&mut w, a, loc, &[(PRE_ROLL_TOTAL, 69)]);
    assert_eq!((trace.result, trace.roll), ("gem_skip_ore", Some(69)));
    assert_eq!(count(&w, a, 440), 0);
    assert!(w.locations.find_type(loc.coords, ORE_ROCK).is_some());
    assert!(w.players.get(a.slot as usize).unwrap().location_interaction.is_some());

    let (mut w, a, loc) = fixture(ORE_ROCK, 440, true, false);
    let xp = w.data(a).unwrap().stats.fine_xp(14);
    let (trace, _) = attempt(&mut w, a, loc, &[(PRE_ROLL_GLORY_TOTAL, 128), (256, 0), (255, 0)]);
    assert_eq!((trace.result, trace.roll), ("success", Some(0)));
    assert_eq!(count(&w, a, 440), 1);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(14) - xp, 500);
    assert!(w.locations.find_type(loc.coords, 2).is_some(), "ore success depletes as before");
}

#[test]
fn gem_rocks_use_their_own_table_xp_and_glory_tripled_rates_without_a_pre_roll() {
    let mut thresholds = vec![];
    for glory in [false, true] {
        let (mut w, a, loc) = fixture(GEM_ROCK, 0, glory, false);
        let xp = w.data(a).unwrap().stats.fine_xp(14);
        // Success, then the gem-rock /128 draw (61 = jade), then depletion; no pre-roll.
        let (trace, events) = attempt(&mut w, a, loc, &[(256, 0), (GEM_ROCK_TOTAL, 61), (255, 0)]);
        assert_eq!((trace.result, trace.product), ("success", 1627));
        assert_eq!(count(&w, a, 1627), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(14) - xp, 650, "65 xp");
        assert!(said(&events, "You manage to mine some uncut jade."));
        assert!(w.locations.find_type(loc.coords, 2).is_some(), "gem rock depletes");
        thresholds.push(trace.threshold.unwrap());
    }
    let rate = |low, high| crate::skills::success_rate(low, high, 40, 99) * 256.0;
    assert_eq!(thresholds, [rate(28, 70), rate(84, 210)]);
}

#[test]
fn the_coal_bag_path_is_unaffected_and_gems_never_enter_the_bag() {
    let stored = |w: &World, a| w.data(a).unwrap().variables.bit(w.coal_bag.unwrap().stored).unwrap();
    let (mut w, a, loc) = fixture(ORE_ROCK, 453, false, true);
    let (trace, _) = attempt(&mut w, a, loc, &[(PRE_ROLL_TOTAL, 500), (256, 0), (255, 0)]);
    assert_eq!((trace.result, trace.stored_delta, stored(&w, a)), ("success", 1, 1));
    assert_eq!(count(&w, a, 453), 0, "coal still goes into the open bag");

    let (mut w, a, loc) = fixture(ORE_ROCK, 453, false, true);
    let (trace, _) = attempt(&mut w, a, loc, &[(PRE_ROLL_TOTAL, 126)]);
    assert_eq!((trace.result, stored(&w, a), count(&w, a, 1617)), ("gem_found", 0, 1));
}

#[test]
fn invalid_gem_settings_are_rejected() {
    let rocks: std::collections::BTreeMap<u32, Rock> = w_rocks();
    let mut g = gems();
    assert!(g.validate(&rocks).is_ok());
    g.pre_roll_gems[0].0 = 31;
    assert!(g.validate(&rocks).is_err(), "nothing + gems must be 128");
    let mut g = gems();
    g.gem_rock_table.pop();
    assert!(g.validate(&rocks).is_err());
    let mut g = gems();
    g.names.remove(&1629);
    assert!(g.validate(&rocks).is_err());
    let mut g = gems();
    g.pre_roll_rocks.insert(GEM_ROCK);
    assert!(g.validate(&rocks).is_err(), "gem rocks never pre-roll");
    let mut g = gems();
    g.gem_rocks.insert(99);
    assert!(g.validate(&rocks).is_err());
}
fn w_rocks() -> std::collections::BTreeMap<u32, Rock> {
    let (w, _, _) = fixture(ORE_ROCK, 440, false, false);
    w.mining.unwrap().rocks
}
