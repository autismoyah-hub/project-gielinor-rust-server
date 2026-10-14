use super::*;
impl World {
    pub(super) fn process_tower_ash(&mut self, slot: usize) {
        self.try_tower_login_restore(slot);
        let Some(config) = self.tower_restore.as_ref() else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        let inside = crate::tower_layout::contains(config.origin, p.movement.coords);
        if inside {
            p.tower_tastes
                .sample_minds(&p.data.inventory, &config.mind_runes);
        }
        let sync = p.data.tower_diary.and_then(|diary| {
            p.tower_tastes.grant_mind_spend(
                diary,
                config.skills.map(|s| p.data.stats.base(s) as i32),
                p.data.tower_arrival.staff_taken,
                p.protection.access_protected(p.has_modal),
            )
        });
        let actor = p.id;
        let bones = config.bones;
        let agility = config.agility;
        if let Some(sync) = sync {
            self.publish_tower_diary_sync(actor, sync, bones);
        }
        let p = self.players.get_mut(slot).unwrap();
        if let Some(xp) = inside
            .then(|| p.tower_tastes.movement_xp(p.movement.coords))
            .flatten()
        {
            match crate::skills::advance(
                &mut p.data.stats,
                agility,
                xp,
                p.data.xp_rate_hundreds as f64 / 100.0,
                99,
            ) {
                Ok(advancement) => {
                    if self.tutorial_rules_completed_varp.is_some()
                        && p.stat_events
                            .advance(&crate::tutorial_rules::bindings(), agility, advancement)
                            .is_err()
                    {
                        p.pending_logout = true;
                        self.events.push(WorldEvent::ActionRejected {
                            actor,
                            error: ActionError::QueueFull,
                        });
                    }
                    self.events.push(WorldEvent::SkillAdvanced {
                        actor,
                        skill: agility,
                        advancement,
                    });
                }
                Err(_) => self.events.push(WorldEvent::ActionRejected {
                    actor,
                    error: ActionError::InvalidStat,
                }),
            }
        }
    }

    pub(super) fn try_tower_login_restore(&mut self, slot: usize) {
        let Some(config) = self.tower_restore.as_ref() else {
            return;
        };
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        if !p.tower_restore_pending || p.saving || p.protection.access_protected(p.has_modal) {
            return;
        }
        // towerBusy is not active until Tower command coroutine content is added.
        // Every current suspended action is covered by protected access above.
        let actor = p.id;
        let Some(diary) = p.data.tower_diary else {
            p.tower_restore_pending = false;
            self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::UnknownTowerDiary,
            });
            return;
        };
        p.tower_restore_pending = false;
        let out = config.kit.bank_extras(
            &mut p.data.inventory,
            &p.data.equipment,
            &mut p.data.bank,
            &self.object_types,
            diary,
        );
        if let Some(text) = out.message {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: text.into(),
            });
        }
        let reconcile = crate::tower_kit::reconcile_tans(
            diary,
            &p.data.inventory,
            &p.data.bank,
            &config.leather,
            config.skills.map(|s| p.data.stats.base(s) as i32),
            p.data.tower_arrival.staff_taken,
        );
        let bones = config.bones;
        if let Some(sync) = reconcile.sync {
            self.publish_tower_diary_sync(actor, sync, bones);
        }
        if let Some(text) = reconcile.message {
            self.events.push(WorldEvent::GameMessage { actor, text });
        }
        let config = self.tower_restore.as_ref().unwrap();
        let p = self.players.get_mut(slot).unwrap();
        let restored = config
            .staff
            .prepare_restore(&mut p.data, &self.object_types);
        for text in restored.messages {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: text.into(),
            });
        }
        if let Some(inventory_slot) = restored.equip_slot {
            // ProtectedAccess.invEquip directly invokes equipment handling;
            // it does not dispatch a held action or clear the current route.
            let p = self.players.get_mut(slot).unwrap();
            let item = p.data.inventory.get(inventory_slot).unwrap().id;
            let result = crate::equipment::equip(
                &mut p.data.inventory,
                &mut p.data.equipment,
                inventory_slot,
                &self.object_types,
                &self.wear_types,
                &p.data.stats,
            );
            if let Ok(events) = &result {
                for event in events {
                    if let crate::equipment::Event::Changed { slot, id } = event {
                        self.worn_changed(actor, *slot, *id);
                    }
                }
            }
            self.events.push(WorldEvent::EquipmentResult {
                actor,
                item,
                unequipping: false,
                result,
            });
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "You wield the staff of air.".into(),
            });
        }
        let config = self.tower_restore.as_ref().unwrap();
        let p = self.players.get_mut(slot).unwrap();
        let diary = p.data.tower_diary.unwrap();
        let mut messages = config
            .rewards
            .before_food(&mut p.data, &self.object_types, diary);
        if let Some(text) = config.food.restore(&mut p.data, &self.object_types) {
            messages.push(text);
        }
        if let Some(text) = config
            .staff
            .restore_talisman(&mut p.data, &self.object_types)
        {
            messages.push(text);
        }
        messages.extend(
            config
                .rewards
                .after_talisman(&mut p.data, &self.object_types, diary),
        );
        for text in messages {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: text.into(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Insert, Item},
        tower_diary::Progress,
        tower_restore,
    };
    fn setup(known: bool, inside: bool) -> (World, ActorId) {
        let mut w = World::default();
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let coords = if inside {
            origin.translate(56, 56, 0).unwrap()
        } else {
            CoordGrid::new(3200, 3200, 0).unwrap()
        };
        w.collision.allocate(coords.x(), coords.z(), 0);
        fn reward(id: i32) -> tower_restore::Item {
            tower_restore::Item {
                id,
                count: 1,
                owned: [id].into(),
                waiting: "waiting",
                restored: "restored",
            }
        }
        let mut kit = crate::tower_kit::Config::default();
        kit.named[1].insert(10);
        let staff = crate::tower_staff::Config {
            staff: 1381,
            talisman: 1448,
            staff_names: [1381].into(),
            staff_owned: [1381].into(),
            talisman_owned: [1448].into(),
        };
        w.tower_restore = Some(tower_restore::HandlerConfig {
            timer: 9,
            agility: 16,
            mind_runes: [558].into(),
            origin,
            skills: [12, 13, 11],
            bones: 526,
            kit,
            leather: [1741].into(),
            staff,
            rewards: tower_restore::Config {
                axe: reward(10),
                needle: reward(11),
                thread: reward(12),
                pickaxe: reward(13),
                tinderbox: reward(14),
                scimitar: reward(15),
                net: reward(16),
                knife: reward(17),
                hammer: reward(18),
            },
            food: Default::default(),
        });
        w.wear_types.types.insert(
            1381,
            crate::equipment::WearType {
                primary: Some(3),
                secondary: [None; 2],
                requirements: vec![],
            },
        );
        let mut data = PlayerData::new(1, "Restore".into(), coords, Stats::default());
        if known {
            data.tower_diary = Some(Progress {
                schema11: true,
                ..Default::default()
            });
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a)
    }
    fn through(w: &mut World, clock: i32) {
        while w.clock.0 <= clock {
            w.tick().unwrap();
        }
    }
    #[test]
    fn restore_waits_for_first_original_soft_timer_and_runs_once() {
        let (mut w, a) = setup(true, true);
        w.data_mut(a).unwrap().tower_diary.as_mut().unwrap().done[0] = true;
        through(&mut w, 6);
        assert!(w.data(a).unwrap().inventory.get(0).is_none());
        through(&mut w, 7);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 10);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(0, None)
            .unwrap();
        through(&mut w, 31);
        assert!(w.data(a).unwrap().inventory.get(0).is_none());
    }
    #[test]
    fn protected_player_retries_next_ash_tick_without_losing_pending_restore() {
        let (mut w, a) = setup(true, true);
        w.data_mut(a).unwrap().tower_diary.as_mut().unwrap().done[0] = true;
        w.set_modal_open(a, true).unwrap();
        through(&mut w, 7);
        assert!(w.data(a).unwrap().inventory.get(0).is_none());
        assert!(
            w.players
                .get(a.slot as usize)
                .unwrap()
                .tower_restore_pending
        );
        w.set_modal_open(a, false).unwrap();
        through(&mut w, 14);
        assert!(w.data(a).unwrap().inventory.get(0).is_none());
        through(&mut w, 15);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 10);
    }
    #[test]
    fn unknown_marks_fail_before_banking_and_outside_login_never_arms_restore() {
        for (known, inside) in [(false, true), (true, false)] {
            let (mut w, a) = setup(known, inside);
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: 10,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            through(&mut w, 23);
            assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 10);
            let rejects = w
                .drain_events()
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        WorldEvent::ActionRejected {
                            error: ActionError::UnknownTowerDiary,
                            ..
                        }
                    )
                })
                .count();
            assert_eq!(rejects, usize::from(!known));
        }
    }
    #[test]
    fn bank_then_reconcile_then_staff_equip_preserves_order_and_actual_containers() {
        let (mut w, a) = setup(true, true);
        let p = w.data_mut(a).unwrap();
        p.tower_arrival.staff_taken = true;
        p.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 10,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        p.inventory
            .insert(&Default::default(), Insert::new(1741, 5))
            .unwrap();
        through(&mut w, 7);
        let p = w.data(a).unwrap();
        assert_eq!(p.bank.get(0).unwrap().id, 10);
        assert_eq!(p.tower_diary.unwrap().counts[0], 5);
        assert_eq!(p.tower_diary.unwrap().bounty, 1);
        assert_eq!(p.equipment.get(3).unwrap().id, 1381);
        assert!(p.inventory.slots().iter().flatten().any(|i| i.id == 1448));
        let text: Vec<_> = w
            .drain_events()
            .into_iter()
            .filter_map(|e| match e {
                WorldEvent::GameMessage { text, .. } => Some(text),
                WorldEvent::TowerDiary { .. } => Some("DIARY".into()),
                WorldEvent::EquipmentResult { result, .. } => {
                    assert!(result.is_ok());
                    Some("EQUIP".into())
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            text,
            [
                "Diary tools wait in the bank. Tan 5 hides for the axe.",
                "DIARY",
                "The chest holds a bronze axe.",
                "Hides tanned 5/5.",
                "You still have the staff of air.",
                "You still have the mind talisman.",
                "EQUIP",
                "You wield the staff of air."
            ]
        );
    }
    #[test]
    fn direct_restore_equip_preserves_interaction_route() {
        let (mut w, a) = setup(true, true);
        let p = w.players.get_mut(a.slot as usize).unwrap();
        p.data.tower_arrival.staff_taken = true;
        let destination = p.movement.coords.translate(3, 0, 0).unwrap();
        let loc = crate::locations::Location::new(destination, 77, 10, 0).unwrap();
        p.location_interaction = Some((loc, LocationAction::Operation(1)));
        p.movement.interacting = true;
        p.movement.route([destination], Speed::Walk);
        w.try_tower_login_restore(a.slot as usize);
        let p = w.players.get(a.slot as usize).unwrap();
        assert_eq!(p.data.equipment.get(3).unwrap().id, 1381);
        assert!(p.movement.interacting);
        assert_eq!(p.movement.destination(), Some(destination));
        assert!(p.location_interaction.is_some());
    }
    #[test]
    fn ash_timer_awards_sampled_movement_xp_with_account_rate_and_no_idle_repeat() {
        let (mut w, a) = setup(true, true);
        w.data_mut(a).unwrap().xp_rate_hundreds = 500;
        let start = w.data(a).unwrap().coords;
        for x in start.x()..=start.x() + 4 {
            for z in start.z() - 1..=start.z() + 1 {
                w.collision.set(x, z, 0, 0);
            }
        }
        through(&mut w, 7);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(16), 0);
        let dest = start.translate(3, 0, 0).unwrap();
        w.submit(
            a,
            Action::Walk {
                destination: dest,
                run: false,
            },
        )
        .unwrap();
        through(&mut w, 14);
        assert_eq!(w.data(a).unwrap().coords, dest);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(16), 0);
        through(&mut w, 15);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(16), 37);
        through(&mut w, 23);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(16), 37);
    }
    #[test]
    fn ash_timer_samples_rune_spend_while_protected_and_credits_on_later_free_tick() {
        let (mut w, a) = setup(true, true);
        w.object_types.stackable.insert(558);
        let p = w.data_mut(a).unwrap();
        let diary = p.tower_diary.as_mut().unwrap();
        diary.done[..6].fill(true);
        diary.counts[4] = 5;
        p.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 558,
                    count: 2,
                    vars: 0,
                }),
            )
            .unwrap();
        through(&mut w, 7);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(0, None)
            .unwrap();
        w.set_modal_open(a, true).unwrap();
        through(&mut w, 15);
        assert!(!w.data(a).unwrap().tower_diary.unwrap().actions[2]);
        assert!(
            w.players
                .get(a.slot as usize)
                .unwrap()
                .tower_tastes
                .mind_pending
        );
        w.set_modal_open(a, false).unwrap();
        through(&mut w, 23);
        let progress = w.data(a).unwrap().tower_diary.unwrap();
        assert!(progress.actions[2]);
        assert_eq!(progress.bounty, 7);
        assert!(
            !w.players
                .get(a.slot as usize)
                .unwrap()
                .tower_tastes
                .mind_pending
        );
    }
}
