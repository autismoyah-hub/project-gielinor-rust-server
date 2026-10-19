//! Scenery crossings commit authoritative occupancy and emit native exact movement.
use super::*;
use crate::{location_content::Crossing, locations::Location};
impl World {
    pub(super) fn process_fence_crossing(
        &mut self,
        slot: usize,
        location: Location,
        rule: Crossing,
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
            return;
        };
        let from = p.movement.coords;
        let to = match rule.destination(location, ty.width, ty.length, from) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        let (dx, dz) = (from.x() - to.x(), from.z() - to.z());
        let (Ok(dx), Ok(dz)) = (i8::try_from(dx), i8::try_from(dz)) else {
            self.events.push(WorldEvent::LocationError(
                "Crossing relative offset outside native range",
            ));
            return;
        };
        let direction = if to.z() > from.z() {
            1024
        } else if to.z() < from.z() {
            0
        } else if to.x() > from.x() {
            1536
        } else {
            512
        };
        if self.clock.0 < 0 || self.clock.0.checked_add(rule.protection_ticks()).is_none() {
            return;
        }
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        if p.movement
            .teleport(&mut self.collision, to, self.clock.0, true)
            .is_err()
        {
            self.events
                .push(WorldEvent::LocationError("Crossing exact move failed"));
            return;
        }
        p.data.coords = to;
        if rule.protection_ticks() > 0 {
            p.protection.processed_clock = self.clock.0;
            p.protection.delay(rule.protection_ticks());
        }
        self.events.push(WorldEvent::PlayerExactMove {
            actor,
            from,
            to,
            offsets: [dx, dz, 0, 0],
            delays: [30, 94],
            direction,
            sequence: 839,
            sequence_delay: 30,
        });
        self.events.push(WorldEvent::LocationClimbed {
            actor,
            trace: crate::location_content::ClimbTransition {
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
            panic!("crossing must not roll")
        }
    }
    #[test]
    fn exact_crossings_reached_both_sides_all_angles_emit_relative_animation_and_keep_items_xp_and_models(
    ) {
        for (rule, width, length) in [(Crossing::Stile, 1, 2), (Crossing::Fence, 1, 1)] {
            for angle in 0..4 {
                for reverse in [false, true] {
                    let mut w = World::default();
                    for x in (3192..3224).step_by(8) {
                        for z in (3192..3224).step_by(8) {
                            w.collision.allocate(x, z, 0);
                        }
                    }
                    let l = Location::new(CoordGrid::new(3204, 3204, 0).unwrap(), 100, 10, angle)
                        .unwrap();
                    let (from, to) = match rule {
                        Crossing::Stile => {
                            let far = if angle % 2 == 0 {
                                l.coords.translate(0, 1, 0).unwrap()
                            } else {
                                l.coords.translate(1, 0, 0).unwrap()
                            };
                            if reverse {
                                (far, l.coords)
                            } else {
                                (l.coords, far)
                            }
                        }
                        Crossing::Fence => {
                            let step = if reverse { 1 } else { -1 };
                            let (dx, dz) = if angle % 2 == 0 { (step, 0) } else { (0, step) };
                            (
                                l.coords.translate(dx, dz, 0).unwrap(),
                                l.coords.translate(-dx, -dz, 0).unwrap(),
                            )
                        }
                    };
                    w.locations.load(l);
                    w.location_types.insert(
                        100,
                        LocCollision {
                            width,
                            length,
                            block_walk: 2,
                            block_range: true,
                            break_route_finding: false,
                        },
                    );
                    w.location_handlers
                        .insert((100, 1), Handler::Crossing(rule));
                    w.queue_login(
                        1,
                        PlayerData::new(1, "Crossing".into(), from, Stats::default()),
                    )
                    .unwrap();
                    w.tick().unwrap();
                    let a = w.actors().next().unwrap();
                    let inv = w.data(a).unwrap().inventory.clone();
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
                    assert_eq!(w.data(a).unwrap().coords, to);
                    let e = w.drain_events();
                    let exact = e
                        .iter()
                        .find_map(|e| {
                            if let WorldEvent::PlayerExactMove {
                                from,
                                to,
                                offsets,
                                delays,
                                direction,
                                sequence,
                                sequence_delay,
                                ..
                            } = e
                            {
                                Some((
                                    *from,
                                    *to,
                                    *offsets,
                                    *delays,
                                    *direction,
                                    *sequence,
                                    *sequence_delay,
                                ))
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    assert_eq!(exact.0, from);
                    assert_eq!(exact.1, to);
                    assert_eq!(
                        exact.2,
                        [
                            i8::try_from(from.x() - to.x()).unwrap(),
                            i8::try_from(from.z() - to.z()).unwrap(),
                            0,
                            0
                        ]
                    );
                    assert_eq!(exact.3, [30, 94]);
                    assert_eq!(
                        exact.4,
                        if to.z() > from.z() {
                            1024
                        } else if to.z() < from.z() {
                            0
                        } else if to.x() > from.x() {
                            1536
                        } else {
                            512
                        }
                    );
                    assert_eq!((exact.5, exact.6), (839, 30));
                    assert_eq!(w.protection(a).unwrap().delayed(), rule == Crossing::Fence);
                    for _ in 0..3 {
                        w.tick_with_random(&mut Never).unwrap();
                    }
                    assert!(!w.protection(a).unwrap().delayed());
                    assert_eq!(w.data(a).unwrap().inventory, inv);
                    assert_eq!(w.data(a).unwrap().stats, stats);
                    assert_eq!(w.locations.revision(l), revision);
                }
            }
        }
    }
    #[test]
    fn exact_crossing_invalid_plane_geometry_edges_or_stale_revision_refuse_before_movement() {
        let l = Location::new(CoordGrid::new(16383, 3204, 0).unwrap(), 100, 10, 0).unwrap();
        assert!(Crossing::Fence.destination(l, 1, 1, l.coords).is_err());
        assert!(Crossing::Stile.destination(l, 0, 2, l.coords).is_err());
        assert!(Crossing::Fence
            .destination(l, 1, 1, CoordGrid::new(16383, 3204, 1).unwrap())
            .is_err());
        let mut w = World::default();
        let at = CoordGrid::new(3204, 3204, 0).unwrap();
        let l = Location::new(at, 100, 10, 0).unwrap();
        w.locations.load(l);
        w.location_types.insert(
            100,
            LocCollision {
                width: 1,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
        );
        w.queue_login(1, PlayerData::new(1, "Stale".into(), at, Stats::default()))
            .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.players
            .get_mut(a.slot as usize)
            .unwrap()
            .location_revision = Some(u64::MAX);
        w.drain_events();
        w.process_fence_crossing(a.slot as usize, l, Crossing::Stile);
        assert_eq!(w.data(a).unwrap().coords, at);
        assert!(w.drain_events().is_empty());
    }
}
