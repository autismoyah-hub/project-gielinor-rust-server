use super::*;
use crate::{prayer::State, tower_offering::Continuation};
impl World {
    pub(super) fn begin_tower_offering(&mut self, actor: ActorId) -> Result<(), ActionError> {
        self.player(actor)?;
        self.prayer.as_ref().ok_or(ActionError::InvalidItem)?;
        let config = self
            .tower_offering
            .as_ref()
            .ok_or(ActionError::InvalidLocation)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if !crate::tower_layout::contains(config.origin, p.movement.coords) {
            return Ok(());
        }
        p.data.tower_diary.ok_or(ActionError::UnknownTowerDiary)?;
        config
            .bones
            .take_from_bank(&mut p.data.inventory, &mut p.data.bank, &self.object_types);
        let Some(slot) = config.bones.slot(&p.data.inventory) else {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "The altar wants bones. Kill a cow or chicken, then use the bones here."
                    .into(),
            });
            return Ok(());
        };
        let bone = p.data.inventory.get(slot).unwrap().id;
        let continuation = Continuation {
            before: config.bones.count(&p.data.inventory),
        };
        let xp = config.xp.get(&bone).copied().flatten();
        self.begin_offering(actor, bone, config.animation, xp, Some(continuation))
    }
    pub(super) fn finish_tower_offering(
        &mut self,
        actor: ActorId,
        continuation: Continuation,
        offered: bool,
    ) {
        let Some(config) = self.tower_offering.as_ref() else {
            return;
        };
        let Some(staff) = self.tower_staff.as_ref() else {
            return;
        };
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.data.coords = p.movement.coords;
        let altar = config
            .origin
            .translate(60, 52, 0)
            .expect("validated tower bounds");
        let result = continuation.complete(
            &mut p.data,
            &config.bones,
            &staff.inventory,
            altar,
            config.skills,
            offered,
        );
        let drop_bones = config.drop_bones;
        let completed = match result {
            Ok(Some(c)) => c,
            Ok(None) => return,
            Err(error) => {
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
                return;
            }
        };
        self.publish_tower_diary_sync(actor, completed.sync, drop_bones);
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: completed.summary,
        });
        if completed.crate_message {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "The altar accepts the offering. Search the staff crate.".into(),
            });
        }
        if completed.visit_crate {
            // afterAltarOffering only invokes this within two squares of the
            // hub altar, so walkToMagic always takes the hub-south branch.
            let p = self.players.get_mut(actor.slot as usize).unwrap();
            p.prayer = Some(State::CrateWalk {
                step: 0,
                attempts: 0,
                pending: None,
            });
            self.process_tower_crate_walk(actor.slot as usize);
        }
    }
    pub(super) fn process_tower_crate_walk(&mut self, slot: usize) {
        let Some(p) = self.players.get(slot) else {
            return;
        };
        if p.saving {
            return;
        }
        let actor = p.id;
        let Some(State::CrateWalk {
            mut step,
            mut attempts,
            pending,
        }) = p.prayer
        else {
            return;
        };
        let Some(config) = self.tower_staff.as_ref() else {
            return;
        };
        let origin = config.origin;
        let location = config.location;
        if let Some(destination) = pending {
            // ProtectedAccess.walk queues a destination in PlayerMain. Its
            // route becomes available during the next tick, not immediately.
            let route = crate::routing::coordinate_route(
                &self.collision,
                p.movement.coords,
                destination,
                p.movement.size,
                true,
                25,
                Strategy::Normal,
            );
            let p = self.players.get_mut(slot).unwrap();
            p.movement.temporary_speed = None;
            p.movement.route(route.waypoints, Speed::Walk);
            p.prayer = Some(State::CrateWalk {
                step,
                attempts,
                pending: None,
            });
        }
        let p = self.players.get_mut(slot).unwrap();
        p.protection.processed_clock = self.clock.0;
        if p.protection.delayed() {
            return;
        }
        // arrive makes at most8 attempts per destination. playerWalk suspends
        // for distance-1 cycles, then arrive adds8; no output occurs in between.
        while step < 3 {
            let (x, z, label) = match step {
                0 => (56, 48, None),
                1 => (56, 40, Some("magicStand")),
                _ => (57, 43, Some("crateStand")),
            };
            let destination = origin.translate(x, z, 0).expect("validated tower bounds");
            let distance = p.movement.coords.chebyshev_distance(destination);
            if attempts >= 8 || distance <= if label.is_some() { 0 } else { 1 } {
                if let Some(label) = label {
                    self.events.push(WorldEvent::GameMessage {
                        actor,
                        text: format!(
                            "{label} {},{} now {},{}",
                            destination.lx(),
                            destination.lz(),
                            p.movement.coords.lx(),
                            p.movement.coords.lz()
                        ),
                    });
                }
                step += 1;
                attempts = 0;
                continue;
            }
            p.movement.abort();
            p.prayer = Some(State::CrateWalk {
                step,
                attempts: attempts + 1,
                pending: Some(destination),
            });
            p.protection.suspended = true;
            p.protection.current_clock = self.clock.0;
            p.protection.delay((distance - 1).max(0) + 8);
            return;
        }
        p.prayer = None;
        p.protection.suspended = false;
        let tiles = [
            (58, 43),
            (57, 43),
            (54, 49),
            (53, 49),
            (55, 50),
            (54, 51),
            (53, 51),
            (55, 49),
        ]
        .map(|(x, z)| origin.translate(x, z, 0).expect("validated tower bounds"));
        if let Some(loc) = tiles
            .iter()
            .find_map(|&coords| self.locations.find_type(coords, location))
        {
            if let Err(error) = self.request_location(actor, loc.coords, loc.id, 1, false) {
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
            }
        } else {
            let coords = tiles
                .map(|c| {
                    format!(
                        "CoordGrid({}_{}_{}_{}_{})",
                        c.level(),
                        c.mx(),
                        c.mz(),
                        c.lx(),
                        c.lz()
                    )
                })
                .join(", ");
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: format!("No loc at {coords}"),
            });
        }
    }
}
