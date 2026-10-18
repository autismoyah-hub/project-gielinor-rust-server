use super::*;
use crate::counted_assembly::{Recipe, Request};
impl World {
    pub(super) fn request_counted_assembly(
        &mut self,
        actor: ActorId,
        recipe: Recipe,
        held: crate::held_use::Request,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if recipe.members && !p.member {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: "You need to be a member to make that item.",
            });
            return Ok(());
        }
        if i32::from(p.data.stats.current(recipe.skill)) < i32::from(recipe.level) {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: crate::assembly::level_message(recipe.skill),
            });
            return Ok(());
        }
        let request = Request::bind(
            recipe,
            held,
            &p.data.inventory,
            p.movement.coords,
            p.movement.last_movement,
        )
        .ok_or(ActionError::InvalidItem)?;
        self.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .counted_assembly_request = Some(request);
        Ok(())
    }
    pub fn take_counted_assembly_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .counted_assembly_request
            .take())
    }
    pub fn complete_counted_assembly(
        &mut self,
        actor: ActorId,
        request: Request,
        token: u64,
        count: i32,
    ) -> Result<(), ActionError> {
        if !self.resume_quantity_dialog(actor, token)? {
            return Ok(());
        }
        let p = self.player(actor)?;
        if p.pending_logout || p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        if p.movement.coords != request.origin || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidItem);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        match request.complete(
            &mut p.data.inventory,
            &self.object_types,
            &p.data.stats,
            p.member,
            count,
        ) {
            Ok(count) => {
                self.events.push(WorldEvent::CountedAssemblyMade {
                    actor,
                    first: request.recipe.first,
                    second: request.recipe.second,
                    output: request.recipe.output,
                    count,
                });
                self.events.push(WorldEvent::GameMessage {
                    actor,
                    text: request.recipe.message.into(),
                });
            }
            Err(outcome) => self.events.push(WorldEvent::LocationMessage {
                actor,
                message: crate::assembly::outcome_message_for_skill(outcome, request.recipe.skill),
            }),
        }
        Ok(())
    }
}
