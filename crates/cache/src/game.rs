//! Read-only game-cache loading. No cache packing or repair is performed here.
use crate::{
    archive::Archive,
    definitions::{InventoryType, ItemType, StatType},
    disk::DiskStore,
    invalid,
    locations::LocationType,
    maps::{self, Tiles},
    npcs::NpcType,
    variables,
};
use gielinor_protocol::buffer::DecodeError;
use gielinor_world::{
    inventory::Catalog,
    loc_collision,
    variables::{Varbit, Varp},
    CoordGrid,
};
use std::{collections::BTreeMap, io, path::Path};

pub struct GameCache {
    store: DiskStore,
    archives: BTreeMap<u8, Archive>,
    limit: usize,
}
impl GameCache {
    pub fn open(path: &Path, limit: usize) -> io::Result<Self> {
        Ok(Self {
            store: DiskStore::open(path, limit)?,
            archives: BTreeMap::new(),
            limit,
        })
    }
    fn archive(&mut self, id: u8) -> io::Result<()> {
        if !self.archives.contains_key(&id) {
            self.archives
                .insert(id, Archive::load(&mut self.store, id, self.limit)?);
        }
        Ok(())
    }
    pub fn groups(&mut self, archive: u8) -> io::Result<Vec<u32>> {
        self.archive(archive)?;
        Ok(self.archives[&archive]
            .index
            .groups
            .keys()
            .copied()
            .collect())
    }
    pub fn files(
        &mut self,
        archive: u8,
        group: u32,
        key: Option<[u32; 4]>,
    ) -> io::Result<Option<BTreeMap<u32, Vec<u8>>>> {
        self.archive(archive)?;
        self.archives[&archive].files(&mut self.store, group, key)
    }
    /// Missing archive indexes have zero CRC, as in the reference checksum table.
    pub fn checksums(&mut self) -> io::Result<[u32; 23]> {
        let mut values = [0; 23];
        for (id, value) in values.iter_mut().enumerate() {
            if self.store.read(255, id as u32)?.is_some() {
                self.archive(id as u8)?;
                *value = self.archives[&(id as u8)].index_checksum;
            }
        }
        Ok(values)
    }
    fn definitions<T>(
        &mut self,
        group: u32,
        decode: impl Fn(u32, &[u8]) -> Result<T, DecodeError>,
    ) -> io::Result<BTreeMap<u32, T>> {
        let files = self
            .files(2, group, None)?
            .ok_or_else(|| invalid(&format!("Missing config group {group}")))?;
        files
            .into_iter()
            .map(|(id, bytes)| {
                decode(id, &bytes)
                    .map(|d| (id, d))
                    .map_err(|e| invalid(&format!("Config {group}:{id}: {e:?}")))
            })
            .collect()
    }
    pub fn headbars(&mut self) -> io::Result<BTreeMap<u32, crate::headbars::HeadbarType>> {
        self.definitions(33, |_, b| crate::headbars::HeadbarType::decode(b))
    }
    pub fn db_rows(&mut self) -> io::Result<BTreeMap<u32, crate::db::Row>> {
        self.definitions(38, |_, b| crate::db::Row::decode(b))
    }
    pub fn db_tables(&mut self) -> io::Result<BTreeMap<u32, crate::db::Table>> {
        self.definitions(39, |_, b| crate::db::Table::decode(b))
    }
    /// MusicRepository checks actual AreaType membership, not symbol coverage.
    pub fn area_ids(&mut self) -> io::Result<std::collections::BTreeSet<u16>> {
        let types = self.definitions(18, |id, bytes| {
            let mut reader = gielinor_protocol::buffer::Reader::new(bytes);
            while reader.remaining() > 0 {
                match reader.u8()? {
                    0 => break,
                    200 => {
                        reader.u32()?;
                    }
                    _ => return Err(DecodeError::InvalidValue("unknown area opcode")),
                }
            }
            u16::try_from(id).map_err(|_| DecodeError::InvalidValue("area id"))
        })?;
        Ok(types.into_values().collect())
    }
    pub fn sequences(&mut self) -> io::Result<BTreeMap<u32, crate::sequences::SequenceType>> {
        self.definitions(12, |_, b| crate::sequences::SequenceType::decode(b))
    }
    pub fn definitions_all(&mut self) -> io::Result<Definitions> {
        Ok(Definitions {
            inventories: self.definitions(5, |_, b| InventoryType::decode(b))?,
            locations: self.definitions(6, |_, b| LocationType::decode(b))?,
            npcs: self.definitions(9, |_, b| NpcType::decode(b))?,
            items: self.definitions(10, |_, b| ItemType::decode(b))?,
            varbits: self.definitions(14, |_, b| variables::varbit(b))?,
            varps: self.definitions(16, |id, b| {
                variables::varp(
                    u16::try_from(id).map_err(|_| DecodeError::InvalidValue("varp id"))?,
                    b,
                )
            })?,
            stats: self.definitions(200, |_, b| StatType::decode(b))?,
        })
    }
    pub fn names(&mut self) -> io::Result<crate::names::Names> {
        let mut names = crate::names::Names::default();
        for group in self.groups(24)? {
            let files = self
                .files(24, group, None)?
                .ok_or_else(|| invalid("Missing game-value group"))?;
            names
                .group(group, &files)
                .map_err(|e| invalid(&format!("Game-value group {group}: {e:?}")))?;
        }
        Ok(names)
    }
    pub fn parameters(&mut self) -> io::Result<BTreeMap<u32, crate::parameters::ParameterType>> {
        self.definitions(11, |_, bytes| {
            crate::parameters::ParameterType::decode(bytes)
        })
    }
    pub fn enums(&mut self) -> io::Result<BTreeMap<u32, crate::enums::EnumType>> {
        Ok(self
            .definitions(8, |_, bytes| crate::enums::EnumType::decode(bytes))?
            .into_iter()
            .filter_map(|(id, value)| value.map(|value| (id, value)))
            .collect())
    }
    pub fn components(&mut self) -> io::Result<BTreeMap<u32, crate::components::ComponentType>> {
        let mut components = BTreeMap::new();
        for interface in self.groups(3)? {
            if interface > u16::MAX as u32 {
                return Err(invalid("Interface id exceeds component packing"));
            }
            let files = self
                .files(3, interface, None)?
                .ok_or_else(|| invalid("Missing interface group"))?;
            for (child, bytes) in files {
                if child > u16::MAX as u32 || components.len() >= 100000 {
                    return Err(invalid("Component limit"));
                }
                let id = (interface << 16) | child;
                let value =
                    crate::components::ComponentType::decode(id, &bytes).map_err(|error| {
                        invalid(&format!("Component {interface}:{child}: {error:?}"))
                    })?;
                components.insert(id, value);
            }
        }
        Ok(components)
    }
    pub fn map_square(&mut self, square: u16) -> io::Result<Option<MapSquare>> {
        let Some(mut files) = self.files(5, square as u32, None)? else {
            return Ok(None);
        };
        let (Some(terrain), Some(locations)) = (files.remove(&0), files.remove(&1)) else {
            return Ok(None);
        };
        if terrain.len() < 1024 {
            return Ok(None);
        }
        let decode = || -> Result<MapSquare, DecodeError> {
            Ok(MapSquare {
                id: square,
                tiles: Tiles::decode(&terrain)?,
                locations: maps::locations(&locations, 1_000_000)?,
                npcs: files
                    .get(&100)
                    .map(|b| maps::npcs(b))
                    .transpose()?
                    .unwrap_or_default(),
                objects: files
                    .get(&101)
                    .map(|b| maps::objects(b))
                    .transpose()?
                    .unwrap_or_default(),
                areas: files
                    .get(&102)
                    .map(|b| crate::areas::MapAreas::decode(b))
                    .transpose()?,
            })
        };
        decode()
            .map(Some)
            .map_err(|e| invalid(&format!("Map {square}: {e:?}")))
    }
}
pub struct Definitions {
    pub inventories: BTreeMap<u32, InventoryType>,
    pub locations: BTreeMap<u32, LocationType>,
    pub npcs: BTreeMap<u32, NpcType>,
    pub items: BTreeMap<u32, ItemType>,
    pub varbits: BTreeMap<u32, Varbit>,
    pub varps: BTreeMap<u32, Varp>,
    pub stats: BTreeMap<u32, StatType>,
}
impl Definitions {
    pub fn inventory_catalog(&self) -> Catalog {
        let mut catalog = Catalog::default();
        for (&id, item) in &self.items {
            item.register(id as i32, &mut catalog);
        }
        catalog
    }
}
pub struct MapSquare {
    pub id: u16,
    pub tiles: Tiles,
    pub locations: Vec<maps::Location>,
    pub npcs: Vec<maps::Spawn>,
    pub objects: Vec<maps::GroundObject>,
    pub areas: Option<crate::areas::MapAreas>,
}
#[derive(Clone, Copy, Debug)]
pub struct StaticLocation {
    pub id: u32,
    pub coords: CoordGrid,
    pub shape: u8,
    pub angle: u8,
}
impl MapSquare {
    pub fn coords(&self, x: u8, z: u8, level: u8) -> io::Result<CoordGrid> {
        CoordGrid::new(
            ((self.id >> 8) as i32) * 64 + x as i32,
            ((self.id & 255) as i32) * 64 + z as i32,
            level as i32,
        )
        .map_err(|_| invalid("Map coordinate outside world"))
    }
    /// Apply all terrain before locations, since terrain assignment replaces flags.
    pub fn terrain_collision(&self, collision: &mut gielinor_world::collision::CollisionMap) {
        self.tiles.apply_collision(collision, self.id);
    }
    pub fn location_collision(
        &self,
        definitions: &Definitions,
        collision: &mut gielinor_world::collision::CollisionMap,
    ) -> io::Result<Vec<StaticLocation>> {
        let mut locations = Vec::with_capacity(self.locations.len());
        for &loc in &self.locations {
            let Some(level) = self.tiles.visual_level(loc) else {
                continue;
            };
            let ty = definitions.locations.get(&loc.id).ok_or_else(|| {
                invalid(&format!("Unknown location {} in map {}", loc.id, self.id))
            })?;
            let coords = self.coords(loc.x, loc.z, level)?;
            loc_collision::toggle(collision, coords, loc.shape, loc.angle, ty.collision, true)
                .map_err(invalid)?;
            locations.push(StaticLocation {
                id: loc.id,
                coords,
                shape: loc.shape,
                angle: loc.angle,
            });
        }
        Ok(locations)
    }
}
