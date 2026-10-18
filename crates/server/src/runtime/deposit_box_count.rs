//! Deposit modal X prompts reuse the shared protected numeric continuation.
use super::*;
use gielinor_world::deposit_box_items::CountPurpose;
#[derive(Clone, Copy)]
pub(super) struct CountDialog {
    pub token: u64,
    pub parent: gielinor_world::ui::Component,
    pub interface: u16,
    pub purpose: CountPurpose,
}
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn deposit_box_count_button(
        &mut self,
        actor: ActorId,
        button: &gielinor_protocol::buttons::Button,
    ) -> io::Result<bool> {
        let Some(p) = self.deposit_box_presentation else {
            return Ok(false);
        };
        let Some(parent) = self
            .active
            .get(&actor)
            .and_then(|a| a.ui.modals.parent(p.interface))
        else {
            return Ok(false);
        };
        let purpose = if button.component == p.quantity_x.0 && button.op == 1 {
            CountPurpose::Mode
        } else if button.component == p.grid.0 && button.op == 5 {
            let Ok(slot) = usize::try_from(button.slot) else {
                return Ok(true);
            };
            if slot >= 28
                || !self
                    .world
                    .data(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                    .inventory
                    .get(slot)
                    .is_some_and(|i| i.id == button.object)
            {
                return Ok(true);
            }
            CountPurpose::Item {
                slot,
                expected: button.object,
            }
        } else {
            return Ok(false);
        };
        let packet = gielinor_protocol::client_script::run(
            108,
            &[gielinor_protocol::client_script::Argument::String(
                "Enter amount:".into(),
            )],
        )
        .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let token = match self.world.start_modal_count_dialog(actor) {
            Ok(t) => t,
            Err(error) => {
                self.events
                    .push(Event::World(WorldEvent::ActionRejected { actor, error }));
                return Ok(true);
            }
        };
        let active = self.active.get_mut(&actor).unwrap();
        active.deposit_count = Some(CountDialog {
            token,
            parent,
            interface: p.interface,
            purpose,
        });
        active.ui.input_dialog_open = true;
        if let Some(connection) = active.connection {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(true)
    }
    pub(super) fn resume_deposit_count(&mut self, actor: ActorId, count: i32) -> io::Result<bool> {
        let Some(dialog) = self
            .active
            .get_mut(&actor)
            .and_then(|a| a.deposit_count.take())
        else {
            return Ok(false);
        };
        if self
            .world
            .quantity_dialog_token(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            != Some(dialog.token)
        {
            return Ok(false);
        }
        let active = self.active.get_mut(&actor).unwrap();
        active.ui.input_dialog_open = false;
        if active.ui.modals.get(dialog.parent) != Some(dialog.interface) {
            self.world
                .cancel_quantity_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            return Ok(true);
        }
        let accepted =
            match self
                .world
                .complete_deposit_box_count(actor, dialog.token, dialog.purpose, count)
            {
                Ok(accepted) => accepted,
                Err(error) => {
                    self.events
                        .push(Event::World(WorldEvent::ActionRejected { actor, error }));
                    false
                }
            };
        // Acceptance refers to this continuation, not proof of a successful bank transfer.
        self.events.push(Event::CountDialogInput {
            actor,
            count,
            accepted,
        });
        Ok(true)
    }
}
