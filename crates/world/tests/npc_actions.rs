use gielinor_world::{
    movement::Speed,
    npc::{BlockWalk, Mode, Npc, Restrict},
    npc_interaction::{Event, Handler},
    npc_options::Definition,
    simulation::{Action, ActorId, NpcId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
fn setup() -> (World, ActorId, NpcId) {
    let mut w = World::default();
    for x in (3192..3224).step_by(8) {
        for z in (3192..3224).step_by(8) {
            w.collision.allocate(x, z, 0);
        }
    }
    let c = |x, z| CoordGrid::new(x, z, 0).unwrap();
    let mut npc = Npc::new(1, c(3205, 3200), 1, Restrict::Normal, BlockWalk::Npc).unwrap();
    npc.mode = Mode::None;
    let target = w.spawn_npc(npc).unwrap();
    w.npc_options.types.insert(
        1,
        Definition {
            operations: [
                Some("Talk-to".into()),
                None,
                Some("Bank".into()),
                None,
                None,
            ],
            ..Default::default()
        },
    );
    w.npc_handlers.insert((1, 3), Handler::Bank);
    w.queue_login(
        1,
        PlayerData::new(1, "Walker".into(), c(3200, 3200), Stats::default()),
    )
    .unwrap();
    w.tick().unwrap();
    let actor = w.actors().next().unwrap();
    w.drain_events();
    (w, actor, target)
}
struct PersonRandom;
#[test]
fn tutorial_fishing_routes_once_and_later_catches_do_not_reapply_focus() {
    use gielinor_world::variables::{Transmit, Varp};
    let (mut w, actor, target) = setup();
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    w.tutorial_fishing = Some(gielinor_world::tutorial_fishing::Config {
        progression: gielinor_world::tutorial_progression::Config {
            step: varp(5003),
            completed: varp(5004),
        },
        shrimp: 55,
        animation: 621,
        skill: 10,
        max_level: 99,
        global_xp_rate: 1.0,
    });
    w.npc_handlers.insert((1, 1), Handler::TutorialFishing);
    w.data_mut(actor)
        .unwrap()
        .variables
        .set(varp(5003), 6, true);
    for catch in 1..=2 {
        w.submit(
            actor,
            Action::OperateNpc {
                target,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        let mut events = vec![];
        for _ in 0..12 {
            w.tick().unwrap();
            events.extend(w.drain_events());
        }
        assert_eq!(w.data(actor).unwrap().stats.xp(10), 10 * catch);
        assert_eq!(w.data(actor).unwrap().variables.get(5003), 8);
        assert_eq!(
            w.data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 55)
                .count(),
            catch as usize
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 10, .. }))
                .count(),
            1
        );
        assert_eq!(events.iter().any(|e|matches!(e,WorldEvent::TutorialInventoryProgress {effects,..} if !effects.is_empty())),catch==1);
    }
}
fn start_quest(w: &mut World, actor: ActorId, target: NpcId, progress: i32) {
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, progress)].into());
    w.npc_handlers
        .insert((1, 1), Handler::QuestTutor { step_varp: 5003 });
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
    panic!("Quest dialogue did not begin");
}
#[test]
fn quest_tutor_captures_start_step_and_final_continue_opens_journal_or_ladder() {
    use gielinor_world::{dialogue::Event as D, tutorial_steps::Step};
    for (initial, pages, expected, tab) in [
        (20, 1, Step::QuestList, Some(2)),
        (21, 5, Step::QuestLadder, None),
    ] {
        let (mut w, actor, target) = setup();
        start_quest(&mut w, actor, target, initial);
        w.data_mut(actor).unwrap().variables =
            gielinor_world::variables::Variables::restore([(5003, 53)].into());
        for _ in 1..pages {
            let events = continue_survival(&mut w, actor, -1);
            assert!(!events.iter().any(|e| matches!(
                e,
                WorldEvent::Dialogue {
                    event: D::TutorialStep { .. },
                    ..
                }
            )));
        }
        let events = continue_survival(&mut w, actor, -1);
        assert!(events.iter().any(|e|matches!(e,WorldEvent::Dialogue {event:D::TutorialStep {step,open_tab,..},..} if *step==expected && *open_tab==tab)));
        assert_eq!(w.dialogue_token(actor).unwrap(), None);
    }
}
#[test]
fn quest_recap_rewinds_later_progress_only_after_last_chat_and_cancellation_is_safe() {
    use gielinor_world::{dialogue::Event as D, tutorial_steps::Step};
    for (choice, pages) in [(1, 2), (2, 1)] {
        let (mut w, actor, target) = setup();
        start_quest(&mut w, actor, target, 53);
        continue_survival(&mut w, actor, -1);
        let token = w.dialogue_token(actor).unwrap();
        continue_survival(&mut w, actor, 3);
        assert_eq!(w.dialogue_token(actor).unwrap(), token);
        continue_survival(&mut w, actor, choice);
        for _ in 1..pages {
            continue_survival(&mut w, actor, -1);
        }
        let events = continue_survival(&mut w, actor, -1);
        assert!(events.iter().any(|e| matches!(
            e,
            WorldEvent::Dialogue {
                event: D::TutorialStep {
                    step: Step::QuestLadder,
                    open_tab: None,
                    ..
                },
                ..
            }
        )));
    }
    let (mut w, actor, target) = setup();
    start_quest(&mut w, actor, target, 20);
    assert!(w.cancel_dialogue(actor).unwrap());
    w.tick().unwrap();
    assert!(!w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: D::TutorialStep { .. },
            ..
        }
    )));
}
fn start_chef(w: &mut World, actor: ActorId, target: NpcId, progress: i32) {
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, progress)].into());
    w.npc_handlers.insert(
        (1, 1),
        Handler::MasterChef(gielinor_world::tutorial_chef::Config {
            step_varp: 5003,
            flour: 1933,
            water: 1929,
        }),
    );
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
    panic!("Chef dialogue did not begin");
}
#[test]
fn chef_checks_ingredients_at_final_continue_and_commits_each_independently() {
    use gielinor_world::{dialogue::Event as D, inventory::Item, tutorial_steps::Step};
    for free in 0..=2 {
        let (mut w, actor, target) = setup();
        start_chef(&mut w, actor, target, 16);
        let inv = &mut w.data_mut(actor).unwrap().inventory;
        for slot in 0..(28 - free) {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 995,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let events = continue_survival(&mut w, actor, -1);
        let inv = &w.data(actor).unwrap().inventory;
        assert_eq!(
            inv.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 1933)
                .count(),
            usize::from(free >= 1)
        );
        assert_eq!(
            inv.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 1929)
                .count(),
            usize::from(free >= 2)
        );
        assert!(events.iter().any(|e| matches!(
            e,
            WorldEvent::Dialogue {
                event: D::TutorialStep {
                    step: Step::ChefDough,
                    ..
                },
                ..
            }
        )));
    }
    let (mut w, actor, target) = setup();
    start_chef(&mut w, actor, target, 15);
    continue_survival(&mut w, actor, -1);
    continue_survival(&mut w, actor, -1);
    // Items arriving while dialogue is suspended must suppress duplicate gifts.
    let inv = &mut w.data_mut(actor).unwrap().inventory;
    inv.restore_slot(
        0,
        Some(Item {
            id: 1933,
            count: 1,
            vars: 0,
        }),
    )
    .unwrap();
    inv.restore_slot(
        1,
        Some(Item {
            id: 1929,
            count: 1,
            vars: 0,
        }),
    )
    .unwrap();
    continue_survival(&mut w, actor, -1);
    assert_eq!(
        w.data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .count(),
        2
    );
}
#[test]
fn chef_declining_recap_finishes_immediately_and_reads_current_progress() {
    use gielinor_world::{dialogue::Event as D, tutorial_steps::Step};
    for progress in [2, 17, 18, 53] {
        let (mut w, actor, target) = setup();
        start_chef(&mut w, actor, target, 53);
        continue_survival(&mut w, actor, -1);
        w.data_mut(actor).unwrap().variables =
            gielinor_world::variables::Variables::restore([(5003, progress)].into());
        let token = w.dialogue_token(actor).unwrap();
        continue_survival(&mut w, actor, 0);
        assert_eq!(w.dialogue_token(actor).unwrap(), token);
        let events = continue_survival(&mut w, actor, 3);
        assert_eq!(
            events.iter().any(|e| matches!(
                e,
                WorldEvent::Dialogue {
                    event: D::TutorialStep {
                        step: Step::ChefExit,
                        ..
                    },
                    ..
                }
            )),
            progress >= 18
        );
        assert_eq!(w.dialogue_token(actor).unwrap(), None);
        assert!(!w.protection(actor).unwrap().suspended);
    }
}
#[test]
fn chef_cancel_before_final_continue_does_not_grant_ingredients() {
    let (mut w, actor, target) = setup();
    start_chef(&mut w, actor, target, 15);
    continue_survival(&mut w, actor, -1);
    assert!(w.cancel_dialogue(actor).unwrap());
    w.tick().unwrap();
    assert!(w
        .data(actor)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .all(Option::is_none));
}
fn start_survival(w: &mut World, actor: ActorId, target: NpcId, progress: i32) {
    use gielinor_world::variables::{Transmit, Varp};
    w.data_mut(actor).unwrap().variables.set(
        Varp {
            id: 5003,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        progress,
        false,
    );
    w.npc_handlers.insert(
        (1, 1),
        Handler::SurvivalTutor(gielinor_world::tutorial_survival::Config {
            step_varp: 5003,
            net: 303,
            axe: 1351,
            tinderbox: 590,
        }),
    );
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
    panic!("Survival dialogue did not begin");
}
fn continue_survival(w: &mut World, actor: ActorId, choice: i32) -> Vec<WorldEvent> {
    let token = w.dialogue_token(actor).unwrap().unwrap();
    w.submit(actor, Action::ResumeDialogue { token, choice })
        .unwrap();
    // Duplicate resume must not grant duplicate items or select twice.
    w.submit(actor, Action::ResumeDialogue { token, choice })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    w.drain_events()
}
#[test]
fn survival_gifts_wait_for_continue_and_tools_commit_independently() {
    use gielinor_world::{dialogue::Event as D, inventory::Item, tutorial_steps::Step};
    for (progress, free, next, items) in [
        (5, 1, Step::SurvivalInv, vec![303]),
        (9, 1, Step::SurvivalWc, vec![1351]),
        (9, 2, Step::SurvivalWc, vec![1351, 590]),
        (5, 0, Step::SurvivalInv, vec![]),
    ] {
        let (mut w, actor, target) = setup();
        start_survival(&mut w, actor, target, progress);
        continue_survival(&mut w, actor, -1);
        assert_eq!(w.data(actor).unwrap().inventory.free_space(), 28);
        // Change inventory while suspended, then acknowledge the final page.
        for slot in 0..28 - free {
            w.data_mut(actor)
                .unwrap()
                .inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 999,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let events = continue_survival(&mut w, actor, -1);
        let granted: Vec<_> = w
            .data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|item| item.id != 999)
            .map(|item| item.id)
            .collect();
        assert_eq!(granted, items);
        assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::Dialogue {event:D::Survival {step,..},..} if *step==next)).count(),1);
        assert!(w.dialogue_token(actor).unwrap().is_none());
    }
}
#[test]
fn survival_recap_uses_current_progress_and_invalid_choices_keep_token() {
    use gielinor_world::{
        dialogue::Event as D,
        variables::{Transmit, Varp},
    };
    for progress in [2, 13] {
        let (mut w, actor, target) = setup();
        start_survival(&mut w, actor, target, 53);
        continue_survival(&mut w, actor, -1);
        let token = w.dialogue_token(actor).unwrap();
        continue_survival(&mut w, actor, 0);
        assert_eq!(w.dialogue_token(actor).unwrap(), token);
        continue_survival(&mut w, actor, 5);
        w.data_mut(actor).unwrap().variables.set(
            Varp {
                id: 5003,
                permanent: true,
                transmit: Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
            progress,
            false,
        );
        continue_survival(&mut w, actor, -1);
        let events = continue_survival(&mut w, actor, -1);
        assert_eq!(
            events.iter().any(|e| matches!(
                e,
                WorldEvent::Dialogue {
                    event: D::Survival { .. },
                    ..
                }
            )),
            progress == 13
        );
        assert!(w.dialogue_token(actor).unwrap().is_none());
    }
}
#[test]
fn survival_cancel_before_final_page_does_not_grant_items() {
    let (mut w, actor, target) = setup();
    start_survival(&mut w, actor, target, 5);
    continue_survival(&mut w, actor, -1);
    let token = w.dialogue_token(actor).unwrap().unwrap();
    w.cancel_dialogue(actor).unwrap();
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    assert_eq!(w.data(actor).unwrap().inventory.free_space(), 28);
    assert!(!w.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: gielinor_world::dialogue::Event::Survival { .. },
            ..
        }
    )));
}
struct PersonOutcome {
    flyer: bool,
    response: u32,
}
impl gielinor_world::npc::RandomSource for PersonOutcome {
    fn below(&mut self, exclusive: u32) -> u32 {
        if exclusive == 128 {
            u32::from(!self.flyer)
        } else {
            self.response % exclusive
        }
    }
}
#[test]
fn flyer_is_awarded_once_after_acknowledgement_and_existing_flyer_excludes_reward() {
    use gielinor_world::dialogue::{Event as D, Page};
    let (mut w, actor, target) = setup();
    w.person_flyer = Some(956);
    let token = start_person(&mut w, actor, target);
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonOutcome {
        flyer: true,
        response: 0,
    })
    .unwrap();
    assert!(w
        .data(actor)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .all(Option::is_none));
    let token = w.dialogue_token(actor).unwrap().unwrap();
    assert!(w.drain_events().iter().any(|event| matches!(
        event,
        WorldEvent::Dialogue {
            event: D::Page {
                page: Page::Chat {
                    text: "Have this flyer...",
                    ..
                },
                ..
            },
            ..
        }
    )));
    for _ in 0..2 {
        w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
            .unwrap();
    }
    w.tick_with_random(&mut PersonOutcome {
        flyer: true,
        response: 0,
    })
    .unwrap();
    assert_eq!(
        w.data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|item| item.id == 956)
            .map(|item| item.count)
            .sum::<i32>(),
        1
    );
    assert_eq!(w.dialogue_token(actor).unwrap(), None);
    let token = start_person(&mut w, actor, target);
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonOutcome {
        flyer: true,
        response: 0,
    })
    .unwrap();
    assert!(w.drain_events().iter().any(|event| matches!(
        event,
        WorldEvent::Dialogue {
            event: D::Page {
                page: Page::Chat {
                    text: "I'm fine, how are you?",
                    ..
                },
                ..
            },
            ..
        }
    )));
}
#[test]
fn hostile_reply_retaliates_only_after_continue() {
    let (mut w, actor, target) = setup();
    let token = start_person(&mut w, actor, target);
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonOutcome {
        flyer: false,
        response: 17,
    })
    .unwrap();
    assert!(w.npc(target).unwrap().retaliate_due.is_empty());
    let token = w.dialogue_token(actor).unwrap().unwrap();
    for _ in 0..2 {
        w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
            .unwrap();
    }
    w.tick_with_random(&mut PersonOutcome {
        flyer: false,
        response: 17,
    })
    .unwrap();
    assert_eq!(w.npc(target).unwrap().aggressive_player, Some(actor));
    assert_eq!(w.npc(target).unwrap().retaliate_due.len(), 1);
    assert_eq!(w.dialogue_token(actor).unwrap(), None);
}
#[test]
fn npc_chat_face_holds_route_until_the_player_leaves_close_range() {
    let (mut w, actor, target) = setup();
    let token = start_person(&mut w, actor, target);
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    assert_eq!(w.npc(target).unwrap().dialogue_face, Some(actor));
    assert_eq!(w.npc(target).unwrap().face_player, Some(actor));
    w.submit(
        actor,
        Action::MoveClick {
            destination: CoordGrid::new(3200, 3200, 0).unwrap(),
            control: false,
        },
    )
    .unwrap();
    for _ in 0..6 {
        w.tick_with_random(&mut PersonRandom).unwrap();
    }
    assert_eq!(w.npc(target).unwrap().dialogue_face, None);
    assert_eq!(w.npc(target).unwrap().face_player, None);
}
#[test]
fn structured_movement_cancels_dialogue_and_its_continuation() {
    let (mut w, actor, target) = setup();
    let token = start_person(&mut w, actor, target);
    w.submit(
        actor,
        Action::MoveClick {
            destination: CoordGrid::new(3203, 3201, 0).unwrap(),
            control: false,
        },
    )
    .unwrap();
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    let dialogue: Vec<_> = w
        .drain_events()
        .into_iter()
        .filter_map(|event| match event {
            WorldEvent::Dialogue { event, .. } => Some(event),
            _ => None,
        })
        .collect();
    assert_eq!(dialogue.len(), 1);
    assert!(matches!(
        dialogue[0],
        gielinor_world::dialogue::Event::Closed
    ));
    assert!(!w.protection(actor).unwrap().suspended);
}
impl gielinor_world::npc::RandomSource for PersonRandom {
    fn below(&mut self, exclusive: u32) -> u32 {
        if exclusive == 128 {
            1
        } else {
            2 % exclusive
        }
    }
}
fn start_person(w: &mut World, actor: ActorId, target: NpcId) -> u64 {
    w.npc_handlers.insert((1, 1), Handler::Person);
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        for event in w.drain_events() {
            if let WorldEvent::Dialogue {
                event: gielinor_world::dialogue::Event::Page { token, page, .. },
                ..
            } = event
            {
                assert!(matches!(
                    page,
                    gielinor_world::dialogue::Page::Chat {
                        speaker: gielinor_world::dialogue::Speaker::Player,
                        ..
                    }
                ));
                return token;
            }
        }
    }
    panic!("person interaction did not arrive");
}
#[test]
fn tutorial_guide_progress_requires_last_continue_and_captures_initial_step() {
    use gielinor_world::{
        dialogue::Event as D,
        tutorial_steps::Step,
        variables::{Transmit, Varp},
    };
    for (initial, count, next, switch) in [
        (2, 4, Step::GielinorSettings, true),
        (3, 2, Step::GielinorDoor, false),
        (53, 3, Step::GielinorDoor, false),
    ] {
        let (mut w, actor, target) = setup();
        let ty = Varp {
            id: 5003,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        w.data_mut(actor).unwrap().variables.set(ty, initial, false);
        w.npc_handlers
            .insert((1, 1), Handler::TutorialGuide { step_varp: 5003 });
        w.submit(
            actor,
            Action::OperateNpc {
                target,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        let mut token = None;
        for _ in 0..8 {
            w.tick_with_random(&mut PersonRandom).unwrap();
            for event in w.drain_events() {
                if let WorldEvent::Dialogue {
                    event:
                        D::Page {
                            token: page_token, ..
                        },
                    ..
                } = event
                {
                    token = Some(page_token);
                }
            }
            if token.is_some() {
                break;
            }
        }
        let mut token = token.expect("guide did not start");
        // The Kotlin coroutine chooses its branch before suspending.
        w.data_mut(actor).unwrap().variables.set(ty, 45, false);
        for page in 0..count {
            for _ in 0..2 {
                w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
                    .unwrap();
            }
            w.tick_with_random(&mut PersonRandom).unwrap();
            let events: Vec<_> = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::Dialogue { event, .. } = e {
                        Some(event)
                    } else {
                        None
                    }
                })
                .collect();
            if page + 1 == count {
                assert!(
                    matches!(events.as_slice(), [D::TutorialGuide { npc, step, open_settings }, D::Closed] if *npc == target && *step == next && *open_settings == switch)
                );
                assert!(!w.protection(actor).unwrap().suspended);
            } else {
                assert_eq!(events.len(), 1, "duplicate Continue must not advance twice");
                let D::Page {
                    token: next_token, ..
                } = events[0]
                else {
                    panic!("premature tutorial effect");
                };
                token = next_token;
            }
        }
    }
}

#[test]
fn closing_tutorial_guide_before_continue_never_advances_progress() {
    let (mut w, actor, target) = setup();
    w.npc_handlers
        .insert((1, 1), Handler::TutorialGuide { step_varp: 5003 });
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            break;
        }
    }
    let token = w.dialogue_token(actor).unwrap().unwrap();
    w.cancel_dialogue(actor).unwrap();
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    assert!(!w
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::Dialogue { .. })));
    assert!(!w.protection(actor).unwrap().suspended);
}
#[test]
fn duplicate_continuations_cannot_skip_pages_and_closed_dialogue_cannot_resume() {
    use gielinor_world::dialogue::{Event as D, Page};
    let (mut w, actor, target) = setup();
    let token = start_person(&mut w, actor, target);
    assert!(w.protection(actor).unwrap().suspended);
    for _ in 0..2 {
        w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
            .unwrap();
    }
    w.tick_with_random(&mut PersonRandom).unwrap();
    let pages: Vec<_> = w
        .drain_events()
        .into_iter()
        .filter_map(|event| match event {
            WorldEvent::Dialogue {
                event: D::Page { token, page, .. },
                ..
            } => Some((token, page)),
            _ => None,
        })
        .collect();
    assert_eq!(pages.len(), 1);
    assert!(matches!(
        &pages[0].1,
        Page::Chat {
            text: "How can I help you?",
            ..
        }
    ));
    let token = pages[0].0;
    assert!(w.cancel_dialogue(actor).unwrap());
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    assert!(!w
        .drain_events()
        .iter()
        .any(|event| matches!(event, WorldEvent::Dialogue { .. })));
    assert!(!w.protection(actor).unwrap().suspended);
}
#[test]
fn dialogue_captures_npc_generation_and_rejects_invalid_chat_choices() {
    let (mut w, actor, target) = setup();
    let token = start_person(&mut w, actor, target);
    w.submit(actor, Action::ResumeDialogue { token, choice: 1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    assert!(!w
        .drain_events()
        .iter()
        .any(|event| matches!(event, WorldEvent::Dialogue { .. })));
    let npc = w.despawn_npc(target).unwrap();
    assert_ne!(w.spawn_npc(npc).unwrap(), target);
    w.submit(actor, Action::ResumeDialogue { token, choice: -1 })
        .unwrap();
    w.tick_with_random(&mut PersonRandom).unwrap();
    assert!(w.drain_events().iter().any(|event| matches!(
        event,
        WorldEvent::Dialogue {
            event: gielinor_world::dialogue::Event::Closed,
            ..
        }
    )));
    assert!(!w.protection(actor).unwrap().suspended);
}
#[test]
fn banker_op_routes_until_two_tile_approach_then_stops() {
    let (mut w, actor, target) = setup();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 3,
            control: false,
        },
    )
    .unwrap();
    let mut bank = 0;
    for _ in 0..6 {
        w.tick().unwrap();
        bank += w
            .drain_events()
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    WorldEvent::NpcInteraction {
                        event: Event::Bank,
                        ..
                    }
                )
            })
            .count();
    }
    assert_eq!(bank, 1);
    assert_eq!(w.observe(actor).unwrap().coords.x(), 3203);
}
#[test]
fn recycled_npc_index_does_not_inherit_a_pending_interaction() {
    let (mut w, actor, target) = setup();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 3,
            control: false,
        },
    )
    .unwrap();
    w.tick().unwrap();
    w.drain_events();
    let npc = w.despawn_npc(target).unwrap();
    let replacement = w.spawn_npc(npc).unwrap();
    assert_ne!(target, replacement);
    for _ in 0..5 {
        w.tick().unwrap();
        assert!(!w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::NpcInteraction {
                event: Event::Bank,
                ..
            }
        )));
    }
}
#[test]
fn invalid_option_preserves_existing_route_and_walk_cancels_valid_target() {
    let (mut w, actor, target) = setup();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 3,
            control: false,
        },
    )
    .unwrap();
    w.tick().unwrap();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 2,
            control: false,
        },
    )
    .unwrap();
    w.tick().unwrap();
    assert_eq!(w.observe(actor).unwrap().coords.x(), 3202);
    w.submit(
        actor,
        Action::MoveClick {
            destination: CoordGrid::new(3202, 3203, 0).unwrap(),
            control: false,
        },
    )
    .unwrap();
    for _ in 0..5 {
        w.tick().unwrap();
        assert!(!w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::NpcInteraction {
                event: Event::Bank,
                ..
            }
        )));
    }
    assert_eq!(w.observe(actor).unwrap().coords.z(), 3203);
}
#[test]
fn moving_target_is_followed_and_control_click_uses_temporary_run() {
    let (mut w, actor, target) = setup();
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 3,
            control: true,
        },
    )
    .unwrap();
    w.tick().unwrap();
    assert_eq!(w.observe(actor).unwrap().coords.x(), 3202);
    w.npc_mut(target)
        .unwrap()
        .movement
        .route([CoordGrid::new(3210, 3200, 0).unwrap()], Speed::Walk);
    let mut bank = 0;
    for _ in 0..12 {
        w.tick().unwrap();
        bank += w
            .drain_events()
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    WorldEvent::NpcInteraction {
                        event: Event::Bank,
                        ..
                    }
                )
            })
            .count();
    }
    assert_eq!(bank, 1);
}

fn start_mining(w: &mut World, actor: ActorId, target: NpcId, progress: i32) {
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, progress)].into());
    w.npc_handlers.insert(
        (1, 1),
        Handler::MiningTutor(gielinor_world::tutorial_mining_tutor::Config {
            step_varp: 5003,
            pickaxe: 1265,
            hammer: 2347,
        }),
    );
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
    panic!("Mining Instructor did not begin");
}
fn owns(w: &World, actor: ActorId, id: i32) -> bool {
    w.data(actor)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .any(|item| item.id == id)
}
#[test]
fn mining_first_talk_defers_pickaxe_and_captures_branch_even_if_progress_changes() {
    use gielinor_world::{
        dialogue::{Event as D, Page, Speaker},
        tutorial_steps::Step,
    };
    let (mut w, actor, target) = setup();
    start_mining(&mut w, actor, target, 23);
    let events = continue_survival(&mut w, actor, -1);
    assert!(events.iter().any(|event| matches!(
        event,
        WorldEvent::Dialogue {
            event: D::Page {
                page: Page::NamedChat {
                    speaker: Speaker::Player,
                    text: "You can call me ${player.displayName}.",
                    ..
                },
                ..
            },
            ..
        }
    )));
    assert!(!owns(&w, actor, 1265));
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, 53)].into());
    continue_survival(&mut w, actor, -1);
    continue_survival(&mut w, actor, -1);
    assert!(!owns(&w, actor, 1265));
    let events = continue_survival(&mut w, actor, -1);
    assert!(owns(&w, actor, 1265));
    assert!(events.iter().any(|event| matches!(
        event,
        WorldEvent::Dialogue {
            event: D::TutorialStep {
                step: Step::MineOres,
                ..
            },
            ..
        }
    )));
}
#[test]
fn mining_hammer_checks_inventory_at_final_continue_and_full_inventory_blocks_progress() {
    use gielinor_world::{dialogue::Event as D, inventory::Item, tutorial_steps::Step};
    for held_hammer in [false, true] {
        let (mut w, actor, target) = setup();
        start_mining(&mut w, actor, target, 26);
        continue_survival(&mut w, actor, -1);
        for slot in 0..28 {
            w.data_mut(actor)
                .unwrap()
                .inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if held_hammer && slot == 0 { 2347 } else { 995 },
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let events = continue_survival(&mut w, actor, -1);
        assert_eq!(
            events.iter().any(|event| matches!(
                event,
                WorldEvent::Dialogue {
                    event: D::TutorialStep {
                        step: Step::MineDagger,
                        ..
                    },
                    ..
                }
            )),
            held_hammer
        );
        assert_eq!(
            events.iter().any(|event| matches!(
                event,
                WorldEvent::NpcInteraction {
                    event: Event::Message(gielinor_world::tutorial_mining_tutor::NO_ROOM),
                    ..
                }
            )),
            !held_hammer
        );
        assert_eq!(w.dialogue_token(actor).unwrap(), None);
    }
    let (mut w, actor, target) = setup();
    start_mining(&mut w, actor, target, 26);
    continue_survival(&mut w, actor, -1);
    assert!(!owns(&w, actor, 2347));
    continue_survival(&mut w, actor, -1);
    assert!(owns(&w, actor, 2347));
}
#[test]
fn mining_recap_grants_before_first_page_and_rechecks_progress_after_all_choices() {
    use gielinor_world::{dialogue::Event as D, tutorial_steps::Step};
    for (choice, pages) in [(1, 6), (2, 6), (3, 6), (4, 1)] {
        for current in [2, 27, 28, 53] {
            let (mut w, actor, target) = setup();
            start_mining(&mut w, actor, target, 53);
            assert!(owns(&w, actor, 2347));
            continue_survival(&mut w, actor, -1);
            let token = w.dialogue_token(actor).unwrap();
            continue_survival(&mut w, actor, 5);
            assert_eq!(w.dialogue_token(actor).unwrap(), token);
            continue_survival(&mut w, actor, choice);
            w.data_mut(actor).unwrap().variables =
                gielinor_world::variables::Variables::restore([(5003, current)].into());
            for _ in 1..pages {
                continue_survival(&mut w, actor, -1);
            }
            let events = continue_survival(&mut w, actor, -1);
            assert_eq!(
                events.iter().any(|event| matches!(
                    event,
                    WorldEvent::Dialogue {
                        event: D::TutorialStep {
                            step: Step::MineGate,
                            ..
                        },
                        ..
                    }
                )),
                current >= 28
            );
            assert_eq!(w.dialogue_token(actor).unwrap(), None);
        }
    }
}
#[test]
fn mining_cancellation_does_not_grant_deferred_tools_but_keeps_recap_grant() {
    for (progress, granted) in [(23, false), (26, false), (28, true)] {
        let (mut w, actor, target) = setup();
        start_mining(&mut w, actor, target, progress);
        w.cancel_dialogue(actor).unwrap();
        w.tick().unwrap();
        assert!(!owns(&w, actor, 1265));
        assert_eq!(owns(&w, actor, 2347), granted);
    }
}

fn start_combat_tutor(w: &mut World, actor: ActorId, target: NpcId, progress: i32) {
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, progress)].into());
    w.npc_handlers.insert(
        (1, 1),
        Handler::CombatTutor(gielinor_world::tutorial_combat_tutor::Config {
            step_varp: 5003,
            dagger: 1205,
            sword: 1277,
            shield: 1171,
            bow: 841,
            arrows: 882,
        }),
    );
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
    panic!("Combat Instructor did not begin");
}
#[test]
fn combat_tutor_waits_for_continue_then_grants_gear_and_steps_even_when_bag_is_full() {
    use gielinor_world::{dialogue::Event as D, inventory::Item, tutorial_steps::Step};
    for free in 0..=2 {
        let (mut w, actor, target) = setup();
        w.data_mut(actor)
            .unwrap()
            .equipment
            .restore_slot(
                3,
                Some(Item {
                    id: 1205,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        start_combat_tutor(&mut w, actor, target, 32);
        assert!(!owns(&w, actor, 1277));
        for slot in 0..28 - free {
            w.data_mut(actor)
                .unwrap()
                .inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 995,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let events = continue_survival(&mut w, actor, -1);
        assert_eq!(owns(&w, actor, 1277), free >= 1);
        assert_eq!(owns(&w, actor, 1171), free >= 2);
        assert!(events.iter().any(|event| matches!(
            event,
            WorldEvent::Dialogue {
                event: D::TutorialStep {
                    step: Step::CombatGear,
                    ..
                },
                ..
            }
        )));
    }
}
#[test]
fn combat_tutor_worn_tab_and_cancellation_preserve_dialogue_timing() {
    use gielinor_world::{dialogue::Event as D, tutorial_steps::Step};
    let (mut w, actor, target) = setup();
    start_combat_tutor(&mut w, actor, target, 29);
    for _ in 0..3 {
        let events = continue_survival(&mut w, actor, -1);
        assert!(!events.iter().any(|event| matches!(
            event,
            WorldEvent::Dialogue {
                event: D::TutorialStep { .. },
                ..
            }
        )));
    }
    let events = continue_survival(&mut w, actor, -1);
    assert!(events.iter().any(|event| matches!(
        event,
        WorldEvent::Dialogue {
            event: D::TutorialStep {
                step: Step::CombatWorn,
                open_tab: Some(4),
                ..
            },
            ..
        }
    )));
    start_combat_tutor(&mut w, actor, target, 36);
    w.cancel_dialogue(actor).unwrap();
    w.tick().unwrap();
    assert!(!owns(&w, actor, 841));
    assert!(!owns(&w, actor, 882));
}

fn start_magic_tutor(w: &mut World, actor: ActorId, target: NpcId, progress: i32) {
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, progress)].into());
    w.object_types.stackable.extend([556, 558]);
    w.npc_handlers.insert(
        (1, 1),
        Handler::MagicTutor(gielinor_world::tutorial_magic_tutor::Config {
            step_varp: 5003,
            air: 556,
            mind: 558,
        }),
    );
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
}
#[test]
fn magic_tutor_grants_at_resume_and_rechecks_both_rune_stacks() {
    use gielinor_world::{dialogue::Event as D, inventory::Item, tutorial_steps::Step};
    let (mut w, actor, target) = setup();
    start_magic_tutor(&mut w, actor, target, 48);
    assert!(!owns(&w, actor, 556));
    w.data_mut(actor)
        .unwrap()
        .inventory
        .restore_slot(
            0,
            Some(Item {
                id: 556,
                count: 25,
                vars: 0,
            }),
        )
        .unwrap();
    let events = continue_survival(&mut w, actor, -1);
    assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().count, 25);
    assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().count, 5);
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: D::TutorialStep {
                step: Step::MagicRunes,
                open_tab: None,
                ..
            },
            ..
        }
    )));
    assert!(w.dialogue_token(actor).unwrap().is_none());
}
#[test]
fn magic_tutor_custom_titles_and_self_teleport_branch_keep_suspension_boundaries() {
    use gielinor_world::{
        dialogue::{Event as D, Page},
        tutorial_magic_tutor as magic,
        tutorial_steps::Step,
    };
    let (mut w, actor, target) = setup();
    start_magic_tutor(&mut w, actor, target, 51);
    let events = continue_survival(&mut w, actor, -1);
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: D::Page {
                page: Page::TitledChoice {
                    title: magic::GO_TITLE,
                    ..
                },
                ..
            },
            ..
        }
    )));
    let token = w.dialogue_token(actor).unwrap();
    continue_survival(&mut w, actor, 3);
    assert_eq!(w.dialogue_token(actor).unwrap(), token);
    let events = continue_survival(&mut w, actor, 1);
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: D::Page {
                page: Page::TitledChoice {
                    title: magic::PATH_TITLE,
                    ..
                },
                ..
            },
            ..
        }
    )));
    continue_survival(&mut w, actor, 3);
    let events = continue_survival(&mut w, actor, -1);
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: D::Page {
                page: Page::Choice(_),
                ..
            },
            ..
        }
    )));
    let events = continue_survival(&mut w, actor, 2);
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::Dialogue {
            event: D::TutorialStep {
                step: Step::HomeTele,
                open_tab: Some(6),
                ..
            },
            ..
        }
    )));
    assert!(w.dialogue_token(actor).unwrap().is_none());
}
#[test]
fn magic_tutor_missing_runes_refill_without_a_dialogue_and_cancelled_gift_is_not_applied() {
    let (mut w, actor, target) = setup();
    start_magic_tutor(&mut w, actor, target, 49);
    assert!(w.dialogue_token(actor).unwrap().is_none());
    assert!(owns(&w, actor, 556) && owns(&w, actor, 558));
    let (mut w, actor, target) = setup();
    start_magic_tutor(&mut w, actor, target, 48);
    assert!(w.cancel_dialogue(actor).unwrap());
    assert!(!owns(&w, actor, 556) && !owns(&w, actor, 558));
}

#[test]
fn magic_tutor_direct_confirmation_uses_captured_branch_and_already_admitted_access() {
    use gielinor_world::{
        tutorial_leave, tutorial_progression,
        variables::{Transmit, Variables, Varp},
    };
    let (mut w, actor, target) = setup();
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    w.tutorial_leave = Some(tutorial_leave::Config {
        progression: tutorial_progression::Config {
            step: varp(5003),
            completed: varp(5004),
        },
        items: std::array::from_fn(|i| 100 + i as i32),
        coins: 995,
    });
    w.object_types.stackable.extend(109..=114);
    w.object_types.stackable.insert(995);
    start_magic_tutor(&mut w, actor, target, 51);
    continue_survival(&mut w, actor, -1);
    continue_survival(&mut w, actor, 1);
    continue_survival(&mut w, actor, 3);
    continue_survival(&mut w, actor, -1);
    // Kotlin captures the entry branch, then invokes giveLeaveKit unconditionally.
    w.data_mut(actor).unwrap().variables = Variables::restore([(5003, 0), (5004, 1)].into());
    let events = continue_survival(&mut w, actor, 1);
    let data = w.data(actor).unwrap();
    assert_eq!(data.coords, tutorial_leave::Config::destination());
    assert_eq!(
        (data.variables.get(5003), data.variables.get(5004)),
        (53, 1)
    );
    assert_eq!(data.inventory.get(0).unwrap().id, 100);
    assert_eq!(data.bank.get(0).unwrap().count, 25);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, WorldEvent::TutorialLeaveProgress { .. }))
            .count(),
        1
    );
    assert!(w.dialogue_token(actor).unwrap().is_none());
}

fn start_lecture_tutor(
    w: &mut World,
    actor: ActorId,
    target: NpcId,
    kind: gielinor_world::tutorial_lecture_tutor::Kind,
    raw: i32,
) {
    w.data_mut(actor).unwrap().variables =
        gielinor_world::variables::Variables::restore([(5003, raw)].into());
    w.npc_handlers.insert(
        (1, 1),
        Handler::LectureTutor(gielinor_world::tutorial_lecture_tutor::Config {
            kind,
            step_varp: 5003,
        }),
    );
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation: 1,
            control: false,
        },
    )
    .unwrap();
    for _ in 0..8 {
        w.tick_with_random(&mut PersonRandom).unwrap();
        w.drain_events();
        if w.dialogue_token(actor).unwrap().is_some() {
            return;
        }
    }
    panic!("Lecture tutor did not begin");
}
#[test]
fn lecture_tutors_defer_progress_to_final_continue_despite_progress_changes() {
    use gielinor_world::{
        dialogue::Event as D, tutorial_lecture_tutor::Kind, tutorial_steps::Step,
        variables::Variables,
    };
    for (kind, raw, pages, step, tab) in [
        (Kind::Prayer, 38, 2, Step::PrayerTab, Some(5)),
        (Kind::Prayer, 39, 4, Step::PrayerExit, None),
        (Kind::Account, 43, 5, Step::AccountMgmt, Some(8)),
        (Kind::Account, 44, 16, Step::AccountExit, None),
    ] {
        let (mut w, actor, target) = setup();
        start_lecture_tutor(&mut w, actor, target, kind, raw);
        w.data_mut(actor).unwrap().variables = Variables::restore([(5003, 53)].into());
        for page in 0..pages {
            let events = continue_survival(&mut w, actor, -1);
            let updates: Vec<_> = events
                .iter()
                .filter_map(|e| match e {
                    WorldEvent::Dialogue {
                        event: D::TutorialStep { step, open_tab, .. },
                        ..
                    } => Some((*step, *open_tab)),
                    _ => None,
                })
                .collect();
            assert_eq!(
                updates,
                if page + 1 == pages {
                    vec![(step, tab)]
                } else {
                    vec![]
                }
            );
        }
        assert!(w.dialogue_token(actor).unwrap().is_none());
    }
}
#[test]
fn lecture_recap_decline_has_distinct_completion_timing() {
    use gielinor_world::{dialogue::Event as D, tutorial_lecture_tutor::Kind};
    for kind in [Kind::Prayer, Kind::Account] {
        let (mut w, actor, target) = setup();
        start_lecture_tutor(&mut w, actor, target, kind, 53);
        continue_survival(&mut w, actor, -1);
        let events = continue_survival(&mut w, actor, 2);
        let completed = events.iter().any(|e| {
            matches!(
                e,
                WorldEvent::Dialogue {
                    event: D::TutorialStep { .. },
                    ..
                }
            )
        });
        assert_eq!(completed, kind == Kind::Account);
        if kind == Kind::Prayer {
            let events = continue_survival(&mut w, actor, -1);
            assert!(events.iter().any(|e| matches!(
                e,
                WorldEvent::Dialogue {
                    event: D::TutorialStep { .. },
                    ..
                }
            )));
        }
        assert!(w.dialogue_token(actor).unwrap().is_none());
    }
}

fn tanner_setup() -> (World, ActorId, NpcId) {
    use gielinor_world::{
        tower_diary::Progress,
        tower_tanning::{Config, HandlerConfig},
    };
    let (mut w, actor, target) = setup();
    w.tower_tanning = Some(HandlerConfig {
        origin: CoordGrid::new(3192, 3192, 0).unwrap(),
        inventory: Config {
            hides: [1739].into(),
            leather: 1741,
        },
        skills: [12, 13, 11],
        bones: 526,
    });
    for op in [1, 3] {
        w.npc_handlers.insert((1, op), Handler::TowerTanner);
    }
    w.data_mut(actor).unwrap().tower_diary = Some(Progress {
        schema11: true,
        ..Default::default()
    });
    (w, actor, target)
}
fn tanner_click(w: &mut World, actor: ActorId, target: NpcId, operation: u8) -> Vec<WorldEvent> {
    w.submit(
        actor,
        Action::OperateNpc {
            target,
            operation,
            control: false,
        },
    )
    .unwrap();
    let mut events = vec![];
    for _ in 0..12 {
        w.tick().unwrap();
        events.extend(w.drain_events());
    }
    events
}
#[test]
fn tanner_routes_updates_diary_and_orders_task_reward_and_result() {
    use gielinor_world::inventory::Item;
    let (mut w, actor, target) = tanner_setup();
    for slot in 0..5 {
        w.data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(
                slot,
                Some(Item {
                    id: 1739,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    let events = tanner_click(&mut w, actor, target, 1);
    let observed: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            WorldEvent::GameMessage { text, .. } => Some(text.clone()),
            WorldEvent::TowerDiary { marks, .. } => {
                assert_eq!(marks.bounty, 1);
                Some("DIARY".into())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        observed,
        [
            "I'm a tanner. Bring cowhides. I don't keep a shop.",
            "DIARY",
            "The chest holds a bronze axe.",
            "The tanner tans 5 cowhide(s) into leather. Hides 5/5."
        ]
    );
    let data = w.data(actor).unwrap();
    assert_eq!(
        data.inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 1741)
            .count(),
        5
    );
    assert_eq!(data.tower_diary.unwrap().bounty, 1);
    assert!(data.tower_arrival.diary_progress);
    assert!(data.stats.xp(12) == 0, "Tanning does not award crafting XP");
    w.request_logout(actor).unwrap();
    assert!(w.drain_events().iter().any(|event| matches!(event,
        WorldEvent::Save { data, .. } if data.tower_diary.unwrap().counts[0] == 5)));
}
#[test]
fn tanner_unknown_diary_does_not_consume_inventory_or_fabricate_marks() {
    use gielinor_world::{inventory::Item, simulation::ActionError};
    let (mut w, actor, target) = tanner_setup();
    let data = w.data_mut(actor).unwrap();
    data.tower_diary = None;
    data.inventory
        .restore_slot(
            0,
            Some(Item {
                id: 1739,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    let events = tanner_click(&mut w, actor, target, 3);
    assert!(events.iter().any(|e| matches!(
        e,
        WorldEvent::ActionRejected {
            error: ActionError::UnknownTowerDiary,
            ..
        }
    )));
    assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 1739);
    assert!(w.data(actor).unwrap().tower_diary.is_none());
}
#[test]
fn tanner_empty_inventory_sends_hint_without_diary_sync() {
    let (mut w, actor, target) = tanner_setup();
    let events = tanner_click(&mut w, actor, target, 3);
    assert!(events
        .iter()
        .any(|e| matches!(e, WorldEvent::GameMessage { text, .. }
        if text == "Bring cowhides. The tanner tans; he does not keep a shop.")));
    assert!(!events
        .iter()
        .any(|e| matches!(e, WorldEvent::TowerDiary { .. })));
}
#[test]
fn tanner_rechecks_tower_bounds_after_routing() {
    use gielinor_world::inventory::Item;
    let (mut w, actor, target) = tanner_setup();
    w.tower_tanning.as_mut().unwrap().origin = CoordGrid::new(6000, 6000, 0).unwrap();
    w.data_mut(actor)
        .unwrap()
        .inventory
        .restore_slot(
            0,
            Some(Item {
                id: 1739,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    let events = tanner_click(&mut w, actor, target, 3);
    assert!(!events.iter().any(|e| matches!(
        e,
        WorldEvent::GameMessage { .. } | WorldEvent::TowerDiary { .. }
    )));
    assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 1739);
}

#[test]
fn tanner_talk_checks_npc_bounds_while_tan_checks_only_player_bounds() {
    use gielinor_world::inventory::Item;
    for (op, converted) in [(1, false), (3, true)] {
        let (mut w, actor, target) = tanner_setup();
        // Tanner is on x=3205, immediately outside this tower's east edge.
        w.tower_tanning.as_mut().unwrap().origin = CoordGrid::new(3077, 3192, 0).unwrap();
        w.data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 1739,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        tanner_click(&mut w, actor, target, op);
        assert_eq!(
            w.data(actor).unwrap().inventory.get(0).unwrap().id,
            if converted { 1741 } else { 1739 }
        );
    }
}

#[test]
fn diary_transition_to_imp_reward_drops_private_bones_even_with_saved_flag() {
    use gielinor_world::inventory::Item;
    for flag in [false, true] {
        let (mut w, actor, target) = tanner_setup();
        w.set_observer_id(actor, 900).unwrap();
        let data = w.data_mut(actor).unwrap();
        let diary = data.tower_diary.as_mut().unwrap();
        diary.done[..9].fill(true);
        diary.pending[0] = true;
        diary.imp_bones = flag;
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 1739,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let events = tanner_click(&mut w, actor, target, 3);
        let data = w.data(actor).unwrap();
        assert!(data.tower_diary.unwrap().imp_bones);
        assert_eq!(data.tower_diary.unwrap().bounty, 10);
        let bones = w.objects.at(data.coords).next().unwrap().snapshot();
        assert_eq!(
            (bones.item, bones.count, bones.receiver, bones.owner),
            (526, 1, Some(900), Some(900))
        );
        assert!(!bones.visible_to(Some(901)));
        assert!(events
            .iter()
            .any(|e| matches!(e, WorldEvent::GameMessage { text, .. }
            if text == "The chest holds 8 coins.")));
        w.data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 1739,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let events = tanner_click(&mut w, actor, target, 3);
        assert!(!events
            .iter()
            .any(|e| matches!(e, WorldEvent::GameMessage { text, .. }
            if text == "The chest holds 8 coins.")));
        assert_eq!(w.objects.count(), 1);
    }
}
