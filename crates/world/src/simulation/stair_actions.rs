//! Source straight-flight landings through the existing authoritative telejump.
use super::*;
use crate::{location_content::Stairs, locations::Location};
impl World {
    pub(super) fn process_straight_stairs(
        &mut self,
        slot: usize,
        location: Location,
        rule: Stairs,
    ) {
        let p = self.players.get(slot).unwrap();
        let Some(revision) = self.locations.revision(location) else {
            return;
        };
        if p.saving
            || p.pending_logout
            || p.has_modal
            || p.data.stats.current(3) <= 0
            || p.location_revision != Some(revision)
        {
            return;
        }
        let Some(ty) = self.location_types.get(&location.id) else {
            self.events
                .push(WorldEvent::LocationError("Unknown staircase type"));
            return;
        };
        let destination = match rule.destination(location, ty.width, ty.length) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        let origin = p.movement.coords;
        self.finish_location_climb(slot, location, origin, destination);
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
            panic!("straight staircase must not roll")
        }
    }
    #[test]
    fn straight_stair_rotations_match_recorded_native_footprints_including_narrow_reverse_flights()
    {
        for (dx, dz, dl, w, l, expected) in [
            (1, 3, 1, 2, 3, [(1, 3), (3, 0), (0, -1), (-1, 1)]),
            (1, -2, -1, 2, 2, [(1, -2), (-2, 0), (0, 3), (3, 1)]),
            (0, -1, 1, 1, 3, [(0, -1), (-1, 0), (0, 3), (3, 0)]),
            (0, 3, -1, 1, 2, [(0, 3), (3, 0), (0, -2), (-2, 0)]),
        ] {
            for angle in 0..4 {
                let node =
                    Location::new(CoordGrid::new(3204, 3204, 1).unwrap(), 100, 10, angle).unwrap();
                let dest = Stairs {
                    dx,
                    dz,
                    delta_level: dl,
                }
                .destination(node, w, l)
                .unwrap();
                let offset = expected[angle as usize];
                assert_eq!(
                    (dest.x(), dest.z(), dest.level()),
                    (3204 + offset.0, 3204 + offset.1, 1 + dl)
                );
            }
        }
    }
    #[test]
    fn straight_stair_real_reached_dispatch_jumps_immediately_from_location_anchor_without_animation_items_or_xp(
    ) {
        for (angle, dx, dz) in [(0, 1, 3), (1, 3, 0), (2, 0, -1), (3, -1, 1)] {
            let mut w = World::default();
            for level in 0..4 {
                for x in (3192..3224).step_by(8) {
                    for z in (3192..3224).step_by(8) {
                        w.collision.allocate(x, z, level);
                    }
                }
            }
            let l = Location::new(CoordGrid::new(3204, 3204, 1).unwrap(), 100, 10, angle).unwrap();
            w.locations.load(l);
            w.location_types.insert(
                100,
                LocCollision {
                    width: 2,
                    length: 3,
                    block_walk: 2,
                    block_range: true,
                    break_route_finding: false,
                },
            );
            w.location_handlers.insert(
                (100, 1),
                Handler::Stairs(Stairs {
                    dx: 1,
                    dz: 3,
                    delta_level: 1,
                }),
            );
            w.queue_login(
                1,
                PlayerData::new(
                    1,
                    "Stairs".into(),
                    CoordGrid::new(3203, 3204, 1).unwrap(),
                    Stats::default(),
                ),
            )
            .unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            let inventory = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let revision = w.locations.revision(l);
            w.drain_events();
            w.submit(
                a,
                Action::OperateLocation {
                    coords: l.coords,
                    id: 100,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(
                w.data(a).unwrap().coords,
                CoordGrid::new(3204 + dx, 3204 + dz, 2).unwrap()
            );
            let e = w.drain_events();
            assert_eq!(
                e.iter()
                    .filter(|e| matches!(e, WorldEvent::LocationClimbed { .. }))
                    .count(),
                1
            );
            assert!(!e.iter().any(|e| matches!(
                e,
                WorldEvent::PlayerAnimation { .. } | WorldEvent::PlayerSound { .. }
            )));
            assert_eq!(w.data(a).unwrap().inventory, inventory);
            assert_eq!(w.data(a).unwrap().stats, stats);
            assert_eq!(w.locations.revision(l), revision);
        }
    }
    #[test]
    fn straight_stair_landing_refuses_invalid_level_world_edge_or_zero_geometry() {
        for (coords, rule) in [
            (
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stairs {
                    dx: 1,
                    dz: -2,
                    delta_level: -1,
                },
            ),
            (
                CoordGrid::new(16383, 3200, 0).unwrap(),
                Stairs {
                    dx: 1,
                    dz: 3,
                    delta_level: 1,
                },
            ),
        ] {
            let l = Location::new(coords, 100, 10, 0).unwrap();
            assert!(rule.destination(l, 2, 3).is_err());
        }
        let l = Location::new(CoordGrid::new(3200, 3200, 1).unwrap(), 100, 10, 0).unwrap();
        assert!(Stairs {
            dx: 1,
            dz: 3,
            delta_level: 1
        }
        .destination(l, 0, 3)
        .is_err());
    }
    #[test]
    fn spiral_landings_use_adjusted_footprints_and_direct_two_floor_translation() {
        use crate::location_content::Spiral;
        for (width, length, dl, expected) in [
            (2, 2, 1, [(2, 0), (0, -1), (-1, 1), (1, 2)]),
            (2, 2, -1, [(1, -1), (-2, 0), (0, 2), (2, 1)]),
            (1, 1, -2, [(0, -1), (-1, 0), (0, 1), (1, 0)]),
            (2, 3, 1, [(2, 0), (0, -1), (-1, 2), (2, 2)]),
            (2, 3, -1, [(1, -1), (-3, 0), (0, 3), (3, 1)]),
        ] {
            for angle in 0..4 {
                let l =
                    Location::new(CoordGrid::new(3204, 3204, 2).unwrap(), 100, 10, angle).unwrap();
                let d = Spiral {
                    delta_level: dl,
                    arrival_delay: false,
                }
                .destination(l, width, length)
                .unwrap();
                let e = expected[angle as usize];
                assert_eq!((d.x(), d.z(), d.level()), (3204 + e.0, 3204 + e.1, 2 + dl));
            }
        }
    }
    #[test]
    fn spiral_direct_operations_reach_and_jump_without_animation_or_inventory_changes() {
        use crate::location_content::Spiral;
        for (dl, width, expected) in [
            (1, 2, [(2, 0), (0, -1), (-1, 1), (1, 2)]),
            (-1, 1, [(0, -1), (-1, 0), (0, 1), (1, 0)]),
            (2, 2, [(2, 0), (0, -1), (-1, 1), (1, 2)]),
            (-2, 1, [(0, -1), (-1, 0), (0, 1), (1, 0)]),
        ] {
            for angle in 0..4 {
                let mut w = World::default();
                for level in 0..4 {
                    for x in (3192..3224).step_by(8) {
                        for z in (3192..3224).step_by(8) {
                            w.collision.allocate(x, z, level);
                        }
                    }
                }
                let level = if dl > 0 { 0 } else { 2 };
                let l = Location::new(CoordGrid::new(3204, 3204, level).unwrap(), 100, 10, angle)
                    .unwrap();
                w.locations.load(l);
                w.location_types.insert(
                    100,
                    LocCollision {
                        width,
                        length: width,
                        block_walk: 2,
                        block_range: true,
                        break_route_finding: false,
                    },
                );
                w.location_handlers.insert(
                    (100, 2),
                    Handler::Spiral(Spiral {
                        delta_level: dl,
                        arrival_delay: false,
                    }),
                );
                w.queue_login(
                    1,
                    PlayerData::new(
                        1,
                        "Spiral".into(),
                        CoordGrid::new(3203, 3204, level).unwrap(),
                        Stats::default(),
                    ),
                )
                .unwrap();
                w.tick().unwrap();
                let a = w.actors().next().unwrap();
                let inv = w.data(a).unwrap().inventory.clone();
                let stats = w.data(a).unwrap().stats.clone();
                w.drain_events();
                w.submit(
                    a,
                    Action::OperateLocation {
                        coords: l.coords,
                        id: 100,
                        operation: 2,
                        control: false,
                    },
                )
                .unwrap();
                w.tick_with_random(&mut Never).unwrap();
                let e = expected[angle as usize];
                assert_eq!(
                    w.data(a).unwrap().coords,
                    CoordGrid::new(3204 + e.0, 3204 + e.1, level + dl).unwrap()
                );
                assert!(!w.drain_events().iter().any(|e| matches!(
                    e,
                    WorldEvent::PlayerAnimation { .. } | WorldEvent::PlayerSound { .. }
                )));
                assert_eq!(w.data(a).unwrap().inventory, inv);
                assert_eq!(w.data(a).unwrap().stats, stats);
            }
        }
    }
}
impl World {
    pub(super) fn process_spiral_stairs(
        &mut self,
        slot: usize,
        location: Location,
        rule: crate::location_content::Spiral,
    ) {
        let p = self.players.get(slot).unwrap();
        let Some(revision) = self.locations.revision(location) else {
            return;
        };
        if p.saving
            || p.pending_logout
            || p.has_modal
            || p.data.stats.current(3) <= 0
            || p.location_revision != Some(revision)
        {
            return;
        }
        let Some(ty) = self.location_types.get(&location.id) else {
            self.events
                .push(WorldEvent::LocationError("Unknown spiral staircase type"));
            return;
        };
        let destination = match rule.destination(location, ty.width, ty.length) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        let origin = p.movement.coords;
        self.finish_location_climb(slot, location, origin, destination);
    }
}
