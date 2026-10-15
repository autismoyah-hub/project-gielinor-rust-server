//! Ground-object registry and timed lifecycle from ObjRegistry / ObjRepository.
use crate::{inventory::Catalog, tick::MapClock, CoordGrid};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Temporary,
    Permanent,
    Private,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub coords: CoordGrid,
    pub item: u16,
    pub count: i32,
    pub scope: Scope,
    pub created: i32,
    pub receiver: Option<i64>,
    pub owner: Option<i64>,
}
impl Object {
    pub fn visible_to(&self, observer: Option<i64>) -> bool {
        self.scope != Scope::Private || observer == self.receiver
    }
}
#[derive(Clone, Debug)]
pub struct Handle {
    id: u64,
    value: Rc<RefCell<Object>>,
}
impl Handle {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn snapshot(&self) -> Object {
        self.value.borrow().clone()
    }
}
#[derive(Clone, Debug)]
pub enum Event {
    Added(u64, Object),
    Deleted(u64, Object),
    Revealed(u64, Object),
    Count(u64, Object, i32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidObject,
    DummyItem,
    NonStackableLimit,
    IdentityExhausted,
}
enum Added {
    Merge(Handle),
    New(Vec<Handle>),
}
struct Timed {
    object: Handle,
    delete: i32,
    reveal: i32,
}
struct Respawn {
    object: Handle,
    at: i32,
}
struct Delayed {
    object: Object,
    at: i32,
    duration: i32,
}
#[derive(Default)]
pub struct Objects {
    next: u64,
    tiles: HashMap<CoordGrid, Vec<Handle>>,
    zones: HashMap<CoordGrid, Vec<Handle>>,
    timers: Vec<Timed>,
    respawns: Vec<Respawn>,
    delayed: Vec<Delayed>,
    events: Vec<Event>,
}
impl Objects {
    fn validate_add(object: &Object, catalog: &Catalog) -> Result<u64, Error> {
        if object.count <= 0 || object.coords == CoordGrid::NULL {
            return Err(Error::InvalidObject);
        }
        if catalog.dummy.contains(&(object.item as i32)) {
            return Err(Error::DummyItem);
        }
        let stackable = catalog.stackable.contains(&(object.item as i32));
        if !stackable && object.count > 128 {
            return Err(Error::NonStackableLimit);
        }
        // add allocates the request handle even when it merges, then allocates
        // single-copy handles when a nonstackable quantity is split.
        Ok(1 + if !stackable && object.count > 1 {
            object.count as u64
        } else {
            0
        })
    }
    /// Prevalidate every insertion and its worst-case identity allocation before
    /// touching stacks, timers or events. The world thread owns this registry,
    /// so no other insertion can invalidate the reservation during this call.
    pub fn add_batch(
        &mut self,
        objects: &[Object],
        catalog: &Catalog,
        clock: MapClock,
        duration: i32,
        reveal: i32,
    ) -> Result<Vec<Vec<Handle>>, Error> {
        let mut maximum = self.next;
        for object in objects {
            maximum = maximum
                .checked_add(Self::validate_add(object, catalog)?)
                .ok_or(Error::IdentityExhausted)?;
        }
        Ok(objects
            .iter()
            .map(|object| {
                self.add(object.clone(), catalog, clock, duration, reveal)
                    .expect("prevalidated ground insertion on exclusively owned registry")
            })
            .collect())
    }
    /// Kotlin ObjStackEntryList preserves insertion order across all tiles in a zone.
    pub fn in_zone(&self, base: CoordGrid) -> impl Iterator<Item = &Handle> {
        self.zones.get(&zone_base(base)).into_iter().flatten()
    }
    pub fn at(&self, coords: CoordGrid) -> impl Iterator<Item = &Handle> {
        self.tiles.get(&coords).into_iter().flatten()
    }
    pub fn count(&self) -> usize {
        self.tiles.values().map(Vec::len).sum()
    }
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
    fn handle(&mut self, object: Object) -> Result<Handle, Error> {
        self.next = self.next.checked_add(1).ok_or(Error::IdentityExhausted)?;
        Ok(Handle {
            id: self.next,
            value: Rc::new(RefCell::new(object)),
        })
    }
    fn register(&mut self, handle: Handle, catalog: &Catalog) -> Result<Added, Error> {
        let object = handle.snapshot();
        Self::validate_add(&object, catalog)?;
        let stackable = catalog.stackable.contains(&(object.item as i32));
        let merge = self
            .at(object.coords)
            .find(|entry| {
                let old = entry.value.borrow();
                old.item == object.item
                    && old.scope == Scope::Private
                    && old.receiver.is_some()
                    && old.receiver == object.receiver
                    && stackable
                    && (old.count as i64 + object.count as i64) <= i32::MAX as i64
            })
            .cloned();
        if let Some(existing) = merge {
            let old = existing.value.borrow().count;
            existing.value.borrow_mut().count += object.count;
            self.events
                .push(Event::Count(existing.id, existing.snapshot(), old));
            return Ok(Added::Merge(existing));
        }
        let entries = if !stackable && object.count > 1 {
            self.next
                .checked_add(object.count as u64)
                .ok_or(Error::IdentityExhausted)?;
            let mut entries = Vec::with_capacity(object.count as usize);
            for _ in 0..object.count {
                // The reference's singleCopy omits ownerId; preserve that behavior.
                entries.push(self.handle(Object {
                    count: 1,
                    owner: None,
                    ..object.clone()
                })?);
            }
            entries
        } else {
            vec![handle]
        };
        for entry in &entries {
            self.events.push(Event::Added(entry.id, entry.snapshot()));
        }
        self.tiles
            .entry(object.coords)
            .or_default()
            .extend(entries.iter().cloned());
        self.zones
            .entry(zone_base(object.coords))
            .or_default()
            .extend(entries.iter().cloned());
        Ok(Added::New(entries))
    }
    pub fn add(
        &mut self,
        object: Object,
        catalog: &Catalog,
        clock: MapClock,
        duration: i32,
        reveal: i32,
    ) -> Result<Vec<Handle>, Error> {
        let handle = self.handle(object)?;
        let result = self.register(handle, catalog)?;
        match result {
            Added::Merge(entry) => {
                if duration != i32::MAX {
                    if let Some(timer) = self.timers.iter_mut().find(|t| t.object.id == entry.id) {
                        timer.delete = clock.deadline(duration);
                        timer.reveal = clock.deadline(reveal);
                    }
                }
                Ok(vec![entry])
            }
            Added::New(entries) => {
                if duration != i32::MAX {
                    for entry in &entries {
                        self.timers.push(Timed {
                            object: entry.clone(),
                            delete: clock.deadline(duration),
                            reveal: clock.deadline(reveal),
                        });
                    }
                }
                Ok(entries)
            }
        }
    }
    fn remove(&mut self, handle: &Handle) -> bool {
        let object = handle.snapshot();
        let Some(entries) = self.tiles.get_mut(&object.coords) else {
            return false;
        };
        let Some(index) = entries
            .iter()
            .position(|e| e.id == handle.id && Rc::ptr_eq(&e.value, &handle.value))
        else {
            return false;
        };
        entries.remove(index);
        if entries.is_empty() {
            self.tiles.remove(&object.coords);
        }
        let zone = zone_base(object.coords);
        if let Some(entries) = self.zones.get_mut(&zone) {
            entries.retain(|entry| entry.id != handle.id);
            if entries.is_empty() {
                self.zones.remove(&zone);
            }
        }
        self.events.push(Event::Deleted(handle.id, object));
        true
    }
    pub fn delete(&mut self, handle: &Handle, clock: MapClock, respawn_ticks: i32) -> bool {
        if !self.remove(handle) {
            return false;
        }
        if handle.value.borrow().scope == Scope::Permanent && respawn_ticks != i32::MAX {
            self.respawns.push(Respawn {
                object: handle.clone(),
                at: clock.deadline(respawn_ticks),
            });
        }
        true
    }
    fn reveal(&mut self, handle: &Handle) {
        let coords = handle.value.borrow().coords;
        if !self
            .at(coords)
            .any(|entry| entry.id == handle.id && Rc::ptr_eq(&entry.value, &handle.value))
        {
            return;
        }
        handle.value.borrow_mut().scope = Scope::Temporary;
        self.events
            .push(Event::Revealed(handle.id, handle.snapshot()));
    }
    /// Called after map-clock increment. Reveals use equality, deletions use >=.
    pub fn process_durations(&mut self, clock: MapClock, catalog: &Catalog) -> Result<(), Error> {
        let mut index = 0;
        while index < self.respawns.len() {
            if clock.0 < self.respawns[index].at {
                index += 1;
                continue;
            }
            self.register(self.respawns[index].object.clone(), catalog)?;
            self.respawns.remove(index);
        }
        let mut index = 0;
        while index < self.timers.len() {
            let timer = &self.timers[index];
            let object = timer.object.clone();
            let reveal = clock.0 == timer.reveal;
            let delete = clock.0 >= timer.delete;
            if reveal {
                self.reveal(&object);
            }
            if delete {
                self.remove(&object);
                self.timers.remove(index);
            } else {
                index += 1;
            }
        }
        Ok(())
    }
    pub fn add_delayed(&mut self, object: Object, clock: MapClock, delay: i32, duration: i32) {
        self.delayed.push(Delayed {
            object,
            at: clock.deadline(delay),
            duration,
        });
    }
    /// Called during world-main, before the map clock increments.
    pub fn process_delayed(&mut self, clock: MapClock, catalog: &Catalog) -> Result<(), Error> {
        let mut index = 0;
        while index < self.delayed.len() {
            let entry = &self.delayed[index];
            if clock.0 < entry.at {
                index += 1;
                continue;
            }
            self.add(entry.object.clone(), catalog, clock, entry.duration, 100)?;
            self.delayed.remove(index);
        }
        Ok(())
    }
}
pub fn zone_base(coords: CoordGrid) -> CoordGrid {
    CoordGrid::new(coords.x() & !7, coords.z() & !7, coords.level()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batch_rejection_preserves_existing_stacks_timers_events_and_identity() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        for kind in 0..4 {
            let mut objects = Objects::default();
            let existing = objects
                .add(object(Scope::Private, 10), &catalog, MapClock(0), 10, 5)
                .unwrap()
                .remove(0);
            objects.drain_events();
            let first = object(Scope::Private, 3);
            let mut invalid = object(Scope::Private, 1);
            if kind == 0 {
                invalid.count = 0;
            }
            if kind == 1 {
                invalid.coords = CoordGrid::NULL;
            }
            if kind == 2 {
                invalid.item = 100;
                invalid.count = 129;
            }
            if kind == 3 {
                objects.next = u64::MAX - 1;
            }
            let identity = objects.next;
            assert!(objects
                .add_batch(&[first, invalid], &catalog, MapClock(3), 100, 50)
                .is_err());
            assert_eq!(objects.next, identity);
            assert_eq!(existing.snapshot().count, 10);
            assert_eq!(objects.count(), 1);
            assert!(objects.drain_events().is_empty());
            assert_eq!(
                (objects.timers[0].delete, objects.timers[0].reveal),
                (10, 5)
            );
        }
    }
    #[test]
    fn batch_reuses_normal_merge_split_and_private_lifecycle() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        let mut objects = Objects::default();
        let existing = objects
            .add(object(Scope::Private, 10), &catalog, MapClock(0), 10, 5)
            .unwrap()
            .remove(0);
        objects.drain_events();
        let mut bones = object(Scope::Private, 2);
        bones.item = 526;
        let batch = objects
            .add_batch(
                &[object(Scope::Private, 5), bones],
                &catalog,
                MapClock(2),
                200,
                100,
            )
            .unwrap();
        assert_eq!(batch[0][0].id(), existing.id());
        assert_eq!(existing.snapshot().count, 15);
        assert_eq!(batch[1].len(), 2);
        assert_eq!(objects.count(), 3);
        assert!(batch
            .iter()
            .flatten()
            .all(|h| !h.snapshot().visible_to(Some(8))));
        objects.process_durations(MapClock(102), &catalog).unwrap();
        assert!(batch
            .iter()
            .flatten()
            .all(|h| h.snapshot().visible_to(Some(8))));
        objects.process_durations(MapClock(202), &catalog).unwrap();
        assert_eq!(objects.count(), 0);
    }
    fn object(scope: Scope, count: i32) -> Object {
        Object {
            coords: CoordGrid::from_packed(0),
            item: 995,
            count,
            scope,
            created: 0,
            receiver: Some(7),
            owner: Some(7),
        }
    }
    #[test]
    fn configured_private_profiles_reveal_at_100_and_expire_at_200_or_300() {
        for duration in [200, 300] {
            let catalog = Catalog::default();
            let mut objects = Objects::default();
            let handle = objects
                .add(
                    object(Scope::Private, 1),
                    &catalog,
                    MapClock(0),
                    duration,
                    100,
                )
                .unwrap()
                .remove(0);
            objects.drain_events();
            for clock in 1..duration {
                objects
                    .process_durations(MapClock(clock), &catalog)
                    .unwrap();
                assert_eq!(handle.snapshot().visible_to(Some(8)), clock >= 100);
                assert_eq!(objects.count(), 1);
            }
            let events = objects.drain_events();
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::Revealed(..)))
                    .count(),
                1
            );
            objects
                .process_durations(MapClock(duration), &catalog)
                .unwrap();
            assert_eq!(objects.count(), 0);
            assert_eq!(
                objects
                    .drain_events()
                    .iter()
                    .filter(|e| matches!(e, Event::Deleted(..)))
                    .count(),
                1
            );
        }
    }
    #[test]
    fn coincident_reveal_expiry_is_ordered_and_stale_timer_cannot_delete_replacement() {
        let catalog = Catalog::default();
        let mut objects = Objects::default();
        let first = objects
            .add(object(Scope::Private, 1), &catalog, MapClock(0), 5, 5)
            .unwrap()
            .remove(0);
        objects.drain_events();
        objects.process_durations(MapClock(5), &catalog).unwrap();
        let events = objects.drain_events();
        assert!(
            matches!(events.as_slice(),[Event::Revealed(id,_),Event::Deleted(other,_)] if *id==first.id() && id==other)
        );
        let stale = objects
            .add(object(Scope::Private, 1), &catalog, MapClock(6), 5, 5)
            .unwrap()
            .remove(0);
        assert!(objects.delete(&stale, MapClock(7), 0));
        let replacement = objects
            .add(object(Scope::Private, 1), &catalog, MapClock(7), 10, 8)
            .unwrap()
            .remove(0);
        assert_ne!(stale.id(), replacement.id());
        objects.drain_events();
        objects.process_durations(MapClock(11), &catalog).unwrap();
        assert_eq!(objects.count(), 1);
        assert_eq!(replacement.snapshot().scope, Scope::Private);
        assert!(objects.drain_events().is_empty());
        assert!(!objects.delete(&stale, MapClock(11), 0));
    }
    #[test]
    fn private_merges_reset_timers_public_stacks_and_overflow_do_not_merge() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        let mut objects = Objects::default();
        let first = objects
            .add(object(Scope::Private, 10), &catalog, MapClock(0), 10, 5)
            .unwrap()
            .remove(0);
        let merged = objects
            .add(object(Scope::Private, 20), &catalog, MapClock(3), 10, 5)
            .unwrap()
            .remove(0);
        assert_eq!(first.id(), merged.id());
        assert_eq!(first.snapshot().count, 30);
        objects.process_durations(MapClock(5), &catalog).unwrap();
        assert_eq!(first.snapshot().scope, Scope::Private);
        objects.process_durations(MapClock(8), &catalog).unwrap();
        assert_eq!(first.snapshot().scope, Scope::Temporary);
        objects
            .add(
                object(Scope::Private, i32::MAX),
                &catalog,
                MapClock(8),
                i32::MAX,
                100,
            )
            .unwrap();
        assert_eq!(objects.count(), 2);
        objects
            .add(
                object(Scope::Private, 1),
                &catalog,
                MapClock(8),
                i32::MAX,
                100,
            )
            .unwrap();
        assert_eq!(objects.count(), 3);
        objects.process_durations(MapClock(13), &catalog).unwrap();
        assert_eq!(objects.count(), 2);
    }
    #[test]
    fn nonstackable_split_limits_and_permanent_respawn_preserve_identity() {
        let catalog = Catalog::default();
        let mut objects = Objects::default();
        assert_eq!(
            objects
                .add(object(Scope::Private, 129), &catalog, MapClock(0), 10, 5)
                .err(),
            Some(Error::NonStackableLimit)
        );
        assert_eq!(objects.count(), 0);
        let split = objects
            .add(
                object(Scope::Permanent, 2),
                &catalog,
                MapClock(0),
                i32::MAX,
                100,
            )
            .unwrap();
        assert_eq!(split.len(), 2);
        assert_eq!(split[0].snapshot().owner, None);
        assert!(objects.delete(&split[0], MapClock(0), 2));
        assert!(!objects.delete(&split[0], MapClock(0), 2));
        objects.process_durations(MapClock(1), &catalog).unwrap();
        assert_eq!(objects.count(), 1);
        objects.process_durations(MapClock(2), &catalog).unwrap();
        assert_eq!(objects.count(), 2);
        assert!(objects
            .at(CoordGrid::from_packed(0))
            .any(|e| e.id() == split[0].id()));
    }
    #[test]
    fn skipped_reveal_and_infinite_merge_duration_match_reference_quirks() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        let mut objects = Objects::default();
        let entry = objects
            .add(object(Scope::Private, 1), &catalog, MapClock(0), 10, 5)
            .unwrap()
            .remove(0);
        objects
            .add(
                object(Scope::Private, 1),
                &catalog,
                MapClock(1),
                i32::MAX,
                1,
            )
            .unwrap();
        objects.process_durations(MapClock(6), &catalog).unwrap();
        assert_eq!(entry.snapshot().scope, Scope::Private);
        objects.process_durations(MapClock(10), &catalog).unwrap();
        assert_eq!(objects.count(), 0);
    }
}
