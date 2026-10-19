//! Delayed bookcase text uses the existing interruptible player queue.
use super::*;
use crate::locations::Location;
pub(super) const QUEUE: i32 = -2391062;
pub(super) const REPLIES: [&str; 3] = [
    "None of them look very interesting.",
    "You don't find anything that you'd ever want to read.",
    "You find nothing to interest you.",
];
#[derive(Clone, Copy)]
pub(super) struct Pending {
    pub target: Location,
    revision: u64,
    origin: CoordGrid,
    last_movement: i32,
}
impl World {
    pub(super) fn start_bookcase(&mut self, slot: usize, location: Location) {
        let Some(revision) = self.locations.revision(location) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if p.location_revision != Some(revision) {
            return;
        }
        p.queues.clear_weak();
        let pending = Pending {
            target: location,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        };
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                2,
                Deferred::Bookcase(pending),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::LocationSpam {
            actor: p.id,
            message: "You search the books...",
        });
    }
}
pub(super) fn complete(
    p: &mut Player,
    job: Pending,
    revision: Option<u64>,
    events: &mut Vec<WorldEvent>,
    random: &mut impl RandomSource,
) {
    if p.saving
        || p.pending_logout
        || p.has_modal
        || p.data.stats.current(3) <= 0
        || p.location_interaction.is_some()
        || p.npc_interaction.is_some()
        || p.object_interaction.is_some()
        || p.movement.coords != job.origin
        || p.movement.last_movement != job.last_movement
        || revision != Some(job.revision)
    {
        return;
    }
    let draw = random.below(3);
    let Some(&message) = REPLIES.get(draw as usize) else {
        events.push(WorldEvent::LocationError("Invalid bookcase random draw"));
        return;
    };
    events.push(WorldEvent::LocationMessage {
        actor: p.id,
        message,
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{loc_collision::LocCollision, location_content::Handler, stats::Stats};
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
    fn fixture() -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let l = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 380, 10, 0).unwrap();
        w.locations.load(l);
        w.location_types.insert(
            380,
            LocCollision {
                width: 1,
                length: 2,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
        );
        w.location_handlers.insert((380, 1), Handler::Bookcase);
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Books".into(),
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
    fn search(w: &mut World, a: ActorId, l: Location) {
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
    fn bookcase_real_dispatch_spam_then_exact_two_tick_single_random_reply_without_items_xp_or_stock(
    ) {
        for draw in 0..3 {
            let (mut w, a, l) = fixture();
            let before = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let rev = w.locations.revision(l);
            let mut rng = Draw {
                value: draw,
                calls: vec![],
            };
            search(&mut w, a, l);
            w.tick_with_random(&mut rng).unwrap();
            let events = w.drain_events();
            assert!(events.iter().any(|e| matches!(
                e,
                WorldEvent::LocationSpam {
                    message: "You search the books...",
                    ..
                }
            )));
            assert!(rng.calls.is_empty());
            assert!(!events
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationMessage { .. })));
            w.tick_with_random(&mut rng).unwrap();
            assert!(rng.calls.is_empty());
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationMessage { .. })));
            w.tick_with_random(&mut rng).unwrap();
            assert_eq!(rng.calls, vec![3]);
            let e = w.drain_events();
            assert_eq!(e.iter().filter(|e|matches!(e,WorldEvent::LocationMessage{message,..}if *message==REPLIES[draw as usize])).count(),1);
            for _ in 0..4 {
                w.tick_with_random(&mut rng).unwrap();
            }
            assert_eq!(rng.calls, vec![3]);
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.data(a).unwrap().stats, stats);
            assert_eq!(w.locations.revision(l), rev);
        }
    }
    #[test]
    fn bookcase_replaced_generation_or_held_action_cancels_before_reply_rng() {
        for mode in 0..2 {
            let (mut w, a, l) = fixture();
            let mut rng = Draw {
                value: 0,
                calls: vec![],
            };
            search(&mut w, a, l);
            w.tick_with_random(&mut rng).unwrap();
            w.drain_events();
            if mode == 0 {
                w.change_location(l, false).unwrap();
                w.change_location(l, true).unwrap();
            } else {
                w.submit(a, Action::EatItemSlot { slot: 0, item: 999 })
                    .unwrap();
            }
            for _ in 0..4 {
                w.tick_with_random(&mut rng).unwrap();
            }
            assert!(rng.calls.is_empty());
            assert!(!w.drain_events().iter().any(
                |e| matches!(e,WorldEvent::LocationMessage{message,..}if REPLIES.contains(message))
            ));
        }
    }
}
