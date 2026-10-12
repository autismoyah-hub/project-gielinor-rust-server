use super::*;
use crate::dialogue::{Conversation, Effect, Event, Page, Pending, Speaker};
impl World {
    pub fn dialogue_token(&self, actor: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self
            .player(actor)?
            .dialogue
            .as_ref()
            .map(|dialogue| dialogue.token))
    }
    pub fn cancel_dialogue(&mut self, actor: ActorId) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let cancelled = p.dialogue.take().is_some();
        if cancelled {
            p.protection.suspended = false;
        }
        Ok(cancelled)
    }
    pub(super) fn start_person(&mut self, actor: ActorId, npc: NpcId) -> Result<(), ActionError> {
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: Conversation::person(),
            },
        )
    }
    fn show_dialogue(&mut self, actor: ActorId, mut pending: Pending) -> Result<(), ActionError> {
        let Some(page) = pending.conversation.pages.front().cloned() else {
            return Ok(());
        };
        if matches!(
            page,
            Page::Chat {
                speaker: Speaker::Npc,
                ..
            }
        ) {
            if let Some(npc) = self.npc_mut(pending.npc) {
                if npc.attacking_player.is_none() && npc.escape_player.is_none() {
                    npc.movement.abort();
                    npc.movement.interacting = false;
                    npc.face_player(Some(actor));
                    npc.dialogue_face = Some(actor);
                }
            }
        }
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        p.dialog_generation = p
            .dialog_generation
            .checked_add(1)
            .ok_or(ActionError::QueueFull)?;
        pending.token = p.dialog_generation;
        self.events.push(WorldEvent::Dialogue {
            actor,
            event: Event::Page {
                token: pending.token,
                npc: pending.npc,
                page,
            },
        });
        p.dialogue = Some(pending);
        p.protection.suspended = true;
        Ok(())
    }
    pub(super) fn process_dialogue_face(&mut self, id: NpcId) -> bool {
        let Some(npc) = self.npc(id) else {
            return false;
        };
        let Some(actor) = npc.dialogue_face else {
            return false;
        };
        let a = npc.movement.coords;
        let size = npc.movement.size;
        let close = self.player(actor).is_ok_and(|p| {
            let b = p.movement.coords;
            a.level() == b.level()
                && a.x() <= b.x() + p.movement.size
                && b.x() <= a.x() + size
                && a.z() <= b.z() + p.movement.size
                && b.z() <= a.z() + size
        });
        let npc = self.npc_mut(id).unwrap();
        npc.movement.abort();
        if !close {
            npc.face_player(None);
            npc.movement.interacting = false;
        }
        true
    }
    pub(super) fn resume_dialogue(
        &mut self,
        actor: ActorId,
        token: u64,
        choice: i32,
        random: &mut impl RandomSource,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.dialogue.as_ref().is_none_or(|d| d.token != token) {
            return Ok(());
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let mut pending = p.dialogue.take().unwrap();
        p.protection.suspended = false;
        let Some(npc) = self.npc(pending.npc) else {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
            return Ok(());
        };
        if !npc.visible || npc.stats[3] <= 0 {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
            return Ok(());
        }
        match pending.conversation.pages.front() {
            Some(Page::Choice(_)) => {
                let Some(conversation) = Conversation::choice(choice) else {
                    self.players.get_mut(actor.slot as usize).unwrap().dialogue = Some(pending);
                    self.players
                        .get_mut(actor.slot as usize)
                        .unwrap()
                        .protection
                        .suspended = true;
                    return Ok(());
                };
                pending.conversation = conversation;
            }
            Some(Page::Chat { .. }) if choice == -1 => {
                pending.conversation.pages.pop_front();
            }
            _ => {
                self.players.get_mut(actor.slot as usize).unwrap().dialogue = Some(pending);
                self.players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .protection
                    .suspended = true;
                return Ok(());
            }
        }
        if pending.conversation.pages.is_empty() {
            match pending.conversation.after {
                Effect::RollPersonResponse => {
                    let rolled = random.below(128) == 0;
                    let p = self.player(actor)?;
                    let flyer = rolled
                        && p.data.inventory.free_space() > 0
                        && self.person_flyer.is_some_and(|id| {
                            !p.data
                                .inventory
                                .slots()
                                .iter()
                                .flatten()
                                .any(|item| item.id == id)
                        });
                    let roll = if flyer { 0 } else { random.below(22) };
                    pending.conversation = Conversation::response(flyer, roll).unwrap();
                }
                Effect::GiveFlyer => {
                    if let Some(id) = self.person_flyer {
                        let p = self.players.get_mut(actor.slot as usize).unwrap();
                        // Kotlin invAdd is non-strict: a now-full inventory cannot duplicate an item.
                        let mut insert = crate::inventory::Insert::new(id, 1);
                        insert.strict = false;
                        let _ = p.data.inventory.insert(&self.object_types, insert);
                    }
                }
                Effect::Retaliate => {
                    let clock = self.clock.0;
                    if let Some(npc) = self.npc_mut(pending.npc) {
                        npc.aggressive_player = Some(actor);
                        npc.last_combat = npc.last_combat.max(clock);
                        npc.retaliate_due.push_back(clock.wrapping_add(1));
                    }
                }
                Effect::None => {}
            }
        }
        if pending.conversation.pages.is_empty() {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
            Ok(())
        } else {
            self.show_dialogue(actor, pending)
        }
    }
}
