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
