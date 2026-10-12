//! Location registry in world coordinates. Constructed regions must explicitly
//! register their bounds after their source locations have been transformed.
use crate::{ground::zone_base, loc_map::ByteMap, CoordGrid};
use std::collections::HashMap;
pub const DELETED_ID: u32 = 131071;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub coords: CoordGrid,
    pub id: u32,
    pub shape: u8,
    pub angle: u8,
    pub layer: u8,
}
impl Location {
    pub fn new(coords: CoordGrid, id: u32, shape: u8, angle: u8) -> Result<Self, &'static str> {
        if coords == CoordGrid::NULL || id > DELETED_ID || shape > 22 || angle > 3 {
            return Err("Invalid location");
        }
        Ok(Self {
            coords,
            id,
            shape,
            angle,
            layer: layer(shape).unwrap(),
        })
    }
    fn key(self) -> u8 {
        (self.layer << 6) | ((self.coords.x() as u8 & 7) << 3) | (self.coords.z() as u8 & 7)
    }
    fn packed(self) -> u32 {
        self.id | ((self.shape as u32) << 17) | ((self.angle as u32) << 22)
    }
}
fn layer(shape: u8) -> Option<u8> {
    Some(match shape {
        0..=3 => 0,
        4..=8 => 1,
        9..=21 => 2,
        22 => 3,
        _ => return None,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Add(Location),
    Delete(Location),
    Animate { location: Location, sequence: u16 },
}
impl Event {
    pub fn location(self) -> Location {
        match self {
            Self::Add(v) | Self::Delete(v) | Self::Animate { location: v, .. } => v,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Changed {
    Map,
    Spawned,
    Missing,
}
#[derive(Default, Debug)]
pub struct Mutation {
    pub collision: Vec<(Location, bool)>,
    pub updates: Vec<Event>,
}
#[derive(Default)]
pub struct Locations {
    map: HashMap<CoordGrid, ByteMap<Location>>,
    spawned: HashMap<CoordGrid, ByteMap<Location>>,
    constructed: Vec<(CoordGrid, u16)>,
}
impl Locations {
    /// Authorize mutation only after the region owner has prepared transformed
    /// locations/collision. Floor 1 is pinned; dynamic lifetime removal is separate.
    pub fn register_constructed(
        &mut self,
        origin: CoordGrid,
        size: u16,
    ) -> Result<(), &'static str> {
        let length = size as i32;
        if !matches!(size, 128 | 256)
            || origin.level() != 0
            || origin.x() < 6432
            || origin.x() % 8 != 0
            || origin.z() % 8 != 0
            || origin.x() + length > 16384
            || origin.z() + length > 16384
        {
            return Err("Invalid constructed location region");
        }
        if self.constructed.iter().any(|(other, other_size)| {
            origin.x() < other.x() + *other_size as i32
                && other.x() < origin.x() + length
                && origin.z() < other.z() + *other_size as i32
                && other.z() < origin.z() + length
        }) {
            return Err("Overlapping constructed location regions");
        }
        self.constructed.push((origin, size));
        Ok(())
    }
    /// Roll back only the affected spawn zone if collision validation fails.
    pub fn change(
        &mut self,
        loc: Location,
        add: bool,
        apply: impl FnOnce(&Mutation) -> Result<(), &'static str>,
    ) -> Result<(Changed, Mutation), &'static str> {
        let zone = zone_base(loc.coords);
        let before = self.spawned.get(&zone).cloned();
        let result = if add {
            self.add(loc)?
        } else {
            self.delete(loc)
        };
        if let Err(error) = apply(&result.1) {
            if let Some(before) = before {
                self.spawned.insert(zone, before);
            } else {
                self.spawned.remove(&zone);
            }
            return Err(error);
        }
        Ok(result)
    }
    /// Map loader has already applied collision. Later entries on a layer replace earlier ones.
    pub fn load(&mut self, loc: Location) {
        self.map
            .entry(zone_base(loc.coords))
            .or_default()
            .insert(loc.key(), loc);
    }
    pub fn spawned(&self, zone: CoordGrid) -> impl Iterator<Item = Location> + '_ {
        self.spawned
            .get(&zone_base(zone))
            .into_iter()
            .flat_map(|m| m.entries().map(|(_, v)| v))
    }
    pub fn all(&self, zone: CoordGrid) -> Vec<Location> {
        let zone = zone_base(zone);
        let mut processed = [false; 256];
        let mut result = Vec::new();
        if let Some(spawns) = self.spawned.get(&zone) {
            for (k, v) in spawns.entries() {
                processed[k as usize] = true;
                if v.id != DELETED_ID {
                    result.push(v);
                }
            }
        }
        if let Some(map) = self.map.get(&zone) {
            for (k, v) in map.entries() {
                if !processed[k as usize] {
                    result.push(v);
                }
            }
        }
        result
    }
    pub fn find_type(&self, coords: CoordGrid, id: u32) -> Option<Location> {
        let zone = zone_base(coords);
        if let Some(found) = self.spawned.get(&zone).and_then(|m| {
            m.entries()
                .map(|(_, v)| v)
                .find(|v| v.coords == coords && v.id == id)
        }) {
            return (found.id != DELETED_ID).then_some(found);
        }
        let found = self
            .map
            .get(&zone)?
            .entries()
            .map(|(_, v)| v)
            .find(|v| v.coords == coords && v.id == id)?;
        self.spawned
            .get(&zone)
            .and_then(|m| m.get(found.key()))
            .is_none()
            .then_some(found)
    }
    pub fn find_shape(&self, coords: CoordGrid, shape: u8) -> Option<Location> {
        let key = (layer(shape)? << 6) | ((coords.x() as u8 & 7) << 3) | (coords.z() as u8 & 7);
        let zone = zone_base(coords);
        let spawned = self.spawned.get(&zone).and_then(|m| m.get(key));
        if let Some(found) = spawned.filter(|v| v.shape == shape) {
            return (found.id != DELETED_ID).then_some(found);
        }
        let found = self.map.get(&zone)?.get(key).filter(|v| v.shape == shape)?;
        spawned.is_none().then_some(found)
    }
    pub fn add(&mut self, loc: Location) -> Result<(Changed, Mutation), &'static str> {
        if loc.coords.x() >= 6432
            && !self.constructed.iter().any(|(origin, size)| {
                (0..*size as i32).contains(&(loc.coords.x() - origin.x()))
                    && (0..*size as i32).contains(&(loc.coords.z() - origin.z()))
            })
        {
            return Err("Instanced location requires region registry");
        }
        let zone = zone_base(loc.coords);
        let key = loc.key();
        let mut mutation = Mutation::default();
        let exact = self
            .map
            .get(&zone)
            .and_then(|m| m.get(key))
            .is_some_and(|v| v.packed() == loc.packed());
        let spawned = self.spawned.entry(zone).or_default();
        if let Some(old) = spawned.remove(key).filter(|v| v.packed() != 0) {
            mutation.collision.push((old, false));
        }
        if !exact {
            spawned.insert(key, loc);
        }
        mutation.collision.push((loc, true));
        mutation.updates.push(Event::Add(loc));
        Ok((
            if exact {
                Changed::Map
            } else {
                Changed::Spawned
            },
            mutation,
        ))
    }
    pub fn delete(&mut self, loc: Location) -> (Changed, Mutation) {
        let zone = zone_base(loc.coords);
        let key = loc.key();
        let mut mutation = Mutation::default();
        let map = self.map.get(&zone).and_then(|m| m.get(key));
        if let Some(spawned) = self.spawned.get_mut(&zone) {
            if spawned
                .get(key)
                .is_some_and(|v| v.packed() != 0 && v.packed() == loc.packed())
            {
                spawned.remove(key);
                mutation.collision.push((loc, false));
                if let Some(masked) = map {
                    mutation.collision.push((masked, true));
                    mutation.updates.push(Event::Add(masked));
                } else {
                    mutation.updates.push(Event::Delete(loc));
                }
                return (Changed::Spawned, mutation);
            }
        }
        if map.is_some_and(|v| v.packed() == loc.packed()) {
            self.spawned.entry(zone).or_default().insert(
                key,
                Location {
                    id: DELETED_ID,
                    ..loc
                },
            );
            mutation.collision.push((loc, false));
            mutation.updates.push(Event::Delete(loc));
            (Changed::Map, mutation)
        } else {
            (Changed::Missing, mutation)
        }
    }
}

#[cfg(test)]
mod constructed_tests {
    use super::*;
    #[test]
    fn only_registered_region_accepts_mutations_and_cache_deletes_survive() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let old = Location::new(origin.translate(56, 56, 0).unwrap(), 100, 10, 0).unwrap();
        let mut locations = Locations::default();
        locations.load(old);
        let replacement = Location::new(old.coords, 101, 10, 0).unwrap();
        assert!(locations.add(replacement).is_err());
        locations.register_constructed(origin, 128).unwrap();
        assert!(locations
            .register_constructed(origin.translate(120, 0, 0).unwrap(), 128)
            .is_err());
        assert_eq!(locations.delete(old).0, Changed::Map);
        assert!(locations.find_shape(old.coords, 10).is_none());
        assert!(locations
            .spawned(old.coords)
            .any(|loc| loc.id == DELETED_ID));
        locations.add(replacement).unwrap();
        assert_eq!(locations.find_shape(old.coords, 10), Some(replacement));
        let upper = Location::new(origin.translate(127, 127, 3).unwrap(), 102, 10, 0).unwrap();
        assert!(locations.add(upper).is_ok());
        let outside = Location::new(origin.translate(128, 127, 0).unwrap(), 102, 10, 0).unwrap();
        assert!(locations.add(outside).is_err());
        assert!(locations
            .register_constructed(CoordGrid::new(3200, 3200, 0).unwrap(), 128)
            .is_err());
    }
}
