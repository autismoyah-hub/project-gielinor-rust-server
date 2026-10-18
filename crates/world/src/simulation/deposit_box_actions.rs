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
    /// Consume the same one-use continuation as anvil/Make-X; no second dialog engine.
    pub fn complete_deposit_box_count(
        &mut self,
        actor: ActorId,
        token: u64,
        purpose: crate::deposit_box_items::CountPurpose,
        count: i32,
    ) -> Result<bool, ActionError> {
        let config = self.deposit_box.ok_or(ActionError::InvalidBank)?;
        let quantities = config.quantities.ok_or(ActionError::InvalidBank)?;
        let player = self.player(actor)?;
        if player.saving || player.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if !self.resume_quantity_dialog(actor, token)? {
            return Ok(false);
        }
        // ProtectedAccess.countDialog applies Kotlin absoluteValue; MIN stays negative.
        let count = count.wrapping_abs();
        if count <= 0 {
            return Ok(false);
        }
        let bank = self.banking.as_ref().ok_or(ActionError::InvalidBank)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let result = match purpose {
            crate::deposit_box_items::CountPurpose::Mode => quantities
                .input(&mut p.data, count)
                .map(|events| (events, false)),
            crate::deposit_box_items::CountPurpose::Item { slot, expected } => quantities
                .deposit_amount(bank, &mut p.data, &self.object_types, slot, expected, count),
        };
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
        Ok(true)
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
    fn count_world() -> (World, ActorId) {
        let mut w = World::default();
        let bank = crate::bank_actions::tests::config();
        let mut data = PlayerData::new(1, "Box X".into(), CoordGrid::ZERO, Stats::default());
        bank.capacity.set(&mut data.variables, 80).unwrap();
        let q = crate::deposit_box_items::tests::quantities();
        data.variables.set(q.requested, 7, true);
        data.inventory
            .restore_slot(
                3,
                Some(Item {
                    id: 995,
                    count: 20,
                    vars: 0,
                }),
            )
            .unwrap();
        w.banking = Some(bank);
        w.object_types.stackable.insert(995);
        w.deposit_box = Some(crate::deposit_box::Config {
            animation: 834,
            quantities: Some(q),
        });
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.set_modal_open(actor, true).unwrap();
        w.drain_events();
        (w, actor)
    }
    #[test]
    fn deposit_box_worn_item_action_unequips_to_bag_and_rechecks_identity_capacity_and_delay() {
        for case in 0..4 {
            let mut w = World::default();
            let mut data = PlayerData::new(1, "Box worn".into(), CoordGrid::ZERO, Stats::default());
            data.equipment
                .restore_slot(
                    3,
                    Some(Item {
                        id: 2,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            if case == 1 {
                for slot in 0..28 {
                    data.inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 99,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            w.tick().unwrap();
            let actor = w.actors().next().unwrap();
            w.set_modal_open(actor, true).unwrap();
            w.drain_events();
            if case == 3 {
                w.delay_player(actor, 2).unwrap();
            }
            let before = w.data(actor).unwrap().clone();
            w.submit(
                actor,
                Action::UnequipItemSlot {
                    slot: 3,
                    item: if case == 2 { 99 } else { 2 },
                },
            )
            .unwrap();
            w.tick().unwrap();
            let events = w.drain_events();
            let after = w.data(actor).unwrap();
            assert_eq!(after.bank, before.bank);
            assert_eq!(after.stats, before.stats);
            if case == 0 {
                assert!(after.equipment.get(3).is_none());
                assert_eq!(after.inventory.get(0).unwrap().id, 2);
                assert!(events.iter().any(|e| matches!(
                    e,
                    WorldEvent::EquipmentResult {
                        unequipping: true,
                        result: Ok(_),
                        ..
                    }
                )));
            } else {
                assert_eq!(after.inventory, before.inventory);
                assert_eq!(after.equipment.slots(), before.equipment.slots());
                if case == 3 {
                    assert!(after.equipment.modified().contains(&0));
                }
            }
            assert!(!events.iter().any(|e| matches!(e, WorldEvent::Bank { .. })));
        }
    }
    #[test]
    fn deposit_box_count_input_presets_custom_signed_boundaries_and_one_use() {
        use crate::deposit_box_items::CountPurpose;
        for (input, accepted, mode, requested) in [
            (1, true, 0, 7),
            (5, true, 1, 7),
            (10, true, 4, 7),
            (8, true, 3, 8),
            (-5, true, 1, 7),
            (0, false, 0, 7),
            (i32::MIN, false, 0, 7),
            (i32::MAX, true, 3, i32::MAX),
        ] {
            let (mut w, actor) = count_world();
            let token = w.start_modal_count_dialog(actor).unwrap();
            assert_eq!(
                w.complete_deposit_box_count(actor, token, CountPurpose::Mode, input)
                    .unwrap(),
                accepted
            );
            let q = w.deposit_box.unwrap().quantities.unwrap();
            let data = w.data(actor).unwrap();
            assert_eq!(q.mode.get(&data.variables).unwrap(), mode);
            assert_eq!(data.variables.get(q.requested.id), requested);
            assert_eq!(data.inventory.get(3).unwrap().count, 20);
            assert!(data.bank.get(0).is_none());
            assert_eq!(w.quantity_dialog_token(actor).unwrap(), None);
            assert!(!w
                .complete_deposit_box_count(actor, token, CountPurpose::Mode, 99)
                .unwrap());
        }
    }
    #[test]
    fn deposit_box_count_item_has_no_preset_write_and_canceled_or_delayed_inputs_cannot_transfer() {
        use crate::deposit_box_items::CountPurpose;
        let (mut w, actor) = count_world();
        let purpose = CountPurpose::Item {
            slot: 3,
            expected: 995,
        };
        let token = w.start_modal_count_dialog(actor).unwrap();
        assert!(w
            .complete_deposit_box_count(actor, token, purpose, 5)
            .unwrap());
        assert_eq!(w.data(actor).unwrap().inventory.get(3).unwrap().count, 15);
        assert_eq!(w.data(actor).unwrap().bank.get(0).unwrap().count, 5);
        assert_eq!(w.data(actor).unwrap().variables.get(1794), 7);
        assert!(!w
            .complete_deposit_box_count(actor, token, purpose, 5)
            .unwrap());
        let token = w.start_modal_count_dialog(actor).unwrap();
        w.cancel_count_dialog(actor).unwrap();
        assert!(!w
            .complete_deposit_box_count(actor, token, purpose, 5)
            .unwrap());
        let token = w.start_modal_count_dialog(actor).unwrap();
        w.delay_player(actor, 2).unwrap();
        assert_eq!(
            w.complete_deposit_box_count(actor, token, purpose, 5),
            Err(ActionError::Delayed)
        );
        assert_eq!(w.quantity_dialog_token(actor).unwrap(), None);
        assert_eq!(w.data(actor).unwrap().inventory.get(3).unwrap().count, 15);
        assert_eq!(w.data(actor).unwrap().bank.get(0).unwrap().count, 5);
    }
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
