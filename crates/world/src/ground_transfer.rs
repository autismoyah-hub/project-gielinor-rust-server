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
        if clock.0 < self.ready_at {
            return Err(Error::NotReady);
        }
        if !objects.delete(&self.object, clock, self.respawn_ticks) {
            return Ok(Taken::Gone);
        }
        let object = self.object.snapshot();
        // Preserve the original ordering: takeClose removes before the second transaction.
        // If inventory capacity changed during the delay, Kotlin loses the ground object.
        // Expose this result explicitly rather than silently reporting success.
        match inventory.insert(catalog, Insert::new(object.item as i32, object.count)) {
            Ok(result) => Ok(Taken::Added(result.completed)),
            Err(_) => Ok(Taken::RemovedWithoutSpace),
        }
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
    fn delayed_pickup_preserves_reference_second_transaction_failure() {
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
            pickup.finish(&mut objects, &mut inventory, &catalog, MapClock(1)),
            Ok(Taken::RemovedWithoutSpace)
        );
        assert_eq!(objects.count(), 0);
        assert_eq!(inventory.get(0).unwrap().id, 2);
    }
}
