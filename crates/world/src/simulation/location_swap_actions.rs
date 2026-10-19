//! Plain location toggles share authoritative mutation and restoration queues.
use super::*;
use crate::{
    location_content::{Swap, SwapTransition},
    locations::{Changed, Location},
};
impl World {
    pub(super) fn process_location_swap(&mut self, slot: usize, current: Location, rule: Swap) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if self.locations.revision(current) != p.location_revision {
            return;
        }
        let Some(root) = self.locations.map_origin(current) else {
            self.events.push(WorldEvent::LocationError(
                "Location switch requires an underlying cache node",
            ));
            return;
        };
        if root.id != current.id && root.id != rule.next {
            self.events.push(WorldEvent::LocationError(
                "Location switch cache root is outside its pair",
            ));
            return;
        }
        let deadline = match self.clock.0.checked_add(rule.duration) {
            Some(t) if self.clock.0 >= 0 && rule.duration > 0 && rule.next != current.id => t,
            _ => {
                self.events.push(WorldEvent::LocationError(
                    "Invalid location switch deadline or next stage",
                ));
                return;
            }
        };
        if !self.location_types.contains_key(&rule.next)
            || !self.location_types.contains_key(&root.id)
        {
            self.events
                .push(WorldEvent::LocationError("Unknown location switch type"));
            return;
        }
        let next = match Location::new(current.coords, rule.next, current.shape, current.angle) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        let changed = match self.change_location(next, true) {
            Ok(v) => v,
            Err(e) => {
                self.events.push(WorldEvent::LocationError(e));
                return;
            }
        };
        let restoration = if changed == Changed::Spawned {
            let rev = self
                .locations
                .revision(next)
                .expect("committed location switch");
            self.location_timers
                .replace(root, next, rev, self.clock, rule.duration);
            Some((root, deadline))
        } else {
            None
        };
        if let Some(sequence) = rule.animation {
            self.events
                .push(WorldEvent::PlayerAnimation { actor, sequence });
        }
        if let Some(sound) = rule.sound {
            self.events.push(WorldEvent::PlayerSound { actor, sound });
        }
        self.events.push(WorldEvent::LocationSwitched {
            actor,
            trace: SwapTransition {
                tick: self.clock.0,
                from: current,
                to: next,
                restore: restoration.map(|v| v.0),
                restore_at: restoration.map(|v| v.1),
            },
        });
    }
}
/// Actor-owned continuation; the existing protection clock supplies its delay.
#[derive(Clone, Copy)]
pub(super) struct Pending {
    location: Location,
    revision: u64,
    rule: Swap,
    origin: CoordGrid,
    last_movement: i32,
}
impl World {
    pub(super) fn start_delayed_location_swap(
        &mut self,
        slot: usize,
        current: Location,
        rule: Swap,
        delay: i32,
    ) {
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        let Some(revision) = self
            .locations
            .revision(current)
            .filter(|v| p.location_revision == Some(*v))
        else {
            return;
        };
        let root = self.locations.map_origin(current);
        if delay <= 0
            || rule.duration <= 0
            || rule.next == current.id
            || self.clock.0 < 0
            || self
                .clock
                .0
                .checked_add(delay)
                .and_then(|v| v.checked_add(rule.duration))
                .is_none()
            || root.is_none_or(|r| {
                (r.id != current.id && r.id != rule.next)
                    || !self.location_types.contains_key(&r.id)
            })
            || !self.location_types.contains_key(&rule.next)
            || Location::new(current.coords, rule.next, current.shape, current.angle).is_err()
        {
            self.events
                .push(WorldEvent::LocationError("Invalid delayed location switch"));
            return;
        }
        let p = self.players.get_mut(slot).unwrap();
        p.pending_location_action = Some(Pending {
            location: current,
            revision,
            rule,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        });
        p.protection.processed_clock = self.clock.0;
        p.protection.delay(delay);
        p.protection.suspended = true;
        if let Some(sequence) = rule.animation {
            self.events
                .push(WorldEvent::PlayerAnimation { actor, sequence });
        }
        if let Some(sound) = rule.sound {
            self.events.push(WorldEvent::PlayerSound { actor, sound });
        }
    }
    pub(super) fn resume_location_action(&mut self, slot: usize) {
        let pending = {
            let Some(p) = self.players.get_mut(slot) else {
                return;
            };
            if p.pending_location_action.is_none() {
                return;
            }
            p.protection.processed_clock = self.clock.0;
            if p.saving || p.pending_logout || p.has_modal || p.data.stats.current(3) <= 0 {
                p.pending_location_action = None;
                p.protection.suspended = false;
                return;
            }
            if p.protection.delayed() {
                return;
            }
            let pending = p.pending_location_action.take().unwrap();
            p.protection.suspended = false;
            if p.location_interaction.is_some()
                || p.npc_interaction.is_some()
                || p.object_interaction.is_some()
                || p.movement.coords != pending.origin
                || p.movement.last_movement != pending.last_movement
            {
                return;
            }
            pending
        };
        if self.locations.revision(pending.location) != Some(pending.revision) {
            return;
        }
        let mut rule = pending.rule;
        rule.animation = None;
        rule.sound = None;
        self.process_location_swap(slot, pending.location, rule);
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
            panic!("plain location switch must not roll")
        }
    }
    fn rule(next: u32, sound: Option<u16>) -> Swap {
        Swap {
            next,
            animation: Some(536),
            sound,
            duration: 5,
            arrival_delay: true,
        }
    }
    fn fixture(root_id: u32) -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        for id in [375, 378] {
            w.location_types.insert(
                id,
                LocCollision {
                    width: 1,
                    length: 1,
                    block_walk: 2,
                    block_range: true,
                    break_route_finding: false,
                },
            );
        }
        let l = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), root_id, 10, 0).unwrap();
        w.locations.load(l);
        w.location_handlers
            .insert((375, 1), Handler::TimedSwap(rule(378, Some(52))));
        w.location_handlers
            .insert((378, 2), Handler::Message("You find nothing of interest."));
        w.location_handlers
            .insert((378, 3), Handler::TimedSwap(rule(375, None)));
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Chests".into(),
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
    fn toggle(w: &mut World, a: ActorId, l: Location, operation: u8) -> SwapTransition {
        w.submit(
            a,
            Action::OperateLocation {
                coords: l.coords,
                id: l.id,
                operation,
                control: false,
            },
        )
        .unwrap();
        for _ in 0..6 {
            w.tick_with_random(&mut Never).unwrap();
            if let Some(t) = w.drain_events().into_iter().find_map(|e| {
                if let WorldEvent::LocationSwitched { trace, .. } = e {
                    Some(trace)
                } else {
                    None
                }
            }) {
                return t;
            }
        }
        panic!("switch did not commit")
    }
    #[test]
    fn location_switch_restores_underlying_closed_or_open_cache_root_at_exact_deadline() {
        for root in [375, 378] {
            let (mut w, a, l) = fixture(root);
            let before = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let t = toggle(&mut w, a, l, if root == 375 { 1 } else { 3 });
            assert_eq!(t.restore, Some(l));
            assert_eq!(t.restore_at, Some(t.tick + 5));
            assert!(t.json(a).contains("\"restore_tick\":"));
            for _ in 0..4 {
                w.tick_with_random(&mut Never).unwrap();
                assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(t.to));
            }
            w.tick_with_random(&mut Never).unwrap();
            assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.data(a).unwrap().stats, stats);
        }
    }
    #[test]
    fn location_switch_shut_to_map_resets_without_reopen_timer_and_old_timer_cannot_close_new_opening(
    ) {
        let (mut w, a, l) = fixture(375);
        let first = toggle(&mut w, a, l, 1);
        let shut = toggle(&mut w, a, first.to, 3);
        assert_eq!(shut.to, l);
        assert!(shut.restore_at.is_none());
        let second = toggle(&mut w, a, l, 1);
        while w.clock.0 <= first.restore_at.unwrap() {
            w.tick_with_random(&mut Never).unwrap();
        }
        assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(second.to));
        while w.clock.0 <= second.restore_at.unwrap() {
            w.tick_with_random(&mut Never).unwrap();
        }
        assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e,WorldEvent::LocationRestoration(t)if t.result=="stale")));
    }
    #[test]
    fn location_switch_invalid_root_deadline_or_unknown_stage_has_no_mutation_or_presentation() {
        for mode in 0..3 {
            let (mut w, a, l) = fixture(375);
            let mut r = rule(378, Some(52));
            if mode == 0 {
                r.duration = 0;
            } else if mode == 1 {
                r.next = 999;
            } else {
                w.locations = crate::locations::Locations::default();
                w.locations.add(l).unwrap();
            }
            let before = w.locations.revision(l);
            w.players
                .get_mut(a.slot as usize)
                .unwrap()
                .location_revision = before;
            w.process_location_swap(a.slot as usize, l, r);
            assert_eq!(w.locations.revision(l), before);
            assert_eq!(w.locations.find_type(l.coords, l.id), Some(l));
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::LocationSwitched { .. }
                    | WorldEvent::PlayerAnimation { .. }
                    | WorldEvent::PlayerSound { .. }
            )));
        }
    }
    fn delayed_fixture() -> (World, ActorId, Location) {
        let (mut w, a, l) = fixture(375);
        w.location_handlers.insert(
            (375, 1),
            Handler::DelayedSwap {
                rule: rule(378, Some(64)),
                delay: 1,
            },
        );
        w.location_handlers.insert(
            (378, 3),
            Handler::DelayedSwap {
                rule: rule(375, None),
                delay: 1,
            },
        );
        w.location_handlers.insert(
            (378, 2),
            Handler::ArrivedMessage("You search the drawers but find nothing."),
        );
        (w, a, l)
    }
    #[test]
    fn delayed_location_switch_presents_once_then_changes_after_one_tick_and_search_does_not_reset_timer(
    ) {
        let (mut w, a, l) = delayed_fixture();
        let before = w.data(a).unwrap().inventory.clone();
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
        w.tick_with_random(&mut Never).unwrap();
        assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
        let e = w.drain_events();
        assert!(e
            .iter()
            .any(|e| matches!(e, WorldEvent::PlayerAnimation { sequence: 536, .. })));
        assert!(e
            .iter()
            .any(|e| matches!(e, WorldEvent::PlayerSound { sound: 64, .. })));
        assert!(!e
            .iter()
            .any(|e| matches!(e, WorldEvent::LocationSwitched { .. })));
        w.tick_with_random(&mut Never).unwrap();
        let e = w.drain_events();
        let t = e
            .iter()
            .find_map(|e| {
                if let WorldEvent::LocationSwitched { trace, .. } = e {
                    Some(*trace)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(t.to.id, 378);
        assert!(!e.iter().any(|e| matches!(
            e,
            WorldEvent::PlayerAnimation { .. } | WorldEvent::PlayerSound { .. }
        )));
        w.submit(
            a,
            Action::OperateLocation {
                coords: l.coords,
                id: 378,
                operation: 2,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(&mut Never).unwrap();
        assert!(w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::LocationMessage {
                message: "You search the drawers but find nothing.",
                ..
            }
        )));
        while w.clock.0 <= t.restore_at.unwrap() {
            w.tick_with_random(&mut Never).unwrap();
        }
        assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
        assert_eq!(w.data(a).unwrap().inventory, before);
        assert_eq!(w.data(a).unwrap().stats, stats);
    }
    #[test]
    fn delayed_location_switch_cancels_replaced_target_held_launch_logout_or_modal_before_model_change(
    ) {
        for mode in 0..4 {
            let (mut w, a, l) = delayed_fixture();
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
            w.tick_with_random(&mut Never).unwrap();
            w.drain_events();
            match mode {
                0 => {
                    w.change_location(l, false).unwrap();
                    w.change_location(l, true).unwrap();
                }
                1 => w
                    .submit(a, Action::EatItemSlot { slot: 0, item: 999 })
                    .unwrap(),
                2 => w.request_logout(a).unwrap(),
                _ => w.players.get_mut(a.slot as usize).unwrap().has_modal = true,
            };
            for _ in 0..3 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert_eq!(w.locations.find_shape(l.coords, l.shape), Some(l));
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationSwitched { .. })));
        }
    }
}
