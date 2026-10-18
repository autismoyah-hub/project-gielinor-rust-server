//! Original24 ordinary cooked meat only. No potion or chemistry setup.
use gielinor_world::{
    food::Edible,
    hits::{Hit, Style, Target},
    inventory::Item,
    npc::RandomSource,
    simulation::{Action, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
struct NoRandom;
impl RandomSource for NoRandom {
    fn below(&mut self, _: u32) -> u32 {
        panic!("ordinary food life probe drew random")
    }
}
#[test]
fn observe_original_food_after_zero_hp_and_actual_unresolved_death() {
    let mut cases = 0;
    let mut violations = 0;
    for member in [false, true] {
        for slot in [0usize, 13, 27] {
            for phase in 0..3 {
                let mut w = World::default();
                w.collision.allocate(3224, 3216, 0);
                w.foods.insert(
                    2142,
                    Edible {
                        id: 2142,
                        heal: 3,
                        delay: 3,
                        replacement: None,
                        message: "You eat the meat.".into(),
                    },
                );
                let mut s = Stats::default();
                s.set_base(3, 10);
                s.set_current(3, 1);
                s.set_fine_xp(3, 11540).unwrap();
                let mut p = PlayerData::new(
                    1,
                    "Original24 ordinary food life observer".into(),
                    CoordGrid::new(3224, 3218, 0).unwrap(),
                    s,
                );
                for i in 0..28 {
                    p.inventory
                        .restore_slot(
                            i,
                            Some(Item {
                                id: if i == slot || i == (slot + 1) % 28 {
                                    2142
                                } else {
                                    1931
                                },
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                w.queue_login(1, p).unwrap();
                w.tick_with_random(&mut NoRandom).unwrap();
                let a = w.actors().next().unwrap();
                w.set_membership(a, member).unwrap();
                w.drain_events();
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
                    w.tick_with_random(&mut NoRandom).unwrap();
                    assert!(w
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
                    assert_eq!(w.data(a).unwrap().stats.current(3), 0);
                    if phase == 2 {
                        w.data_mut(a).unwrap().stats.set_current(3, 10);
                    }
                }
                let before = w.data(a).unwrap().clone();
                let submitted = w
                    .submit(a, Action::EatItemSlot { slot, item: 2142 })
                    .is_ok();
                if submitted {
                    w.tick_with_random(&mut NoRandom).unwrap();
                }
                let d = w.data(a).unwrap();
                let changed =
                    d.inventory.slots() != before.inventory.slots() || d.stats != before.stats;
                violations += usize::from(changed);
                cases += 1;
                println!("PG_ORIGINAL_FOOD_LIFE member={member} slot={slot} phase={phase} submitted={submitted} hp_before={} hp_after={} selectedRetained={} violation={changed}",before.stats.current(3),d.stats.current(3),d.inventory.get(slot).is_some_and(|i|i.id==2142));
            }
        }
    }
    assert_eq!(cases, 18);
    println!("PG_ORIGINAL_FOOD_LIFE_SUMMARY cases={cases} violations={violations};diagnostic observation completion is not acceptancePASS if violations>0");
}
