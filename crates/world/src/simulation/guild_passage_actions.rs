use super::*;
use crate::{guild_passages::Kind, locations::Location};
impl World {
    pub(super) fn process_guild_passage(&mut self, slot: usize, door: Location) {
        let Some(gate) = self.guild_passages.get(&door.id) else {
            return;
        };
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        if p.pending_logout
            || p.saving
            || p.death_queued
            || p.data.stats.current(3) as u8 == 0
            || p.protection.access_protected(p.has_modal)
        {
            return;
        }
        let plan = match gate.prepare(
            door,
            p.movement.coords,
            &p.data.stats,
            &p.data.equipment,
            &self.object_types,
            p.member,
        ) {
            Ok(plan) => plan,
            Err(error) => {
                self.events.push(WorldEvent::LocationMessage {
                    actor,
                    message: gate.message(error),
                });
                return;
            }
        };
        let crafting = gate.kind == Kind::Crafting;
        // Preflight every fallible footprint and destination before any live write.
        if self.locations.revision(door).is_none()
            || !self.collision.allocated(
                plan.destination.x(),
                plan.destination.z(),
                plan.destination.level(),
            )
        {
            self.events.push(WorldEvent::LocationError(
                "Invalid guild passage destination",
            ));
            return;
        }
        let mut scratch = CollisionMap::default();
        for (loc, add) in [(door, false), (plan.opened, true)] {
            let Some(&ty) = self.location_types.get(&loc.id) else {
                self.events
                    .push(WorldEvent::LocationError("Missing guild door collision"));
                return;
            };
            if ty.width != 1
                || ty.length != 1
                || crate::loc_collision::toggle(
                    &mut scratch,
                    loc.coords,
                    loc.shape,
                    loc.angle,
                    ty,
                    add,
                )
                .is_err()
            {
                self.events
                    .push(WorldEvent::LocationError("Invalid guild door footprint"));
                return;
            }
        }
        // Same shared LocRepository duration queues as the donor's delete/add(...,3).
        if let Err(error) = self
            .change_location_for(door, false, 3)
            .and_then(|_| self.change_location_for(plan.opened, true, 3))
        {
            self.events.push(WorldEvent::LocationError(error));
            return;
        }
        let p = self.players.get_mut(slot).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        if p.movement
            .teleport(&mut self.collision, plan.destination, self.clock.0, false)
            .is_err()
        {
            self.events
                .push(WorldEvent::LocationError("Invalid guild passage movement"));
            return;
        }
        p.data.coords = plan.destination;
        if plan.delay > 0 {
            p.protection.delay(plan.delay);
        }
        if crafting && plan.entering {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: "Welcome to the Crafting Guild.",
            });
        }
        self.events.push(WorldEvent::ClearMapFlag(actor));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        guild_passages::{Gate, Outfit},
        inventory::Item,
        loc_collision::LocCollision,
    };
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Guild passage must not draw RNG");
        }
    }
    fn fixture(kind: Kind, inside: bool) -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3232).step_by(8) {
            for z in (3192..3232).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let (id, opened, skill, level, outfit, outfit_slot) = if kind == Kind::Crafting {
            (14910, 1539, 12, 40, 1757, 4)
        } else {
            (24958, 24959, 7, 32, 1949, 0)
        };
        let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), id, 0, 0).unwrap();
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(id, ty);
        w.location_types.insert(opened, ty);
        w.locations.load(loc);
        crate::loc_collision::toggle(&mut w.collision, loc.coords, 0, 0, ty, true).unwrap();
        w.guild_passages.insert(
            id,
            Gate {
                kind,
                opened,
                outfits: vec![Outfit {
                    id: outfit,
                    slot: outfit_slot,
                    members: false,
                }],
            },
        );
        // Prove the special handler prevents a possible generic-door eligibility bypass.
        w.doors.insert(
            id,
            crate::doors::Door {
                kind: crate::doors::Kind::ClosedSingle,
                next: opened,
                sound: 1,
            },
        );
        let north = (kind == Kind::Crafting) != inside;
        let from = loc
            .coords
            .translate(0, if north { 1 } else { -1 }, 0)
            .unwrap();
        let mut stats = Stats::default();
        stats.set_base(skill, level);
        stats.set_current(skill, level);
        let mut data = PlayerData::new(1, "Guild visitor".into(), from, stats);
        data.equipment
            .restore_slot(
                outfit_slot,
                Some(Item {
                    id: outfit,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a, loc)
    }
    fn click(w: &mut World, a: ActorId, loc: Location) {
        assert!(w.has_location_handler(loc.id, 1));
        w.submit(
            a,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
    }
    #[test]
    fn guild_passages_actual_op1_denials_exit_and_three_tick_shared_location_restoration() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            for case in 0..4 {
                let inside = case == 3;
                let (mut w, a, loc) = fixture(kind, inside);
                let before = w.data(a).unwrap().clone();
                if case == 1 || inside {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(if kind == Kind::Crafting { 4 } else { 0 }, None)
                        .unwrap();
                }
                if case == 2 || inside {
                    w.data_mut(a)
                        .unwrap()
                        .stats
                        .set_current(if kind == Kind::Crafting { 12 } else { 7 }, 1);
                }
                let state = w.data(a).unwrap().clone();
                let began = w.clock.0;
                click(&mut w, a, loc);
                if case == 1 || case == 2 {
                    assert_eq!(w.data(a).unwrap().coords, before.coords);
                    assert_eq!(w.data(a).unwrap().inventory, state.inventory);
                    assert_eq!(w.data(a).unwrap().stats, state.stats);
                    assert_eq!(w.locations.find_type(loc.coords, loc.id), Some(loc));
                    continue;
                }
                let g = &w.guild_passages[&loc.id];
                let expected = g
                    .prepare(
                        loc,
                        before.coords,
                        &state.stats,
                        &state.equipment,
                        &w.object_types,
                        false,
                    )
                    .unwrap();
                assert_eq!(w.data(a).unwrap().coords, expected.destination);
                assert_eq!(w.data(a).unwrap().inventory, state.inventory);
                assert_eq!(w.data(a).unwrap().equipment, state.equipment);
                assert_eq!(w.data(a).unwrap().stats, state.stats);
                assert_eq!(w.locations.find_type(loc.coords, loc.id), None);
                assert_eq!(
                    w.locations
                        .find_type(expected.opened.coords, expected.opened.id),
                    Some(expected.opened)
                );
                assert!(!w.has_location_handler(expected.opened.id, 1));
                assert_eq!(w.clock.0, began + 1);
                // The action cycle includes its Late increment: two more cycles reach +3.
                for _ in 0..1 {
                    w.tick_with_random(&mut NoDraw).unwrap();
                    assert!(w
                        .locations
                        .find_type(expected.opened.coords, expected.opened.id)
                        .is_some());
                }
                w.tick_with_random(&mut NoDraw).unwrap();
                assert_eq!(w.clock.0, began + 3);
                assert_eq!(w.locations.find_type(loc.coords, loc.id), Some(loc));
                assert!(w
                    .locations
                    .find_type(expected.opened.coords, expected.opened.id)
                    .is_none());
            }
        }
    }
    #[test]
    fn guild_passages_preflight_collision_failure_leaves_location_movement_and_rewards_unchanged() {
        let (mut w, a, loc) = fixture(Kind::Crafting, false);
        w.location_types.remove(&1539);
        let before = w.data(a).unwrap().clone();
        click(&mut w, a, loc);
        assert_eq!(w.locations.find_type(loc.coords, loc.id), Some(loc));
        assert_eq!(w.data(a).unwrap().coords, before.coords);
        assert_eq!(w.data(a).unwrap().inventory, before.inventory);
        assert_eq!(w.data(a).unwrap().stats, before.stats);
    }
}
