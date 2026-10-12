use super::*;
use gielinor_protocol::{
    client_script::{self, Argument},
    interfaces,
};
use gielinor_world::{
    dialogue::{Event as DialogueEvent, Page, Speaker},
    simulation::Action,
    ui::Component,
};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_dialogue(
        &mut self,
        presentation: crate::dialogue::Presentation,
        person_group: i32,
        flyer: i32,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure dialogue before admission"));
        }
        for (&id, ty) in &self.definitions.npcs {
            if ty.content_group == person_group {
                self.world.npc_handlers.insert(
                    (id as u16, 1),
                    gielinor_world::npc_interaction::Handler::Person,
                );
            }
        }
        self.world.person_flyer = Some(flyer);
        self.dialogue_presentation = Some(presentation);
        Ok(())
    }
    fn dialogue_packet(&mut self, actor: ActorId, packet: Packet) -> io::Result<()> {
        if let Some(connection) = self.connection(actor) {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub(super) fn resume_dialogue_pause(
        &mut self,
        actor: ActorId,
        pause: gielinor_protocol::dialog::Pause,
    ) -> io::Result<()> {
        let Some(active) = self.active.get(&actor) else {
            return Ok(());
        };
        let Some(dialogue) = active.dialogue.as_ref() else {
            return Ok(());
        };
        if dialogue.button.0 != pause.component
            || !active
                .ui
                .modals
                .contains_interface(dialogue.button.interface())
        {
            return Ok(());
        }
        let Some(definition) = self.components.get(&pause.component) else {
            return Ok(());
        };
        if !active
            .ui
            .events
            .enabled(dialogue.button, pause.subcomponent, definition.events, 1)
        {
            return Ok(());
        }
        if self
            .world
            .protection(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .delayed()
        {
            return Ok(());
        }
        let token = dialogue.token;
        if self
            .world
            .dialogue_token(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            != Some(token)
        {
            return Ok(());
        }
        if !dialogue.remaining.is_empty() {
            self.show_dialogue_page(actor)?;
        } else {
            self.world
                .submit(
                    actor,
                    Action::ResumeDialogue {
                        token,
                        choice: pause.subcomponent,
                    },
                )
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        Ok(())
    }
    pub(super) fn dialogue_event(
        &mut self,
        actor: ActorId,
        event: &DialogueEvent,
    ) -> io::Result<()> {
        if self.connection(actor).is_none() {
            return Ok(());
        }
        let Some(presentation) = self.dialogue_presentation.as_ref() else {
            return Ok(());
        };
        match event {
            DialogueEvent::Closed => {
                let parent = presentation.parent;
                self.active.get_mut(&actor).unwrap().dialogue = None;
                self.close_interface(actor, parent)
            }
            DialogueEvent::Page { token, npc, page } => {
                let (interface, remaining) = match page {
                    Page::Chat { speaker, text, .. } => (
                        if *speaker == Speaker::Player {
                            presentation.player
                        } else {
                            presentation.npc
                        },
                        presentation
                            .font
                            .pages(text)
                            .map_err(io::Error::other)?
                            .into(),
                    ),
                    Page::Choice(_) => (presentation.choice, Default::default()),
                };
                let button = Component(
                    (interface as u32) << 16
                        | if matches!(page, Page::Choice(_)) {
                            1
                        } else {
                            5
                        },
                );
                self.active.get_mut(&actor).unwrap().dialogue = Some(crate::dialogue::Active {
                    token: *token,
                    npc: *npc,
                    page: page.clone(),
                    remaining,
                    button,
                });
                self.show_dialogue_page(actor)
            }
        }
    }
    fn show_dialogue_page(&mut self, actor: ActorId) -> io::Result<()> {
        let presentation = self.dialogue_presentation.as_ref().unwrap().clone();
        // Take before replacing the modal, so external-close hooks do not cancel this conversation.
        let Some(mut dialogue) = self.active.get_mut(&actor).and_then(|a| a.dialogue.take()) else {
            return Ok(());
        };
        let protocol =
            |e: gielinor_protocol::buffer::DecodeError| io::Error::other(format!("{e:?}"));
        let mode = match dialogue.page {
            Page::Chat { .. } => 0,
            Page::Choice(_) => 1,
        };
        let text_page = dialogue.remaining.pop_front();
        let title = match &dialogue.page {
            Page::Chat {
                speaker: Speaker::Player,
                ..
            } => self
                .world
                .data(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .display_name
                .clone(),
            _ => self
                .world
                .npc(dialogue.npc)
                .and_then(|npc| self.definitions.npcs.get(&(npc.type_id as u32)))
                .map(|ty| ty.name.clone())
                .unwrap_or_default(),
        };
        if let Some((text, _)) = &text_page {
            self.dialogue_packet(
                actor,
                gielinor_protocol::chat::game(115, None, &format!("{title}|{text}"))
                    .map_err(protocol)?,
            )?;
        }
        self.set_varbit(actor, presentation.unclamp, mode)?;
        self.dialogue_packet(actor, client_script::run(2379, &[]).map_err(protocol)?)?;
        self.open_interface(
            actor,
            Some(presentation.parent),
            dialogue.button.interface(),
            false,
        )?;
        match &dialogue.page {
            Page::Chat {
                speaker,
                expression,
                ..
            } => {
                let (text, lines) =
                    text_page.ok_or_else(|| io::Error::other("Missing dialogue page"))?;
                let base = (dialogue.button.interface() as u32) << 16;
                let head = if *speaker == Speaker::Player {
                    interfaces::player_head(base | 2)
                } else {
                    let Some(npc) = self.world.npc(dialogue.npc) else {
                        self.world
                            .cancel_dialogue(actor)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        self.close_interface(actor, presentation.parent)?;
                        return Ok(());
                    };
                    interfaces::npc_head(base | 2, npc.type_id)
                };
                self.dialogue_packet(actor, head)?;
                self.dialogue_packet(
                    actor,
                    interfaces::animation(
                        base | 2,
                        Some(presentation.animation(*expression, lines)),
                    ),
                )?;
                self.dialogue_packet(actor, interfaces::text(base | 4, &title).map_err(protocol)?)?;
                self.dialogue_packet(actor, interfaces::text(base | 6, &text).map_err(protocol)?)?;
                self.dialogue_packet(
                    actor,
                    client_script::run(
                        600,
                        &[
                            Argument::Int(1),
                            Argument::Int(1),
                            Argument::Int(gielinor_world::dialogue_text::line_height(lines)),
                            Argument::Int((base | 6) as i32),
                        ],
                    )
                    .map_err(protocol)?,
                )?;
                self.set_interface_events(actor, dialogue.button, -1, -1, 1)?;
                self.dialogue_packet(
                    actor,
                    interfaces::text(dialogue.button.0, "Click here to continue")
                        .map_err(protocol)?,
                )?;
            }
            Page::Choice(options) => {
                self.dialogue_packet(
                    actor,
                    client_script::run(
                        58,
                        &[
                            Argument::String("Select an option".into()),
                            Argument::String(options.join("|")),
                        ],
                    )
                    .map_err(protocol)?,
                )?;
                self.set_interface_events(actor, dialogue.button, 1, options.len() as i32, 1)?;
            }
        }
        self.active.get_mut(&actor).unwrap().dialogue = Some(dialogue);
        Ok(())
    }
}
