use super::*;
impl World {
    pub(super) fn boost_worn_item(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        let rule = *self
            .worn_stat_boosts
            .get(&item)
            .ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.pending_logout
            || p.saving
            || p.death_queued
            || p.data.stats.current(3) as u8 == 0
            || p.protection.access_protected(p.has_modal)
        {
            return Err(ActionError::Delayed);
        }
        let prepared = match rule.prepare(
            item,
            slot,
            &p.data.equipment,
            &self.object_types,
            &p.data.stats,
            p.member,
        ) {
            Ok(v) => v,
            Err(crate::worn_stat_boost::Error::Level) => {
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: "You need level 99 in that skill to use this cape's Boost option.",
                });
                return Ok(());
            }
            Err(crate::worn_stat_boost::Error::Members) => {
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: "You need membership to use this cape's Boost option.",
                });
                return Ok(());
            }
            Err(_) => return Err(ActionError::InvalidItem),
        };
        // Shared stat trigger admission succeeds before publishing the staged stat image.
        p.record_stat_update(
            self.tutorial_rules_completed_varp.is_some(),
            prepared.update,
        )?;
        p.begin_held_action();
        p.queues.clear_weak();
        p.data.stats = prepared.stats;
        self.events.push(WorldEvent::Stat {
            actor,
            update: prepared.update,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, worn_stat_boost::Rule};
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Cape Boost must not draw RNG");
        }
    }
    fn fixture(item: i32) -> (World, ActorId) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        for id in [9780, 9781] {
            w.worn_stat_boosts.insert(
                id,
                Rule {
                    skill: 12,
                    slot: 1,
                    button_operation: 2,
                    members: true,
                },
            );
        }
        let mut stats = Stats::default();
        stats.set_base(12, 99);
        stats.set_current(12, 99);
        stats
            .set_xp(12, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        let mut data = PlayerData::new(
            1,
            "Master crafter".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.equipment
            .restore_slot(
                1,
                Some(Item {
                    id: item,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        w.drain_events();
        (w, a)
    }
    #[test]
    fn worn_crafting_cape_actual_action_updates_stat_once_without_xp_or_item_change() {
        for item in [9780, 9781] {
            let (mut w, a) = fixture(item);
            let before = w.data(a).unwrap().clone();
            for (iteration, notify) in [(0, true), (1, false)] {
                w.submit(a, Action::BoostWornItem { slot: 1, item })
                    .unwrap();
                w.tick_with_random(&mut NoDraw).unwrap();
                let events = w.drain_events();
                let d = w.data(a).unwrap();
                assert_eq!(d.stats.current(12), 100);
                assert_eq!(d.stats.base(12), 99);
                assert_eq!(d.stats.fine_xp(12), before.stats.fine_xp(12));
                assert_eq!(d.inventory, before.inventory);
                assert_eq!(d.equipment, before.equipment);
                assert!(events.iter().any(|e|matches!(e,WorldEvent::Stat{update,..}if update.skill==12&&update.notify==notify)),"iteration{iteration}");
                assert!(!events
                    .iter()
                    .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
            }
        }
    }
    #[test]
    fn worn_cape_action_rechecks_late_member_base_slot_vars_identity_and_modal_without_xp() {
        for case in 0..7 {
            let (mut w, a) = fixture(9780);
            let slot = if case == 6 { 4 } else { 1 };
            w.submit(a, Action::BoostWornItem { slot, item: 9780 })
                .unwrap();
            match case {
                0 => w.set_membership(a, false).unwrap(),
                1 => w.data_mut(a).unwrap().stats.set_base(12, 98),
                2 => {
                    let d = w.data_mut(a).unwrap();
                    let mut i = d.equipment.get(1).unwrap();
                    i.vars = 9;
                    d.equipment.restore_slot(1, Some(i)).unwrap();
                }
                3 => {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(1, None)
                        .unwrap();
                }
                4 => {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 9781,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                5 => w.players.get_mut(a.slot as usize).unwrap().has_modal = true,
                6 => {}
                _ => unreachable!(),
            }
            let before = w.data(a).unwrap().clone();
            w.tick_with_random(&mut NoDraw).unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(d.stats, before.stats, "case{case}");
            assert_eq!(d.inventory, before.inventory);
            assert_eq!(d.equipment, before.equipment);
        }
    }
}
