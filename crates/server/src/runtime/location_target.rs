use super::*;
use gielinor_world::{simulation::Action, ui::Component, CoordGrid};
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn location_target(
        &mut self,
        actor: ActorId,
        target: gielinor_protocol::loc_input::Target,
    ) -> io::Result<()> {
        let err = |e| io::Error::other(format!("{e:?}"));
        if self.world.protection(actor).map_err(err)?.delayed() {
            return Ok(());
        }
        let level = self.world.observe(actor).map_err(err)?.coords.level();
        let coords = CoordGrid::new(target.x as i32, target.z as i32, level).ok();
        let exists = coords.is_some_and(|coords| {
            self.world
                .locations
                .find_type(coords, target.id as u32)
                .is_some()
        });
        let enabled = self
            .components
            .get(&target.source)
            .is_some_and(|component| {
                self.active[&actor].ui.location_target(
                    Component(target.source),
                    target.source_slot,
                    component.events,
                )
            });
        let component=Component(target.source);let ui=&self.active[&actor].ui;
        let source_open=ui.modals.contains_interface(component.interface())||ui.overlays.contains_interface(component.interface());
        let flags=ui.events.get(component,target.source_slot).unwrap_or_else(||self.components.get(&target.source).map_or(0,|c|u64::from(c.events as u32)));
        self.events.push(Event::LocationTargetChecked{actor,target,exists,source_open,flags,enabled});
        if !exists || !enabled {
            if let Some(connection) = self.active[&actor].connection {
                let packet = gielinor_protocol::game::clear_map_flag();
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
            self.events
                .push(Event::World(WorldEvent::ClearMapFlag(actor)));
        }
        // Capture permissions before ifClose removes the selected modal. Even
        // rejected interface/loc targets clear pending actions in Kotlin.
        let aborted = self.world.close_dialogs(actor).map_err(err)?;
        let pause = self.world.cancel_dialogue(actor).map_err(err)?;
        let active = self.active.get_mut(&actor).unwrap();
        let connection = active.connection;
        if aborted || pause {
            active.ui.input_dialog_open = false;
            if let Some(connection) = connection {
                self.transport.send(connection, 117, &[])?;
            }
        }
        let parents: Vec<_> = active
            .ui
            .modals
            .entries()
            .map(|(parent, _)| parent)
            .collect();
        for parent in parents {
            self.close_interface(actor, parent)?;
        }
        self.world
            .cancel_location_target_action(actor)
            .map_err(err)?;
        if !exists || !enabled {
            return Ok(());
        }
        self.world
            .reset_location_target_facing(actor)
            .map_err(err)?;
        if let Err(error) = self.world.submit(
            actor,
            Action::UseLocation {
                coords: coords.unwrap(),
                id: target.id as u32,
                slot: target.source_slot,
                object: target.source_object,
                control: target.control,
            },
        ) {
            if let Some(connection) = connection {
                self.events.push(Event::PacketRejected {
                    connection,
                    actor,
                    opcode: 62,
                    reason: format!("{error:?}"),
                });
            }
        }
        Ok(())
    }
}
