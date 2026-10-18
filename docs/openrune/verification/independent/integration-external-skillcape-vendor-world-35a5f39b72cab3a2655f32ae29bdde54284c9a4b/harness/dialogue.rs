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
#[test]
fn both_npc_purchase_boundaries_and_replayed_choice_cannot_charge_twice() {
    for kind in [Kind::Crafting, Kind::Cooking] {
        let (mut w, a, n, c) = fixture(kind);
        let before = w.data(a).unwrap().clone();
        let q = quote(&mut w, a, n);
        assert_eq!(w.data(a).unwrap().inventory, before.inventory);
        let next = reply(&mut w, a, q, 2);
        if kind == Kind::Cooking {
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            let (t, p) = next.expect("no purchase acknowledgement");
            assert!(matches!(
                p,
                Page::Chat {
                    speaker: Speaker::Player,
                    ..
                }
            ));
            reply(&mut w, a, t, -1);
        }
        assert_eq!(units(&w, a, 995), 1);
        assert_eq!(units(&w, a, c.capes[0]), 1);
        assert_eq!(units(&w, a, c.hood), 1);
        assert_eq!(w.data(a).unwrap().stats, before.stats);
        assert_eq!(w.data(a).unwrap().equipment, before.equipment);
        assert_eq!(
            w.data(a).unwrap().inventory.get(27),
            before.inventory.get(27)
        );
        let paid = w.data(a).unwrap().inventory.clone();
        reply(&mut w, a, q, 2);
        assert_eq!(w.data(a).unwrap().inventory, paid);
    }
    println!("PG_VENDOR_DIALOGUE2 actual publicNPC/quote/purchase timing Craftimmediate/CookafterplayerContinue normalcape+hood exact99000 noXPgear noReplay PASS");
}
#[test]
fn late_refusals_at_the_actual_payment_boundary() {
    let mut cases = 0;
    for kind in [Kind::Crafting, Kind::Cooking] {
        for mode in 0..10 {
            let (mut w, a, n, c) = fixture(kind);
            let q = quote(&mut w, a, n);
            let payment = if kind == Kind::Cooking {
                let (t, _) = reply(&mut w, a, q, 2).unwrap();
                t
            } else {
                q
            };
            match mode {
                0 => {
                    w.set_membership(a, false).unwrap();
                }
                1 => w.data_mut(a).unwrap().stats.set_base(c.skill, 98),
                2 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        8,
                        Some(Item {
                            id: 995,
                            count: 98999,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                3 => {
                    for s in 0..28 {
                        if s != 8 {
                            w.data_mut(a)
                                .unwrap()
                                .inventory
                                .restore_slot(
                                    s,
                                    Some(Item {
                                        id: 1931,
                                        count: 1,
                                        vars: 0,
                                    }),
                                )
                                .unwrap();
                        }
                    }
                }
                4 => {
                    w.object_types.dummy.insert(c.hood);
                }
                5 => {
                    w.npc_handlers.remove(&(c.npc, 1));
                }
                6 => {
                    w.npc_mut(n).unwrap().visible = false;
                }
                7 => {
                    w.cancel_dialogue(a).unwrap();
                }
                8 => {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                }
                _ => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            8,
                            Some(Item {
                                id: 995,
                                count: 99001,
                                vars: 7,
                            }),
                        )
                        .unwrap();
                }
            }
            let before = w.data(a).unwrap().clone();
            reply(
                &mut w,
                a,
                payment,
                if kind == Kind::Cooking { -1 } else { 2 },
            );
            assert_eq!(
                w.data(a).unwrap().inventory,
                before.inventory,
                "kind{kind:?} mode{mode}"
            );
            assert_eq!(w.data(a).unwrap().stats, before.stats);
            assert_eq!(w.data(a).unwrap().equipment, before.equipment);
            cases += 1;
        }
    }
    assert_eq!(cases, 20);
    println!("PG_VENDOR_LATE20 bothNPCs actualpayment member/base/money/space/catalog/handler/NPCvisibility/Close/life/coinvars refusals noPartialMutation PASS");
}
