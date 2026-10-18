use super::*;
impl World {
    pub(super) fn deposit_box_all(
        &mut self,
        actor: ActorId,
        worn: bool,
    ) -> Result<(), ActionError> {
        let config = self.deposit_box.ok_or(ActionError::InvalidBank)?;
        let p = self.player(actor)?;
        if p.protection.modal_button_protected() {
            return Err(ActionError::Protected);
        }
        let bank = self.banking.as_ref().ok_or(ActionError::InvalidBank)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let result = crate::deposit_box::deposit_all(bank, &mut p.data, &self.object_types, worn);
        let changed = result.as_ref().is_ok_and(|(_, changed)| *changed);
        let result = result.map(|(events, _)| events);
        if let Ok(events) = &result {
            for event in events {
                if let crate::bank_actions::Event::WornDeposited { slot, item } = event {
                    self.worn_changed(actor, *slot, *item);
                }
            }
        }
        self.events.push(WorldEvent::Bank { actor, result });
        if changed {
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: config.animation,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    #[test]
    fn deposit_box_action_dispatch_bulk_empty_and_worn_use_shared_engine() {
        for worn in [false, true] {
            let mut w = World::default();
            let bank = crate::bank_actions::tests::config();
            let mut data =
                PlayerData::new(1, "Box action".into(), CoordGrid::ZERO, Stats::default());
            bank.capacity.set(&mut data.variables, 80).unwrap();
            let source = if worn {
                &mut data.equipment
            } else {
                &mut data.inventory
            };
            source
                .restore_slot(
                    3,
                    Some(Item {
                        id: 2,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            w.banking = Some(bank);
            w.deposit_box = Some(crate::deposit_box::Config { animation: 834 });
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            w.tick().unwrap();
            let actor = w.actors().next().unwrap();
            w.drain_events();
            w.submit(actor, Action::DepositBoxAll { worn }).unwrap();
            w.tick().unwrap();
            let events = w.drain_events();
            assert!(events
                .iter()
                .any(|e| matches!(e, WorldEvent::Bank { result: Ok(_), .. })));
            assert!(events
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 834, .. })));
            assert!(w
                .data(actor)
                .unwrap()
                .bank
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == 2));
            let source = if worn {
                &w.data(actor).unwrap().equipment
            } else {
                &w.data(actor).unwrap().inventory
            };
            assert!(source.get(3).is_none());
            w.submit(actor, Action::DepositBoxAll { worn }).unwrap();
            w.tick().unwrap();
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { .. })));
        }
    }
}
