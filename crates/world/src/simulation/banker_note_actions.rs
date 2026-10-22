//! Held notes reuse the NPC route, common dialogue and selected exchange transaction.
use super::*;
use crate::{
    banknote_exchange::{self as exchange, NpcContext},
    dialogue::{Conversation, Effect, Event, Expression, Page, Pending, Speaker},
    npc_interaction::Handler,
    raycast::{self, Bounds},
};
impl World {
    fn banker_note_visible(&self, actor: ActorId, raw: u16) -> Option<u16> {
        let p = self.player(actor).ok()?;
        let definition = self.npc_options.types.get(&raw)?;
        if !definition.morphs.is_empty() || definition.fallback.unwrap_or(0) > 0 {
            self.npc_options.morph(raw, &p.data.variables)
        } else {
            Some(raw)
        }
    }
    pub(super) fn banker_note_identity(
        &self,
        actor: ActorId,
        npc: NpcId,
        raw: u16,
        visible: u16,
    ) -> bool {
        if self.player(actor).is_err() {
            return false;
        }
        let Some(n) = self.npc(npc) else { return false };
        n.type_id == raw
            && n.visible
            && !n.delayed
            && n.stats[3] > 0
            && self.banker_note_visible(actor, raw) == Some(visible)
            && self
                .npc_handlers
                .get(&(visible, 3))
                .or_else(|| self.npc_handlers.get(&(raw, 3)))
                == Some(&Handler::Bank)
            && self.can_operate_npc(actor, npc, 3)
    }
    pub(super) fn request_banker_note(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        slot: i32,
        note: i32,
        control: bool,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if self.banknote_npc_sad.is_none() || p.death_queued || p.data.stats.current(3) as u8 == 0 {
            return Err(ActionError::Protected);
        }
        let n = self.npc(npc).ok_or(ActionError::InvalidItem)?;
        let raw = n.type_id;
        let visible = self
            .banker_note_visible(actor, raw)
            .ok_or(ActionError::InvalidItem)?;
        if !self.banker_note_identity(actor, npc, raw, visible) {
            return Err(ActionError::InvalidItem);
        }
        let selected = usize::try_from(slot).map_err(|_| ActionError::InvalidItem)?;
        if !self.held_use_types.contains_key(&note)
            || !p
                .data
                .inventory
                .get(selected)
                .is_some_and(|i| i.id == note && i.count > 0)
        {
            return Err(ActionError::InvalidItem);
        }
        self.request_npc_inner(actor, npc, 3, control, None)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if let Some(pending) = p
            .npc_interaction
            .as_mut()
            .filter(|i| i.target == npc && i.handler == Some(Handler::Bank))
        {
            pending.handler = Some(Handler::Banknote {
                slot: selected,
                note,
                raw,
                visible,
            });
        }
        Ok(())
    }
    fn banker_note_context_valid(&self, actor: ActorId, npc: NpcId, ctx: NpcContext) -> bool {
        let Ok(p) = self.player(actor) else {
            return false;
        };
        if p.saving
            || self.banknote_npc_sad.is_none()
            || p.pending_logout
            || p.death_queued
            || p.data.stats.current(3) as u8 == 0
            || p.protection.delayed()
            || p.movement.coords != ctx.origin
            || p.movement.last_movement != ctx.last_movement
            || !self.banker_note_identity(actor, npc, ctx.raw, ctx.visible)
        {
            return false;
        }
        let n = self.npc(npc).unwrap();
        let a = p.movement.coords;
        let b = n.movement.coords;
        let size = n.movement.size;
        a.level() == b.level()
            && a.x() >= b.x() - 3
            && a.x() <= b.x() + size - 1 + 3
            && a.z() >= b.z() - 3
            && a.z() <= b.z() + size - 1 + 3
            && !(a.x() < b.x() + size
                && a.x() + p.movement.size > b.x()
                && a.z() < b.z() + size
                && a.z() + p.movement.size > b.z())
            && raycast::clear(
                &self.collision,
                Bounds {
                    coords: a,
                    width: p.movement.size,
                    length: p.movement.size,
                },
                Bounds {
                    coords: b,
                    width: size,
                    length: size,
                },
                true,
                0x100000,
            )
    }
    fn banker_note_notice(&self, ctx: NpcContext, text: &'static str) -> Conversation {
        Conversation {
            pages: [Page::Chat {
                speaker: Speaker::NpcNoTurn,
                expression: Expression::Sad(
                    self.banknote_npc_sad
                        .expect("admitted banker note presentation"),
                ),
                text,
            }]
            .into(),
            after: Effect::Banknote {
                ctx,
                confirm: false,
            },
        }
    }
    pub(super) fn start_banker_note(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        slot: usize,
        note: i32,
        raw: u16,
        visible: u16,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        let ctx = NpcContext {
            raw,
            visible,
            slot,
            note,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        };
        if !self.banker_note_context_valid(actor, npc, ctx) {
            return Err(ActionError::Protected);
        }
        let conversation =
            match exchange::prepare(&p.data.inventory, &self.object_types, slot, note) {
                Ok(_) => Conversation {
                    pages: [Page::TitledChoice {
                        title: exchange::QUESTION,
                        options: exchange::OPTIONS,
                    }]
                    .into(),
                    after: Effect::Banknote { ctx, confirm: true },
                },
                Err(exchange::Error::NotNote) => self.banker_note_notice(
                    ctx,
                    "Hand me a banknote, and I'll try to convert it to an item.",
                ),
                Err(exchange::Error::NoSpace) => self.banker_note_notice(ctx, exchange::FULL),
                Err(_) => return Err(ActionError::InvalidItem),
            };
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation,
            },
        )
    }
    pub(super) fn banker_note_choice(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        ctx: NpcContext,
        choice: i32,
    ) -> Result<Option<Conversation>, ActionError> {
        if !self.banker_note_context_valid(actor, npc, ctx) {
            return Ok(Some(Conversation {
                pages: Default::default(),
                after: Effect::None,
            }));
        }
        if !(1..=2).contains(&choice) {
            return Ok(None);
        }
        if choice == 2 {
            return Ok(Some(Conversation {
                pages: Default::default(),
                after: Effect::None,
            }));
        }
        let p = self.player(actor)?;
        if p.dialog_generation == u64::MAX {
            return Ok(Some(Conversation {
                pages: Default::default(),
                after: Effect::None,
            }));
        }
        let prepared =
            match exchange::prepare(&p.data.inventory, &self.object_types, ctx.slot, ctx.note) {
                Ok(p) => p,
                Err(exchange::Error::NoSpace) => {
                    return Ok(Some(self.banker_note_notice(ctx, exchange::FULL)))
                }
                Err(_) => {
                    return Ok(Some(Conversation {
                        pages: Default::default(),
                        after: Effect::None,
                    }))
                }
            };
        self.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .inventory = prepared.inventory;
        Ok(Some(Conversation {
            pages: [Page::Object {
                item: prepared.output as u16,
                zoom: 400,
                text: exchange::MESSAGE,
            }]
            .into(),
            after: Effect::Banknote {
                ctx,
                confirm: false,
            },
        }))
    }
    pub(super) fn validate_banker_note_dialogue(&mut self, slot: usize) {
        let Some(p) = self.players.get(slot) else {
            return;
        };
        let Some(d) = p.dialogue.as_ref() else { return };
        let Effect::Banknote { ctx, .. } = d.conversation.after else {
            return;
        };
        let actor = p.id;
        let npc = d.npc;
        if !self.banker_note_context_valid(actor, npc, ctx) {
            let p = self.players.get_mut(slot).unwrap();
            p.dialogue = None;
            p.protection.suspended = false;
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        npc::{BlockWalk, Mode, Restrict},
        npc_options::Definition,
        stats::Stats,
    };
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Banknote exchange must not draw RNG")
        }
    }
    fn fixture(distance: i32, free: usize) -> (World, ActorId, NpcId) {
        let mut w = World::default();
        for x in (3192..3232).step_by(8) {
            for z in (3192..3232).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        w.object_types = exchange::tests::catalog();
        w.banknote_npc_sad = Some([610, 611, 612, 613]);
        for id in [1511, 1512, 1513] {
            w.held_use_types
                .insert(id, crate::held_use::Type { id, content: 0 });
        }
        w.npc_handlers.insert((394, 3), Handler::Bank);
        w.npc_options.types.insert(
            394,
            Definition {
                operations: [
                    Some("Talk-to".into()),
                    None,
                    Some("Bank".into()),
                    Some("Collect".into()),
                    None,
                ],
                ..Default::default()
            },
        );
        let mut data = PlayerData::new(
            1,
            "Banker notes".into(),
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
        for i in (0..28).filter(|i| *i != 13).take(free) {
            data.inventory.restore_slot(i, None).unwrap();
        }
        data.inventory
            .restore_slot(
                13,
                Some(Item {
                    id: 1512,
                    count: 5,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let a = w.actors().next().unwrap();
        let mut n = Npc::new(
            394,
            CoordGrid::new(3200 + distance, 3200, 0).unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::None,
        )
        .unwrap();
        n.mode = Mode::None;
        let n = w.spawn_npc(n).unwrap();
        w.drain_events();
        (w, a, n)
    }
    fn use_note(w: &mut World, a: ActorId, n: NpcId, id: i32) {
        w.submit(
            a,
            Action::UseNpc {
                target: n,
                slot: 13,
                object: id,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
    }
    fn reply(w: &mut World, a: ActorId, token: u64, choice: i32) {
        w.submit(a, Action::ResumeDialogue { token, choice })
            .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
    }
    fn totals(w: &World, a: ActorId) -> (i64, i64) {
        let inv = &w.data(a).unwrap().inventory;
        let total = |id| {
            inv.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == id)
                .map(|i| i.count as i64)
                .sum()
        };
        (total(1512), total(1511))
    }
    #[test]
    fn banker_note_partial_exchange_no_yes_object_continue_three_tile_approach_and_no_replay() {
        for member in [false, true] {
            for distance in [1, 2, 3] {
                let (mut w, a, n) = fixture(distance, 2);
                w.set_membership(a, member).unwrap();
                let before = w.data(a).unwrap().clone();
                use_note(&mut w, a, n, 1512);
                assert_eq!(w.data(a).unwrap().coords, before.coords);
                let no = w.dialogue_token(a).unwrap().unwrap();
                reply(&mut w, a, no, 2);
                assert_eq!(totals(&w, a), (5, 0));
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                use_note(&mut w, a, n, 1512);
                let yes = w.dialogue_token(a).unwrap().unwrap();
                reply(&mut w, a, yes, 1);
                assert_eq!(totals(&w, a), (3, 2));
                let result = w.dialogue_token(a).unwrap().unwrap();
                assert_ne!(result, yes);
                assert!(w.drain_events().iter().any(|e| matches!(
                    e,
                    WorldEvent::Dialogue {
                        event: Event::Page {
                            page: Page::Object {
                                item: 1511,
                                zoom: 400,
                                text: exchange::MESSAGE
                            },
                            ..
                        },
                        ..
                    }
                )));
                reply(&mut w, a, yes, 1);
                assert_eq!(totals(&w, a), (3, 2));
                assert_eq!(w.dialogue_token(a).unwrap(), Some(result));
                reply(&mut w, a, result, 9);
                assert_eq!(w.dialogue_token(a).unwrap(), Some(result));
                reply(&mut w, a, result, 0);
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                reply(&mut w, a, result, 0);
                assert_eq!(totals(&w, a), (3, 2));
                assert_eq!(w.data(a).unwrap().stats, before.stats);
                assert_eq!(w.data(a).unwrap().bank, before.bank);
                assert_eq!(w.data(a).unwrap().equipment, before.equipment);
            }
        }
    }
    #[test]
    fn banker_note_full_and_non_note_use_sad_npc_without_turn_or_inventory_change() {
        for full in [false, true] {
            let (mut w, a, n) = fixture(1, if full { 0 } else { 2 });
            if !full {
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
            let before = w.data(a).unwrap().clone();
            use_note(&mut w, a, n, if full { 1512 } else { 1513 });
            let token = w.dialogue_token(a).unwrap().unwrap();
            assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Dialogue{event:Event::Page{page:Page::Chat{speaker:Speaker::NpcNoTurn,expression:Expression::Sad([610,611,612,613]),text},..},..}if *text==if full{exchange::FULL}else{"Hand me a banknote, and I'll try to convert it to an item."})));
            assert!(w.npc(n).unwrap().dialogue_face.is_none());
            reply(&mut w, a, token, -1);
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
    #[test]
    fn banker_note_morph_change_missing_morph_and_replacement_npc_cannot_confirm_old_identity() {
        for mode in 0..3 {
            let (mut w, a, n) = fixture(1, 2);
            if mode < 2 {
                let base = w.npc_options.types[&394].clone();
                w.npc_options.types.insert(395, base.clone());
                w.npc_options.types.insert(396, base);
                w.npc_handlers.insert((395, 3), Handler::Bank);
                w.npc_handlers.insert((396, 3), Handler::Bank);
                w.npc_options.varps.insert(77);
                let root = w.npc_options.types.get_mut(&394).unwrap();
                root.varp = Some(77);
                root.morphs = vec![Some(395), if mode == 0 { Some(396) } else { None }];
            }
            use_note(&mut w, a, n, 1512);
            let token = w.dialogue_token(a).unwrap().unwrap();
            let before = w.data(a).unwrap().inventory.clone();
            if mode < 2 {
                w.data_mut(a).unwrap().variables.set(
                    crate::variables::Varp {
                        id: 77,
                        permanent: false,
                        transmit: crate::variables::Transmit::Never,
                        bit_protect: false,
                        client_code: 0,
                    },
                    1,
                    true,
                );
            } else {
                let replacement = w.despawn_npc(n).unwrap();
                let new = w.spawn_npc(replacement).unwrap();
                assert_ne!(n, new);
            }
            reply(&mut w, a, token, 1);
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            assert_eq!(w.data(a).unwrap().inventory, before);
            if mode == 1 {
                use_note(&mut w, a, n, 1512);
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                assert_eq!(w.data(a).unwrap().inventory, before);
            }
        }
    }

    #[test]
    fn banker_note_rechecks_live_count_and_late_full_pack_without_replaying_confirmation() {
        let (mut w, a, n) = fixture(1, 2);
        use_note(&mut w, a, n, 1512);
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
        assert_eq!(totals(&w, a), (0, 1));
        let (mut w, a, n) = fixture(1, 2);
        use_note(&mut w, a, n, 1512);
        let token = w.dialogue_token(a).unwrap().unwrap();
        for slot in [0, 1] {
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 1513,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let before = w.data(a).unwrap().inventory.clone();
        reply(&mut w, a, token, 1);
        assert_eq!(w.data(a).unwrap().inventory, before);
        let notice = w.dialogue_token(a).unwrap().unwrap();
        assert_ne!(notice, token);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(0, None)
            .unwrap();
        reply(&mut w, a, notice, -1);
        assert_eq!(totals(&w, a), (5, 0));
        assert_eq!(w.dialogue_token(a).unwrap(), None);
    }

    #[test]
    fn banker_note_actual_death_before_or_during_confirmation_cannot_revive_at_restored_hp() {
        use crate::hits::{Hit, Style, Target};
        for late in [false, true] {
            for restored in [false, true] {
                let (mut w, a, n) = fixture(1, 2);
                let token = if late {
                    use_note(&mut w, a, n, 1512);
                    w.dialogue_token(a).unwrap()
                } else {
                    None
                };
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
                w.tick_with_random(&mut NoDraw).unwrap();
                assert!(w
                    .drain_events()
                    .iter()
                    .any(|e| matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)));
                if restored {
                    w.data_mut(a).unwrap().stats.set_current(3, 10);
                }
                let before = w.data(a).unwrap().clone();
                if let Some(token) = token {
                    reply(&mut w, a, token, 1);
                } else {
                    use_note(&mut w, a, n, 1512);
                }
                for _ in 0..3 {
                    w.tick_with_random(&mut NoDraw).unwrap();
                }
                assert_eq!(w.dialogue_token(a).unwrap(), None);
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert_eq!(w.data(a).unwrap().stats, before.stats);
            }
        }
    }

    #[test]
    fn banker_note_stale_item_target_and_actor_context_never_convert_or_revive() {
        for mode in 0..6 {
            let (mut w, a, n) = fixture(1, 2);
            use_note(&mut w, a, n, 1512);
            let token = w.dialogue_token(a).unwrap().unwrap();
            match mode {
                0 => {
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
                1 => {
                    w.npc_mut(n).unwrap().type_id = 395;
                }
                2 => {
                    w.npc_mut(n).unwrap().visible = false;
                }
                3 => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .movement
                        .last_movement += 1;
                }
                4 => {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                }
                _ => {
                    w.npc_handlers.remove(&(394, 3));
                }
            }
            let before = w.data(a).unwrap().inventory.clone();
            reply(&mut w, a, token, 1);
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.dialogue_token(a).unwrap(), None);
            assert!(!w.protection(a).unwrap().suspended);
            w.data_mut(a).unwrap().stats.set_current(3, 10);
            reply(&mut w, a, token, 1);
            assert_eq!(w.data(a).unwrap().inventory, before);
        }
    }
}
