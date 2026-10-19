//! Ranged signpost reads use the shared bounds route and leave presentation to the host.
use super::*;
use crate::locations::Location;
impl World {
    pub(super) fn process_signpost(&mut self, slot: usize, location: Location) {
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
        let origin = p.movement.coords;
        if origin.translate(0, -6, 0).is_err() || origin.translate(0, 3, 0).is_err() {
            self.events.push(WorldEvent::LocationError(
                "Signpost camera target outside world",
            ));
            return;
        }
        self.events.push(WorldEvent::SignpostRead {
            actor: p.id,
            location,
            origin,
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
            panic!("signpost must not roll")
        }
    }
    fn fixture(x: i32) -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3184..3232).step_by(8) {
            for z in (3184..3232).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let l = Location::new(CoordGrid::new(3204, 3204, 0).unwrap(), 18493, 10, 0).unwrap();
        w.locations.load(l);
        w.location_types.insert(
            l.id,
            LocCollision {
                width: 2,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
        );
        w.location_handlers.insert((l.id, 1), Handler::Signpost);
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Reader".into(),
                CoordGrid::new(x, 3204, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a, l)
    }
    #[test]
    fn signpost_approach_reads_at_five_without_walking_to_edge_or_rewards() {
        for start in [3198, 3199] {
            let (mut w, a, l) = fixture(start);
            let inv = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
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
            let mut reads = vec![];
            for _ in 0..4 {
                w.tick_with_random(&mut Never).unwrap();
                reads.extend(w.drain_events().into_iter().filter_map(|e| {
                    if let WorldEvent::SignpostRead { origin, .. } = e {
                        Some(origin)
                    } else {
                        None
                    }
                }));
            }
            assert_eq!(reads, [CoordGrid::new(3199, 3204, 0).unwrap()]);
            assert_eq!(w.data(a).unwrap().coords, reads[0]);
            assert_eq!(w.data(a).unwrap().inventory, inv);
            assert_eq!(w.data(a).unwrap().stats, stats);
        }
    }
    #[test]
    fn signpost_bounds_distance_uses_rotated_extent_size_plane_and_nonnegative_range() {
        let c = CoordGrid::new(3204, 3204, 0).unwrap();
        let r = crate::reach::Rectangle {
            coords: c,
            width: 1,
            length: 2,
            angle: 1,
            blocked: 15,
            exclusive: true,
        };
        assert!(r.within_distance(CoordGrid::new(3210, 3204, 0).unwrap(), 1, 5));
        assert!(!r.within_distance(CoordGrid::new(3211, 3204, 0).unwrap(), 1, 5));
        assert!(r.within_distance(CoordGrid::new(3198, 3204, 0).unwrap(), 2, 5));
        assert!(!r.within_distance(CoordGrid::new(3204, 3204, 1).unwrap(), 1, 5));
        assert!(!r.within_distance(c, 1, -1));
    }
}
