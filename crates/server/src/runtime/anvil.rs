use super::*;
use gielinor_world::{
    anvil::{self, Open, Selection},
    ui::Component,
};
fn err(e: impl std::fmt::Debug) -> io::Error {
    io::Error::other(format!("{e:?}"))
}
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_anvil(&mut self, config: crate::anvil::Config) -> io::Result<()> {
        if !self.connections.is_empty()
            || !self.active.is_empty()
            || self
                .world
                .smithing
                .is_none_or(|make| make.skill != config.rules.skill)
            || !self.components.contains_key(&config.parent.0)
            || config
                .quantity_buttons
                .keys()
                .any(|id| !self.components.contains_key(id))
            || config
                .rules
                .products
                .values()
                .flat_map(|grid| grid.keys())
                .any(|&slot| {
                    !self
                        .components
                        .contains_key(&Component::new(config.interface, slot).0)
                })
            || config
                .locations
                .iter()
                .any(|id| !self.definitions.locations.contains_key(id))
            || config
                .bars
                .iter()
                .any(|id| !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure valid anvil definitions after smithing and before admission",
            ));
        }
        for &id in &config.bars {
            self.world.held_use_types.insert(
                id,
                gielinor_world::held_use::Type {
                    id,
                    content: self.definitions.items[&(id as u32)].content_group,
                },
            );
        }
        self.world.anvils = config.locations.clone();
        self.world.smithing_bars = config.bars.clone();
        self.anvil = Some(config);
        Ok(())
    }
    fn smithing_message(&mut self, actor: ActorId, message: &str) -> io::Result<()> {
        if let Some(connection) = self.connection(actor) {
            let packet = gielinor_protocol::chat::game(0, None, message).map_err(err)?;
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub fn open_anvil(&mut self, actor: ActorId, preferred: Option<i32>) -> io::Result<()> {
        let config = self
            .anvil
            .as_ref()
            .ok_or_else(|| io::Error::other("Anvil is not configured"))?;
        let data = self.world.data(actor).map_err(err)?;
        let result = config.rules.prepare(
            &data.inventory,
            &self.world.object_types,
            &data.stats,
            preferred,
        );
        let (parent, interface, bar_type) = (config.parent, config.interface, config.bar_type_id);
        match result {
            Open::Denied(message) => self.smithing_message(actor, message),
            Open::Tier(tier) => {
                self.set_varbit(actor, bar_type, tier)?;
                // Player.ifOpenMainModal prepares the top-level background first.
                self.run_client_script(
                    actor,
                    2524,
                    &[
                        gielinor_protocol::client_script::Argument::Int(-1),
                        gielinor_protocol::client_script::Argument::Int(-1),
                    ],
                )?;
                self.open_interface(actor, Some(parent), interface, false)
            }
        }
    }
    pub(super) fn anvil_button(&mut self, actor: ActorId, component: u32) -> io::Result<()> {
        let Some(config) = self
            .anvil
            .as_ref()
            .filter(|c| c.interface == Component(component).interface())
        else {
            return Ok(());
        };
        if let Some(count) = config.quantity_buttons.get(&component).copied() {
            return match count {
                Some(count) => {
                    self.set_varp(actor, config.quantity.id as u32, anvil::quantity(count))
                }
                None => self.open_anvil_count(actor),
            };
        }
        let data = self.world.data(actor).map_err(err)?;
        let tier = config.bar_type.get(&data.variables).map_err(err)?;
        let result = config.rules.select(
            &data.inventory,
            &self.world.object_types,
            &data.stats,
            tier,
            Component(component).child(),
        );
        let quantity = config.quantity.id;
        match result {
            Selection::Ignore => Ok(()),
            Selection::Denied(message) => self.smithing_message(actor, &message),
            Selection::Start { request } => {
                self.close_dialogs_and_modals(actor)?;
                let count =
                    anvil::quantity(self.world.data(actor).map_err(err)?.variables.get(quantity));
                self.world
                    .start_smithing(actor, request, count)
                    .map_err(err)
            }
        }
    }
    fn open_anvil_count(&mut self, actor: ActorId) -> io::Result<()> {
        let config = self
            .anvil
            .as_ref()
            .ok_or_else(|| io::Error::other("Anvil is not configured"))?;
        let (parent, interface) = (config.parent, config.interface);
        if self.active[&actor].ui.modals.get(parent) != Some(interface) {
            return Ok(());
        }
        let packet = gielinor_protocol::client_script::run(
            108,
            &[gielinor_protocol::client_script::Argument::String(
                "Enter amount:".into(),
            )],
        )
        .map_err(err)?;
        let token = self.world.start_modal_count_dialog(actor).map_err(err)?;
        let active = self.active.get_mut(&actor).unwrap();
        active.anvil_count = Some(crate::anvil::CountDialog {
            token,
            parent,
            interface,
        });
        active.ui.input_dialog_open = true;
        if let Some(connection) = active.connection {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub(super) fn resume_anvil_count(&mut self, actor: ActorId, count: i32) -> io::Result<bool> {
        let Some(dialog) = self.active.get(&actor).and_then(|a| a.anvil_count) else {
            return Ok(false);
        };
        if self.world.quantity_dialog_token(actor).map_err(err)? != Some(dialog.token) {
            self.active.get_mut(&actor).unwrap().anvil_count = None;
            return Ok(false);
        }
        let active = self.active.get_mut(&actor).unwrap();
        active.anvil_count = None;
        active.ui.input_dialog_open = false;
        if active.ui.modals.get(dialog.parent) != Some(dialog.interface) {
            self.world.cancel_quantity_dialog(actor).map_err(err)?;
            return Ok(true);
        }
        match self.world.resume_quantity_dialog(actor, dialog.token) {
            Ok(true) => {
                let id = self
                    .anvil
                    .as_ref()
                    .ok_or_else(|| io::Error::other("Anvil configuration disappeared"))?
                    .quantity
                    .id;
                self.set_varp(actor, id as u32, anvil::count_input(count))?;
                self.events.push(Event::CountDialogInput {
                    actor,
                    count,
                    accepted: true,
                });
            }
            Ok(false) | Err(gielinor_world::simulation::ActionError::Delayed) => {
                self.events.push(Event::CountDialogInput {
                    actor,
                    count,
                    accepted: false,
                })
            }
            Err(error) => return Err(err(error)),
        }
        Ok(true)
    }
}
