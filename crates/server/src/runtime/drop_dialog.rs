use super::*;
use gielinor_protocol::{
    client_script::{self, Argument},
    interfaces,
};
use gielinor_world::{
    drop_actions::{Event as DropEvent, Prompt},
    simulation::Action,
    ui::Component,
};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_dropping(
        &mut self,
        presentation: crate::drop_dialog::Presentation,
        config: gielinor_world::drop_actions::Config,
    ) -> io::Result<()> {
        if !self.active.is_empty() {
            return Err(io::Error::other("Configure drop dialogs before admission"));
        }
        self.drop_presentation = Some(presentation);
        self.world.dropping = Some(config);
        Ok(())
    }
    fn drop_packet(&mut self, actor: ActorId, packet: Packet) -> io::Result<()> {
        if let Some(connection) = self.connection(actor) {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub(super) fn resume_drop_pause(
        &mut self,
        actor: ActorId,
        pause: gielinor_protocol::dialog::Pause,
    ) -> io::Result<()> {
        let Some(active) = self.active.get(&actor) else {
            return Ok(());
        };
        let Some((token, expected)) = active.drop_dialog else {
            return Ok(());
        };
        let component = Component(pause.component);
        if component != expected || !active.ui.modals.contains_interface(component.interface()) {
            return Ok(());
        }
        let Some(definition) = self.components.get(&pause.component) else {
            return Ok(());
        };
        if !active
            .ui
            .events
            .enabled(component, pause.subcomponent as i32, definition.events, 1)
        {
            return Ok(());
        }
        self.world
            .submit(
                actor,
                Action::ResumeDrop {
                    token,
                    choice: pause.subcomponent as i32,
                },
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    pub(super) fn drop_event(&mut self, actor: ActorId, event: &DropEvent) -> io::Result<()> {
        let protocol =
            |e: gielinor_protocol::buffer::DecodeError| io::Error::other(format!("{e:?}"));
        match event {
            DropEvent::Sound(id) => {
                self.drop_packet(actor, gielinor_protocol::game::synth_sound(*id, 1, 0))
            }
            DropEvent::Message(message) => self.drop_packet(
                actor,
                gielinor_protocol::chat::game(0, None, message).map_err(protocol)?,
            ),
            DropEvent::Committed { .. } => Ok(()),
            DropEvent::Closed => {
                let parent = self.drop_presentation.map(|p| p.parent);
                let was_open = self
                    .active
                    .get_mut(&actor)
                    .and_then(|a| a.drop_dialog.take())
                    .is_some();
                if was_open {
                    if let Some(parent) = parent {
                        self.close_interface(actor, parent)?;
                    }
                }
                Ok(())
            }
            DropEvent::Prompt { token, prompt } => {
                if self.connection(actor).is_none() {
                    return Ok(());
                }
                let presentation = self
                    .drop_presentation
                    .ok_or_else(|| io::Error::other("Drop dialog presentation missing"))?;
                let (interface, button, start, end, mode) = match prompt {
                    Prompt::Object { .. } => (
                        presentation.object_interface,
                        presentation.object_button,
                        0,
                        1,
                        1,
                    ),
                    Prompt::Choice { .. } => (
                        presentation.choice_interface,
                        presentation.choice_button,
                        1,
                        2,
                        1,
                    ),
                    Prompt::Destroy { .. } => (
                        presentation.destroy_interface,
                        presentation.destroy_button,
                        0,
                        1,
                        0,
                    ),
                };
                // Replacing a page belongs to the same coroutine; external closes cancel it.
                self.active.get_mut(&actor).unwrap().drop_dialog = None;
                if let Prompt::Object { text, .. } = prompt {
                    self.drop_packet(
                        actor,
                        gielinor_protocol::chat::game(115, None, text).map_err(protocol)?,
                    )?;
                }
                self.set_varbit(actor, presentation.unclamp, mode)?;
                self.drop_packet(actor, client_script::run(2379, &[]).map_err(protocol)?)?;
                self.open_interface(actor, Some(presentation.parent), interface, false)?;
                match prompt {
                    Prompt::Object { id, text } => {
                        self.drop_packet(
                            actor,
                            client_script::run(
                                2868,
                                &[Argument::String("Click here to continue".into())],
                            )
                            .map_err(protocol)?,
                        )?;
                        self.set_interface_events(actor, button, start, end, 1)?;
                        self.drop_packet(
                            actor,
                            interfaces::object(presentation.object_model.0, *id as u16, 400),
                        )?;
                        self.drop_packet(
                            actor,
                            interfaces::text(presentation.object_text.0, text).map_err(protocol)?,
                        )?;
                    }
                    Prompt::Choice { title, yes, no } => {
                        self.drop_packet(
                            actor,
                            client_script::run(
                                58,
                                &[
                                    Argument::String(title.clone()),
                                    Argument::String(format!("{yes}|{no}")),
                                ],
                            )
                            .map_err(protocol)?,
                        )?;
                        self.set_interface_events(actor, button, start, end, 1)?;
                    }
                    Prompt::Destroy {
                        id,
                        count,
                        title,
                        text,
                    } => {
                        self.drop_packet(
                            actor,
                            client_script::run(
                                814,
                                &[
                                    Argument::Int(*id),
                                    Argument::Int(*count),
                                    Argument::String(title.clone()),
                                    Argument::String(text.clone()),
                                ],
                            )
                            .map_err(protocol)?,
                        )?;
                        self.set_interface_events(actor, button, start, end, 1)?;
                    }
                }
                self.active.get_mut(&actor).unwrap().drop_dialog = Some((*token, button));
                Ok(())
            }
        }
    }
}
