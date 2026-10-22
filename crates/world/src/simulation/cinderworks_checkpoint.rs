//! A content checkpoint uses the host's existing account transaction. No reward
//! becomes spendable until its inventory and paid marker are durably saved.
use super::*;
#[derive(Debug)]
pub(super) struct Pending {
    inventory: Inventory,
    run: crate::tower_run::Run,
    floors: crate::tower_progression::Progress,
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
        if p.pending_location_action.take().is_some() {
            p.protection.suspended = false;
        }
        p.queues.clear_weak();
        let mut data = p.data.clone();
        data.coords = p.movement.coords;
        data.inventory = inventory.clone();
        data.tower_run = run;
        data.tower_floors.reconcile_cinderworks(run);
        p.cinderworks_checkpoint = Some(Pending {
            inventory,
            run,
            floors: data.tower_floors.clone(),
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
        p.data.tower_floors = pending.floors;
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

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (World, ActorId) {
        let mut world = World::default();
        let at = CoordGrid::new(3200, 3200, 0).unwrap();
        world.collision.allocate(3200, 3200, 0);
        let mut data = PlayerData::new(1, "Snapshot fixture".into(), at, Stats::default());
        data.inventory
            .restore_slot(
                0,
                Some(crate::inventory::Item {
                    id: 995,
                    count: 2,
                    vars: 0,
                }),
            )
            .unwrap();
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        world.drain_events();
        (world, actor)
    }
    fn stage(world: &mut World, actor: ActorId) {
        let mut proposed = world.data(actor).unwrap().inventory.clone();
        proposed
            .restore_slot(
                0,
                Some(crate::inventory::Item {
                    id: 995,
                    count: 12,
                    vars: 0,
                }),
            )
            .unwrap();
        let mut run = world.data(actor).unwrap().tower_run;
        assert!(run.begin());
        world.checkpoint_cinderworks(actor, proposed, run, "Unit checkpoint acknowledged".into());
    }
    #[test]
    fn proposed_inventory_and_run_are_not_visible_before_ack() {
        let (mut world, actor) = fixture();
        stage(&mut world, actor);
        assert!(world.cinderworks_checkpoint_pending(actor));
        assert_eq!(
            world.data(actor).unwrap().inventory.slots()[0]
                .as_ref()
                .unwrap()
                .count,
            2
        );
        assert_eq!(world.data(actor).unwrap().tower_run.id, 0);
        let staged = world
            .drain_events()
            .into_iter()
            .find_map(|e| match e {
                WorldEvent::CinderworksCheckpoint { data, .. } => Some(data),
                _ => None,
            })
            .unwrap();
        assert_eq!(staged.inventory.slots()[0].as_ref().unwrap().count, 12);
        assert_eq!(staged.tower_run.id, 1);
        world.cinderworks_checkpoint_saved(actor).unwrap();
        assert!(!world.cinderworks_checkpoint_pending(actor));
        assert_eq!(
            world.data(actor).unwrap().inventory.slots()[0]
                .as_ref()
                .unwrap()
                .count,
            12
        );
        assert_eq!(world.data(actor).unwrap().tower_run.id, 1);
        assert!(world.cinderworks_checkpoint_saved(actor).is_err());
        assert_eq!(
            world.data(actor).unwrap().inventory.slots()[0]
                .as_ref()
                .unwrap()
                .count,
            12
        );
    }
    #[test]
    fn interrupted_logout_waits_for_checkpoint_then_saves_acknowledged_state() {
        let (mut world, actor) = fixture();
        stage(&mut world, actor);
        world.drain_events();
        world.request_shutdown(actor).unwrap();
        for _ in 0..3 {
            world.tick().unwrap();
            assert!(!world
                .drain_events()
                .into_iter()
                .any(|e| matches!(e, WorldEvent::Save { .. })));
            assert_eq!(
                world.data(actor).unwrap().inventory.slots()[0]
                    .as_ref()
                    .unwrap()
                    .count,
                2
            );
        }
        world.cinderworks_checkpoint_saved(actor).unwrap();
        world.tick().unwrap();
        let saved = world
            .drain_events()
            .into_iter()
            .find_map(|e| match e {
                WorldEvent::Save { data, .. } => Some(data),
                _ => None,
            })
            .unwrap();
        assert_eq!(saved.inventory.slots()[0].as_ref().unwrap().count, 12);
        assert_eq!(saved.tower_run.id, 1);
    }

    #[test]
    fn permanent_floor_unlock_waits_for_victory_save_acknowledgement() {
        use crate::{tower_progression::FloorId, tower_run::Service};
        let (mut world, actor) = fixture();
        let mut run = crate::tower_run::Run::default();
        run.begin();
        for service in [Service::Forge, Service::Kitchen, Service::Altar] { run.service(service); }
        run.defeat_imp(); run.defeat_warden();
        let inventory = world.data(actor).unwrap().inventory.clone();
        world.checkpoint_cinderworks(actor, inventory.clone(), run, "Synthetic victory".into());
        assert!(!world.data(actor).unwrap().tower_floors.completed(FloorId::CINDERWORKS));
        let staged = world.drain_events().into_iter().find_map(|event| match event {
            WorldEvent::CinderworksCheckpoint { data, .. } => Some(data), _ => None,
        }).unwrap();
        assert!(staged.tower_floors.completed(FloorId::CINDERWORKS));
        assert_eq!(staged.tower_floors.achieved(FloorId::CINDERWORKS),31);
        assert_eq!(world.data(actor).unwrap().tower_floors.achieved(FloorId::CINDERWORKS),0);
        assert!(world.cinderworks_checkpoint_pending(actor));
        world.cinderworks_checkpoint_saved(actor).unwrap();
        assert!(world.data(actor).unwrap().tower_floors.completed(FloorId::CINDERWORKS));
        assert_eq!(world.data(actor).unwrap().tower_floors.achieved(FloorId::CINDERWORKS),31);
        assert_eq!(world.data(actor).unwrap().inventory, inventory);
        assert!(world.data(actor).unwrap().tower_run.reward_pending);
        assert!(world.cinderworks_checkpoint_saved(actor).is_err());
    }
}
