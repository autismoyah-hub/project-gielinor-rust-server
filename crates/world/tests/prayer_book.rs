use gielinor_world::{
    bank_actions::Binding,
    prayer_book::{Config, Event, Prayer, Regen},
    stats::Stats,
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
fn binding(base: u16, lsb: u8, msb: u8) -> Binding {
    Binding {
        base: Varp {
            id: base,
            permanent: false,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit { base, lsb, msb },
    }
}
fn setup() -> (Config, Variables, Stats) {
    let prayer = |id| Prayer {
        id,
        name: format!("Prayer {id}"),
        level: 1,
        sound: 100 + id as u16,
        enabled: binding(0, id, id),
        overhead: None,
        unlock: None,
        unlock_state: 1,
        defence: None,
        locked_message: None,
        regen: None,
    };
    (
        Config {
            prayer_skill: 5,
            defence_skill: 1,
            all_active: binding(0, 0, 2),
            quick_active: binding(1, 0, 0),
            drain_counter: binding(2, 0, 30),
            disable_sound: 9,
            prayers: BTreeMap::from([(0, prayer(0)), (1, prayer(1)), (2, prayer(2))]),
            collisions: BTreeMap::from([(0, vec![0, 1]), (1, vec![0, 1])]),
            components: BTreeMap::new(),
        },
        Variables::default(),
        Stats::default(),
    )
}
#[test]
fn collision_switch_preserves_unrelated_prayer_and_counter() {
    let (mut c, mut v, s) = setup();
    c.prayers.get_mut(&0).unwrap().regen = Some(Regen::RapidHeal);
    c.toggle(0, &mut v, &s, None, true).unwrap();
    c.toggle(2, &mut v, &s, None, true).unwrap();
    v.set(c.drain_counter.base, 57, false);
    let events = c.toggle(1, &mut v, &s, None, true).unwrap();
    assert_eq!(v.bit(c.all_active.bit).unwrap(), 6);
    assert_eq!(v.bit(c.drain_counter.bit).unwrap(), 57);
    assert!(events.contains(&Event::Regen {
        kind: Regen::RapidHeal,
        enabled: false
    }));
    assert!(!events.contains(&Event::Sound(c.disable_sound)));
    assert!(!events.contains(&Event::Drain(false)));
}
#[test]
fn last_manual_disable_clears_quick_prayer_and_drain() {
    let (c, mut v, s) = setup();
    c.toggle(0, &mut v, &s, None, true).unwrap();
    v.set(c.quick_active.base, 1, false);
    v.set(c.drain_counter.base, 40, false);
    let events = c.toggle(0, &mut v, &s, None, true).unwrap();
    assert_eq!((v.get(0), v.get(1), v.get(2)), (0, 0, 0));
    assert_eq!(events.last(), Some(&Event::Drain(false)));
}
#[test]
fn zero_current_points_short_circuits_requirements_but_allows_disabling() {
    let (mut c, mut v, mut s) = setup();
    c.prayers.get_mut(&0).unwrap().level = 99;
    s.set_current(5, 0);
    assert_eq!(
        c.toggle(0, &mut v, &s, None, true).unwrap(),
        vec![Event::RefreshCombatRange]
    );
    v.set(c.all_active.base, 1, false);
    assert!(c
        .toggle(0, &mut v, &s, None, true)
        .unwrap()
        .contains(&Event::Drain(false)));
}
#[test]
fn base_level_controls_requirement_and_failure_message_priority() {
    let (mut c, mut v, mut s) = setup();
    let p = c.prayers.get_mut(&0).unwrap();
    p.level = 70;
    p.defence = Some(70);
    p.unlock = Some(binding(3, 0, 1));
    p.unlock_state = 2;
    p.locked_message = Some("Unlock this<br>prayer.".into());
    s.set_current(5, 99);
    let events = c.toggle(0, &mut v, &s, None, true).unwrap();
    assert!(
        matches!(&events[1],Event::Message { text,modal:true } if text.contains("level of 70"))
    );
    s.set_base(5, 70);
    let events = c.protected_failure(0, &v, &s).unwrap().unwrap();
    assert_eq!(
        events[0],
        Event::Message {
            text: "Unlock this prayer.".into(),
            modal: false
        }
    );
    s.set_base(1, 70);
    v.set(binding(3, 0, 1).base, 2, false);
    assert_eq!(c.protected_failure(0, &v, &s).unwrap(), None);
    assert!(c
        .toggle(0, &mut v, &s, None, true)
        .unwrap()
        .contains(&Event::Drain(true)));
}
#[test]
fn overhead_disable_only_clears_matching_icon() {
    let (mut c, mut v, s) = setup();
    c.prayers.get_mut(&0).unwrap().overhead = Some(2);
    assert!(c
        .toggle(0, &mut v, &s, None, true)
        .unwrap()
        .contains(&Event::Overhead(Some(2))));
    assert!(!c
        .toggle(0, &mut v, &s, Some(1), true)
        .unwrap()
        .contains(&Event::Overhead(None)));
    c.toggle(0, &mut v, &s, None, true).unwrap();
    assert!(c
        .toggle(0, &mut v, &s, Some(2), true)
        .unwrap()
        .contains(&Event::Overhead(None)));
}

#[test]
fn depletion_clears_prayers_without_resetting_regeneration_effects() {
    let (c, mut v, s) = setup();
    c.toggle(0, &mut v, &s, None, true).unwrap();
    v.set(c.quick_active.base, 1, false);
    v.set(c.drain_counter.base, 42, false);
    let events = c.disable_all(&mut v, Some(2), true).unwrap();
    assert_eq!((v.get(0), v.get(1), v.get(2)), (0, 0, 0));
    assert!(events.contains(&Event::Overhead(None)));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Regen { .. })));
    assert!(!c
        .disable_all(&mut v, Some(15), true)
        .unwrap()
        .contains(&Event::Overhead(None)));
}

#[test]
fn runtime_depletes_next_cycle_and_clears_only_reference_timers() {
    use gielinor_world::{player_regen, player_timers::Timers, prayer_drain, prayer_runtime};
    let (book, mut vars, mut stats) = setup();
    let runtime = prayer_runtime::Config {
        drain: prayer_drain::Config {
            enabled: book.all_active,
            counter: book.drain_counter,
            resistance: binding(3, 0, 30),
            skill: 5,
            effects: vec![(0, 1)],
        },
        book,
        regeneration: player_regen::Config {
            skills: vec![0, 3, 5],
            hitpoints: 3,
            prayer: 5,
            bracelet: 11133,
            drained_timer: 2,
            boosted_timer: 3,
            health_timer: 4,
            rapid_timer: 5,
        },
        drain_timer: 7,
        depletion_sound: 10,
    };
    let mut timers = Timers::default();
    runtime.regeneration.login(&mut timers, 0);
    runtime.drain.update_resistance(&mut vars, 0, true).unwrap();
    runtime
        .toggle(0, &mut vars, &stats, None, &mut timers, 0, true)
        .unwrap();
    timers.schedule(5, 0, 100).unwrap();
    let mut depleted = false;
    for clock in 1..=62 {
        timers.process(clock, |t, id| {
            if id == 7 {
                let output = runtime
                    .tick(&mut vars, &mut stats, Some(2), t, true)
                    .unwrap();
                if output.preserve_queue == Some(false) {
                    assert_eq!(clock, 62);
                    depleted = true;
                }
            }
            true
        });
        if clock == 61 {
            assert_eq!(stats.current(5), 0);
            assert!(!depleted);
            assert!(timers.get(7).is_some());
        }
    }
    assert!(depleted);
    assert_eq!(vars.get(0), 0);
    assert_eq!(vars.get(2), 0);
    assert!(timers.get(7).is_none());
    assert!(timers.get(5).is_none());
    assert_eq!(timers.get(3).unwrap().expiry, 100);
    assert_eq!(timers.get(4).unwrap().expiry, 100);
}

fn world_setup() -> (
    gielinor_world::simulation::World,
    gielinor_world::simulation::ActorId,
) {
    use gielinor_world::{
        player_regen, prayer_drain, prayer_runtime,
        simulation::{PlayerData, World},
        CoordGrid,
    };
    let (book, _, stats) = setup();
    let regen = player_regen::Config {
        skills: vec![0, 3, 5],
        hitpoints: 3,
        prayer: 5,
        bracelet: 11133,
        drained_timer: 2,
        boosted_timer: 3,
        health_timer: 4,
        rapid_timer: 5,
    };
    let mut world = World::default();
    world.regeneration = Some(regen.clone());
    world.prayer_book = Some(prayer_runtime::Config {
        drain: prayer_drain::Config {
            enabled: book.all_active,
            counter: book.drain_counter,
            resistance: binding(3, 0, 30),
            skill: 5,
            effects: vec![(0, 1), (1, 6), (2, 30)],
        },
        book,
        regeneration: regen,
        drain_timer: 7,
        depletion_sound: 10,
    });
    world
        .queue_login(
            1,
            PlayerData::new(
                1,
                "Prayers".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                stats,
            ),
        )
        .unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    (world, actor)
}
#[test]
fn world_soft_drain_continues_during_protection_and_cleans_up_next_cycle() {
    let (mut w, a) = world_setup();
    assert_eq!(w.data(a).unwrap().variables.get(3), 60);
    w.select_prayer(a, 0).unwrap();
    w.delay_player(a, 100).unwrap();
    for _ in 0..62 {
        w.tick().unwrap();
    }
    assert_eq!(w.data(a).unwrap().stats.current(5), 0);
    assert_eq!(w.data(a).unwrap().variables.get(0), 1);
    w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().variables.get(0), 0);
    assert!(w.protection(a).unwrap().delayed());
}
#[test]
fn protected_click_queue_caps_at_ten_and_unmet_requirements_do_not_queue() {
    use gielinor_world::simulation::WorldEvent;
    let (mut w, a) = world_setup();
    w.delay_player(a, 5).unwrap();
    for _ in 0..11 {
        w.select_prayer(a, 0).unwrap();
    }
    for _ in 0..6 {
        w.tick().unwrap();
    }
    let events = w.drain_events();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(
                e,
                WorldEvent::PrayerBook {
                    event: Event::Drain(true),
                    ..
                }
            ))
            .count(),
        5
    );
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(
                e,
                WorldEvent::PrayerBook {
                    event: Event::Drain(false),
                    ..
                }
            ))
            .count(),
        5
    );
    assert_eq!(w.data(a).unwrap().variables.get(0), 0);
    w.prayer_book
        .as_mut()
        .unwrap()
        .book
        .prayers
        .get_mut(&1)
        .unwrap()
        .level = 99;
    w.delay_player(a, 10).unwrap();
    w.select_prayer(a, 1).unwrap();
    assert!(w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::PrayerBook {
            event: Event::Message { modal: false, .. },
            ..
        }
    )));
    for _ in 0..11 {
        w.tick().unwrap();
    }
    assert_eq!(w.prayer_notice_token(a).unwrap(), None);
}
#[test]
fn requirement_notice_is_protected_and_only_matching_token_can_resume_it() {
    let (mut w, a) = world_setup();
    w.prayer_book
        .as_mut()
        .unwrap()
        .book
        .prayers
        .get_mut(&0)
        .unwrap()
        .level = 99;
    w.select_prayer(a, 0).unwrap();
    let token = w.prayer_notice_token(a).unwrap().unwrap();
    assert!(w.protection(a).unwrap().suspended);
    assert!(!w.close_prayer_notice(a, token + 1).unwrap());
    assert!(w.close_prayer_notice(a, token).unwrap());
    assert!(!w.protection(a).unwrap().suspended);
    assert!(!w.close_prayer_notice(a, token).unwrap());
}
