use super::*;
impl World {
    pub(super) fn process_ordinary_death(&mut self, random: &mut impl RandomSource) {
        let Some(config) = self.ordinary_death.clone() else {
            return;
        };
        let slots: Vec<_> = self
            .players
            .iter()
            .filter(|(_, p)| {
                !p.saving && (p.ordinary_death.is_some() || p.data.stats.current(3) as u8 == 0)
            // Tutorial protection has its own stat-change event and must run first.
            && !self.tutorial_rules_completed_varp.is_some_and(|id| p.data.variables.get(id) != 1)
            && p.cinderworks_recovery_at.is_none()
            && !self.cinderworks.as_ref().is_some_and(|c| p.data.tower_run.id > 0
                && crate::tower_layout::contains(c.origin, p.movement.coords))
            })
            .map(|(slot, _)| slot)
            .collect();
        for slot in slots {
            let p = self.players.get_mut(slot).unwrap();
            let actor = p.id;
            if p.ordinary_death.is_none() {
                // Select before stopping the action, as the reference does. This
                // is a uniform permutation, not a claim about Kotlin RNG seeds.
                let mut candidates = Vec::with_capacity(25);
                for dx in -2..=2 {
                    for dz in -2..=2 {
                        if let Ok(c) = config.centre.translate(dx, dz, 0) {
                            candidates.push(c);
                        }
                    }
                }
                for i in (1..candidates.len()).rev() {
                    candidates.swap(i, random.below((i + 1) as u32) as usize);
                }
                let bounds = |coords| crate::raycast::Bounds {
                    coords,
                    width: 1,
                    length: 1,
                };
                let destination = candidates
                    .into_iter()
                    .find(|&c| {
                        crate::raycast::clear(
                            &self.collision,
                            bounds(config.centre),
                            bounds(c),
                            false,
                            0x100000,
                        )
                    })
                    .unwrap_or(config.centre);
                p.ordinary_death = Some(crate::player_death::Pending {
                    destination,
                    deadline: self.clock.deadline(2),
                    animated: false,
                });
                p.begin_held_action();
                p.queues.clear();
                p.hits = Default::default();
                p.consumables = Default::default();
                p.npc_interaction = None;
                p.location_interaction = None;
                p.object_interaction = None;
                p.aggressive_npc = None;
                p.retaliate_npcs.clear();
                p.shop_session = None;
                p.bank_count = None;
                p.main_modal = None;
                p.prayer_notice = None;
                p.has_modal = false;
                p.death_queued = true;
                p.movement.rooted = true;
                p.movement.abort();
                p.movement.interacting = false;
                p.protection.current_clock = self.clock.0;
                p.protection.suspended = false;
                p.protection.delay(6);
                self.events.push(WorldEvent::CloseModals { actor });
                continue;
            }
            let pending = p.ordinary_death.unwrap();
            if self.clock.0 < pending.deadline {
                continue;
            }
            if !pending.animated {
                self.events.push(WorldEvent::PlayerCombatAnimation {
                    actor,
                    sequence: config.animation,
                });
                p.ordinary_death = Some(crate::player_death::Pending {
                    animated: true,
                    deadline: self.clock.deadline(4),
                    ..pending
                });
                continue;
            }
            // Missing destination collision must never turn this into an in-place
            // heal. Keep the job/root and retry after the map becomes available.
            if p.movement
                .teleport(&mut self.collision, pending.destination, self.clock.0, true)
                .is_err()
            {
                p.protection.current_clock = self.clock.0;
                p.protection.delay(1);
                continue;
            }
            p.data.coords = pending.destination;
            if let Some(music) = &self.music {
                let updates = p
                    .data
                    .variables
                    .set_bit(music.clocks.base, music.clocks.bit, 0, true)
                    .expect("startup-validated music binding")
                    .into_iter()
                    .collect();
                self.events
                    .push(WorldEvent::SessionVariables { actor, updates });
            }
            if let Some(prayer) = &self.prayer_book {
                let events = prayer
                    .book
                    .disable_all(&mut p.data.variables, p.overhead, true)
                    .expect("startup-validated prayer bindings");
                p.soft_timers.remove(prayer.drain_timer);
                p.soft_timers.remove(prayer.regeneration.rapid_timer);
                let mut queues = std::mem::take(&mut p.queues);
                prayer_book_actions::publish(
                    p,
                    &mut queues,
                    crate::prayer_runtime::Output {
                        events,
                        stat: None,
                        preserve_queue: Some(false),
                    },
                    &mut self.events,
                )
                .expect("disable cannot add a queue");
                p.queues = queues;
            }
            if let Some(regen) = &self.regeneration {
                regen.login(&mut p.soft_timers, self.clock.0);
            }
            if let Some(session) = &self.session_vars {
                p.soft_timers
                    .schedule(session.special_timer, self.clock.0, 50)
                    .unwrap();
            }
            let updates = p
                .data
                .variables
                .set(config.special_attack, 0, true)
                .into_iter()
                .collect();
            self.events
                .push(WorldEvent::CombatVariables { actor, updates });
            for &skill in &config.skills {
                let update = p
                    .data
                    .stats
                    .apply(skill, crate::stat_ops::Operation::Restore, 0, 0, skill == 3)
                    .unwrap();
                if update.notify {
                    self.events.push(WorldEvent::Stat { actor, update });
                }
            }
            p.hits = Default::default();
            p.queues.clear();
            p.aggressive_npc = None;
            p.retaliate_npcs.clear();
            p.last_npc_combat = -1000;
            p.last_pvp_combat = -1000;
            p.logout_blocked_until = self.clock.0;
            p.movement.rooted = false;
            p.death_queued = false;
            p.ordinary_death = None;
            p.protection.suspended = false;
            p.protection.delay_until = self.clock.0;
            self.events.push(WorldEvent::ClearMapFlag(actor));
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: u16::MAX,
            });
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "Oh dear, you are dead!".into(),
            });
            self.events.push(WorldEvent::OrdinaryDeathRecovered {
                actor,
                stop_music: config.stop_music,
                jingle: config.jingle,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> crate::player_death::Config {
        crate::player_death::Config {
            centre: CoordGrid::new(3221, 3218, 0).unwrap(),
            animation: 836,
            skills: (0..23).collect(),
            special_attack: crate::variables::Varp {
                id: 301,
                permanent: false,
                transmit: crate::variables::Transmit::Different,
                bit_protect: false,
                client_code: 0,
            },
            stop_music: 147,
            jingle: 90,
        }
    }
    fn setup(hp: u8, destination: bool) -> (World, ActorId) {
        let mut world = World::default();
        world.ordinary_death = Some(config());
        world.regeneration = Some(crate::player_regen::Config {
            skills: (0..23).collect(),
            hitpoints: 3,
            prayer: 5,
            bracelet: 11133,
            drained_timer: 1,
            boosted_timer: 2,
            health_timer: 3,
            rapid_timer: 4,
        });
        let coords = CoordGrid::new(3300, 3300, 0).unwrap();
        world.collision.allocate(coords.x(), coords.z(), 0);
        if destination {
            world.collision.allocate(3221, 3218, 0);
        }
        let mut data =
            PlayerData::new(1, "Recovery".into(), coords, crate::stats::Stats::default());
        for skill in 0..23 {
            data.stats.set_base(skill, 20);
            data.stats
                .set_current(skill, if skill % 2 == 0 { 25 } else { 3 });
            data.stats.set_fine_xp(skill, 1999 + skill as i32).unwrap();
        }
        data.stats.set_current(3, hp as i8);
        data.inventory
            .restore_slot(
                0,
                Some(crate::inventory::Item {
                    id: 1470,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.equipment
            .restore_slot(
                3,
                Some(crate::inventory::Item {
                    id: 13,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.bank
            .restore_slot(
                100,
                Some(crate::inventory::Item {
                    id: 995,
                    count: 777,
                    vars: 0,
                }),
            )
            .unwrap();
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        world.drain_events();
        (world, actor)
    }
    #[test]
    fn lethal_hit_runs_two_then_four_ticks_and_conserves_earned_state() {
        let (mut world, actor) = setup(20, true);
        let before = world.data(actor).unwrap().clone();
        world
            .queue_hit(
                crate::hits::Target::Player(actor),
                crate::hits::Hit {
                    source: None,
                    style: crate::hits::Style::Typeless,
                    damage: 20,
                    display: None,
                },
                0,
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 0);
        world.tick().unwrap();
        let started = world.clock.0 - 1;
        assert!(world
            .players
            .get(actor.slot as usize)
            .unwrap()
            .ordinary_death
            .is_some());
        assert!(!world.movement_input_allowed(actor).unwrap());
        world.drain_events();
        for delta in 1..=6 {
            world.tick().unwrap();
            let events = world.drain_events();
            assert_eq!(world.clock.0 - 1, started + delta);
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(
                        e,
                        WorldEvent::PlayerCombatAnimation { sequence: 836, .. }
                    ))
                    .count(),
                usize::from(delta == 2)
            );
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. }))
                    .count(),
                usize::from(delta == 6)
            );
            if delta < 6 {
                assert_eq!(world.data(actor).unwrap().stats.current(3), 0);
            }
        }
        let after = world.data(actor).unwrap();
        assert!((after.coords.x() - 3221).abs() <= 2 && (after.coords.z() - 3218).abs() <= 2);
        assert_eq!(after.inventory, before.inventory);
        assert_eq!(after.equipment, before.equipment);
        assert_eq!(after.tower_run, before.tower_run);
        assert_eq!(after.bank, before.bank);
        assert_eq!(after.other_inventories, before.other_inventories);
        for skill in 0..23 {
            assert_eq!(after.stats.base(skill), before.stats.base(skill));
            assert_eq!(after.stats.fine_xp(skill), before.stats.fine_xp(skill));
            assert_eq!(after.stats.current(skill), before.stats.base(skill));
        }
        assert!(world.movement_input_allowed(actor).unwrap());
        let p = world.players.get(actor.slot as usize).unwrap();
        for timer in [1, 2, 3] {
            assert_eq!(p.soft_timers.get(timer).unwrap().expiry, started + 6 + 100);
        }
        for _ in 0..8 {
            world.tick().unwrap();
        }
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. })));
    }
    #[test]
    fn saved_zero_health_reconstructs_formal_recovery_after_a_mid_death_restart() {
        let (mut first, actor) = setup(0, true);
        first.tick().unwrap();
        first.tick().unwrap();
        let saved = first.data(actor).unwrap().clone();
        assert_eq!(saved.stats.current(3), 0);
        let mut world = World::default();
        world.ordinary_death = Some(config());
        world
            .collision
            .allocate(saved.coords.x(), saved.coords.z(), 0);
        world.collision.allocate(3221, 3218, 0);
        world.queue_login(2, saved.clone()).unwrap();
        for _ in 0..8 {
            world.tick().unwrap();
        }
        let restored = world.data(world.actors().next().unwrap()).unwrap();
        assert_eq!(restored.stats.current(3), 20);
        assert_ne!(restored.coords, saved.coords);
        assert_eq!(restored.inventory, saved.inventory);
        assert_eq!(restored.equipment, saved.equipment);
        assert_eq!(restored.bank, saved.bank);
        for skill in 0..23 {
            assert_eq!(restored.stats.fine_xp(skill), saved.stats.fine_xp(skill));
        }
        assert_eq!(
            world
                .drain_events()
                .iter()
                .filter(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. }))
                .count(),
            1
        );
    }
    #[test]
    fn absent_respawn_collision_keeps_death_pending_past_health_regeneration() {
        let (mut world, actor) = setup(0, false);
        let coords = world.data(actor).unwrap().coords;
        for _ in 0..105 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().stats.current(3), 0);
        assert_eq!(world.data(actor).unwrap().coords, coords);
        assert!(!world.movement_input_allowed(actor).unwrap());
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. })));
        world.collision.allocate(3221, 3218, 0);
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 20);
    }
    #[test]
    fn unsigned_boosted_health_and_tutorial_rules_do_not_enter_ordinary_recovery() {
        let (mut world, actor) = setup(180, true);
        for _ in 0..8 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().stats.current(3) as u8, 180);
        assert!(world
            .players
            .get(actor.slot as usize)
            .unwrap()
            .ordinary_death
            .is_none());
        world.tutorial_rules_completed_varp = Some(900);
        world.data_mut(actor).unwrap().stats.set_current(3, 0);
        world.tick().unwrap();
        assert!(world
            .players
            .get(actor.slot as usize)
            .unwrap()
            .ordinary_death
            .is_none());
    }
    #[test]
    fn completion_resets_prayers_and_stat_special_timers_but_preserves_unrelated_timers() {
        let (mut world, actor) = setup(0, true);
        let binding = |id, msb| crate::bank_actions::Binding {
            base: crate::variables::Varp {
                id,
                permanent: false,
                transmit: crate::variables::Transmit::Different,
                bit_protect: false,
                client_code: 0,
            },
            bit: crate::variables::Varbit {
                base: id,
                lsb: 0,
                msb,
            },
        };
        let active = binding(700, 30);
        let quick = binding(701, 0);
        let counter = binding(702, 30);
        let resistance = binding(703, 20);
        let regen = world.regeneration.clone().unwrap();
        world.prayer_book = Some(crate::prayer_runtime::Config {
            book: crate::prayer_book::Config {
                prayer_skill: 5,
                defence_skill: 1,
                all_active: active,
                quick_active: quick,
                drain_counter: counter,
                disable_sound: 0,
                prayers: Default::default(),
                collisions: Default::default(),
                components: Default::default(),
            },
            drain: crate::prayer_drain::Config {
                enabled: active,
                counter,
                resistance,
                skill: 5,
                effects: vec![],
            },
            regeneration: regen,
            drain_timer: 8,
            depletion_sound: 0,
        });
        world.session_vars = Some(crate::session_vars::Config {
            clock_timer: 9,
            special_timer: 10,
            milliseconds: binding(704, 10),
            seconds: binding(705, 6),
            map_clock: binding(706, 30).base,
            playtime: binding(707, 30).base,
            energy: binding(708, 30).base,
            new_account: binding(709, 0),
        });
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        for (bit, value) in [(active, 15), (counter, 900), (quick, 1)] {
            p.data
                .variables
                .set_bit(bit.base, bit.bit, value, true)
                .unwrap();
        }
        p.data.variables.set(config().special_attack, 1, true);
        p.overhead = Some(2);
        for timer in [4, 8, 10, 919] {
            p.soft_timers.schedule(timer, world.clock.0, 80).unwrap();
        }
        let unrelated = p.soft_timers.get(919).unwrap();
        let mut random = crate::npc::SeededRandom(1);
        world.process_ordinary_death(&mut random);
        world.clock.0 += 2;
        world.process_ordinary_death(&mut random);
        world.clock.0 += 4;
        world.process_ordinary_death(&mut random);
        let p = world.players.get(actor.slot as usize).unwrap();
        for bit in [active, counter, quick] {
            assert_eq!(p.data.variables.bit(bit.bit).unwrap(), 0);
        }
        assert_eq!(p.data.variables.get(301), 0);
        assert_eq!(p.overhead, None);
        assert!(p.soft_timers.get(4).is_none());
        assert!(p.soft_timers.get(8).is_none());
        assert_eq!(p.soft_timers.get(10).unwrap().expiry, world.clock.0 + 50);
        for id in [1, 2, 3] {
            assert_eq!(p.soft_timers.get(id).unwrap().expiry, world.clock.0 + 100);
        }
        assert_eq!(p.soft_timers.get(919), Some(unrelated));
    }
    #[test]
    fn cinderworks_retains_sanctuary_priority_and_earned_run() {
        let (mut world, actor, _) = super::super::cinderworks_encounter_actions::tests::setup(
            crate::cinderworks_encounter::Boss::Imp,
        );
        world.ordinary_death = Some(config());
        let before = world.data(actor).unwrap().clone();
        world.data_mut(actor).unwrap().stats.set_current(3, 0);
        for _ in 0..7 {
            world.tick().unwrap();
        }
        let after = world.data(actor).unwrap();
        assert_eq!(after.coords, CoordGrid::new(10420, 110, 0).unwrap());
        assert_eq!(after.tower_run, before.tower_run);
        assert_eq!(after.inventory, before.inventory);
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. })));
    }
}
