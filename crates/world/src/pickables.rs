//! Single-item pickables over existing inventory and reached-location systems.
use crate::{
    inventory::{Catalog, Error, Insert, Inventory, StackType},
    locations::Location,
    simulation::PlayerData,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    pub item: i32,
    pub members: bool,
    pub animation: u16,
    pub sound: u16,
    pub picked_message: &'static str,
    pub full_message: &'static str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ignored,
    Message(&'static str),
    Picked(Rule),
}
impl Outcome {
    pub fn message(self) -> Option<&'static str> {
        match self {
            Self::Ignored => None,
            Self::Message(m) => Some(m),
            Self::Picked(r) => Some(r.picked_message),
        }
    }
}
/// A grant owns its private inventory snapshot and can be committed only once.
#[derive(Debug)]
pub struct Prepared {
    inventory: Inventory,
    rule: Rule,
}
impl Prepared {
    pub fn commit(self, player: &mut PlayerData) -> Rule {
        player.inventory = self.inventory;
        self.rule
    }
}
#[derive(Debug)]
pub enum Preparation {
    Refused(Outcome),
    Grant(Prepared),
}
impl Rule {
    pub fn prepare(
        self,
        player: &PlayerData,
        catalog: &Catalog,
        member: bool,
    ) -> Result<Preparation, Error> {
        if !catalog.ordinary_nonstackable(self.item) {
            return Err(Error::InvalidCount);
        }
        if player.inventory.stack != StackType::Normal || player.inventory.placeholders {
            return Err(Error::InvalidCapacity);
        }
        if self.members && !member {
            return Ok(Preparation::Refused(Outcome::Message(
                "You need to be a member to pick this.",
            )));
        }
        if player.inventory.free_space() == 0 {
            return Ok(Preparation::Refused(Outcome::Message(self.full_message)));
        }
        let mut inventory = player.inventory.clone();
        let inserted = inventory.insert(catalog, Insert::new(self.item, 1))?;
        if inserted.completed != 1 {
            return Err(Error::NotEnoughSpace);
        }
        Ok(Preparation::Grant(Prepared {
            inventory,
            rule: self,
        }))
    }
    /// Perpetual rows use the same prepared grant without a location change.
    pub fn perform(
        self,
        player: &mut PlayerData,
        catalog: &Catalog,
        member: bool,
    ) -> Result<Outcome, Error> {
        Ok(match self.prepare(player, catalog, member)? {
            Preparation::Refused(outcome) => outcome,
            Preparation::Grant(plan) => Outcome::Picked(plan.commit(player)),
        })
    }
}
/// Validated before drawing randomness or preparing any live mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Depletion {
    pub numerator: u32,
    pub denominator: u32,
    pub delay: i32,
}
impl Depletion {
    pub fn deadline(self, now: i32) -> Result<i32, &'static str> {
        if self.numerator == 0
            || self.denominator == 0
            || self.numerator > self.denominator
            || self.delay <= 0
            || now < 0
        {
            return Err("Invalid pickable depletion");
        }
        now.checked_add(self.delay)
            .ok_or("Pickable depletion clock overflow")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChanceTransition {
    pub transition: Transition,
    pub roll: u32,
    pub denominator: u32,
    pub depleted: bool,
}
impl ChanceTransition {
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        format!(
            "{{\"schema\":1,\"roll\":{},\"denominator\":{},\"depleted\":{},\"transition\":{}}}",
            self.roll,
            self.denominator,
            self.depleted,
            self.transition.json(actor)
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Next { location: u32 },
    Respawn { empty: u32, full: u32, delay: i32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Advance {
    pub replacement: Location,
    pub restoration: Option<(Location, i32)>,
    pub deadline: Option<i32>,
}
impl Step {
    /// Validate the complete transition and deadline before inventory or location mutation.
    pub fn prepare(self, current: Location, now: i32) -> Result<Advance, &'static str> {
        if now < 0 {
            return Err("Invalid pickable clock");
        }
        let make = |id| {
            Location::new(current.coords, id, current.shape, current.angle)
                .map_err(|_| "Invalid pickable location")
        };
        match self {
            Self::Next { location } => {
                if location == current.id {
                    return Err("Pickable stage must advance");
                }
                Ok(Advance {
                    replacement: make(location)?,
                    restoration: None,
                    deadline: None,
                })
            }
            Self::Respawn { empty, full, delay } => {
                if empty == current.id || empty == full || delay <= 0 {
                    return Err("Invalid pickable restoration");
                }
                let deadline = now
                    .checked_add(delay)
                    .ok_or("Pickable restoration clock overflow")?;
                Ok(Advance {
                    replacement: make(empty)?,
                    restoration: Some((make(full)?, delay)),
                    deadline: Some(deadline),
                })
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    pub tick: i32,
    pub from: Location,
    pub to: Location,
    pub item: i32,
    pub restore: Option<Location>,
    pub restore_at: Option<i32>,
}
impl Transition {
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        let loc = |l: Location| {
            format!(
                "[{},{},{},{},{},{}]",
                l.coords.x(),
                l.coords.z(),
                l.coords.level(),
                l.id,
                l.shape,
                l.angle
            )
        };
        format!(
            r#"{{"schema":1,"actor_slot":{},"actor_generation":{},"tick":{},"from":{},"to":{},"item":{},"restore":{},"restore_tick":{}}}"#,
            actor.slot,
            actor.generation,
            self.tick,
            loc(self.from),
            loc(self.to),
            self.item,
            self.restore.map(loc).unwrap_or_else(|| "null".into()),
            self.restore_at
                .map(|n| n.to_string())
                .unwrap_or_else(|| "null".into())
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Inventory, Item, Template},
        loc_collision::LocCollision,
        location_content::Handler,
        locations::Location,
        simulation::{Action, ActorId, World, WorldEvent},
        stats::Stats,
        CoordGrid,
    };
    fn rule() -> Rule {
        Rule {
            item: 1000,
            members: true,
            animation: 827,
            sound: 2581,
            picked_message: "You pick some berries.",
            full_message: "You can't carry any more berries.",
        }
    }
    fn data() -> PlayerData {
        PlayerData::new(
            1,
            "Picker".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        )
    }
    #[test]
    fn perpetual_pickable_transaction_refuses_member_full_or_malformed_without_mutation() {
        let r = rule();
        let mut p = data();
        p.inventory = Inventory::new(2, StackType::Normal, false);
        let c = Catalog::default();
        let before = (p.inventory.clone(), p.stats.clone());
        assert!(matches!(
            r.perform(&mut p, &c, false),
            Ok(Outcome::Message(_))
        ));
        assert_eq!((p.inventory.clone(), p.stats.clone()), before);
        for _ in 0..2 {
            assert_eq!(r.perform(&mut p, &c, true), Ok(Outcome::Picked(r)));
        }
        assert_eq!(
            p.inventory
                .slots()
                .iter()
                .flatten()
                .map(|i| i.count)
                .sum::<i32>(),
            2
        );
        assert_eq!(p.stats, before.1);
        let before = p.inventory.clone();
        assert_eq!(
            r.perform(&mut p, &c, true),
            Ok(Outcome::Message(r.full_message))
        );
        assert_eq!(p.inventory, before);
        for mode in 0..7 {
            let mut p = data();
            let mut c = Catalog::default();
            match mode {
                0 => {
                    c.stackable.insert(1000);
                }
                1 => {
                    c.dummy.insert(1000);
                }
                2 => {
                    c.notes.insert(
                        1000,
                        Template {
                            link: 1,
                            template: 799,
                        },
                    );
                }
                3 => {
                    c.placeholders.insert(
                        1000,
                        Template {
                            link: 1,
                            template: 14401,
                        },
                    );
                }
                4 => {
                    c.transforms.insert(
                        1000,
                        Template {
                            link: 1,
                            template: 65534,
                        },
                    );
                }
                5 => p.inventory.stack = StackType::Always,
                _ => p.inventory.placeholders = true,
            }
            let before = (p.inventory.clone(), p.stats.clone());
            assert!(r.perform(&mut p, &c, true).is_err());
            assert_eq!((p.inventory, p.stats), before);
        }
    }
    fn fixture(member: bool, far: bool) -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let loc = Location::new(
            CoordGrid::new(if far { 3204 } else { 3201 }, 3200, 0).unwrap(),
            100,
            10,
            0,
        )
        .unwrap();
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: false,
            break_route_finding: false,
        };
        w.location_types.insert(100, ty);
        w.locations.load(loc);
        crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        for op in [1, 2] {
            w.location_handlers
                .insert((100, op), Handler::Pickable(rule()));
        }
        w.queue_login(1, data()).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, member).unwrap();
        w.drain_events();
        (w, a, loc)
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
    fn settle(w: &mut World) {
        for _ in 0..7 {
            w.tick().unwrap();
        }
    }
    #[test]
    fn perpetual_pickable_actual_reached_operations_grant_once_per_click_without_depletion_or_xp() {
        let (mut w, a, l) = fixture(true, false);
        let rev = w.locations.revision(l);
        let stats = w.data(a).unwrap().stats.clone();
        w.submit(a, action(l, 1)).unwrap();
        settle(&mut w);
        assert_eq!(count(&w, a), 1);
        settle(&mut w);
        assert_eq!(count(&w, a), 1);
        w.submit(a, action(l, 2)).unwrap();
        settle(&mut w);
        assert_eq!(count(&w, a), 2);
        assert_eq!(w.locations.revision(l), rev);
        assert_eq!(w.data(a).unwrap().stats, stats);
        w.submit(a, action(l, 3)).unwrap();
        settle(&mut w);
        assert_eq!(count(&w, a), 2);
        let e = w.drain_events();
        assert_eq!(
            e.iter()
                .filter(|e| matches!(e, WorldEvent::PickableCollected { item: 1000, .. }))
                .count(),
            2
        );
        assert_eq!(
            e.iter()
                .filter(|e| matches!(e, WorldEvent::PlayerAnimation { sequence: 827, .. }))
                .count(),
            2
        );
        assert_eq!(
            e.iter()
                .filter(|e| matches!(e, WorldEvent::PlayerSound { sound: 2581, .. }))
                .count(),
            2
        );
    }
    #[test]
    fn perpetual_pickable_member_or_full_refusal_has_no_pickup_presentation_or_item() {
        for full in [false, true] {
            let (mut w, a, l) = fixture(full, false);
            if full {
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
            let before = (
                w.data(a).unwrap().inventory.slots().to_vec(),
                w.data(a).unwrap().stats.clone(),
            );
            w.submit(a, action(l, 1)).unwrap();
            settle(&mut w);
            assert_eq!(w.data(a).unwrap().inventory.slots(), before.0.as_slice());
            assert_eq!(w.data(a).unwrap().stats, before.1);
            assert_eq!(count(&w, a), 0);
            assert!(!w.drain_events().iter().any(|e| matches!(
                e,
                WorldEvent::PickableCollected { .. }
                    | WorldEvent::PlayerAnimation { .. }
                    | WorldEvent::PlayerSound { .. }
            )));
        }
    }
    #[test]
    fn perpetual_pickable_moved_arrival_waits_and_same_type_replacement_invalidates_old_target() {
        let (mut w, a, l) = fixture(true, true);
        w.submit(a, action(l, 1)).unwrap();
        let mut arrived = false;
        for _ in 0..8 {
            w.tick().unwrap();
            assert_eq!(count(&w, a), 0);
            if w.data(a).unwrap().coords.x() == 3203 {
                arrived = true;
                break;
            }
        }
        assert!(arrived);
        w.tick().unwrap();
        assert_eq!(count(&w, a), 0);
        w.tick().unwrap();
        assert_eq!(count(&w, a), 1);
        let (mut w, a, l) = fixture(true, true);
        w.submit(a, action(l, 1)).unwrap();
        w.tick().unwrap();
        let rev = w.locations.revision(l);
        w.locations.delete(l);
        w.locations.add(l).unwrap();
        assert_ne!(w.locations.revision(l), rev);
        settle(&mut w);
        assert_eq!(count(&w, a), 0);
        assert!(!w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::PickableCollected { .. })));
    }
    #[test]
    fn pickable_preparation_is_private_and_invalid_or_overflowing_stage_cannot_mutate() {
        let r = rule();
        let mut p = data();
        let before = p.inventory.clone();
        let Preparation::Grant(plan) = r.prepare(&p, &Catalog::default(), true).unwrap() else {
            panic!("missing plan")
        };
        assert_eq!(p.inventory, before);
        assert_eq!(plan.commit(&mut p), r);
        assert!(p.inventory.slots().iter().flatten().any(|i| i.id == 1000));
        let l = Location::new(CoordGrid::new(3200, 3200, 0).unwrap(), 100, 10, 0).unwrap();
        for (step, now) in [
            (Step::Next { location: 100 }, 1),
            (
                Step::Respawn {
                    empty: 100,
                    full: 101,
                    delay: 4,
                },
                1,
            ),
            (
                Step::Respawn {
                    empty: 102,
                    full: 102,
                    delay: 4,
                },
                1,
            ),
            (
                Step::Respawn {
                    empty: 102,
                    full: 100,
                    delay: 0,
                },
                1,
            ),
            (
                Step::Respawn {
                    empty: 102,
                    full: 100,
                    delay: 4,
                },
                i32::MAX - 2,
            ),
            (Step::Next { location: 101 }, -1),
        ] {
            assert!(step.prepare(l, now).is_err());
        }
        let a = Step::Respawn {
            empty: 102,
            full: 100,
            delay: 4,
        }
        .prepare(l, 1)
        .unwrap();
        assert_eq!(a.deadline, Some(5));
        assert_eq!(a.restoration.unwrap().0.id, 100);
    }
}
