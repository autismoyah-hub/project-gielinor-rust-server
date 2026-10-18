use super::*;
impl World {
    pub(super) fn swap_worn_item(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
    ) -> Result<(), ActionError> {
        let rule = *self
            .worn_item_swaps
            .get(&item)
            .filter(|r| r.from_slot == slot)
            .ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if p.has_modal || p.death_queued || p.data.stats.current(3) as u8 == 0 {
            return Err(ActionError::Protected);
        }
        let plan = match rule.prepare(
            item,
            &p.data.inventory,
            &p.data.equipment,
            &self.object_types,
            &self.wear_types,
            &p.data.stats,
            p.member,
        ) {
            Ok(plan) => plan,
            Err(error) => {
                self.events.push(WorldEvent::EquipmentResult {
                    actor,
                    item,
                    unequipping: true,
                    result: Err(error),
                });
                return Ok(());
            }
        };
        // All inventory/equipment operations have succeeded on images. Publish the
        // intermediate removal to existing listeners before the final equip, keeping
        // their variable/combat refresh order without a fallible inventory step.
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.data.inventory = plan.removed_inventory;
        p.data.equipment = plan.removed_equipment;
        for event in &plan.unequip_events {
            if let crate::equipment::Event::Changed { slot, id } = event {
                self.worn_changed(actor, *slot, *id);
            }
        }
        self.events.push(WorldEvent::EquipmentResult {
            actor,
            item,
            unequipping: true,
            result: Ok(plan.unequip_events),
        });
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.data.inventory = plan.final_inventory;
        p.data.equipment = plan.final_equipment;
        if let Some(events) = plan.equip_events {
            for event in &events {
                if let crate::equipment::Event::Changed { slot, id } = event {
                    self.worn_changed(actor, *slot, *id);
                }
            }
            self.events.push(WorldEvent::EquipmentResult {
                actor,
                item: rule.output,
                unequipping: false,
                result: Ok(events),
            });
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bank_actions::Binding,
        equipment::WearType,
        inventory::Item,
        variables::{Transmit, Varbit, Varp},
        wearpos,
        worn_item_swap::Rule,
    };
    #[test]
    fn worn_item_swap_real_action_stages_space_member_target_and_listener_order() {
        for member in [false, true] {
            for (input, output, from, to) in [(25644, 29775, 3, 5), (29775, 25644, 5, 3)] {
                for case in 0..7 {
                    let mut w = World::default();
                    w.collision.allocate(3200, 3200, 0);
                    w.worn_item_swaps.insert(
                        input,
                        Rule {
                            output,
                            from_slot: from,
                            to_slot: to,
                            button_operation: 2,
                        },
                    );
                    for (id, slot) in [(25644, 3), (29775, 5)] {
                        w.wear_types.types.insert(
                            id,
                            WearType {
                                primary: Some(slot),
                                secondary: [None, None],
                                requirements: vec![],
                            },
                        );
                    }
                    if case == 4 {
                        w.object_types.dummy.insert(output);
                    }
                    let varp = |id| Varp {
                        id,
                        permanent: true,
                        transmit: Transmit::Always,
                        bit_protect: false,
                        client_code: -1,
                    };
                    let binding = |id| Binding {
                        base: varp(id),
                        bit: Varbit {
                            base: id,
                            lsb: 0,
                            msb: 7,
                        },
                    };
                    w.wearpos = Some(wearpos::Config {
                        special: varp(301),
                        stance: varp(43),
                        category: binding(843),
                        level: binding(8),
                        decimal: binding(9),
                        combat_stats: [0, 1, 2, 3, 4, 6, 5],
                        items: [
                            (
                                25644,
                                wearpos::Item {
                                    category: 17,
                                    ..Default::default()
                                },
                            ),
                            (29775, wearpos::Item::default()),
                            (
                                999,
                                wearpos::Item {
                                    category: 7,
                                    ..Default::default()
                                },
                            ),
                        ]
                        .into(),
                    });
                    let mut data = PlayerData::new(
                        1,
                        "Worn Swap".into(),
                        CoordGrid::new(3200, 3200, 0).unwrap(),
                        Stats::default(),
                    );
                    for slot in 0..if case == 2 { 28 } else { 27 } {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 1511,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    data.inventory
                        .restore_slot(
                            4,
                            Some(Item {
                                id: input,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    data.equipment
                        .restore_slot(
                            from,
                            Some(Item {
                                id: input,
                                count: 1,
                                vars: if case == 3 { 7 } else { 0 },
                            }),
                        )
                        .unwrap();
                    if case == 1 {
                        data.equipment
                            .restore_slot(
                                to,
                                Some(Item {
                                    id: 999,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    w.queue_login(1, data).unwrap();
                    w.tick().unwrap();
                    let a = w.actors().next().unwrap();
                    w.set_membership(a, member).unwrap();
                // Emit listener vars only after the original logged-in clock gate.
                w.tick().unwrap();
                    if case == 6 {
                        w.delay_player(a, 2).unwrap();
                    }
                    let before = w.data(a).unwrap().clone();
                    w.drain_events();
                    w.submit(
                        a,
                        Action::SwapWornItem {
                            slot: from,
                            item: if case == 5 { output } else { input },
                        },
                    )
                    .unwrap();
                    w.tick().unwrap();
                    let after = w.data(a).unwrap();
                    assert_eq!(after.stats, before.stats);
                    if case >= 2 {
                        assert_eq!(after.inventory, before.inventory);
                        assert_eq!(after.equipment, before.equipment);
                    } else {
                        assert!(after.equipment.get(from).is_none());
                        assert_eq!(after.inventory.get(4).unwrap().id, input);
                        if case == 0 && member {
                            assert_eq!(after.equipment.get(to).unwrap().id, output);
                            assert!(after.inventory.get(27).is_none());
                        } else {
                            assert_eq!(after.inventory.get(27).unwrap().id, output);
                            assert_eq!(after.equipment.get(to), before.equipment.get(to));
                        }
                    }
                    let events = w.drain_events();
                    let wear: Vec<_> = events
                        .iter()
                        .filter_map(|e| {
                            if let WorldEvent::WearposEffects { result: Ok(v), .. } = e {
                                Some(v)
                            } else {
                                None
                            }
                        })
                        .collect();
                    assert_eq!(
                        wear.len(),
                        if case >= 2 {
                            0
                        } else if case == 0 && member {
                            2
                        } else {
                            1
                        }
                    );
                    if input == 29775 && case == 0 && member {
                        let categories: Vec<_> = wear
                            .iter()
                            .flat_map(|v| v.iter())
                            .filter_map(|e| {
                                if let wearpos::Event::Variable(v) = e {
                                    (v.id == 843).then_some(v.value)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        assert_eq!(categories, vec![0, 17]);
                    }
                }
            }
        }
    }
}
