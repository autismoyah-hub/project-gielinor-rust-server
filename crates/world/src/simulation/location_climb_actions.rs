//! Single-direction level and dungeon links through the shared protected continuation.
use super::*;
use crate::{
    location_content::{Climb, ClimbTransition},
    locations::Location,
};
impl World {
    pub(super) fn start_location_climb(&mut self, slot: usize, location: Location, rule: Climb) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        let Some(revision) = self
            .locations
            .revision(location)
            .filter(|v| p.location_revision == Some(*v))
        else {
            return;
        };
        let dest = p
            .movement
            .coords
            .translate(0, rule.delta_z, rule.delta_level);
        if rule.delay <= 0
            || self.clock.0 < 0
            || self.clock.0.checked_add(rule.delay).is_none()
            || (rule.delta_z == 0 && rule.delta_level == 0)
            || dest.is_err()
        {
            self.events.push(WorldEvent::LocationError(
                "Invalid climb destination or delay",
            ));
            return;
        }
        let destination = dest.unwrap();
        let p = self.players.get_mut(slot).unwrap();
        p.pending_location_action = Some(location_swap_actions::Pending {
            location,
            revision,
            action: location_swap_actions::Continuation::Climb(destination),
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        });
        p.protection.processed_clock = self.clock.0;
        p.protection.delay(rule.delay);
        p.protection.suspended = true;
        self.events.push(WorldEvent::PlayerAnimation {
            actor,
            sequence: rule.animation,
        });
    }
    pub(super) fn finish_location_climb(
        &mut self,
        slot: usize,
        location: Location,
        from: CoordGrid,
        to: CoordGrid,
    ) {
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        if p.movement
            .teleport(&mut self.collision, to, self.clock.0, true)
            .is_err()
        {
            self.events
                .push(WorldEvent::LocationError("Climb telejump failed"));
            return;
        }
        p.data.coords = to;
        self.events.push(WorldEvent::LocationClimbed {
            actor,
            trace: ClimbTransition {
                tick: self.clock.0,
                location,
                from,
                to,
            },
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        loc_collision::LocCollision, location_content::Handler, npc::RandomSource, stats::Stats,
    };
    struct Never;
    impl RandomSource for Never {
        fn below(&mut self, _: u32) -> u32 {
            panic!("ladder climb must not roll")
        }
    }
    fn fixture(level: i32, z: i32, rule: Climb) -> (World, ActorId, Location) {
        let mut w = World::default();
        for height in 0..4 {
            for x in (3192..3224).step_by(8) {
                for zz in [(z / 8) * 8, 3200, 9600] {
                    w.collision.allocate(x, zz, height);
                }
            }
        }
        let l = Location::new(CoordGrid::new(3201, z, level).unwrap(), 100, 10, 0).unwrap();
        w.locations.load(l);
        w.location_types.insert(
            100,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
        );
        w.location_handlers.insert((100, 1), Handler::Climb(rule));
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Climb".into(),
                CoordGrid::new(3200, z, level).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a, l)
    }
    fn submit(w: &mut World, a: ActorId, l: Location) {
        w.submit(
            a,
            Action::OperateLocation {
                coords: l.coords,
                id: l.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
    }
    #[test]
    fn climbs_animate_then_after_one_tick_telejump_from_actor_coordinates_without_item_xp_or_node_changes(
    ) {
        for (level, z, dlevel, dz) in [
            (0, 3200, 1, 0),
            (1, 3200, -1, 0),
            (0, 3200, 0, 6400),
            (0, 9600, 0, -6400),
        ] {
            let rule = Climb {
                delta_level: dlevel,
                delta_z: dz,
                animation: if dlevel < 0 || dz > 0 { 827 } else { 828 },
                delay: 1,
            };
            let (mut w, a, l) = fixture(level, z, rule);
            let origin = w.data(a).unwrap().coords;
            let dest = origin.translate(0, dz, dlevel).unwrap();
            let inv = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let revision = w.locations.revision(l);
            submit(&mut w, a, l);
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(w.data(a).unwrap().coords, origin);
            let e = w.drain_events();
            assert!(e.iter().any(
                |e| matches!(e,WorldEvent::PlayerAnimation{sequence,..}if *sequence==rule.animation)
            ));
            assert!(!e
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(w.data(a).unwrap().coords, dest);
            let e = w.drain_events();
            let t = e
                .iter()
                .find_map(|e| {
                    if let WorldEvent::LocationClimbed { trace, .. } = e {
                        Some(*trace)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!((t.from, t.to, t.location), (origin, dest, l));
            assert!(t.json(a).contains("\"to\":"));
            assert_eq!(w.data(a).unwrap().inventory, inv);
            assert_eq!(w.data(a).unwrap().stats, stats);
            assert_eq!(w.locations.revision(l), revision);
        }
    }
    #[test]
    fn climb_refuses_world_boundaries_and_cancels_stale_or_held_warmups_without_late_teleport() {
        for (level, z, dlevel, dz) in [
            (0, 3200, -1, 0),
            (3, 3200, 1, 0),
            (0, 1000, 0, -6400),
            (0, 12000, 0, 6400),
        ] {
            let (mut w, a, l) = fixture(
                level,
                z,
                Climb {
                    delta_level: dlevel,
                    delta_z: dz,
                    animation: 828,
                    delay: 1,
                },
            );
            let origin = w.data(a).unwrap().coords;
            submit(&mut w, a, l);
            for _ in 0..3 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert_eq!(w.data(a).unwrap().coords, origin);
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::PlayerAnimation { .. } | WorldEvent::LocationClimbed { .. }
            )));
        }
        for held in [false, true] {
            let (mut w, a, l) = fixture(
                0,
                3200,
                Climb {
                    delta_level: 1,
                    delta_z: 0,
                    animation: 828,
                    delay: 1,
                },
            );
            let origin = w.data(a).unwrap().coords;
            submit(&mut w, a, l);
            w.tick_with_random(&mut Never).unwrap();
            w.drain_events();
            if held {
                w.submit(a, Action::EatItemSlot { slot: 0, item: 999 })
                    .unwrap();
            } else {
                w.change_location(l, false).unwrap();
                w.change_location(l, true).unwrap();
            }
            for _ in 0..3 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert_eq!(w.data(a).unwrap().coords, origin);
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationClimbed { .. })));
        }
    }
}
