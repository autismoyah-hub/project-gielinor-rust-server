use super::*;
impl World {
    pub fn held_conversion(&self, item: i32) -> Option<&crate::held_item_conversion::Rule> {
        self.empty_containers.get(&item)
    }
    pub(super) fn empty_container(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
    ) -> Result<(), ActionError> {
        self.convert_held_item(actor, slot, item)
    }
    pub(super) fn convert_held_item(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
    ) -> Result<(), ActionError> {
        let rule = *self
            .empty_containers
            .get(&item)
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
        let next = rule
            .prepare(item, slot, &p.data.inventory, &self.object_types, p.member)
            .map_err(|_| ActionError::InvalidItem)?;
        p.data.inventory = next;
        if let Some(text) = rule.message {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: text.into(),
            });
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{container_emptying::Rule, inventory::Item, stats::Stats, CoordGrid};
    fn setup() -> (World, ActorId) {
        let mut w = World::default();
        w.empty_containers.insert(
            1783,
            Rule {
                output: 1925,
                members: true,
                button_operation: 6,
                message: None,
            },
        );
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(3200, 3200, 0);
        let mut data = PlayerData::new(1, "Sand handler".into(), coords, Stats::default());
        for slot in 0..28 {
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
        for slot in [4, 19] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 1783,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.set_membership(actor, true).unwrap();
        w.drain_events();
        (w, actor)
    }
    #[test]
    fn container_emptying_full_bag_selected_exchange_no_xp_or_matching_slot_fallback() {
        let (mut w, a) = setup();
        let stats = w.data(a).unwrap().stats.clone();
        w.submit(
            a,
            Action::EmptyContainer {
                slot: 19,
                item: 1783,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory.get(19).unwrap().id, 1925);
        assert_eq!(w.data(a).unwrap().inventory.get(4).unwrap().id, 1783);
        assert_eq!(w.data(a).unwrap().stats, stats);
        assert!(!w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
        let before = w.data(a).unwrap().inventory.clone();
        w.submit(
            a,
            Action::EmptyContainer {
                slot: 19,
                item: 1783,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().inventory, before);
        for mode in 0..5 {
            let (mut w, a) = setup();
            match mode {
                0 => w.set_membership(a, false).unwrap(),
                1 => {
                    w.object_types.dummy.insert(1925);
                }
                2 => {
                    let p = w.players.get_mut(a.slot as usize).unwrap();
                    p.data
                        .inventory
                        .restore_slot(
                            19,
                            Some(Item {
                                id: 1783,
                                count: 1,
                                vars: 9,
                            }),
                        )
                        .unwrap();
                }
                3 => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .protection
                        .delay(3);
                }
                _ => {
                    w.players.get_mut(a.slot as usize).unwrap().has_modal = true;
                }
            }
            let before = w.data(a).unwrap().clone();
            w.submit(
                a,
                Action::EmptyContainer {
                    slot: 19,
                    item: 1783,
                },
            )
            .unwrap();
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
}

#[cfg(test)]
mod conversion_life_tests {
    use super::*;
    use crate::{held_item_conversion::Rule, inventory::Item};
    #[test]
    fn held_conversion_unsigned_alive_hp_and_queued_death_are_distinct() {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.empty_containers.insert(
            1783,
            Rule {
                output: 1925,
                members: true,
                button_operation: 6,
                message: None,
            },
        );
        let mut data = PlayerData::new(
            1,
            "Conversion life fixture".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.inventory
            .restore_slot(
                19,
                Some(Item {
                    id: 1783,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        w.data_mut(a).unwrap().stats.set_current(3, 200u8 as i8);
        w.convert_held_item(a, 19, 1783).unwrap();
        assert_eq!(
            w.data(a)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .find(|i| i.id == 1925)
                .unwrap()
                .count,
            1
        );
        assert_eq!(w.data(a).unwrap().stats.current(3) as u8, 200);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(
                19,
                Some(Item {
                    id: 1783,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.players.get_mut(a.slot as usize).unwrap().death_queued = true;
        let before = w.data(a).unwrap().inventory.clone();
        assert_eq!(
            w.convert_held_item(a, 19, 1783),
            Err(ActionError::Protected)
        );
        assert_eq!(w.data(a).unwrap().inventory, before);
    }
}
