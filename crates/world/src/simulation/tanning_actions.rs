use super::*;
use crate::{
    reach::Rectangle,
    tanning::{hide_slots, Request},
};
impl World {
    fn tanning_reached(&self, actor: ActorId, target: NpcId, kind: u16) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.pending_logout || p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        let npc = self
            .npc(target)
            .filter(|n| n.type_id == kind && n.visible && !n.delayed && n.stats[3] > 0)
            .ok_or(ActionError::InvalidItem)?;
        if !self.can_operate_npc(actor, target, 3)
            || !(Rectangle {
                coords: npc.movement.coords,
                width: npc.movement.size,
                length: npc.movement.size,
                angle: 0,
                blocked: 0,
                exclusive: true,
            })
            .reached(&self.collision, p.movement.coords, p.movement.size)
        {
            return Err(ActionError::InvalidItem);
        }
        Ok(())
    }
    pub fn tanning_request(&self, actor: ActorId, target: NpcId) -> Result<Request, ActionError> {
        let kind = self.npc(target).ok_or(ActionError::InvalidItem)?.type_id;
        self.tanning_reached(actor, target, kind)?;
        let recipes = self.tanning.get(&kind).ok_or(ActionError::InvalidItem)?;
        let p = self.player(actor)?;
        Ok(Request {
            target,
            npc_type: kind,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            choices: recipes
                .iter()
                .filter(|r| !r.members || p.member)
                .map(|&r| (r, hide_slots(&p.data.inventory, r.input)))
                .collect(),
        })
    }
    /// Called only after the existing skill dialog consumes its single-use token.
    pub fn complete_tanning(
        &mut self,
        actor: ActorId,
        request: Request,
        index: usize,
        count: i32,
    ) -> Result<(), ActionError> {
        self.tanning_reached(actor, request.target, request.npc_type)?;
        let p = self.player(actor)?;
        if p.movement.coords != request.origin || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidItem);
        }
        let (recipe, slots) = request.choices.get(index).ok_or(ActionError::InvalidItem)?;
        if !self
            .tanning
            .get(&request.npc_type)
            .is_some_and(|rows| rows.contains(recipe))
        {
            return Err(ActionError::InvalidItem);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let result = crate::tanning::exchange(
            &mut p.data.inventory,
            &self.object_types,
            *recipe,
            slots,
            count,
            p.member,
        );
        let text = match result {
            Ok(amount) => format!("The tanner tans {amount} hide(s)."),
            Err(crate::tanning::Error::Coins) => {
                "You do not have enough coins to tan those hides.".into()
            }
            Err(crate::tanning::Error::Hides) => {
                "You do not have any suitable hides to tan.".into()
            }
            Err(_) => "Those hides cannot be tanned right now.".into(),
        };
        self.events.push(WorldEvent::GameMessage { actor, text });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        npc::{BlockWalk, Npc, Restrict},
        npc_interaction::{Event, Handler},
        tanning::Recipe,
    };
    #[test]
    fn tanning_reached_trade_quantity_and_stale_target_or_movement() {
        for denial in 0..3 {
            let mut w = World::default();
            w.collision.allocate(3200, 3200, 0);
            w.object_types.stackable.insert(995);
            w.tanning.insert(
                3231,
                vec![
                    Recipe {
                        input: 1739,
                        output: 1741,
                        cost: 1,
                        members: false,
                    },
                    Recipe {
                        input: 1739,
                        output: 1743,
                        cost: 3,
                        members: false,
                    },
                ],
            );
            w.npc_handlers.insert((3231, 3), Handler::Tanning);
            let mut def = crate::npc_options::Definition::default();
            def.operations[2] = Some("Trade".into());
            w.npc_options.types.insert(3231, def);
            let target = w
                .spawn_npc(
                    Npc::new(
                        3231,
                        CoordGrid::new(3201, 3200, 0).unwrap(),
                        1,
                        Restrict::NoMove,
                        BlockWalk::None,
                    )
                    .unwrap(),
                )
                .unwrap();
            let mut data = PlayerData::new(
                1,
                "Tanning".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            );
            for slot in 0..27 {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1739,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            data.inventory
                .restore_slot(
                    27,
                    Some(Item {
                        id: 995,
                        count: 80,
                        vars: 0,
                    }),
                )
                .unwrap();
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let actor = w.actors().next().unwrap();
            w.drain_events();
            w.submit(
                actor,
                Action::OperateNpc {
                    target,
                    operation: 3,
                    control: false,
                },
            )
            .unwrap();
            w.tick().unwrap();
            assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::NpcInteraction{actor:a,event:Event::Tanning(n)}if *a==actor&&*n==target)));
            let request = w.tanning_request(actor, target).unwrap();
            let before = w.data(actor).unwrap().clone();
            let token = w.start_quantity_dialog(actor).unwrap();
            assert!(w.resume_quantity_dialog(actor, token).unwrap());
            if denial == 1 {
                w.players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .movement
                    .last_movement += 1;
            }
            if denial == 2 {
                w.despawn_npc(target);
            }
            let result = w.complete_tanning(actor, request, 1, 28);
            let after = w.data(actor).unwrap();
            assert_eq!(after.stats, before.stats);
            if denial == 0 {
                assert!(result.is_ok());
                assert_eq!(after.inventory.get(27).unwrap().count, 2);
                assert_eq!(crate::tanning::hide_slots(&after.inventory, 1739).len(), 1);
                assert_eq!(
                    after
                        .inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 1743)
                        .count(),
                    26
                );
            } else {
                assert!(result.is_err());
                assert_eq!(after.inventory, before.inventory);
            }
            assert!(!w.resume_quantity_dialog(actor, token).unwrap());
        }
    }
}
