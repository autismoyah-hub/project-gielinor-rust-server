use super::*;
use gielinor_protocol::{
    client_script::{self, Argument},
    interfaces,
};
use gielinor_world::prayer_book::Event as PrayerEvent;
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn prayer_tab_event(
        &mut self,
        actor: ActorId,
        event: &PrayerEvent,
    ) -> io::Result<()> {
        if let PrayerEvent::Overhead(icon) = event {
            if let Some(active) = self.active.get_mut(&actor) {
                active.design.overhead = icon.map_or(-1, |v| v as i8);
            }
        }
        let Some(connection) = self.connection(actor) else {
            return Ok(());
        };
        let packet = match event {
            PrayerEvent::Variable(update) => {
                gielinor_protocol::varp::update(update.id, update.value)
            }
            PrayerEvent::Sound(id) => gielinor_protocol::game::synth_sound(*id, 1, 0),
            PrayerEvent::Message { text, .. } => gielinor_protocol::chat::game(0, None, text)
                .map_err(|e| io::Error::other(format!("{e:?}")))?,
            PrayerEvent::RefreshCombatRange => {
                let data = self
                    .world
                    .data(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                let presentation = self
                    .presentation
                    .as_ref()
                    .ok_or_else(|| io::Error::other("Combat presentation missing"))?;
                let combat = presentation.combat_level(data);
                client_script::run(5224, &[Argument::Int(combat as i32)])
                    .map_err(|e| io::Error::other(format!("{e:?}")))?
            }
            // Timers and preserve queues are applied by the world before these effects.
            PrayerEvent::Drain(_) | PrayerEvent::Regen { .. } | PrayerEvent::Overhead(_) => {
                return Ok(())
            }
        };
        self.transport
            .send(connection, packet.opcode, &packet.payload)
    }
    pub(super) fn prayer_notice(
        &mut self,
        actor: ActorId,
        token: u64,
        text: &str,
    ) -> io::Result<()> {
        if self.connection(actor).is_none() {
            return Ok(());
        }
        let font = &self
            .dialogue_presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Dialogue font missing"))?
            .font;
        let mut pages: std::collections::VecDeque<_> =
            font.message_pages(text).map_err(io::Error::other)?.into();
        let first = pages
            .pop_front()
            .ok_or_else(|| io::Error::other("Empty prayer notice"))?;
        self.show_prayer_notice_page(actor, &first)?;
        self.active.get_mut(&actor).unwrap().prayer_notice = Some(crate::message_box::Active {
            token,
            remaining: pages,
        });
        Ok(())
    }
    fn show_prayer_notice_page(
        &mut self,
        actor: ActorId,
        page: &(String, usize),
    ) -> io::Result<()> {
        let p = self
            .message_box
            .ok_or_else(|| io::Error::other("Message box presentation missing"))?;
        let Some(connection) = self.connection(actor) else {
            return Ok(());
        };
        let protocol = |e| io::Error::other(format!("{e:?}"));
        let message = gielinor_protocol::chat::game(115, None, &page.0).map_err(protocol)?;
        self.transport
            .send(connection, message.opcode, &message.payload)?;
        self.open_interface(actor, Some(p.parent), p.interface, false)?;
        for packet in [
            interfaces::text(p.text.0, &page.0).map_err(protocol)?,
            client_script::run(
                600,
                &[
                    Argument::Int(1),
                    Argument::Int(1),
                    Argument::Int(crate::message_box::line_height(page.1)),
                    Argument::Int(p.text.0 as i32),
                ],
            )
            .map_err(protocol)?,
        ] {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        self.set_interface_events(actor, p.button, -1, -1, 1)?;
        for packet in [
            interfaces::text(p.button.0, "Click here to continue").map_err(protocol)?,
            client_script::run(1508, &[Argument::String("0".into())]).map_err(protocol)?,
        ] {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub(super) fn resume_prayer_notice(
        &mut self,
        actor: ActorId,
        pause: gielinor_protocol::dialog::Pause,
    ) -> io::Result<()> {
        let Some(p) = self.message_box else {
            return Ok(());
        };
        let Some(active) = self.active.get(&actor) else {
            return Ok(());
        };
        let Some(notice) = &active.prayer_notice else {
            return Ok(());
        };
        if pause.component != p.button.0 || !active.ui.modals.contains_interface(p.interface) {
            return Ok(());
        }
        let Some(definition) = self.components.get(&p.button.0) else {
            return Ok(());
        };
        if !active
            .ui
            .events
            .enabled(p.button, pause.subcomponent, definition.events, 1)
        {
            return Ok(());
        }
        if self
            .world
            .prayer_notice_token(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            != Some(notice.token)
        {
            return Ok(());
        }
        let delayed = self
            .world
            .protection(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .delayed();
        let mut notice = self
            .active
            .get_mut(&actor)
            .unwrap()
            .prayer_notice
            .take()
            .unwrap();
        if !delayed && !notice.remaining.is_empty() {
            let page = notice.remaining.pop_front().unwrap();
            self.show_prayer_notice_page(actor, &page)?;
            self.active.get_mut(&actor).unwrap().prayer_notice = Some(notice);
        } else {
            self.world
                .close_prayer_notice(actor, notice.token)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            // The input handler already queued this modal for the main pass.
            // Delayed resumes lose protected access and discard remaining pages.
        }
        Ok(())
    }
}
