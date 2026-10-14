use gielinor_world::{
    held_use::{Handler, Request, Scope, Type},
    inventory::Item,
    loc_collision::LocCollision,
    simulation::{Action, PlayerData, World, WorldEvent},
    stats::Stats,
    tutorial_fire::Config,
    tutorial_progression::{Config as Progression, Effect},
    variables::{Transmit, Variables, Varp},
    CoordGrid,
};
fn output(events: Vec<WorldEvent>, out: &mut Vec<String>) {
    for event in events {
        match event {
            WorldEvent::PlayerCombatAnimation { sequence, .. } => {
                out.push(format!("anim:{sequence}"))
            }
            WorldEvent::SkillAdvanced {
                skill, advancement, ..
            } => out.push(format!("xp:{skill}:{:.1}", advancement.added_xp as f64)),
            WorldEvent::LocationMessage { message, .. } => out.push(message.into()),
            WorldEvent::TutorialInventoryProgress { effects, .. } => {
                for effect in effects {
                    if let Effect::Focus { step, .. } = effect {
                        out.push(format!("focus:{}", step as i32));
                    }
                }
            }
            _ => {}
        }
    }
}
#[test]
fn unchanged_fire_script_matches_before_and_after_suspension() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-fire.expected"
    ))
    .expect("Run scripts/tutorial-fire-parity.ps1");
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        progression: Progression {
            step: varp(1),
            completed: varp(2),
        },
        logs: 24650,
        location: 4,
        animation: 2,
        skill: 11,
        max_level: 99,
        global_xp_rate: 1.0,
    };
    let origin = CoordGrid::new(3200, 3200, 0).unwrap();
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 9);
        let num = |i: usize| f[i].parse::<i32>().unwrap();
        let mut w = World::default();
        for x in (3192..3216).step_by(8) {
            for z in (3192..3216).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        w.location_types.insert(
            4,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
        for id in [590, 24650] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        w.held_use_handlers
            .register(Scope::Types, 590, 24650, Handler::TutorialFire(config))
            .unwrap();
        let mut p = PlayerData::new(1, "Oracle".into(), origin, Stats::default());
        p.variables = Variables::restore([(1, num(0)), (2, num(1))].into());
        p.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 590,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        if num(2) == 1 {
            p.inventory
                .restore_slot(
                    1,
                    Some(Item {
                        id: 24650,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, p).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.drain_events();
        let request = Request {
            selected_slot: 0,
            selected_id: 590,
            target_slot: 1,
            target_id: 24650,
        };
        w.prepare_held_use(actor, request).unwrap();
        let result = w.finish_held_use(actor, request);
        if num(2) == 1 {
            result.unwrap();
        }
        let mut actual = vec![];
        output(w.drain_events(), &mut actual);
        let suspended = w.protection(actor).unwrap().suspended;
        if suspended {
            assert_eq!(w.protection(actor).unwrap().delay_until - w.clock.0, 4);
            actual.push("delay:4".into());
        }
        assert_eq!(actual.join("~"), f[5], "{line}");
        assert_eq!(
            w.data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 24650)
                .count() as i32,
            num(4),
            "{line}"
        );
        if num(3) == 0 && suspended {
            w.submit(actor, Action::EatSlot { slot: 0 }).unwrap();
        }
        for _ in 0..6 {
            w.tick().unwrap();
            output(w.drain_events(), &mut actual);
        }
        // Route target and location duration are asserted separately below;
        // these are output doubles, not full reference collision/timer execution.
        let expected = f[8]
            .split('~')
            .filter(|s| !s.starts_with("walk:") && !s.starts_with("loc:"))
            .collect::<Vec<_>>()
            .join("~");
        assert_eq!(actual.join("~"), expected, "{line}");
        assert_eq!(w.data(actor).unwrap().variables.get(1), num(6), "{line}");
        assert_eq!(w.data(actor).unwrap().coords.x(), num(7), "{line}");
        assert_eq!(
            w.locations.find_type(origin, 4).is_some(),
            f[8].contains("loc:3200:4:200"),
            "{line}"
        );
        count += 1;
    }
    assert_eq!(count, 896);
}
