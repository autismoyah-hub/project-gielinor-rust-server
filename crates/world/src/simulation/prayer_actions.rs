use super::*;
use crate::prayer::{Event, State};
impl World {
    pub fn cancel_prayer_dialog(&mut self, actor: ActorId) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let cancelled = matches!(p.prayer, Some(State::Confirm { .. }));
        if cancelled {
            p.prayer = None;
            p.protection.suspended = false;
        }
        Ok(cancelled)
    }
    pub(super) fn start_bury(&mut self, actor: ActorId, slot: usize, expected: Option<i32>) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        p.begin_held_action();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let bone = p
            .data
            .inventory
            .get(slot)
            .ok_or(ActionError::InvalidItem)?
            .id;
        let config = self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        if expected.is_some_and(|item|item!=bone) || !config.bones.contains_key(&bone) {
            return Err(ActionError::InvalidItem);
        }
        let selected=expected.map(|_|slot);
        if let Some(slot)=selected {
            if self.prayer_member_bones.contains(&bone) && !p.member {return Err(ActionError::InvalidItem);}
            config.prepare_selected(&p.data,&self.object_types,bone,slot).map_err(|_|ActionError::InvalidItem)?;
        }
        if config.requires_confirmation(p.movement.coords) && !(selected.is_some() && self.prayer_ashes.contains_key(&bone)) {
            p.dialog_generation = p
                .dialog_generation
                .checked_add(1)
                .ok_or(ActionError::QueueFull)?;
            let token = p.dialog_generation;
            p.prayer = Some(State::Confirm { token, bone, slot:selected });
            p.protection.suspended = true;
            self.events.push(WorldEvent::Prayer {
                actor,
                event: Event::Confirm { token },
            });
            return Ok(());
        }
        self.animate_bury(actor, bone, selected)
    }
    fn animate_bury(&mut self, actor: ActorId, bone: i32, slot: Option<usize>) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        let config = self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        let mut pending=if let Some(slot)=slot {
            if self.prayer_member_bones.contains(&bone) && !p.member {return Err(ActionError::InvalidItem);}
            config.prepare_selected(&p.data,&self.object_types,bone,slot)
        } else {config.prepare(&p.data,bone)}.map_err(|_|ActionError::InvalidItem)?;
        let ash_animation=slot.and_then(|_|self.prayer_ashes.get(&bone)).copied();
        if ash_animation.is_some(){pending.restore=0;}
        p.prayer = Some(State::Bury{pending,slot});
        p.protection.suspended = true;
        p.protection.delay(1);
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Animation(ash_animation.unwrap_or(config.animation)),
        });
        Ok(())
    }
    pub(super) fn resume_bury(
        &mut self,
        actor: ActorId,
        token: u64,
        choice: i32,
    ) -> Result<(), ActionError> {
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        let Some(State::Confirm {
            token: current,
            bone,
            slot,
        }) = p.prayer
        else {
            return Ok(());
        };
        if token != current || !(1..=2).contains(&choice) {
            return Ok(());
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        p.prayer = None;
        p.protection.suspended = false;
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Closed,
        });
        if choice == 1 {
            self.animate_bury(actor, bone, slot)?;
        }
        Ok(())
    }
    /// Start Prayer.offerOnAltar after the caller has verified location reach
    /// and its content gates. The existing player delay/cancellation machinery
    /// owns each three-cycle suspension. This is not a network action itself.
    pub fn begin_altar_offering(
        &mut self,
        actor: ActorId,
        bone: i32,
        animation: u16,
    ) -> Result<(), ActionError> {
        let xp = self
            .prayer
            .as_ref()
            .and_then(|c| c.bones.get(&bone))
            .copied()
            .ok_or(ActionError::InvalidItem)?;
        self.begin_offering(actor, bone, animation, Some(xp), None)
    }
    pub(super) fn begin_offering(
        &mut self,
        actor: ActorId,
        bone: i32,
        animation: u16,
        xp: Option<f64>,
        continuation: Option<crate::tower_offering::Continuation>,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        if self.prayer_ashes.contains_key(&bone){return Err(ActionError::InvalidItem);}
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        if !crate::prayer::contains(&p.data.inventory, bone) {
            self.events.push(WorldEvent::Prayer {
                actor,
                event: Event::OfferingFinished { offered: false },
            });
            return Ok(());
        }
        p.begin_held_action();
        p.prayer = Some(State::Offer {
            bone,
            animation,
            offered: false,
            xp,
            continuation,
        });
        p.protection.suspended = true;
        p.protection.current_clock = self.clock.0;
        p.protection.delay(3);
        self.events.push(WorldEvent::Prayer {
            actor,
            event: Event::Animation(animation),
        });
        Ok(())
    }
    pub(super) fn process_prayer(&mut self, slot: usize, random: &mut impl RandomSource) {
        if self
            .players
            .get(slot)
            .is_some_and(|p| matches!(p.prayer, Some(State::CrateWalk { .. })))
        {
            self.process_tower_crate_walk(slot);
            return;
        }
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        if p.saving {
            return;
        }
        p.protection.processed_clock = self.clock.0;
        if p.protection.delayed() {
            return;
        }
        let (pending, selected, offering) = match p.prayer {
            Some(State::Bury{pending,slot}) => (pending, slot, None),
            Some(State::Offer {
                bone,
                animation,
                offered,
                xp,
                continuation,
            }) => (
                crate::prayer::Pending { bone, restore: 0 },
                None,
                Some((animation, offered, xp, continuation)),
            ),
            _ => return,
        };
        p.prayer = None;
        p.protection.suspended = false;
        let Some(config) = &self.prayer else { return };
        let mut tower_completion = None;
        let completed = if let Some((_, _, xp, _)) = offering {
            config.finish_offering_xp(&mut p.data, &self.object_types, pending.bone, xp)
        } else if let Some(slot)=selected {
            if p.pending_logout || p.has_modal || (self.prayer_member_bones.contains(&pending.bone) && !p.member) {Ok(None)}
            else {crate::prayer_preservation::finish_selected(config,&mut p.data,&self.object_types,pending,slot,self.prayer_preservation.as_ref(),p.member,random)}
        } else {
            config.finish(&mut p.data, &self.object_types, pending)
        };
        match completed {
            Ok(Some(result)) => {
                if self.tutorial_rules_completed_varp.is_some() {
                    if p.stat_events
                        .advance(
                            &crate::tutorial_rules::bindings(),
                            config.skill,
                            result.advancement,
                        )
                        .is_err()
                    {
                        p.pending_logout = true;
                        self.events.push(WorldEvent::ActionRejected {
                            actor: p.id,
                            error: ActionError::QueueFull,
                        });
                    }
                }
                self.events.push(WorldEvent::Prayer {
                    actor: p.id,
                    event: Event::Completed {
                        skill: config.skill,
                        result,
                    },
                });
                self.events.push(WorldEvent::Prayer {
                    actor: p.id,
                    event: Event::Message(if offering.is_some() {
                        "The gods are pleased with your offering."
                    } else if selected.is_some() && self.prayer_ashes.contains_key(&pending.bone) {
                        "You scatter the ashes."
                    } else {
                        "You bury the bones."
                    }),
                });
                if let Some((animation, _, xp, continuation)) = offering {
                    if crate::prayer::contains(&p.data.inventory, pending.bone) {
                        p.prayer = Some(State::Offer {
                            bone: pending.bone,
                            animation,
                            offered: true,
                            xp,
                            continuation,
                        });
                        p.protection.suspended = true;
                        p.protection.delay(3);
                        self.events.push(WorldEvent::Prayer {
                            actor: p.id,
                            event: Event::Animation(animation),
                        });
                    } else {
                        tower_completion = continuation.map(|c| (p.id, c, true));
                        self.events.push(WorldEvent::Prayer {
                            actor: p.id,
                            event: Event::OfferingFinished { offered: true },
                        });
                    }
                }
            }
            Ok(None) => {
                if let Some((_, offered, _, continuation)) = offering {
                    tower_completion = continuation.map(|c| (p.id, c, offered));
                    self.events.push(WorldEvent::Prayer {
                        actor: p.id,
                        event: Event::OfferingFinished { offered },
                    });
                }
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::InvalidStat,
            }),
        }
        if let Some((actor, continuation, offered)) = tower_completion {
            self.finish_tower_offering(actor, continuation, offered);
        }
    }
}

#[cfg(test)]
mod zealot_tests {
    use super::*;
    use crate::{inventory::Item, stats::Stats, CoordGrid};
    struct Fixed {
        draw: u32,
        calls: u32,
    }
    impl RandomSource for Fixed {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 80);
            self.calls += 1;
            self.draw
        }
    }
    #[test]
    fn zealot_actual_selected_burial_dispatch_rechecks_live_gear_and_excludes_ashes() {
        for mode in 0..9 {
            let mut w = World::default();
            w.prayer = Some(crate::prayer::Config {
                skill: 5,
                animation: 827,
                bones: [(526, 4.5), (25766, 10.0)].into(),
                necklace: None,
                restore: Default::default(),
                worn_xp_percent: Default::default(),
                chaos: None,
            });
            let rule = crate::prayer_preservation::Rule {
                pieces: [(25438, 0), (25434, 4), (25436, 7), (25440, 10)],
                excluded: [25766, 25769, 25772, 25775, 25778].into(),
            };
            w.prayer_preservation = Some(rule.clone());
            w.prayer_ashes.insert(25766, 2295);
            let input = if mode == 6 { 25766 } else { 526 };
            let mut p = PlayerData::new(
                1,
                "Zealot".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            );
            p.xp_rate_hundreds = 500;
            for slot in 0..28 {
                p.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: input,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            if mode != 3 {
                for &(id, slot) in &rule.pieces {
                    p.equipment
                        .restore_slot(
                            slot,
                            Some(Item {
                                id,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
            }
            w.queue_login(1, p).unwrap();
            let mut rng = Fixed {
                draw: if mode == 1 { 0 } else { 79 },
                calls: 0,
            };
            w.tick_with_random(&mut rng).unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, true).unwrap();
            w.drain_events();
            w.submit(
                a,
                Action::BuryItemSlot {
                    slot: 27,
                    item: input,
                },
            )
            .unwrap();
            w.tick_with_random(&mut rng).unwrap();
            assert_eq!(rng.calls, 0);
            if mode == 2 {
                for &(_, slot) in &rule.pieces {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(slot, None)
                        .unwrap();
                }
            }
            if mode == 3 {
                for &(id, slot) in &rule.pieces {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            slot,
                            Some(Item {
                                id,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
            }
            if mode == 4 {
                w.set_membership(a, false).unwrap();
            }
            if mode == 5 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(27, None)
                    .unwrap();
            }
            if mode == 7 {
                for &(id, _) in &rule.pieces {
                    w.object_types.dummy.insert(id);
                }
            }
            if mode == 8 {
                w.set_modal_open(a, true).unwrap();
            }
            let gear = w.data(a).unwrap().equipment.clone();
            let inv = w.data(a).unwrap().inventory.clone();
            w.tick_with_random(&mut rng).unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(d.equipment, gear);
            assert_eq!(d.inventory.get(0), inv.get(0));
            if matches!(mode, 5 | 8) {
                assert_eq!(d.inventory, inv);
                assert_eq!(d.stats.fine_xp(5), 0);
                assert_eq!(rng.calls, 0);
                continue;
            }
            let saved = matches!(mode, 0 | 3);
            assert_eq!(d.inventory.get(27).is_some(), saved, "mode {mode}");
            assert_eq!(d.stats.fine_xp(5), if mode == 6 { 500 } else { 225 });
            assert_eq!(rng.calls, u32::from(matches!(mode, 0 | 1 | 3)));
            assert!(w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::Prayer {
                    event: crate::prayer::Event::Completed { .. },
                    ..
                }
            )));
        }
    }
}
