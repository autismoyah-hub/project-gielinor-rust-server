//! A content checkpoint uses the host's existing account transaction. No reward
//! becomes spendable until its inventory and paid marker are durably saved.
use super::*;
#[derive(Debug)]
pub(super) struct Pending {
    inventory: Inventory,
    run: crate::tower_run::Run,
    message: String,
}
impl World {
    pub fn cinderworks_checkpoint_pending(&self, actor: ActorId) -> bool {
        self.player(actor)
            .is_ok_and(|p| p.cinderworks_checkpoint.is_some())
    }
    pub(super) fn checkpoint_cinderworks(
        &mut self,
        actor: ActorId,
        inventory: Inventory,
        run: crate::tower_run::Run,
        message: String,
    ) {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .filter(|p| p.id == actor)
            .unwrap();
        assert!(p.cinderworks_checkpoint.is_none());
        p.movement.abort();
        p.npc_interaction = None;
        p.location_interaction = None;
        p.object_interaction = None;
        p.queues.clear_weak();
        let mut data = p.data.clone();
        data.coords = p.movement.coords;
        data.inventory = inventory.clone();
        data.tower_run = run;
        p.cinderworks_checkpoint = Some(Pending {
            inventory,
            run,
            message,
        });
        self.events
            .push(WorldEvent::CinderworksCheckpoint { actor, data });
    }
    /// The runtime calls this only for the matching successful account-worker
    /// completion. Failed/uncertain writes retain the barrier and retry.
    pub fn cinderworks_checkpoint_saved(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let pending = p
            .cinderworks_checkpoint
            .take()
            .ok_or(ActionError::LoggingOut)?;
        p.data.inventory = pending.inventory;
        p.data.tower_run = pending.run;
        self.events.push(WorldEvent::CinderworksProgress {
            actor,
            run: p.data.tower_run,
        });
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: pending.message,
        });
        Ok(())
    }
}
