use super::*;
impl World {
    pub(super) fn empty_container(
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
        if p.has_modal || p.data.stats.current(3) <= 0 {
            return Err(ActionError::Protected);
        }
        let next = rule
            .prepare(item, slot, &p.data.inventory, &self.object_types, p.member)
            .map_err(|_| ActionError::InvalidItem)?;
        p.data.inventory = next;
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
