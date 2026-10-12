//! Construct an isolated world from already decoded, read-only cache maps.
use gielinor_cache::game::{Definitions, MapSquare, StaticLocation};
use gielinor_world::{
    areas::Areas,
    simulation::{NpcId, World},
    CoordGrid,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy)]
pub struct ObjectSpawn {
    pub id: u16,
    pub coords: CoordGrid,
    pub count: i32,
    pub respawn_ticks: u16,
}
pub struct LoadedWorld {
    pub world: World,
    pub areas: Areas,
    pub locations: Vec<StaticLocation>,
    pub npcs: Vec<NpcId>,
    /// Original spawn descriptors, alongside the active world registry.
    pub object_spawns: Vec<ObjectSpawn>,
    pub squares: BTreeSet<u16>,
}

/// Build locally and publish only on success; a bad map cannot partially mutate a running world.
pub fn build(definitions: &Definitions, maps: &[MapSquare]) -> Result<LoadedWorld, String> {
    let mut loaded = LoadedWorld {
        world: World::default(),
        areas: Areas::default(),
        locations: vec![],
        npcs: vec![],
        object_spawns: vec![],
        squares: BTreeSet::new(),
    };
    loaded.world.object_types = definitions.inventory_catalog();
    loaded.world.npc_options = gielinor_world::npc_options::Catalog {
        types: definitions
            .npcs
            .iter()
            .filter_map(|(&id, n)| {
                Some((
                    u16::try_from(id).ok()?,
                    gielinor_world::npc_options::Definition {
                        operations: n.ops.clone(),
                        varp: n.multi_varp,
                        varbit: n.multi_varbit,
                        fallback: n.multi_default,
                        morphs: n.multi.clone(),
                    },
                ))
            })
            .collect(),
        varps: definitions
            .varps
            .keys()
            .filter_map(|id| u16::try_from(*id).ok())
            .collect(),
        varbits: definitions
            .varbits
            .iter()
            .filter_map(|(&id, &bit)| Some((u16::try_from(id).ok()?, bit)))
            .collect(),
    };
    loaded.world.object_respawns = definitions
        .items
        .iter()
        .filter_map(|(&id, ty)| {
            u16::try_from(id)
                .ok()
                .map(|id| (id, ty.respawn_rate as i32))
        })
        .collect();
    loaded.world.location_approach = definitions
        .locations
        .iter()
        .map(|(&id, ty)| (id, ty.force_approach))
        .collect();
    loaded.world.location_types = definitions
        .locations
        .iter()
        .map(|(&id, ty)| (id, ty.collision))
        .collect();
    for map in maps {
        if !loaded.squares.insert(map.id) {
            return Err(format!("Duplicate map square {}", map.id));
        }
        map.terrain_collision(&mut loaded.world.collision);
    }
    // Locations can cross a square boundary. Later terrain must not erase their collision.
    for map in maps {
        loaded.locations.extend(
            map.location_collision(definitions, &mut loaded.world.collision)
                .map_err(|e| e.to_string())?,
        );
        if let Some(areas) = &map.areas {
            areas
                .register(map.id, &mut loaded.areas)
                .map_err(str::to_owned)?;
        }
        for spawn in &map.npcs {
            let ty = definitions
                .npcs
                .get(&(spawn.id as u32))
                .ok_or_else(|| format!("Unknown NPC {} in map {}", spawn.id, map.id))?;
            let coords = map
                .coords(spawn.x, spawn.z, spawn.level)
                .map_err(|e| e.to_string())?;
            let mut npc = ty
                .spawn(spawn.id, coords)
                .map_err(|e| format!("NPC {} in map {}: {e}", spawn.id, map.id))?;
            npc.respawns = true;
            let id = loaded
                .world
                .spawn_npc(npc)
                .map_err(|_| "NPC capacity exceeded".to_owned())?;
            loaded.npcs.push(id);
        }
        for object in &map.objects {
            let spawn = object.spawn;
            let ty = definitions
                .items
                .get(&(spawn.id as u32))
                .ok_or_else(|| format!("Unknown object {} in map {}", spawn.id, map.id))?;
            if object.count <= 0 {
                return Err(format!("Nonpositive object count in map {}", map.id));
            }
            let coords = map
                .coords(spawn.x, spawn.z, spawn.level)
                .map_err(|e| e.to_string())?;
            loaded
                .world
                .objects
                .add(
                    gielinor_world::ground::Object {
                        coords,
                        item: spawn.id,
                        count: object.count,
                        scope: gielinor_world::ground::Scope::Permanent,
                        created: 0,
                        receiver: None,
                        owner: None,
                    },
                    &loaded.world.object_types,
                    loaded.world.clock,
                    i32::MAX,
                    100,
                )
                .map_err(|e| format!("Object {} in map {}: {e:?}", spawn.id, map.id))?;
            loaded.object_spawns.push(ObjectSpawn {
                id: spawn.id,
                coords,
                count: object.count,
                respawn_ticks: ty.respawn_rate,
            });
        }
    }
    for loc in &loaded.locations {
        loaded.world.locations.load(
            gielinor_world::locations::Location::new(loc.coords, loc.id, loc.shape, loc.angle)
                .map_err(str::to_owned)?,
        );
    }
    loaded.world.areas = loaded.areas.clone();
    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_cache::{
        locations::LocationType,
        maps::{Location, Spawn, Tiles},
        npcs::NpcType,
    };
    use std::collections::BTreeMap;
    fn definitions() -> Definitions {
        Definitions {
            inventories: BTreeMap::new(),
            items: BTreeMap::new(),
            locations: BTreeMap::from([(1, LocationType::decode(&[14, 2, 0]).unwrap())]),
            npcs: BTreeMap::from([(1, NpcType::decode(&[0]).unwrap())]),
            varps: BTreeMap::new(),
            varbits: BTreeMap::new(),
            stats: BTreeMap::new(),
        }
    }
    fn square(id: u16) -> MapSquare {
        MapSquare {
            id,
            tiles: Tiles::decode(&[0, 82, 0, 0]).unwrap(),
            locations: vec![],
            npcs: vec![],
            objects: vec![],
            areas: None,
        }
    }
    #[test]
    fn terrain_precedes_cross_square_location_collision_and_npc_admission() {
        let defs = definitions();
        let mut west = square(0x3232);
        let east = square(0x3332);
        west.locations.push(Location {
            id: 1,
            x: 63,
            z: 0,
            level: 0,
            shape: 10,
            angle: 0,
        });
        west.npcs.push(Spawn {
            id: 1,
            x: 1,
            z: 1,
            level: 0,
        });
        let loaded = build(&defs, &[west, east]).unwrap();
        assert_ne!(loaded.world.collision.get(3264, 3200, 0) & 0x100, 0);
        assert_eq!(loaded.locations.len(), 1);
        assert_eq!(loaded.npcs.len(), 1);
        assert_eq!(
            loaded.world.npc(loaded.npcs[0]).unwrap().movement.coords,
            CoordGrid::new(3201, 3201, 0).unwrap()
        );
    }
    #[test]
    fn duplicate_maps_and_unsupported_npc_modes_fail_before_publication() {
        let mut defs = definitions();
        assert!(build(&defs, &[square(0), square(0)]).is_err());
        defs.npcs.insert(1, NpcType::decode(&[201, 3, 0]).unwrap());
        let mut map = square(0);
        map.npcs.push(Spawn {
            id: 1,
            x: 0,
            z: 0,
            level: 0,
        });
        assert!(build(&defs, &[map])
            .err()
            .unwrap()
            .contains("not implemented"));
    }
}
