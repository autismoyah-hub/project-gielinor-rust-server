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
    pub(super) fn deposit_box_item(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
        operation: u8,
    ) -> Result<(), ActionError> {
        let config = self.deposit_box.ok_or(ActionError::InvalidBank)?;
        let quantities = config.quantities.ok_or(ActionError::InvalidBank)?;
        if self.player(actor)?.protection.modal_button_protected() {
            return Err(ActionError::Protected);
        }
        let bank = self.banking.as_ref().ok_or(ActionError::InvalidBank)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let result =
            quantities.deposit(bank, &mut p.data, &self.object_types, slot, item, operation);
        let changed = result.as_ref().is_ok_and(|(_, changed)| *changed);
        self.events.push(WorldEvent::Bank {
            actor,
            result: result.map(|(events, _)| events),
        });
        if changed {
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: config.animation,
            });
        }
        Ok(())
    }
    pub(super) fn deposit_box_mode(
        &mut self,
        actor: ActorId,
        mode: i32,
    ) -> Result<(), ActionError> {
        let quantities = self
            .deposit_box
            .and_then(|c| c.quantities)
            .ok_or(ActionError::InvalidBank)?;
        if self.player(actor)?.protection.modal_button_protected() {
            return Err(ActionError::Protected);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let result = quantities.select(&mut p.data, mode);
        self.events.push(WorldEvent::Bank { actor, result });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    #[test]
    fn deposit_box_item_action_uses_selected_preset_and_live_item_identity() {
        let mut w = World::default();
        let bank = crate::bank_actions::tests::config();
        let mut data = PlayerData::new(1, "Item action".into(), CoordGrid::ZERO, Stats::default());
        bank.capacity.set(&mut data.variables, 80).unwrap();
        data.inventory
            .restore_slot(
                7,
                Some(Item {
                    id: 995,
                    count: 12,
                    vars: 0,
                }),
            )
            .unwrap();
        w.object_types.stackable.insert(995);
        w.banking = Some(bank);
        w.deposit_box = Some(crate::deposit_box::Config {
            animation: 834,
            quantities: Some(crate::deposit_box_items::tests::quantities()),
        });
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.drain_events();
        w.submit(actor, Action::DepositBoxMode { mode: 4 }).unwrap();
        w.tick().unwrap();
        w.drain_events();
        w.submit(
            actor,
            Action::DepositBoxItem {
                slot: 7,
                item: 995,
                operation: 1,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.data(actor).unwrap().inventory.get(7).unwrap().count, 2);
        assert_eq!(w.data(actor).unwrap().bank.get(0).unwrap().count, 10);
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 834, .. })));
        w.submit(
            actor,
            Action::DepositBoxItem {
                slot: 7,
                item: 2,
                operation: 6,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.data(actor).unwrap().inventory.get(7).unwrap().count, 2);
        assert!(!w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { .. })));
    }
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
            w.deposit_box = Some(crate::deposit_box::Config {
                animation: 834,
                quantities: None,
            });
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
