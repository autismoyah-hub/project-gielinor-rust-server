//! Bank-booth notes share held-location routing, dialogue tokens and inventory.
use super::location_swap_actions::{ChoiceAfter, Continuation, Pending};
use super::*;
use crate::{banknote_exchange as exchange, locations::Location};

impl World {
    pub(super) fn start_banknote_exchange(
        &mut self,
        slot: usize,
        location: Location,
        selected: usize,
        note: i32,
    ) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if p.saving || p.pending_logout || p.death_queued || p.data.stats.current(3) as u8 == 0 {
            self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::Protected,
            });
            return;
        }
        match exchange::prepare(&p.data.inventory, &self.object_types, selected, note) {
            Ok(_) => self.start_location_choice(
                slot,
                location,
                exchange::QUESTION,
                exchange::OPTIONS,
                [None, None],
                ChoiceAfter::Banknote {
                    slot: selected,
                    note,
                },
            ),
            Err(error) => self.banknote_error(actor, error),
        }
    }
    fn banknote_error(&mut self, actor: ActorId, error: exchange::Error) {
        match error {
            exchange::Error::NoSpace => self.events.push(WorldEvent::EngineMessage {
                actor,
                message: exchange::FULL,
            }),
            exchange::Error::NotNote => self.events.push(WorldEvent::EngineMessage {
                actor,
                message: "Nothing interesting happens.",
            }),
            _ => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidItem,
            }),
        }
    }
    pub(super) fn finish_banknote_choice(
        &mut self,
        actor: ActorId,
        pending: Pending,
        slot: usize,
        note: i32,
        yes: bool,
    ) -> Result<(), ActionError> {
        if !yes {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: crate::dialogue::Event::Closed,
            });
            return Ok(());
        }
        let p = self.player(actor)?;
        let Some(token) = p.dialog_generation.checked_add(1) else {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: crate::dialogue::Event::Closed,
            });
            return Err(ActionError::QueueFull);
        };
        let prepared = match exchange::prepare(&p.data.inventory, &self.object_types, slot, note) {
            Ok(p) => p,
            Err(error) => {
                self.events.push(WorldEvent::Dialogue {
                    actor,
                    event: crate::dialogue::Event::Closed,
                });
                self.banknote_error(actor, error);
                return Ok(());
            }
        };
        let output = u16::try_from(prepared.output).expect("prepared native object identity");
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.data.inventory = prepared.inventory;
        p.dialog_generation = token;
        p.pending_location_action = Some(Pending {
            action: Continuation::Message { token },
            ..pending
        });
        p.protection.suspended = true;
        // Replace the existing page without a preceding Closed event: a close
        // callback must not cancel the newly installed continuation.
        self.events.push(WorldEvent::Dialogue {
            actor,
            event: crate::dialogue::Event::LocationObject {
                token,
                item: output,
                zoom: 400,
                text: exchange::MESSAGE,
            },
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, loc_collision::LocCollision, stats::Stats};
    fn setup(slot: usize, free: usize, quantity: i32) -> (World, ActorId, Location) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.object_types = exchange::tests::catalog();
        w.ordinary_bank_booths.insert(18491);
        let location = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 18491, 10, 0).unwrap();
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(18491, ty);
        w.locations.load(location);
        crate::loc_collision::toggle(&mut w.collision, location.coords, 10, 0, ty, true).unwrap();
        for id in [1511, 1512, 1513] {
            w.held_use_types
                .insert(id, crate::held_use::Type { id, content: 0 });
        }
        let mut data = PlayerData::new(
            1,
            "Note exchange".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        for i in 0..28 {
            data.inventory
                .restore_slot(
                    i,
                    Some(Item {
                        id: 1513,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        for i in (0..28).filter(|&i| i != slot).take(free) {
            data.inventory.restore_slot(i, None).unwrap();
        }
        data.inventory
            .restore_slot(
                slot,
                Some(Item {
                    id: 1512,
                    count: quantity,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.drain_events();
        (w, actor, location)
    }
    fn use_note(w: &mut World, a: ActorId, location: Location, slot: usize, item: i32) {
        w.submit(
            a,
            Action::UseLocation {
                coords: location.coords,
                id: location.id,
                slot: slot as i32,
                object: item,
                control: false,
            },
        )
        .unwrap();
        w.tick().unwrap();
    }
    fn reply(w: &mut World, a: ActorId, token: u64, choice: i32) {
        w.submit(a, Action::ResumeDialogue { token, choice })
            .unwrap();
        w.tick().unwrap();
    }
    fn total(w: &World, a: ActorId, id: i32) -> i64 {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .map(|i| i.count as i64)
            .sum()
    }

    #[test]
    fn banknote_world_confirmation_selected_conversion_object_message_and_duplicate_reply() {
        let mut cases = 0;
        for member in [false, true] {
            for slot in [0, 13, 27] {
                let (mut w, a, l) = setup(slot, 2, 5);
                w.set_membership(a, member).unwrap();
                let before = w.data(a).unwrap().clone();
                use_note(&mut w, a, l, slot, 1512);
                let token = w.dialogue_token(a).unwrap().unwrap();
                assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:crate::dialogue::Event::LocationChoice{token:t,title:exchange::QUESTION,options:exchange::OPTIONS},..}if *t==token)));
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                reply(&mut w, a, token, 1);
                assert_eq!((total(&w, a, 1512), total(&w, a, 1511)), (3, 2));
                let notice = w.dialogue_token(a).unwrap().unwrap();
                assert_ne!(notice, token);
                let events = w.drain_events();
                assert!(!events.iter().any(|e| matches!(
                    e,
                    WorldEvent::Dialogue {
                        event: crate::dialogue::Event::Closed,
                        ..
                    }
                )));
                assert!(events.iter().any(|e|matches!(e,WorldEvent::Dialogue{event:crate::dialogue::Event::LocationObject{token:t,item:1511,zoom:400,text:exchange::MESSAGE},..}if *t==notice)));
                let committed = w.data(a).unwrap().clone();
                reply(&mut w, a, token, 1);
                assert_eq!(w.data(a).unwrap().inventory, committed.inventory);
                assert_eq!(w.dialogue_token(a).unwrap(), Some(notice));
                reply(&mut w, a, notice, 9);
                assert_eq!(w.dialogue_token(a).unwrap(), Some(notice));
                reply(&mut w, a, notice, 0);
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                reply(&mut w, a, notice, 0);
                assert_eq!(w.data(a).unwrap().inventory, committed.inventory);
                assert_eq!(w.data(a).unwrap().stats, before.stats);
                assert_eq!(w.data(a).unwrap().bank, before.bank);
                assert_eq!(w.data(a).unwrap().equipment, before.equipment);
                cases += 1;
            }
        }
        assert_eq!(cases, 6);
    }

    #[test]
    fn banknote_world_full_non_note_no_choice_late_slot_and_late_space_refuse_without_publication()
    {
        for mode in 0..5 {
            let (mut w, a, l) = setup(13, if mode == 0 { 0 } else { 2 }, 5);
            if mode == 1 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        13,
                        Some(Item {
                            id: 1513,
                            count: 1,
                            vars: 123,
                        }),
                    )
                    .unwrap();
            }
            let before = w.data(a).unwrap().clone();
            use_note(&mut w, a, l, 13, if mode == 1 { 1513 } else { 1512 });
            if mode < 2 {
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::EngineMessage{message,..}if *message==if mode==0{exchange::FULL}else{"Nothing interesting happens."})));
                continue;
            }
            let token = w.dialogue_token(a).unwrap().unwrap();
            if mode == 3 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        13,
                        Some(Item {
                            id: 1513,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            if mode == 4 {
                for i in [0, 1] {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            i,
                            Some(Item {
                                id: 1513,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
            }
            let late = w.data(a).unwrap().clone();
            reply(&mut w, a, token, if mode == 2 { 2 } else { 1 });
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            assert_eq!(w.data(a).unwrap().inventory, late.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }

    #[test]
    fn banknote_world_uses_current_note_count_and_cancels_choices_and_result_messages() {
        let (mut w, a, l) = setup(13, 2, 5);
        use_note(&mut w, a, l, 13, 1512);
        let token = w.dialogue_token(a).unwrap().unwrap();
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(
                13,
                Some(Item {
                    id: 1512,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        reply(&mut w, a, token, 1);
        assert_eq!((total(&w, a, 1512), total(&w, a, 1511)), (0, 1));
        assert!(w.cancel_dialogue(a).unwrap());
        assert_eq!(w.dialogue_token(a).unwrap(), None);
        assert!(!w.protection(a).unwrap().suspended);
        let (mut w, a, l) = setup(13, 2, 5);
        use_note(&mut w, a, l, 13, 1512);
        let token = w.dialogue_token(a).unwrap().unwrap();
        w.submit(
            a,
            Action::Walk {
                destination: CoordGrid::new(3200, 3201, 0).unwrap(),
                run: false,
            },
        )
        .unwrap();
        w.tick().unwrap();
        reply(&mut w, a, token, 1);
        assert_eq!((total(&w, a, 1512), total(&w, a, 1511)), (5, 0));
    }

    #[test]
    fn banknote_world_stale_booth_and_protection_cancel_without_reviving_or_mutating() {
        for mode in 0..6 {
            let (mut w, a, l) = setup(13, 2, 5);
            use_note(&mut w, a, l, 13, 1512);
            let token = w.dialogue_token(a).unwrap().unwrap();
            let before = w.data(a).unwrap().inventory.clone();
            match mode {
                0 => {
                    w.change_location(l, false).unwrap();
                    w.change_location(l, true).unwrap();
                }
                1 => w.players.get_mut(a.slot as usize).unwrap().saving = true,
                2 => w.players.get_mut(a.slot as usize).unwrap().has_modal = true,
                3 => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .movement
                        .last_movement += 1
                }
                4 => {
                    w.cancel_dialogue(a).unwrap();
                }
                _ => {
                    w.ordinary_bank_booths.remove(&l.id);
                    w.change_location(l, false).unwrap();
                }
            }
            if mode == 1 {
                assert_eq!(
                    w.submit(a, Action::ResumeDialogue { token, choice: 1 }),
                    Err(ActionError::LoggingOut)
                );
                w.tick().unwrap();
            } else {
                reply(&mut w, a, token, 1);
            }
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert!(!w.protection(a).unwrap().suspended);
        }
        // Exhaustion is refused before inventory publication or token wrap.
        let (mut w, a, l) = setup(13, 2, 5);
        use_note(&mut w, a, l, 13, 1512);
        let token = w.dialogue_token(a).unwrap().unwrap();
        let before = w.data(a).unwrap().inventory.clone();
        w.players
            .get_mut(a.slot as usize)
            .unwrap()
            .dialog_generation = u64::MAX;
        reply(&mut w, a, token, 1);
        assert_eq!(w.data(a).unwrap().inventory, before);
        assert_eq!(w.dialogue_token(a).unwrap(), None);
    }

    #[test]
    fn banknote_world_life_before_and_during_confirmation_cannot_consume_or_revive() {
        use crate::hits::{Hit, Style, Target};
        for late in [false, true] {
            for phase in 0..3 {
                let (mut w, a, l) = setup(13, 2, 5);
                let token = if late {
                    use_note(&mut w, a, l, 13, 1512);
                    w.dialogue_token(a).unwrap()
                } else {
                    None
                };
                if phase == 0 {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                } else {
                    w.queue_hit(
                        Target::Player(a),
                        Hit {
                            display: None,
                            source: None,
                            style: Style::Typeless,
                            damage: 99,
                        },
                        0,
                    )
                    .unwrap();
                    w.tick().unwrap();
                    assert!(w.drain_events().iter().any(
                        |e| matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)
                    ));
                    if phase == 2 {
                        w.data_mut(a).unwrap().stats.set_current(3, 10);
                    }
                }
                let before = w.data(a).unwrap().clone();
                if let Some(token) = token {
                    reply(&mut w, a, token, 1);
                } else {
                    use_note(&mut w, a, l, 13, 1512);
                }
                for _ in 0..3 {
                    w.tick().unwrap();
                }
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert_eq!(w.data(a).unwrap().stats, before.stats);
            }
        }
    }
}
