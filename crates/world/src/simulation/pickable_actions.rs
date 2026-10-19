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
}
