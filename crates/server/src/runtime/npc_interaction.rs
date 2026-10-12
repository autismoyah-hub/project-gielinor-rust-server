use super::*;
use gielinor_world::{
    npc_interaction::{Event as NpcEvent, Handler},
    simulation::{Action, NpcId},
    ui::Component,
};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_bankers(
        &mut self,
        groups: &[i32],
        parents: (Component, Component),
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure bankers before admission"));
        }
        for (&id, ty) in &self.definitions.npcs {
            if groups.contains(&ty.content_group) {
                if let Ok(id) = u16::try_from(id) {
                    self.world.npc_handlers.insert((id, 3), Handler::Bank);
                }
            }
        }
        self.npc_bank_parents = Some(parents);
        Ok(())
    }
    pub(super) fn npc_button(
        &mut self,
        actor: ActorId,
        op: gielinor_protocol::npc_input::Operation,
    ) -> io::Result<()> {
        if op.suboperation != 0 {
            return Ok(());
        }
        let delayed = self
            .world
            .protection(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .delayed();
        let target = self.world.npc_at_index(op.index);
        if !delayed
            && (target.is_none()
                || target.is_some_and(|t| self.world.can_operate_npc(actor, t, op.operation)))
        {
            self.close_dialogs_and_modals(actor)?;
        }
        self.world
            .submit(
                actor,
                Action::OperateNpc {
                    target: target.unwrap_or(NpcId {
                        slot: op.index,
                        generation: 0,
                    }),
                    operation: op.operation,
                    control: op.control,
                },
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    pub(super) fn npc_event(&mut self, actor: ActorId, event: NpcEvent) -> io::Result<()> {
        match event {
            NpcEvent::Face(target) => {
                if let Some(active) = self.active.get_mut(&actor) {
                    active.face_npc = Some(target);
                    active.pending_face = None;
                    active.face_changed = true;
                }
            }
            NpcEvent::Bank => {
                if let Some((main, side)) = self.npc_bank_parents {
                    self.open_bank(actor, main, side)?;
                }
            }
            NpcEvent::Message(text) => {
                if let Some(connection) = self.connection(actor) {
                    let packet = gielinor_protocol::chat::game(0, None, text)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
        }
        Ok(())
    }
}
