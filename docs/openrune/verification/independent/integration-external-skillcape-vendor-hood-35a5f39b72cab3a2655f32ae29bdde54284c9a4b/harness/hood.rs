//! Drive only public World actions and emitted dialogue pages/tokens.
use gielinor_world::{
    dialogue::{Event, Page, Speaker},
    inventory::Item,
    npc::{BlockWalk, Mode, Npc, RandomSource, Restrict},
    npc_options::Definition,
    simulation::{Action, ActorId, NpcId, PlayerData, World, WorldEvent},
    skillcape_vendor::{Config, Kind},
    stats::Stats,
    CoordGrid,
};

fn hood_question(w: &mut World, a: ActorId, n: NpcId) -> u64 {
    w.submit(
        a,
        Action::OperateNpc {
            target: n,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    let mut current = None;
    for _ in 0..4 {
        w.tick_with_random(&mut NoDraw).unwrap();
        if let Some(p) = page(w) {
            current = Some(p);
            break;
        }
    }
    let mut current = current.expect("no initial vendor page");
    for _ in 0..16 {
        let (t, p) = current;
        let choice = match p {
            Page::Chat {
                speaker: Speaker::Player,
                text,
                ..
            } if text.to_lowercase().contains("hood") => return t,
            Page::Choice(options) => options
                .iter()
                .position(|s| s.to_lowercase().contains("hood") || s.contains("Yes"))
                .map(|n| n as i32 + 1)
                .expect("no hood route"),
            Page::Chat { .. } | Page::NamedChat { .. } => -1,
            _ => panic!("unexpected vendor page"),
        };
        current = reply(w, a, t, choice).expect("ended before hood question");
    }
    panic!("no hood question")
}
#[test]
fn hood_two_acknowledgements_and_live_authority() {
    let mut cases = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        for cape in 0..2 {
            for worn in [false, true] {
                for mode in 0..5 {
                    let (mut w, a, n, c) = fixture(kind);
                    if worn {
                        w.data_mut(a)
                            .unwrap()
                            .equipment
                            .restore_slot(
                                1,
                                Some(Item {
                                    id: c.capes[cape],
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    } else {
                        w.data_mut(a)
                            .unwrap()
                            .inventory
                            .restore_slot(
                                0,
                                Some(Item {
                                    id: c.capes[cape],
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    let before = w.data(a).unwrap().clone();
                    let q = hood_question(&mut w, a, n);
                    assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                    let (t, p) = reply(&mut w, a, q, -1).expect("no hood acknowledgement");
                    assert!(matches!(
                        p,
                        Page::Chat {
                            speaker: Speaker::Npc,
                            ..
                        }
                    ));
                    assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                    match mode {
                        0 => {}
                        1 => {
                            if worn {
                                w.data_mut(a)
                                    .unwrap()
                                    .equipment
                                    .restore_slot(1, None)
                                    .unwrap();
                            } else {
                                w.data_mut(a)
                                    .unwrap()
                                    .inventory
                                    .restore_slot(0, None)
                                    .unwrap();
                            }
                        }
                        2 => {
                            w.set_membership(a, false).unwrap();
                        }
                        3 => {
                            w.cancel_dialogue(a).unwrap();
                        }
                        _ => {
                            for slot in 0..28 {
                                w.data_mut(a)
                                    .unwrap()
                                    .inventory
                                    .restore_slot(
                                        slot,
                                        Some(Item {
                                            id: 1931,
                                            count: 1,
                                            vars: 0,
                                        }),
                                    )
                                    .unwrap();
                            }
                            if !worn {
                                w.data_mut(a)
                                    .unwrap()
                                    .inventory
                                    .restore_slot(
                                        0,
                                        Some(Item {
                                            id: c.capes[cape],
                                            count: 1,
                                            vars: 0,
                                        }),
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    let boundary = w.data(a).unwrap().clone();
                    reply(&mut w, a, t, -1);
                    let next = w.data(a).unwrap();
                    assert_eq!(next.stats, boundary.stats);
                    assert_eq!(next.equipment, boundary.equipment);
                    if mode == 0 {
                        assert_eq!(units(&w, a, c.hood), 1);
                        assert_eq!(units(&w, a, 995), 99001);
                        let plan = c
                            .prepare_hood(
                                &boundary.inventory,
                                &boundary.equipment,
                                &w.object_types,
                                &boundary.stats,
                                true,
                            )
                            .unwrap();
                        assert_eq!(next.inventory, plan.inventory);
                    } else {
                        assert_eq!(next.inventory, boundary.inventory);
                    }
                    let once = next.inventory.clone();
                    reply(&mut w, a, t, -1);
                    assert_eq!(w.data(a).unwrap().inventory, once);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 40);
    println!("PG_VENDOR_HOOD40 bothNPCs bothcapes held/worn twoackboundaries then success/livecape/member/Close/space exactzeroCost noXPgear/replay PASS");
}
struct NoDraw;
impl RandomSource for NoDraw {
    fn below(&mut self, _: u32) -> u32 {
        panic!("vendor drew random")
    }
}
fn fixture(kind: Kind) -> (World, ActorId, NpcId, Config) {
    let mut w = World::default();
    for x in [3216, 3224] {
        for z in [3216, 3224] {
            w.collision.allocate(x, z, 0);
        }
    }
    let (skill, npc, capes, hood) = if kind == Kind::Crafting {
        (12, 5810, [9780, 9781], 9782)
    } else {
        (7, 2658, [9801, 9802], 9803)
    };
    let c = Config {
        kind,
        npc,
        skill,
        coins: 995,
        capes,
        hood,
        active_stats: 0x007fffff,
    };
    w.object_types.stackable.insert(995);
    w.npc_handlers.insert(
        (npc, 1),
        gielinor_world::npc_interaction::Handler::SkillcapeVendor(c),
    );
    w.npc_options.types.insert(
        npc,
        Definition {
            operations: [Some("Talk-to".into()), None, None, None, None],
            ..Default::default()
        },
    );
    let mut stats = Stats::default();
    stats.set_base(3, 10);
    stats.set_current(3, 10);
    stats.set_base(skill, 99);
    stats.set_current(skill, 99);
    stats.set_fine_xp(skill, 130344311).unwrap();
    let mut data = PlayerData::new(
        1,
        "Independent vendor dialogue".into(),
        CoordGrid::new(3224, 3224, 0).unwrap(),
        stats,
    );
    data.inventory
        .restore_slot(
            8,
            Some(Item {
                id: 995,
                count: 99001,
                vars: 0,
            }),
        )
        .unwrap();
    data.inventory
        .restore_slot(
            27,
            Some(Item {
                id: 1931,
                count: 1,
                vars: 9,
            }),
        )
        .unwrap();
    w.queue_login(1, data).unwrap();
    w.tick_with_random(&mut NoDraw).unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, true).unwrap();
    let mut n = Npc::new(
        npc,
        CoordGrid::new(3225, 3224, 0).unwrap(),
        1,
        Restrict::Normal,
        BlockWalk::None,
    )
    .unwrap();
    n.mode = Mode::None;
    let id = w.spawn_npc(n).unwrap();
    w.drain_events();
    (w, a, id, c)
}
fn page(w: &mut World) -> Option<(u64, Page)> {
    w.drain_events()
        .into_iter()
        .filter_map(|e| {
            if let WorldEvent::Dialogue {
                event: Event::Page { token, page, .. },
                ..
            } = e
            {
                Some((token, page))
            } else {
                None
            }
        })
        .last()
}
fn reply(w: &mut World, a: ActorId, t: u64, choice: i32) -> Option<(u64, Page)> {
    w.submit(a, Action::ResumeDialogue { token: t, choice })
        .unwrap();
    w.tick_with_random(&mut NoDraw).unwrap();
    page(w)
}
fn quote(w: &mut World, a: ActorId, n: NpcId) -> u64 {
    w.submit(
        a,
        Action::OperateNpc {
            target: n,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    let mut seen = None;
    for _ in 0..4 {
        w.tick_with_random(&mut NoDraw).unwrap();
        if let Some(p) = page(w) {
            seen = Some(p);
            break;
        }
    }
    let mut current = seen.expect("no public dialogue page");
    for _ in 0..16 {
        let (t, p) = current;
        let choice = match p {
            Page::Choice(options) => {
                if options.iter().any(|s| s.contains("99,000")) {
                    return t;
                }
                options
                    .iter()
                    .position(|s| s.contains("Yes") || *s == "Skillcape")
                    .map(|n| n as i32 + 1)
                    .expect("no affirmative choice")
            }
            Page::Chat { .. } | Page::NamedChat { .. } => -1,
            _ => panic!("unexpected page"),
        };
        current = reply(w, a, t, choice).expect("dialogue ended before quote");
    }
    panic!("no price quote")
}
fn units(w: &World, a: ActorId, id: i32) -> i32 {
    w.data(a)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .filter(|v| v.id == id)
        .map(|v| v.count)
        .sum()
}
