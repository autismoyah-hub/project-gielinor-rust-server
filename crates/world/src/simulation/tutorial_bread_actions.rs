use super::*;
impl World {
    pub(super) fn process_tutorial_bread(&mut self, slot: usize) {
        let Some(config) = self.tutorial_bread else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        let result = config.cook(
            &mut p.data,
            &self.object_types,
            p.protection.processed_clock > 0,
        );
        match result {
            Ok(crate::tutorial_bread::Outcome::Silent) => {}
            Ok(crate::tutorial_bread::Outcome::Missing) => {
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: crate::tutorial_bread::MISSING,
                })
            }
            Ok(crate::tutorial_bread::Outcome::Cooked {
                advancement,
                progression,
            }) => {
                self.events.push(WorldEvent::PlayerCombatAnimation {
                    actor,
                    sequence: config.animation,
                });
                if self.tutorial_rules_completed_varp.is_some()
                    && p.stat_events
                        .advance(
                            &crate::tutorial_rules::bindings(),
                            config.skill,
                            advancement,
                        )
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
                    skill: config.skill,
                    advancement,
                });
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: "You manage to bake some bread.",
                });
                self.events.push(WorldEvent::TutorialInventoryProgress {
                    actor,
                    effects: progression,
                });
            }
            Err(_) => self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidStat,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Insert,
        loc_collision::{self, LocCollision},
        locations::Location,
        tutorial_bread::Config,
        tutorial_progression,
        variables::{Transmit, Varp},
    };
    fn setup() -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        w.tutorial_bread = Some(Config {
            progression: tutorial_progression::Config {
                step: varp(1),
                completed: varp(2),
            },
            location: 1,
            dough: 10,
            bread: 20,
            animation: 897,
            skill: 7,
            max_level: 99,
            global_xp_rate: 1.0,
        });
        for id in [10, 20, 30] {
            w.held_use_types
                .insert(id, crate::held_use::Type { id, content: 0 });
        }
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: false,
            break_route_finding: false,
        };
        let loc = Location::new(CoordGrid::new(3204, 3200, 0).unwrap(), 1, 10, 0).unwrap();
        w.location_types.insert(1, ty);
        w.locations.load(loc);
        loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        let mut p = PlayerData::new(
            1,
            "Baker".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            crate::stats::Stats::default(),
        );
        p.variables.set(varp(1), 17, false);
        p.inventory
            .insert(&w.object_types, Insert::new(10, 1))
            .unwrap();
        w.queue_login(1, p).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.drain_events();
        (w, actor, loc)
    }
    #[test]
    fn tutorial_ore_ops_route_and_keep_rocks_while_awarding_fractional_xp() {
        let (mut w, actor, loc) = setup();
        let base = w.tutorial_bread.take().unwrap();
        w.tutorial_mining = Some(crate::tutorial_mining::Config {
            progression: base.progression,
            tin_rock: 1,
            copper_rock: 2,
            tin: 438,
            copper: 436,
            pickaxe: 1265,
            animation: 625,
            skill: 14,
            max_level: 99,
            global_xp_rate: 1.0,
        });
        w.location_types.insert(2, w.location_types[&1]);
        let p = w.data_mut(actor).unwrap();
        p.variables.set(base.progression.step, 24, true);
        p.inventory
            .restore_slot(
                0,
                Some(crate::inventory::Item {
                    id: 1265,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        for id in [1, 2] {
            let rock = crate::locations::Location::new(loc.coords, id, 10, 0).unwrap();
            if id == 2 {
                w.change_location(rock, true).unwrap();
            }
            assert!(w.has_location_handler(id, 1));
            w.submit(
                actor,
                Action::OperateLocation {
                    coords: loc.coords,
                    id,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            for _ in 0..12 {
                w.tick().unwrap();
            }
            assert_eq!(w.locations.find_type(loc.coords, id), Some(rock));
            assert_eq!(
                w.data(actor).unwrap().stats.fine_xp(14),
                if id == 1 { 175 } else { 350 }
            );
            assert_eq!(
                w.data(actor).unwrap().variables.get(1),
                if id == 1 { 24 } else { 25 }
            );
        }
        for item in [438, 436] {
            assert!(w
                .data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == item));
        }
    }
    #[test]
    fn shrimp_requires_item_target_and_step_then_cooks_without_losing_later_progress() {
        for step in [11, 12, 13, 20] {
            let (mut w, actor, loc) = setup();
            let base = w.tutorial_bread.take().unwrap();
            w.tutorial_shrimp = Some(crate::tutorial_shrimp::Config {
                progression: base.progression,
                location: loc.id,
                raw: 10,
                cooked: 30,
                animation: 897,
                skill: 7,
                max_level: 99,
                global_xp_rate: 1.0,
            });
            w.data_mut(actor)
                .unwrap()
                .variables
                .set(base.progression.step, step, true);
            assert!(
                !w.has_location_handler(loc.id, 1),
                "fire has no ordinary cooking operation"
            );
            w.submit(
                actor,
                Action::UseLocation {
                    coords: loc.coords,
                    id: loc.id,
                    slot: 0,
                    object: 10,
                    control: false,
                },
            )
            .unwrap();
            for _ in 0..12 {
                w.tick().unwrap();
            }
            let p = w.data(actor).unwrap();
            assert_eq!(
                p.inventory.get(0).unwrap().id,
                if step >= 12 { 30 } else { 10 }
            );
            assert_eq!(p.stats.xp(7), if step >= 12 { 30 } else { 0 });
            assert_eq!(p.variables.get(1), if step == 12 { 13 } else { step });
        }
    }
    #[test]
    fn use_dough_on_range_rechecks_selected_slot_after_routing() {
        for replace in [false, true] {
            let (mut w, actor, loc) = setup();
            w.submit(
                actor,
                Action::UseLocation {
                    coords: loc.coords,
                    id: loc.id,
                    slot: 0,
                    object: 10,
                    control: false,
                },
            )
            .unwrap();
            w.tick().unwrap();
            if replace {
                // A matching dough elsewhere must not authorize a stale slot.
                let p = w.data_mut(actor).unwrap();
                let dough = p.inventory.get(0);
                p.inventory.restore_slot(1, dough).unwrap();
                p.inventory
                    .restore_slot(
                        0,
                        Some(crate::inventory::Item {
                            id: 30,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            for _ in 0..12 {
                w.tick().unwrap();
            }
            let p = w.data(actor).unwrap();
            assert_eq!(p.stats.xp(7), if replace { 0 } else { 40 });
            assert_eq!(p.variables.get(1), if replace { 17 } else { 18 });
            if replace {
                assert_eq!(p.inventory.get(1).unwrap().id, 10);
            }
        }
    }
    #[test]
    fn use_item_route_cancels_on_movement_and_rejects_missing_location() {
        for remove in [false, true] {
            let (mut w, actor, loc) = setup();
            w.submit(
                actor,
                Action::UseLocation {
                    coords: loc.coords,
                    id: loc.id,
                    slot: 0,
                    object: 10,
                    control: false,
                },
            )
            .unwrap();
            w.tick().unwrap();
            if remove {
                w.change_location(loc, false).unwrap();
            } else {
                w.submit(
                    actor,
                    Action::MoveClick {
                        destination: CoordGrid::new(3200, 3201, 0).unwrap(),
                        control: false,
                    },
                )
                .unwrap();
            }
            for _ in 0..12 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(actor).unwrap().stats.xp(7), 0);
            assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 10);
        }
    }
    #[test]
    fn reachable_range_routes_then_cooks_once_with_xp_and_progress_effects() {
        let (mut w, actor, loc) = setup();
        assert!(w.has_location_handler(loc.id, 1));
        assert!(!w.has_location_handler(loc.id, 2));
        w.submit(
            actor,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        let mut events = vec![];
        for _ in 0..12 {
            w.tick().unwrap();
            events.extend(w.drain_events());
        }
        assert_eq!(w.data(actor).unwrap().stats.xp(7), 40);
        assert_eq!(w.data(actor).unwrap().variables.get(1), 18);
        assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 20);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 7, .. }))
                .count(),
            1
        );
        let animation = events
            .iter()
            .position(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 897, .. }))
            .unwrap();
        let xp = events
            .iter()
            .position(|e| matches!(e, WorldEvent::SkillAdvanced { .. }))
            .unwrap();
        let focus = events
            .iter()
            .position(|e| matches!(e, WorldEvent::TutorialInventoryProgress { .. }))
            .unwrap();
        assert!(animation < xp && xp < focus);
        assert!(w.observe(actor).unwrap().coords != CoordGrid::new(3200, 3200, 0).unwrap());
    }
    #[test]
    fn disappearing_range_cannot_consume_ingredients_after_the_click() {
        let (mut w, actor, loc) = setup();
        w.submit(
            actor,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick().unwrap();
        w.drain_events();
        w.change_location(loc, false).unwrap();
        for _ in 0..12 {
            w.tick().unwrap();
            w.drain_events();
        }
        assert_eq!(w.data(actor).unwrap().stats.xp(7), 0);
        assert_eq!(w.data(actor).unwrap().variables.get(1), 17);
        assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 10);
    }
}
