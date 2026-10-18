//! Exact original51 ordinary burial admission/completion life checks; no robe inference.
use gielinor_world::{
    inventory::Item,
    npc::RandomSource,
    prayer::{Config, Event},
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
struct Draw {
    calls: usize,
}
impl RandomSource for Draw {
    fn below(&mut self, _: u32) -> u32 {
        self.calls += 1;
        panic!("ordinary burial unexpected RNG")
    }
}
fn setup(worn: bool, confirm: bool) -> (World, ActorId) {
    let mut w = World::default();
    w.collision.allocate(3216, 3216, 0);
    w.prayer = Some(Config {
        skill: 5,
        animation: 827,
        bones: [(526, 4.5)].into(),
        necklace: None,
        restore: Default::default(),
        worn_xp_percent: Default::default(),
        chaos: if confirm {
            Some([3220, 3220, 3220, 3220])
        } else {
            None
        },
    });
    let mut stats = Stats::default();
    stats.set_base(3, 10);
    stats.set_current(3, 10);
    let mut p = PlayerData::new(
        1,
        "World burial observer".into(),
        CoordGrid::new(3220, 3220, 0).unwrap(),
        stats,
    );
    p.xp_rate_hundreds = 125;
    for s in 0..28 {
        p.inventory
            .restore_slot(
                s,
                Some(Item {
                    id: 526,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    if worn {
        p.equipment
            .restore_slot(
                0,
                Some(Item {
                    id: 25438,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    w.queue_login(1, p).unwrap();
    w.tick_with_random(&mut Draw { calls: 0 }).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, true).unwrap();
    w.drain_events();
    (w, a)
}
fn tick(w: &mut World, r: &mut Draw) {
    w.tick_with_random(r).unwrap();
}
#[test]
fn observe_authoritative_death_and_late_life_boundaries() {
    use gielinor_world::hits::{Hit, Style, Target};
    let mut cases = 0;
    let mut violations = 0;
    for slot in [0, 7, 27] {
        for phase in 0..6 {
            let (mut w, a) = setup(false, false);
            let mut r = Draw { calls: 0 };
            let predead = phase == 0 || phase == 2 || phase == 3;
            if predead {
                if phase == 0 {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                } else {
                    w.queue_hit(
                        Target::Player(a),
                        Hit {
                            display: None,
                            source: None,
                            style: Style::Typeless,
                            damage: 20,
                        },
                        1,
                    )
                    .unwrap();
                    tick(&mut w, &mut r);
                    assert!(w
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
                    if phase == 3 {
                        w.data_mut(a).unwrap().stats.set_current(3, 10);
                    }
                }
            }
            let before = w.data(a).unwrap().clone();
            w.submit(a, Action::BuryItemSlot { slot, item: 526 })
                .unwrap();
            tick(&mut w, &mut r);
            if !predead {
                assert_eq!(w.data(a).unwrap().stats.fine_xp(5), before.stats.fine_xp(5));
                if phase == 1 {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                } else {
                    w.queue_hit(
                        Target::Player(a),
                        Hit {
                            display: None,
                            source: None,
                            style: Style::Typeless,
                            damage: 20,
                        },
                        1,
                    )
                    .unwrap();
                    tick(&mut w, &mut r);
                    assert!(w
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
                    if phase == 5 {
                        w.data_mut(a).unwrap().stats.set_current(3, 10);
                    }
                }
            }
            for _ in 0..4 {
                tick(&mut w, &mut r);
            }
            let p = w.data(a).unwrap();
            let delta = p.stats.fine_xp(5) - before.stats.fine_xp(5);
            let keep = p.inventory.get(slot).is_some();
            let bad = delta != 0 || p.inventory != before.inventory || r.calls != 0;
            violations += usize::from(bad);
            cases += 1;
            println!("PG_ORIGINAL_PRAYER_LIFE slot={slot} phase={phase} hp={} fineXPdelta={delta} selectedRetained={keep} calls={} violation={bad}",p.stats.current(3),r.calls);
        }
    }
    assert_eq!(cases, 18);
    println!("PG_ORIGINAL_PRAYER_LIFE_SUMMARY cases={cases} violations={violations};diagnostic observations only,any violation is acceptance FAIL");
}
