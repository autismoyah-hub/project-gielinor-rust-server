use gielinor_world::{
    player_regen::Config,
    simulation::{PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
#[test]
fn login_timers_regenerate_without_restoring_prayer_or_erasing_boosted_health() {
    let mut world = World::default();
    world.regeneration = Some(Config {
        skills: (0..23).collect(),
        hitpoints: 3,
        prayer: 5,
        bracelet: 11133,
        drained_timer: 2,
        boosted_timer: 3,
        health_timer: 4,
        rapid_timer: 5,
    });
    let mut stats = Stats::default();
    for id in [0, 1, 3, 5] {
        stats.set_base(id, 10);
    }
    stats.set_current(0, 8);
    stats.set_current(1, 12);
    stats.set_current(3, 12);
    stats.set_current(5, 0);
    world
        .queue_login(
            1,
            PlayerData::new(
                1,
                "Regen".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            ),
        )
        .unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    for _ in 0..99 {
        world.tick().unwrap();
    }
    assert_eq!(world.data(actor).unwrap().stats.current(0), 8);
    world.tick().unwrap();
    let stats = &world.data(actor).unwrap().stats;
    assert_eq!(
        [
            stats.current(0),
            stats.current(1),
            stats.current(3),
            stats.current(5)
        ],
        [9, 11, 12, 0]
    );
    let updated = world
        .drain_events()
        .into_iter()
        .filter_map(|e| match e {
            WorldEvent::Stat { update, .. } => Some(update.skill),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(updated, vec![0, 1]);
}
#[test]
fn regeneration_bracelet_heals_two_and_caps_at_base() {
    let c = Config {
        skills: vec![0, 3, 5],
        hitpoints: 3,
        prayer: 5,
        bracelet: 11133,
        drained_timer: 2,
        boosted_timer: 3,
        health_timer: 4,
        rapid_timer: 5,
    };
    let mut s = Stats::default();
    s.set_base(3, 10);
    s.set_current(3, 7);
    assert_eq!(c.tick(4, &mut s, Some(11133))[0].level, 9);
    let next = c.tick(4, &mut s, Some(11133));
    assert_eq!(next[0].level, 10);
    assert!(next[0].clear_hero_points);
    assert!(c.tick(4, &mut s, None).is_empty());
}

#[test]
fn prayer_regeneration_resets_deadlines_and_preserve_waits_for_queued_activation() {
    use gielinor_world::{player_timers::Timers, prayer_book::Regen};
    let c = Config {
        skills: vec![0, 3, 5],
        hitpoints: 3,
        prayer: 5,
        bracelet: 11133,
        drained_timer: 2,
        boosted_timer: 3,
        health_timer: 4,
        rapid_timer: 5,
    };
    let mut timers = Timers::default();
    c.login(&mut timers, 10);
    c.prayer_toggle(&mut timers, 20, Regen::RapidHeal, true);
    assert_eq!(timers.get(4).unwrap().expiry, 70);
    c.prayer_toggle(&mut timers, 30, Regen::RapidHeal, false);
    assert_eq!(timers.get(4).unwrap().expiry, 130);
    c.prayer_toggle(&mut timers, 20, Regen::RapidRestore, true);
    assert_eq!(timers.get(5).unwrap().expiry, 120);
    c.prayer_toggle(&mut timers, 30, Regen::RapidRestore, false);
    assert_eq!(timers.get(5), None);
    c.prayer_toggle(&mut timers, 20, Regen::Preserve, true);
    assert_eq!(timers.get(3).unwrap().expiry, 110);
    c.activate_preserve(&mut timers, 45);
    assert_eq!(timers.get(3).unwrap().expiry, 195);
    c.prayer_toggle(&mut timers, 60, Regen::Preserve, false);
    assert_eq!(timers.get(3).unwrap().expiry, 160);
}
