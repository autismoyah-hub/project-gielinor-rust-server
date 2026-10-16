use super::*;
use crate::skill_multi::{Active as SkillDialog, Dialog, Presentation};
use gielinor_world::ui::Component;

impl<B: AccountBackend> Runtime<B> {
    pub fn open_skill_multi(
        &mut self,
        actor: ActorId,
        presentation: Presentation,
        dialog: Dialog,
    ) -> io::Result<()> {
        fn err(e: impl std::fmt::Debug) -> io::Error {
            io::Error::other(format!("{e:?}"))
        }
        let setup = gielinor_protocol::client_script::run(
            presentation.setup_script,
            &dialog.setup_arguments().map_err(err)?,
        )
        .map_err(err)?;
        if !self.components.contains_key(&presentation.parent.0) {
            return Err(io::Error::other("Missing skillmulti parent"));
        }
        for slot in 0..dialog.options.len() {
            let component = Component::new(
                presentation.interface,
                crate::skill_multi::FIRST_OPTION + slot as u16,
            );
            if !self.components.contains_key(&component.0) {
                return Err(io::Error::other("Missing skillmulti option component"));
            }
        }
        let token = self.world.start_quantity_dialog(actor).map_err(err)?;
        if let Err(error) = self.open_interface(
            actor,
            Some(presentation.parent),
            presentation.interface,
            false,
        ) {
            self.world.cancel_quantity_dialog(actor).map_err(err)?;
            return Err(error);
        }
        self.active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid skillmulti actor"))?
            .skill_multi = Some(SkillDialog {
            smelting: false,
            dough: None,
            token,
            presentation,
            dialog: dialog.clone(),
        });
        if let Some(connection) = self.connection(actor) {
            self.transport
                .send(connection, setup.opcode, &setup.payload)?;
        }
        for slot in 0..dialog.options.len() {
            self.set_interface_events(
                actor,
                Component::new(
                    presentation.interface,
                    crate::skill_multi::FIRST_OPTION + slot as u16,
                ),
                0,
                crate::skill_multi::MAX_COUNT,
                1,
            )?;
        }
        Ok(())
    }
    pub(super) fn resume_skill_multi(
        &mut self,
        actor: ActorId,
        pause: gielinor_protocol::dialog::Pause,
    ) -> io::Result<bool> {
        fn err(e: impl std::fmt::Debug) -> io::Error {
            io::Error::other(format!("{e:?}"))
        }
        let Some(active) = self.active.get(&actor) else {
            return Ok(false);
        };
        let Some(dialog) = active.skill_multi.clone() else {
            return Ok(false);
        };
        if self.world.quantity_dialog_token(actor).map_err(err)? != Some(dialog.token) {
            self.active.get_mut(&actor).unwrap().skill_multi = None;
            return Ok(false);
        }
        let component = Component(pause.component);
        let Some(definition) = self.components.get(&pause.component) else {
            return Ok(true);
        };
        if !active.ui.modals.contains_interface(component.interface())
            && !active.ui.overlays.contains_interface(component.interface())
        {
            return Ok(true);
        }
        if !active
            .ui
            .events
            .enabled(component, pause.subcomponent, definition.events, 1)
        {
            return Ok(true);
        }
        // ResumePauseButtonHandler queues the clicked parent before resuming.
        // ProtectedAccessLost cancels the coroutine; queued UI closure still runs.
        let expected_modal =
            active.ui.modals.get(dialog.presentation.parent) == Some(dialog.presentation.interface);
        let baked = definition.events;
        if !self.active.get_mut(&actor).unwrap().ui.queue_pause_close(
            component,
            pause.subcomponent,
            baked,
        ) {
            return Ok(true);
        }
        if !expected_modal {
            self.world.cancel_quantity_dialog(actor).map_err(err)?;
            self.active.get_mut(&actor).unwrap().skill_multi = None;
            return Ok(true);
        }
        match self.world.resume_quantity_dialog(actor, dialog.token) {
            Ok(true) => (),
            Ok(false) => return Ok(true),
            Err(gielinor_world::simulation::ActionError::Delayed) => {
                self.active.get_mut(&actor).unwrap().skill_multi = None;
                return Ok(true);
            }
            Err(error) => return Err(err(error)),
        }
        self.active.get_mut(&actor).unwrap().skill_multi = None;
        self.close_dialogs_and_modals(actor)?;
        let selection = dialog
            .dialog
            .select(dialog.presentation, component, pause.subcomponent);
        if let (Some(request),Some(selection))=(dialog.dough,selection) {
            if let Err(error)=self.world.start_dough(actor,request,selection.index,selection.count) {
                self.events.push(Event::World(gielinor_world::simulation::WorldEvent::ActionRejected{actor,error}));
            }
        }
        if dialog.smelting {
            if let Some(selection) = selection {
                self.select_smelting(actor, selection)?;
            }
        }
        self.events
            .push(Event::SkillMultiSelected { actor, selection });
        Ok(true)
    }
}
