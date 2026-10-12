use super::*;
use gielinor_protocol::client_script::{self, Argument};
use gielinor_world::{prayer::Event as PrayerEvent, simulation::Action, ui::Component};
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn resume_prayer_pause(
        &mut self,
        actor: ActorId,
        pause: gielinor_protocol::dialog::Pause,
    ) -> io::Result<()> {
        let Some(active) = self.active.get(&actor) else {
            return Ok(());
        };
        let Some((token, button)) = active.prayer_dialog else {
            return Ok(());
        };
        if button.0 != pause.component || !active.ui.modals.contains_interface(button.interface()) {
            return Ok(());
        }
        let Some(definition) = self.components.get(&pause.component) else {
            return Ok(());
        };
        if !active
            .ui
            .events
            .enabled(button, pause.subcomponent, definition.events, 1)
        {
            return Ok(());
        }
        self.world
            .submit(
                actor,
                Action::ResumeBury {
                    token,
                    choice: pause.subcomponent,
                },
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    pub(super) fn prayer_event(&mut self, actor: ActorId, event: &PrayerEvent) -> io::Result<()> {
        let Some(connection) = self.connection(actor) else {
            return Ok(());
        };
        let protocol = |e| io::Error::other(format!("{e:?}"));
        match event {
            PrayerEvent::Confirm { token } => {
                let presentation = self
                    .drop_presentation
                    .ok_or_else(|| io::Error::other("Prayer confirmation presentation missing"))?;
                let button: Component = presentation.choice_button;
                self.active.get_mut(&actor).unwrap().prayer_dialog = None;
                self.set_varbit(actor, presentation.unclamp, 1)?;
                let reset = client_script::run(2379, &[]).map_err(protocol)?;
                self.transport
                    .send(connection, reset.opcode, &reset.payload)?;
                self.open_interface(
                    actor,
                    Some(presentation.parent),
                    presentation.choice_interface,
                    false,
                )?;
                let menu = client_script::run(
                    58,
                    &[
                        Argument::String("Are you sure you want to do that?".into()),
                        Argument::String("Bury the bone|Cancel".into()),
                    ],
                )
                .map_err(protocol)?;
                self.transport
                    .send(connection, menu.opcode, &menu.payload)?;
                self.set_interface_events(actor, button, 1, 2, 1)?;
                self.active.get_mut(&actor).unwrap().prayer_dialog = Some((*token, button));
            }
            PrayerEvent::Closed => {
                if self
                    .active
                    .get_mut(&actor)
                    .unwrap()
                    .prayer_dialog
                    .take()
                    .is_some()
                {
                    if let Some(presentation) = self.drop_presentation {
                        self.close_interface(actor, presentation.parent)?;
                    }
                }
            }
            PrayerEvent::Animation(sequence) => {
                self.active.get_mut(&actor).unwrap().sequence = Some((*sequence, 0))
            }
            PrayerEvent::Completed { skill, .. } => self
                .active
                .get_mut(&actor)
                .unwrap()
                .stats
                .invalidate(*skill),
            PrayerEvent::Message(message) => {
                let packet = gielinor_protocol::chat::game(0, None, message).map_err(protocol)?;
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        Ok(())
    }
}
