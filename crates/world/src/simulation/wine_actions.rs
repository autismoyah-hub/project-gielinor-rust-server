use super::*;
impl World {
    pub(super) fn process_wine_ferment(
        &mut self,
        slot: usize,
        timers: &mut crate::player_timers::Timers,
        timer: u16,
        random: &mut impl RandomSource,
    ) -> bool {
        let Some(config) = crate::wine::for_timer([self.wine, self.zamorak_wine], timer) else {
            timers.remove(timer);
            return true;
        };
        let Some(p) = self.players.get_mut(slot) else {
            return true;
        };
        if p.saving || p.pending_logout {
            return true;
        }
        match config.ferment(
            &mut p.data,
            &self.object_types,
            self.banking.as_ref(),
            random,
        ) {
            Ok(result) => {
                timers.remove(config.timer);
                if let Some(result) = result {
                    if let Some(advancement) = result.advancement {
                        if self.tutorial_rules_completed_varp.is_some()
                            && p.stat_events
                                .advance(&crate::tutorial_rules::bindings(), 7, advancement)
                                .is_err()
                        {
                            p.pending_logout = true;
                            self.events.push(WorldEvent::ActionRejected {
                                actor: p.id,
                                error: ActionError::QueueFull,
                            });
                        }
                        self.events.push(WorldEvent::SkillAdvanced {
                            actor: p.id,
                            skill: 7,
                            advancement,
                        });
                    }
                    if !result.bank_events.is_empty() {
                        self.events.push(WorldEvent::Bank {
                            actor: p.id,
                            result: Ok(result.bank_events),
                        });
                    }
                    if result.failures > 0 {
                        self.events.push(WorldEvent::LocationMessage {
                            actor: p.id,
                            message: "Some of your wine has turned bad.",
                        });
                    }
                }
            }
            Err(_) => { /* Preserve all images and retry via the same20tick soft timer when room/policy recovers. */
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request as Held, Scope, Type},
        inventory::Item,
        wine,
    };
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Guaranteed wine/mixing must not draw RNG")
        }
    }
    fn fixture() -> (World, ActorId, crate::assembly_production::Request) {
        let mut w = World::default();
        let config = wine::tests::config();
        w.wine = Some(config);
        w.object_types = wine::tests::catalog();
        w.collision.allocate(3200, 3200, 0);
        let (mut data, bank) = wine::tests::player(0, 0, 68);
        for s in 0..28 {
            data.inventory
                .restore_slot(
                    s,
                    Some(Item {
                        id: 100,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        for (s, id) in [
            (0, 1987),
            (2, 1987),
            (4, 1987),
            (1, 1937),
            (3, 1937),
            (5, 1937),
        ] {
            data.inventory
                .restore_slot(
                    s,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.banking = Some(bank);
        w.queue_login(1, data).unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let actor = w.actors().next().unwrap();
        w.drain_events();
        for id in [1987, 1937] {
            w.held_use_types.insert(id, Type { id, content: 0 });
        }
        w.held_use_handlers
            .register(
                Scope::Types,
                1987,
                1937,
                Handler::AssemblyProduction {
                    recipe: config.recipe(),
                    cycle_ticks: 2,
                },
            )
            .unwrap();
        let held = Held {
            selected_slot: 1,
            selected_id: 1937,
            target_slot: 0,
            target_id: 1987,
        };
        w.prepare_held_use(actor, held).unwrap();
        w.finish_held_use(actor, held).unwrap();
        let req = w.take_assembly_production_request(actor).unwrap().unwrap();
        assert_eq!(req.wine, Some(config));
        (w, actor, req)
    }
    fn count(w: &World, a: ActorId, id: i32) -> i32 {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .map(|i| i.count)
            .sum()
    }
    #[test]
    fn plain_wine_actual_shared_make_x_first3_cycle2_resets_soft_timer_and_ferments_while_modal() {
        let (mut w, a, req) = fixture();
        let baseline = w.data(a).unwrap().stats.fine_xp(7);
        let start = w.clock.0;
        w.start_assembly_production(a, req, 0, 3).unwrap();
        let mut clocks = Vec::new();
        for _ in 0..12 {
            w.tick_with_random(&mut NoDraw).unwrap();
            for event in w.drain_events() {
                if let WorldEvent::AssemblyMade { trace, .. } = event {
                    clocks.push(trace.committed_at);
                }
            }
        }
        assert_eq!(clocks, vec![start + 3, start + 5, start + 7]);
        assert_eq!(count(&w, a, 1995), 3);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(7), baseline);
        let expiry = w
            .players
            .get(a.slot as usize)
            .unwrap()
            .soft_timers
            .get(wine::TIMER)
            .unwrap()
            .expiry;
        assert_eq!(expiry, clocks[2] + 20);
        w.set_modal_open(a, true).unwrap();
        for _ in 0..25 {
            w.tick_with_random(&mut NoDraw).unwrap();
            if count(&w, a, 1995) == 0 {
                assert!(w.clock.0 >= expiry && w.clock.0 <= expiry + 1);
                break;
            }
        }
        assert_eq!(count(&w, a, 1993), 3);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(7) - baseline, 6000);
        assert!(w
            .players
            .get(a.slot as usize)
            .unwrap()
            .soft_timers
            .get(wine::TIMER)
            .is_none());
    }
    #[test]
    fn plain_wine_shared_queue_interrupt_keeps_finished_jug_and_rechecks_cap_and_captured_config() {
        for mode in 0..4 {
            let (mut w, a, req) = fixture();
            let baseline = w.data(a).unwrap().stats.fine_xp(7);
            if mode == 3 {
                w.wine = None;
                assert!(w.start_assembly_production(a, req, 0, 3).is_err());
                continue;
            }
            w.start_assembly_production(a, req, 0, 3).unwrap();
            for _ in 0..8 {
                w.tick_with_random(&mut NoDraw).unwrap();
                if count(&w, a, 1995) == 1 {
                    break;
                }
            }
            assert_eq!(count(&w, a, 1995), 1);
            if mode == 0 {
                w.players
                    .get_mut(a.slot as usize)
                    .unwrap()
                    .begin_held_action();
            } else if mode == 1 {
                w.data_mut(a)
                    .unwrap()
                    .bank
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 1995,
                            count: 59999,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                let cfg = w.banking.as_ref().unwrap().clone();
                cfg.tabs[9]
                    .set(&mut w.data_mut(a).unwrap().variables, 1)
                    .unwrap();
            } else {
                w.wine = None;
            }
            for _ in 0..8 {
                w.tick_with_random(&mut NoDraw).unwrap();
            }
            assert_eq!(count(&w, a, 1995), 1);
            assert_eq!(count(&w, a, 1987), 2);
            assert_eq!(count(&w, a, 1937), 2);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(7), baseline);
            if mode == 0 {
                for _ in 0..25 {
                    w.tick_with_random(&mut NoDraw).unwrap();
                }
                assert_eq!(count(&w, a, 1993), 1);
                assert_eq!(w.data(a).unwrap().stats.fine_xp(7) - baseline, 2000);
            }
        }
    }

    fn zam_fixture(
        level: i8,
        member: bool,
    ) -> (World, ActorId, Option<crate::assembly_production::Request>) {
        let (mut w, a, _) = fixture();
        let config = wine::tests::zam_config();
        w.zamorak_wine = Some(config);
        w.set_membership(a, member).unwrap();
        let stats = &mut w.data_mut(a).unwrap().stats;
        stats.set_base(7, level);
        stats.set_current(7, level);
        stats
            .set_xp(7, crate::skills::xp_from_level(level as u8).unwrap())
            .unwrap();
        for s in [0, 2, 4] {
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(
                    s,
                    Some(Item {
                        id: 20749,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.held_use_types.insert(
            20749,
            Type {
                id: 20749,
                content: 0,
            },
        );
        w.held_use_handlers
            .register(
                Scope::Types,
                20749,
                1937,
                Handler::AssemblyProduction {
                    recipe: config.recipe(),
                    cycle_ticks: 2,
                },
            )
            .unwrap();
        let held = Held {
            selected_slot: 1,
            selected_id: 1937,
            target_slot: 0,
            target_id: 20749,
        };
        w.prepare_held_use(a, held).unwrap();
        w.finish_held_use(a, held).unwrap();
        let req = w.take_assembly_production_request(a).unwrap();
        (w, a, req)
    }
    #[test]
    fn zamorak_wine_actual_pair_selects_second_family_and_only_resets_its_timer() {
        let (mut w, a, req) = zam_fixture(98, true);
        let req = req.unwrap();
        assert_eq!(req.wine, Some(wine::tests::zam_config()));
        let baseline = w.data(a).unwrap().stats.fine_xp(7);
        let start = w.clock.0;
        w.start_assembly_production(a, req, 0, 2).unwrap();
        let mut clocks = Vec::new();
        for _ in 0..12 {
            w.tick_with_random(&mut NoDraw).unwrap();
            for e in w.drain_events() {
                if let WorldEvent::AssemblyMade { trace, .. } = e {
                    clocks.push(trace.committed_at);
                }
            }
        }
        assert_eq!(clocks, vec![start + 3, start + 5]);
        assert_eq!(count(&w, a, 20752), 2);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(7), baseline);
        let p = w.players.get(a.slot as usize).unwrap();
        assert!(p.soft_timers.get(wine::TIMER).is_none());
        assert_eq!(
            p.soft_timers.get(wine::ZAMORAK_TIMER).unwrap().expiry,
            clocks[1] + 20
        );
        w.set_modal_open(a, true).unwrap();
        for _ in 0..25 {
            w.tick_with_random(&mut NoDraw).unwrap();
        }
        assert_eq!(count(&w, a, 245), 2);
        assert_eq!(count(&w, a, 1995), 0);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(7) - baseline, 4000);
    }
    #[test]
    fn zamorak_wine_live_membership_and_level_guard_mixing_without_blocking_finished_jug_timer() {
        let (w, _, req) = zam_fixture(98, false);
        assert!(req.is_none());
        drop(w);
        let (mut w, a, req) = zam_fixture(64, true);
        let before = w.data(a).unwrap().clone();
        w.start_assembly_production(a, req.unwrap(), 0, 1).unwrap();
        for _ in 0..8 {
            w.tick_with_random(&mut NoDraw).unwrap();
        }
        assert_eq!(w.data(a).unwrap().inventory, before.inventory);
        assert_eq!(w.data(a).unwrap().stats, before.stats);
        let (mut w, a, req) = zam_fixture(98, true);
        let baseline = w.data(a).unwrap().stats.fine_xp(7);
        w.start_assembly_production(a, req.unwrap(), 0, 3).unwrap();
        for _ in 0..8 {
            w.tick_with_random(&mut NoDraw).unwrap();
            if count(&w, a, 20752) == 1 {
                break;
            }
        }
        w.set_membership(a, false).unwrap();
        for _ in 0..30 {
            w.tick_with_random(&mut NoDraw).unwrap();
        }
        assert_eq!(count(&w, a, 20749), 2);
        assert_eq!(count(&w, a, 245), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(7) - baseline, 2000);
    }
    #[test]
    fn both_wine_login_timers_recover_and_normal_reset_does_not_postpone_zamorak() {
        let mut w = World::default();
        w.wine = Some(wine::tests::config());
        w.zamorak_wine = Some(wine::tests::zam_config());
        w.object_types = wine::tests::catalog();
        let (mut p, b) = wine::tests::player(2, 0, 98);
        p.inventory
            .restore_slot(
                1,
                Some(Item {
                    id: 20752,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.banking = Some(b);
        w.queue_login(1, p).unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let a = w.actors().next().unwrap();
        let baseline = w.data(a).unwrap().stats.fine_xp(7);
        let original = w
            .players
            .get(a.slot as usize)
            .unwrap()
            .soft_timers
            .get(wine::ZAMORAK_TIMER)
            .unwrap()
            .expiry;
        for _ in 0..5 {
            w.tick_with_random(&mut NoDraw).unwrap();
        }
        let now = w.clock.0;
        w.players
            .get_mut(a.slot as usize)
            .unwrap()
            .soft_timers
            .schedule(wine::TIMER, now, wine::INTERVAL)
            .unwrap();
        assert_eq!(
            w.players
                .get(a.slot as usize)
                .unwrap()
                .soft_timers
                .get(wine::ZAMORAK_TIMER)
                .unwrap()
                .expiry,
            original
        );
        for _ in 0..25 {
            w.tick_with_random(&mut NoDraw).unwrap();
            if count(&w, a, 245) == 1 {
                assert_eq!(count(&w, a, 1995), 1);
                break;
            }
        }
        assert_eq!(count(&w, a, 245), 1);
        for _ in 0..10 {
            w.tick_with_random(&mut NoDraw).unwrap();
        }
        assert_eq!(count(&w, a, 1993), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(7) - baseline, 4000);
        let p = w.players.get(a.slot as usize).unwrap();
        assert!(
            p.soft_timers.get(wine::TIMER).is_none()
                && p.soft_timers.get(wine::ZAMORAK_TIMER).is_none()
        );
    }
}
