use super::*;
impl World {
    pub fn reset_location_target_facing(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        self.events.push(WorldEvent::ResetFaceEntity { actor });
        Ok(())
    }
    /// InvItems' default LocT handler verifies the selected item at operation
    /// time, after routing. Replacing an item on the way must never use the new
    /// item or consume a matching item from another slot.
    pub(super) fn process_location_use(
        &mut self,
        slot: usize,
        loc: crate::locations::Location,
        inv_slot: i32,
        object: i32,
    ) {
        let p = self.players.get_mut(slot).unwrap();
        let valid = self.held_use_types.contains_key(&object)
            && usize::try_from(inv_slot)
                .ok()
                .and_then(|slot| p.data.inventory.get(slot))
                .is_some_and(|item| item.id == object);
        if !valid {
            if !p.data.inventory.slots().is_empty() {
                let first = p.data.inventory.get(0);
                let _ = p.data.inventory.restore_slot(0, first);
            }
            return;
        }
        if self.start_cinderworks_cooking(slot,loc,object) {return;}
        let p = self.players.get_mut(slot).unwrap();
        if self.water_sources.as_ref().is_some_and(|c| c.output(loc.id, object).is_some()) {
            self.start_water_filling(slot, loc, inv_slot as usize, object);
            return;
        }
        if self
            .milking
            .as_ref()
            .is_some_and(|c| c.cows.contains_key(&loc.id) && object == c.bucket)
        {
            self.start_milking(slot, loc, Some(inv_slot as usize));
            return;
        }
        if self
            .tower_offering
            .as_ref()
            .is_some_and(|c| c.location == loc.id && c.accepted.contains(&object))
        {
            let actor = p.id;
            if let Err(error) = self.begin_tower_offering(actor) {
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
            }
            return;
        }
        if self
            .tutorial_bread
            .is_some_and(|c| c.location == loc.id && c.dough == object)
        {
            self.process_tutorial_bread(slot);
            return;
        }
        if self
            .tutorial_shrimp
            .is_some_and(|c| c.location == loc.id && c.raw == object)
        {
            self.process_tutorial_shrimp(slot);
            return;
        }
        if self
            .furnaces
            .get(&loc.id)
            .is_some_and(|furnace| furnace.accepts_ore(object, &self.smelting_ores))
        {
            self.process_smelting(slot, loc.id);
            return;
        }
        if self.anvils.contains(&loc.id) && self.smithing_bars.contains(&object) {
            self.events.push(WorldEvent::AnvilRequested {
                actor: p.id,
                preferred: Some(object),
            });
            return;
        }
        self.events.push(WorldEvent::EngineMessage {
            actor: p.id,
            message: "Nothing interesting happens.",
        });
    }
    /// Finish clearPendingAction after the host has run modal-close callbacks.
    /// The route itself is retained, as in Kotlin clearInteraction.
    pub fn cancel_location_target_action(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.location_interaction = None;
        p.object_interaction = None;
        p.npc_interaction = None;
        p.movement.interacting = false;
        Ok(())
    }
}
