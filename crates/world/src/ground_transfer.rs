//! Inventory/ground transfers from HeldInteractions.invDropSlot and ObjTakePlugin.
//! Call after interaction reach, protected-access and drop-trigger/dialogue checks.
use crate::{
    ground::{self, Handle, Object, Objects, Scope},
    inventory::{self, Catalog, Delete, Insert, Inventory},
    tick::MapClock,
    CoordGrid,
};
#[derive(Clone, Copy, Debug)]
pub struct PickupEffects {
    pub sound: u16,
    pub animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub enum Effect {
    Sound(u16),
    Animation(u16),
    Message(&'static str),
}
pub const NO_SPACE: &str = "You don't have enough inventory space to hold that item.";
pub const TAKEN: &str = "Too late - it's gone!";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Missing,
    NotVisible,
    OutOfReach,
    NoSpace,
    InvalidCount,
    InvalidType,
    NotReady,
    Inventory(inventory::Error),
    Ground(ground::Error),
}
pub struct Drop {
    pub slot: usize,
    pub count: i32,
    pub coords: CoordGrid,
    pub observer: i64,
    pub duration: i32,
    pub reveal: i32,
}
pub fn drop_slot(
    objects: &mut Objects,
    inventory: &mut Inventory,
    catalog: &Catalog,
    clock: MapClock,
    request: Drop,
) -> Result<Vec<Handle>, Error> {
    let item = inventory.get(request.slot).ok_or(Error::Missing)?;
    let count = item.count.min(request.count);
    if count <= 0 {
        return Err(Error::InvalidCount);
    }
    let id = u16::try_from(item.id).map_err(|_| Error::InvalidType)?;
    let mut next = inventory.clone();
    let mut delete = Delete::new(item.id, count);
    delete.slot = request.slot;
    delete.strict_slot = Some(request.slot);
    let result = next.delete(catalog, delete).map_err(Error::Inventory)?;
    let handles = objects
        .add(
            Object {
                coords: request.coords,
                item: id,
                count: result.completed,
                scope: Scope::Private,
                created: clock.0,
                receiver: Some(request.observer),
                owner: None,
            },
            catalog,
            clock,
            request.duration,
            request.reveal,
        )
        .map_err(Error::Ground)?;
    *inventory = next;
    Ok(handles)
}
pub struct Pickup {
    object: Handle,
    ready_at: i32,
    respawn_ticks: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taken {
    Added(i32),
    Gone,
    RemovedWithoutSpace,
    NoSpace,
}
/// GR-13 retains reference behavior until an explicit runtime policy is selected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PickupPolicy {
    #[default]
    Reference,
    PreserveOnFailure,
}
impl Pickup {
    /// Adjacent pickups are delayed by one cycle. Movement/reach validation is the caller's
    /// responsibility; this bound additionally prevents calling the core from a remote tile.
    pub fn prepare(
        object: &Handle,
        observer: i64,
        coords: CoordGrid,
        inventory: &Inventory,
        catalog: &Catalog,
        clock: MapClock,
        respawn_ticks: i32,
    ) -> Result<Self, Error> {
        let value = object.snapshot();
        if !value.visible_to(Some(observer)) {
            return Err(Error::NotVisible);
        }
        if coords == CoordGrid::NULL
            || coords.level() != value.coords.level()
            || (coords.x() - value.coords.x()).abs() > 1
            || (coords.z() - value.coords.z()).abs() > 1
        {
            return Err(Error::OutOfReach);
        }
        let mut trial = inventory.clone();
        trial
            .insert(catalog, Insert::new(value.item as i32, value.count))
            .map_err(|_| Error::NoSpace)?;
        Ok(Self {
            object: object.clone(),
            ready_at: clock.deadline((coords != value.coords) as i32),
            respawn_ticks,
        })
    }
    pub fn ready_at(&self) -> i32 {
        self.ready_at
    }
    pub fn finish(
        self,
        objects: &mut Objects,
        inventory: &mut Inventory,
        catalog: &Catalog,
        clock: MapClock,
    ) -> Result<Taken, Error> {
        self.finish_with_policy(objects, inventory, catalog, clock, PickupPolicy::Reference)
    }
    pub fn finish_with_policy(
        self,
        objects: &mut Objects,
        inventory: &mut Inventory,
        catalog: &Catalog,
        clock: MapClock,
        policy: PickupPolicy,
    ) -> Result<Taken, Error> {
        if clock.0 < self.ready_at {
            return Err(Error::NotReady);
        }
        if policy == PickupPolicy::Reference {
            if !objects.delete(&self.object, clock, self.respawn_ticks) {
                return Ok(Taken::Gone);
            }
            let object = self.object.snapshot();
            return Ok(
                match inventory.insert(catalog, Insert::new(object.item as i32, object.count)) {
                    Ok(result) => Taken::Added(result.completed),
                    Err(_) => Taken::RemovedWithoutSpace,
                },
            );
        }
        let object = self.object.snapshot();
        if !objects
            .at(object.coords)
            .any(|h| h.id() == self.object.id())
        {
            return Ok(Taken::Gone);
        }
        // Capacity may change during the adjacent-pickup delay. Prepare the
        // insertion before removing the exact handle, then commit both on this
        // exclusively owned world thread. Failed pickups preserve loot/timers.
        // This intentionally fixes the Kotlin/OpenRune remove-before-insert loss.
        let mut next = inventory.clone();
        let Ok(result) = next.insert(catalog, Insert::new(object.item as i32, object.count)) else {
            return Ok(Taken::NoSpace);
        };
        if !objects.delete(&self.object, clock, self.respawn_ticks) {
            return Ok(Taken::Gone);
        }
        *inventory = next;
        Ok(Taken::Added(result.completed))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn coords() -> CoordGrid {
        CoordGrid::new(3200, 3200, 0).unwrap()
    }
    fn drop_request() -> Drop {
        Drop {
            slot: 0,
            count: i32::MAX,
            coords: coords(),
            observer: 1,
            duration: 200,
            reveal: 100,
        }
    }
    #[test]
    fn failed_drop_keeps_inventory_and_private_merge_keeps_counts() {
        let mut objects = Objects::default();
        let mut catalog = Catalog::default();
        let mut inventory = Inventory::new(2, StackType::Normal, false);
        inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 1,
                    count: 30,
                    vars: 0,
                }),
            )
            .unwrap();
        catalog.dummy.insert(1);
        let before = inventory.clone();
        assert!(drop_slot(
            &mut objects,
            &mut inventory,
            &catalog,
            MapClock(0),
            drop_request()
        )
        .is_err());
        assert_eq!(inventory, before);
        assert_eq!(objects.count(), 0);
        catalog.dummy.clear();
        catalog.stackable.insert(1);
        let first = drop_slot(
            &mut objects,
            &mut inventory,
            &catalog,
            MapClock(0),
            drop_request(),
        )
        .unwrap();
        assert!(inventory.get(0).is_none());
        inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 1,
                    count: 20,
                    vars: 0,
                }),
            )
            .unwrap();
        let merged = drop_slot(
            &mut objects,
            &mut inventory,
            &catalog,
            MapClock(1),
            drop_request(),
        )
        .unwrap();
        assert_eq!(merged[0].id(), first[0].id());
        assert_eq!(merged[0].snapshot().count, 50);
    }
    #[test]
    fn competing_pickups_do_not_duplicate_and_private_objects_require_receiver() {
        let mut objects = Objects::default();
        let catalog = Catalog::default();
        let mut inventory = Inventory::new(2, StackType::Normal, false);
        let object = objects
            .add(
                Object {
                    coords: coords(),
                    item: 1,
                    count: 1,
                    scope: Scope::Private,
                    created: 0,
                    receiver: Some(1),
                    owner: None,
                },
                &catalog,
                MapClock(0),
                200,
                100,
            )
            .unwrap()
            .remove(0);
        assert!(matches!(
            Pickup::prepare(&object, 2, coords(), &inventory, &catalog, MapClock(0), 100),
            Err(Error::NotVisible)
        ));
        let a =
            Pickup::prepare(&object, 1, coords(), &inventory, &catalog, MapClock(0), 100).unwrap();
        let b =
            Pickup::prepare(&object, 1, coords(), &inventory, &catalog, MapClock(0), 100).unwrap();
        assert_eq!(
            a.finish(&mut objects, &mut inventory, &catalog, MapClock(0)),
            Ok(Taken::Added(1))
        );
        assert_eq!(
            b.finish(&mut objects, &mut inventory, &catalog, MapClock(0)),
            Ok(Taken::Gone)
        );
        assert_eq!(inventory.free_space(), 1);
    }
    #[test]
    fn stack_growth_during_pickup_is_revalidated_without_losing_or_duplicating_loot() {
        let mut objects = Objects::default();
        let mut catalog = Catalog::default();
        catalog.stackable.insert(1);
        let mut inventory = Inventory::new(1, StackType::Normal, false);
        inventory
            .insert(&catalog, Insert::new(1, i32::MAX - 5))
            .unwrap();
        let drop = Object {
            coords: coords(),
            item: 1,
            count: 5,
            scope: Scope::Private,
            created: 0,
            receiver: Some(1),
            owner: Some(1),
        };
        let handle = objects
            .add(drop.clone(), &catalog, MapClock(0), 200, 100)
            .unwrap()
            .remove(0);
        let pending = Pickup::prepare(
            &handle,
            1,
            coords().translate(1, 0, 0).unwrap(),
            &inventory,
            &catalog,
            MapClock(0),
            i32::MAX,
        )
        .unwrap();
        let merged = objects
            .add(
                Object { count: 10, ..drop },
                &catalog,
                MapClock(1),
                200,
                100,
            )
            .unwrap();
        assert_eq!(merged[0].id(), handle.id());
        let before = inventory.clone();
        objects.drain_events();
        assert_eq!(
            pending.finish_with_policy(
                &mut objects,
                &mut inventory,
                &catalog,
                MapClock(1),
                PickupPolicy::PreserveOnFailure
            ),
            Ok(Taken::NoSpace)
        );
        assert_eq!(inventory, before);
        assert_eq!(handle.snapshot().count, 15);
        assert_eq!(objects.count(), 1);
        assert!(objects.drain_events().is_empty());
        inventory.delete(&catalog, Delete::new(1, 10)).unwrap();
        let retry = Pickup::prepare(
            &handle,
            1,
            coords(),
            &inventory,
            &catalog,
            MapClock(2),
            i32::MAX,
        )
        .unwrap();
        assert_eq!(
            retry.finish_with_policy(
                &mut objects,
                &mut inventory,
                &catalog,
                MapClock(2),
                PickupPolicy::PreserveOnFailure
            ),
            Ok(Taken::Added(15))
        );
        assert_eq!(inventory.get(0).unwrap().count, i32::MAX);
        assert_eq!(objects.count(), 0);
        objects.process_durations(MapClock(201), &catalog).unwrap();
        assert_eq!(objects.count(), 0);
    }
    #[test]
    fn delayed_pickup_capacity_failure_preserves_item_and_original_expiry() {
        let mut objects = Objects::default();
        let catalog = Catalog::default();
        let mut inventory = Inventory::new(1, StackType::Normal, false);
        let object = objects
            .add(
                Object {
                    coords: coords(),
                    item: 1,
                    count: 1,
                    scope: Scope::Temporary,
                    created: 0,
                    receiver: None,
                    owner: None,
                },
                &catalog,
                MapClock(0),
                200,
                100,
            )
            .unwrap()
            .remove(0);
        let pickup = Pickup::prepare(
            &object,
            1,
            CoordGrid::new(3201, 3200, 0).unwrap(),
            &inventory,
            &catalog,
            MapClock(0),
            100,
        )
        .unwrap();
        assert_eq!(pickup.ready_at(), 1);
        objects.drain_events();
        inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 2,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        assert_eq!(
            pickup.finish_with_policy(
                &mut objects,
                &mut inventory,
                &catalog,
                MapClock(1),
                PickupPolicy::PreserveOnFailure
            ),
            Ok(Taken::NoSpace)
        );
        assert_eq!(objects.count(), 1);
        assert_eq!(inventory.get(0).unwrap().id, 2);
        assert!(objects.drain_events().is_empty());
        objects.process_durations(MapClock(199), &catalog).unwrap();
        assert_eq!(objects.at(coords()).next().unwrap().id(), object.id());
        objects.process_durations(MapClock(200), &catalog).unwrap();
        assert_eq!(objects.count(), 0);
    }
}
