use super::*;
use crate::{
    locations::Location,
    pickables::{Preparation, Rule, Step, Transition},
};
impl World {
    pub(super) fn emit_pickable(&mut self, actor: ActorId, location: Location, rule: Rule) {
        self.events.push(WorldEvent::PlayerAnimation {
            actor,
            sequence: rule.animation,
        });
        self.events.push(WorldEvent::LocationMessage {
            actor,
            message: rule.picked_message,
        });
        self.events.push(WorldEvent::PlayerSound {
            actor,
            sound: rule.sound,
        });
        self.events.push(WorldEvent::PickableCollected {
            actor,
            location,
            item: rule.item,
        });
    }
    pub(super) fn process_pickable_stage(
        &mut self,
        slot: usize,
        location: Location,
        rule: Rule,
        step: Step,
    ) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if self.locations.revision(location) != p.location_revision {
            return;
        }
        let advance = match step.prepare(location, self.clock.0) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        if !self.location_types.contains_key(&advance.replacement.id)
            || advance
                .restoration
                .is_some_and(|(l, _)| !self.location_types.contains_key(&l.id))
        {
            self.events.push(WorldEvent::LocationError(
                "Unknown pickable replacement type",
            ));
            return;
        }
        let plan = match rule.prepare(&p.data, &self.object_types, p.member) {
            Ok(Preparation::Grant(plan)) => plan,
            Ok(Preparation::Refused(outcome)) => {
                if let Some(message) = outcome.message() {
                    self.events
                        .push(WorldEvent::LocationMessage { actor, message });
                }
                return;
            }
            Err(_) => {
                self.events.push(WorldEvent::LocationError(
                    "Pickable inventory preparation failed",
                ));
                return;
            }
        };
        // The shared location transaction validates collision before changing live state.
        if let Err(error) = self.change_location(advance.replacement, true) {
            self.events.push(WorldEvent::LocationError(error));
            return;
        }
        if let Some((restore, duration)) = advance.restoration {
            let revision = self
                .locations
                .revision(advance.replacement)
                .expect("committed pickable replacement");
            self.location_timers.replace(
                restore,
                advance.replacement,
                revision,
                self.clock,
                duration,
            );
        }
        let p = self.players.get_mut(slot).unwrap();
        let rule = plan.commit(&mut p.data);
        self.emit_pickable(actor, location, rule);
        self.events.push(WorldEvent::PickableAdvanced {
            actor,
            trace: Transition {
                tick: self.clock.0,
                from: location,
                to: advance.replacement,
                item: rule.item,
                restore: advance.restoration.map(|(l, _)| l),
                restore_at: advance.deadline,
            },
        });
    }
}
impl World {
    pub(super) fn process_pickable_chance(
        &mut self,
        slot: usize,
        location: Location,
        rule: Rule,
        depletion: crate::pickables::Depletion,
        random: &mut impl RandomSource,
    ) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if self.locations.revision(location) != p.location_revision {
            return;
        }
        let deadline = match depletion.deadline(self.clock.0) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        // Temporary deletion restores a cache node. A dynamically spawned lookalike
        // may reveal a different underlying node, so it must not enter this path.
        if !self.locations.is_map_location(location)
            || !self.location_types.contains_key(&location.id)
        {
            self.events.push(WorldEvent::LocationError(
                "Pickable depletion requires a current cache location",
            ));
            return;
        }
        let plan = match rule.prepare(&p.data, &self.object_types, p.member) {
            Ok(Preparation::Grant(v)) => v,
            Ok(Preparation::Refused(o)) => {
                if let Some(message) = o.message() {
                    self.events
                        .push(WorldEvent::LocationMessage { actor, message });
                }
                return;
            }
            Err(_) => {
                self.events.push(WorldEvent::LocationError(
                    "Pickable inventory preparation failed",
                ));
                return;
            }
        };
        // Source empty depletion chances mean certain despawn without a random draw.
        let draw = if depletion.numerator == depletion.denominator {
            0
        } else {
            random.below(depletion.denominator)
        };
        if draw >= depletion.denominator {
            self.events
                .push(WorldEvent::LocationError("Invalid pickable random draw"));
            return;
        }
        let roll = draw + 1;
        let depleted = roll <= depletion.numerator;
        let replacement = if depleted {
            if let Err(error) = self.change_location(location, false) {
                self.events.push(WorldEvent::LocationError(error));
                return;
            }
            let revision = self
                .locations
                .deleted_revision(location)
                .expect("validated cache deletion has a tombstone");
            let deleted = Location {
                id: crate::locations::DELETED_ID,
                ..location
            };
            self.location_timers
                .replace(location, deleted, revision, self.clock, depletion.delay);
            deleted
        } else {
            location
        };
        let rule = plan.commit(&mut self.players.get_mut(slot).unwrap().data);
        self.emit_pickable(actor, location, rule);
        self.events.push(WorldEvent::PickableDepletion {
            actor,
            trace: crate::pickables::ChanceTransition {
                transition: Transition {
                    tick: self.clock.0,
                    from: location,
                    to: replacement,
                    item: rule.item,
                    restore: depleted.then_some(location),
                    restore_at: depleted.then_some(deadline),
                },
                roll,
                denominator: depletion.denominator,
                depleted,
            },
        });
    }
}

impl World {
    pub(super) fn process_nettles(
        &mut self,
        slot: usize,
        location: Location,
        rule: Rule,
        excluded_glove: i32,
        mark: crate::hit_display::Mark,
        random: &mut impl RandomSource,
    ) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if self.locations.revision(location) != p.location_revision {
            return;
        }
        if rule.members && !p.member {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: "You need to be a member to pick this.",
            });
            return;
        }
        if p.data
            .equipment
            .get(9)
            .is_some_and(|i| i.id != excluded_glove)
        {
            self.process_pickable_chance(
                slot,
                location,
                rule,
                crate::pickables::Depletion {
                    numerator: 1,
                    denominator: 1,
                    delay: 13,
                },
                random,
            );
            return;
        }
        // Donor handles stinging before inventory space. A full bag offers no protection.
        // It is one incidental typeless hit, not ongoing poison or an immunity check.
        let draw = random.below(2);
        if draw >= 2 {
            self.events.push(WorldEvent::LocationError(
                "Invalid nettle sting random draw",
            ));
            return;
        }
        let hit = crate::hits::Hit {
            display: Some(mark),
            source: None,
            style: crate::hits::Style::Typeless,
            damage: draw as i32 + 1,
        };
        if self
            .queue_hit(crate::hits::Target::Player(actor), hit, 1)
            .is_err()
        {
            self.events
                .push(WorldEvent::LocationError("Nettle sting hit queue refused"));
            return;
        }
        self.events.push(WorldEvent::PlayerAnimation {
            actor,
            sequence: rule.animation,
        });
        self.events.push(WorldEvent::LocationMessage {
            actor,
            message: "The nettles sting you.",
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, loc_collision::LocCollision, location_content::Handler};
    fn rule() -> Rule {
        Rule {
            item: 1000,
            members: false,
            animation: 827,
            sound: 2581,
            picked_message: "You pick some berries.",
            full_message: "You can't carry any more berries.",
        }
    }
    fn fixture() -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: false,
            break_route_finding: false,
        };
        for id in [100, 101, 102] {
            w.location_types.insert(id, ty);
        }
        let l = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 100, 10, 0).unwrap();
        w.locations.load(l);
        crate::loc_collision::toggle(&mut w.collision, l.coords, l.shape, l.angle, ty, true)
            .unwrap();
        for op in [1, 2] {
            w.location_handlers.insert(
                (100, op),
                Handler::PickableStage {
                    rule: rule(),
                    step: Step::Next { location: 101 },
                },
            );
            w.location_handlers.insert(
                (101, op),
                Handler::PickableStage {
                    rule: rule(),
                    step: Step::Respawn {
                        empty: 102,
                        full: 100,
                        delay: 4,
                    },
                },
            );
            w.location_handlers.insert(
                (102, op),
                Handler::EmptyPickable("There are no berries on this bush at the moment."),
            );
        }
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Berries".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a, l)
    }
    fn action(l: Location, operation: u8) -> Action {
        Action::OperateLocation {
            coords: l.coords,
            id: l.id,
            operation,
            control: false,
        }
    }
    fn count(w: &World, a: ActorId) -> i32 {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 1000)
            .map(|i| i.count)
            .sum()
    }
    fn pick(w: &mut World, a: ActorId, l: Location) -> Location {
        w.submit(a, action(l, 1)).unwrap();
        for _ in 0..6 {
            w.tick().unwrap();
            if w.locations.find_type(l.coords, l.id) != Some(l) {
                return w.locations.find_shape(l.coords, l.shape).unwrap();
            }
        }
        panic!("stage did not change")
    }
    #[test]
    fn staged_pickable_two_rewards_empty_message_and_exact_full_root_restoration_use_shared_timer()
    {
        let (mut w, a, full) = fixture();
        let stats = w.data(a).unwrap().stats.clone();
        let one = pick(&mut w, a, full);
        assert_eq!(one.id, 101);
        assert_eq!(count(&w, a), 1);
        assert!(w
            .location_timers
            .replacements_due(w.clock, &w.locations)
            .is_empty());
        let empty = pick(&mut w, a, one);
        assert_eq!(empty.id, 102);
        assert_eq!(count(&w, a), 2);
        let event = w
            .drain_events()
            .into_iter()
            .find_map(|e| {
                if let WorldEvent::PickableAdvanced { trace, .. } = e {
                    trace.restore_at.map(|at| (trace, at))
                } else {
                    None
                }
            })
            .unwrap();
        let deadline = event.1;
        assert_eq!(event.0.restore.unwrap().id, 100);
        assert!(event.0.json(a).contains("\"restore_tick\":"));
        w.submit(a, action(empty, 1)).unwrap();
        w.tick().unwrap();
        assert_eq!(count(&w, a), 2);
        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::LocationMessage{message,..}if *message=="There are no berries on this bush at the moment.")));
        while w.clock.0 < deadline {
            assert_eq!(
                w.locations
                    .find_shape(empty.coords, empty.shape)
                    .unwrap()
                    .id,
                102
            );
            w.tick().unwrap();
        }
        assert_eq!(
            w.locations
                .find_shape(empty.coords, empty.shape)
                .unwrap()
                .id,
            102
        );
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(full.coords, full.shape), Some(full));
        assert_eq!(count(&w, a), 2);
        assert_eq!(w.data(a).unwrap().stats, stats);
        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::LocationRestoration(t)if t.tick==deadline&&t.result=="restored")));
    }
    #[test]
    fn staged_pickable_full_invalid_catalog_and_genuine_collision_error_preserve_stock_and_inventory(
    ) {
        for mode in [0, 2] {
            let (mut w, a, l) = fixture();
            if mode == 0 {
                for slot in 0..28 {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 995,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
            } else {
                w.object_types.dummy.insert(1000);
            }
            let before = (
                w.data(a).unwrap().inventory.slots().to_vec(),
                w.locations.revision(l),
            );
            w.submit(a, action(l, 1)).unwrap();
            for _ in 0..7 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().inventory.slots(), before.0.as_slice());
            assert_eq!(w.locations.revision(l), before.1);
            assert!(w
                .location_timers
                .replacements_due(w.clock, &w.locations)
                .is_empty());
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::PickableCollected { .. }
                    | WorldEvent::PickableAdvanced { .. }
                    | WorldEvent::PlayerAnimation { .. }
                    | WorldEvent::PlayerSound { .. }
            )));
        }
        // Large ordinary footprints are valid: allocation is not a collision error.
        // Inject a real shared-transaction failure at the world coordinate boundary.
        let (mut w, a, _) = fixture();
        w.collision.allocate(3200, 16376, 0);
        let edge = Location::new(CoordGrid::new(3201, 16383, 0).unwrap(), 100, 10, 0).unwrap();
        w.locations.load(edge);
        crate::loc_collision::toggle(
            &mut w.collision,
            edge.coords,
            edge.shape,
            edge.angle,
            w.location_types[&100],
            true,
        )
        .unwrap();
        w.location_types.get_mut(&101).unwrap().length = 2;
        let rev = w.locations.revision(edge);
        w.players
            .get_mut(a.slot as usize)
            .unwrap()
            .location_revision = rev;
        let before = w.data(a).unwrap().inventory.clone();
        w.drain_events();
        w.process_pickable_stage(a.slot as usize, edge, rule(), Step::Next { location: 101 });
        assert_eq!(w.data(a).unwrap().inventory, before);
        assert_eq!(w.locations.revision(edge), rev);
        assert!(w
            .location_timers
            .replacements_due(w.clock, &w.locations)
            .is_empty());
        let events = w.drain_events();
        assert!(events.iter().any(|e| matches!(
            e,
            WorldEvent::LocationError("Location extends outside world")
        )));
        assert!(!events.iter().any(|e| matches!(
            e,
            WorldEvent::PickableCollected { .. }
                | WorldEvent::PickableAdvanced { .. }
                | WorldEvent::PlayerAnimation { .. }
                | WorldEvent::PlayerSound { .. }
        )));
    }
    #[test]
    fn staged_pickable_old_full_packet_and_stale_empty_generation_cannot_duplicate_or_restore() {
        let (mut w, a, l) = fixture();
        let one = pick(&mut w, a, l);
        w.submit(a, action(l, 1)).unwrap();
        w.tick().unwrap();
        assert_eq!(count(&w, a), 1);
        assert_eq!(w.locations.find_shape(one.coords, one.shape), Some(one));
        let empty = pick(&mut w, a, one);
        w.change_location(empty, false).unwrap();
        w.change_location(empty, true).unwrap();
        for _ in 0..6 {
            w.tick().unwrap();
        }
        assert_eq!(
            w.locations.find_shape(empty.coords, empty.shape),
            Some(empty)
        );
        assert_eq!(count(&w, a), 2);
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e,WorldEvent::LocationRestoration(t)if t.result=="stale")));
    }
    #[test]
    fn staged_pickable_two_actors_compete_for_one_current_stage_without_duplicate_reward() {
        let (mut w, a, l) = fixture();
        w.queue_login(
            2,
            PlayerData::new(
                2,
                "Other".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let b = w.actors().find(|id| *id != a).unwrap();
        w.submit(a, action(l, 1)).unwrap();
        w.submit(b, action(l, 1)).unwrap();
        for _ in 0..4 {
            w.tick().unwrap();
        }
        assert_eq!(count(&w, a) + count(&w, b), 1);
        assert_eq!(w.locations.find_shape(l.coords, l.shape).unwrap().id, 101);
    }
    struct Draw {
        value: u32,
        calls: Vec<u32>,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            self.calls.push(n);
            self.value
        }
    }
    fn chance_fixture() -> (World, ActorId, Location) {
        let (mut w, a, l) = fixture();
        w.location_handlers.clear();
        for op in [1, 2] {
            w.location_handlers.insert(
                (l.id, op),
                Handler::PickableChance {
                    rule: rule(),
                    depletion: crate::pickables::Depletion {
                        numerator: 3,
                        denominator: 16,
                        delay: 10,
                    },
                },
            );
        }
        (w, a, l)
    }
    fn chance_pick(
        w: &mut World,
        a: ActorId,
        l: Location,
        random: &mut Draw,
    ) -> crate::pickables::ChanceTransition {
        w.submit(a, action(l, 2)).unwrap();
        for _ in 0..6 {
            w.tick_with_random(random).unwrap();
            if let Some(t) = w.drain_events().into_iter().find_map(|e| {
                if let WorldEvent::PickableDepletion { trace, .. } = e {
                    Some(trace)
                } else {
                    None
                }
            }) {
                return t;
            }
        }
        panic!("no chance harvest")
    }
    #[test]
    fn pickable_chance_uses_inclusive_three_of_sixteen_and_exact_shared_ten_tick_restore() {
        for draw in [0, 2, 3, 15] {
            let (mut w, a, l) = chance_fixture();
            let stats = w.data(a).unwrap().stats.clone();
            let mut rng = Draw {
                value: draw,
                calls: vec![],
            };
            let t = chance_pick(&mut w, a, l, &mut rng);
            assert_eq!(rng.calls, vec![16]);
            assert_eq!(t.roll, draw + 1);
            assert_eq!(t.depleted, draw < 3);
            assert_eq!(count(&w, a), 1);
            assert_eq!(w.data(a).unwrap().stats, stats);
            assert!(t.json(a).contains("\"denominator\":16"));
            if t.depleted {
                assert!(w.locations.find_shape(l.coords, l.shape).is_none());
                assert_eq!(t.transition.restore_at, Some(t.transition.tick + 10));
                for _ in 0..9 {
                    w.tick_with_random(&mut rng).unwrap();
                    assert!(w.locations.find_shape(l.coords, l.shape).is_none());
                }
                w.tick_with_random(&mut rng).unwrap();
                assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
                assert!(w.drain_events().iter().any(
                    |e| matches!(e,WorldEvent::LocationRestoration(t)if t.result=="restored")
                ));
            } else {
                assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
                assert!(t.transition.restore_at.is_none());
            }
            assert_eq!(count(&w, a), 1);
            assert_eq!(rng.calls, vec![16]);
        }
    }
    #[test]
    fn pickable_chance_refuses_full_member_invalid_deadline_or_dynamic_target_before_rng() {
        for mode in 0..4 {
            let (mut w, a, l) = chance_fixture();
            let mut r = rule();
            let mut depletion = crate::pickables::Depletion {
                numerator: 3,
                denominator: 16,
                delay: 10,
            };
            match mode {
                0 => {
                    for slot in 0..28 {
                        w.data_mut(a)
                            .unwrap()
                            .inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: 995,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                }
                1 => r.members = true,
                2 => depletion.denominator = 0,
                _ => {
                    w.locations = crate::locations::Locations::default();
                    w.locations.add(l).unwrap();
                }
            }
            let before = w.data(a).unwrap().inventory.clone();
            let rev = w.locations.revision(l);
            w.players
                .get_mut(a.slot as usize)
                .unwrap()
                .location_revision = rev;
            let mut rng = Draw {
                value: 0,
                calls: vec![],
            };
            w.process_pickable_chance(a.slot as usize, l, r, depletion, &mut rng);
            assert!(rng.calls.is_empty());
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.locations.revision(l), rev);
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::PickableDepletion { .. } | WorldEvent::PickableCollected { .. }
            )));
        }
        for d in [
            crate::pickables::Depletion {
                numerator: 0,
                denominator: 16,
                delay: 10,
            },
            crate::pickables::Depletion {
                numerator: 17,
                denominator: 16,
                delay: 10,
            },
            crate::pickables::Depletion {
                numerator: 3,
                denominator: 16,
                delay: 0,
            },
        ] {
            assert!(d.deadline(1).is_err());
        }
        assert!(crate::pickables::Depletion {
            numerator: 3,
            denominator: 16,
            delay: 10
        }
        .deadline(i32::MAX)
        .is_err());
    }
    #[test]
    fn pickable_deletion_timer_does_not_revive_replaced_or_newer_deleted_generation() {
        for replacement in [false, true] {
            let (mut w, a, l) = chance_fixture();
            let mut rng = Draw {
                value: 0,
                calls: vec![],
            };
            chance_pick(&mut w, a, l, &mut rng);
            w.change_location(l, true).unwrap();
            w.change_location(l, false).unwrap();
            let other = Location { id: 101, ..l };
            if replacement {
                w.change_location(other, true).unwrap();
            }
            for _ in 0..10 {
                w.tick_with_random(&mut rng).unwrap();
            }
            assert_eq!(
                w.locations.find_shape(l.coords, l.shape),
                replacement.then_some(other)
            );
            assert_eq!(count(&w, a), 1);
            assert!(w
                .drain_events()
                .iter()
                .any(|e| matches!(e,WorldEvent::LocationRestoration(t)if t.result=="stale")));
        }
    }
    fn nettle_fixture() -> (World, ActorId, Location) {
        let (mut w, a, l) = fixture();
        w.set_membership(a, true).unwrap();
        let mut r = rule();
        r.members = true;
        w.location_handlers.clear();
        for op in [1, 2] {
            w.location_handlers.insert(
                (l.id, op),
                Handler::Nettles {
                    rule: r,
                    excluded_glove: 25135,
                    mark: crate::hit_display::Mark {
                        target: 65,
                        source: 65,
                        public: Some(66),
                        delay: 0,
                    },
                },
            );
        }
        let data = w.data_mut(a).unwrap();
        data.stats.set_base(3, 10);
        data.stats.set_current(3, 10);
        (w, a, l)
    }
    #[test]
    fn nettle_sting_is_one_delayed_incidental_poison_mark_not_ongoing_poison_or_harvest() {
        for draw in [0, 1] {
            for glove in [None, Some(25135)] {
                let (mut w, a, l) = nettle_fixture();
                if let Some(id) = glove {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            9,
                            Some(Item {
                                id,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                for slot in 0..28 {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 995,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                let before = w.data(a).unwrap().inventory.clone();
                let stats = w.data(a).unwrap().stats.clone();
                let rev = w.locations.revision(l);
                let mut rng = Draw {
                    value: draw,
                    calls: vec![],
                };
                w.submit(a, action(l, 1)).unwrap();
                w.tick_with_random(&mut rng).unwrap();
                assert_eq!(w.data(a).unwrap().stats.current(3), 10);
                assert!(w.drain_events().iter().any(|e| matches!(
                    e,
                    WorldEvent::LocationMessage {
                        message: "The nettles sting you.",
                        ..
                    }
                )));
                w.tick_with_random(&mut rng).unwrap();
                assert_eq!(w.data(a).unwrap().stats.current(3), 9 - draw as i8);
                let impacts: Vec<_> = w
                    .drain_events()
                    .into_iter()
                    .filter_map(|e| {
                        if let WorldEvent::Hit { impact, .. } = e {
                            Some(impact)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(impacts.len(), 1);
                assert_eq!(impacts[0].hit.display.unwrap().target, 65);
                assert_eq!(impacts[0].hit.style, crate::hits::Style::Typeless);
                assert!(impacts[0].hit.source.is_none());
                for _ in 0..35 {
                    w.tick_with_random(&mut rng).unwrap();
                }
                assert_eq!(rng.calls, vec![2]);
                assert_eq!(w.data(a).unwrap().stats.current(3), 9 - draw as i8);
                assert_eq!(w.data(a).unwrap().inventory, before);
                assert_eq!(w.locations.revision(l), rev);
                for skill in 0..24 {
                    assert_eq!(
                        w.data(a).unwrap().stats.fine_xp(skill),
                        stats.fine_xp(skill)
                    );
                }
            }
        }
    }
    #[test]
    fn nettle_real_gloves_harvest_without_rng_then_restore_after13_and_full_bag_or_membership_refuses(
    ) {
        for mode in 0..3 {
            let (mut w, a, l) = nettle_fixture();
            w.data_mut(a)
                .unwrap()
                .equipment
                .restore_slot(
                    9,
                    Some(Item {
                        id: 1059,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            if mode == 1 {
                for slot in 0..28 {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 995,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
            }
            if mode == 2 {
                w.set_membership(a, false).unwrap();
            }
            let mut rng = Draw {
                value: 0,
                calls: vec![],
            };
            w.submit(a, action(l, 2)).unwrap();
            w.tick_with_random(&mut rng).unwrap();
            assert!(rng.calls.is_empty());
            assert_eq!(w.data(a).unwrap().stats.current(3), 10);
            if mode == 0 {
                assert_eq!(count(&w, a), 1);
                assert!(w.locations.find_shape(l.coords, l.shape).is_none());
                for _ in 0..12 {
                    w.tick_with_random(&mut rng).unwrap();
                    assert!(w.locations.find_shape(l.coords, l.shape).is_none());
                }
                w.tick_with_random(&mut rng).unwrap();
                assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
            } else {
                assert_eq!(count(&w, a), 0);
                assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
            }
        }
    }
}
