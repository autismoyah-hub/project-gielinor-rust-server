//! Build a detached small constructed region from read-only normal map state.
//! Registry publication, allocation lifetimes and gameplay stamps belong to the server.
use crate::game::{Definitions, StaticLocation};
use gielinor_world::{
    collision::CollisionMap, loc_collision, region_rotation, tower_layout::Template, CoordGrid,
};

pub struct Built {
    pub collision: CollisionMap,
    pub locations: Vec<StaticLocation>,
    /// Locations whose rotated footprint moved their anchor to another zone.
    pub remapped: Vec<(CoordGrid, CoordGrid)>,
}

pub fn build_small(
    origin: CoordGrid,
    template: &Template,
    normal: &CollisionMap,
    locations: &[StaticLocation],
    definitions: &Definitions,
) -> Result<Built, String> {
    if origin.level() != 0
        || origin.x() % 8 != 0
        || origin.z() % 8 != 0
        || origin.x() < 8
        || origin.z() < 8
        || origin.x() + 136 > 16384
        || origin.z() + 136 > 16384
    {
        return Err("Constructed origin requires an aligned, padded 128-tile region".into());
    }
    let mut built = Built {
        collision: CollisionMap::default(),
        locations: Vec::new(),
        remapped: Vec::new(),
    };
    for (&(x, z, level), source) in template {
        if x >= 16
            || z >= 16
            || level > 3
            || source.x > 1023
            || source.z > 2047
            || source.level > 3
            || source.rotation > 3
        {
            return Err("Invalid constructed zone mapping".into());
        }
        let destination = CoordGrid::new(
            origin.x() + x as i32 * 8,
            origin.z() + z as i32 * 8,
            level as i32,
        )
        .map_err(|e| format!("{e:?}"))?;
        let sx = source.x as i32 * 8;
        let sz = source.z as i32 * 8;
        if normal.allocated(sx, sz, source.level as i32) {
            built
                .collision
                .allocate(destination.x(), destination.z(), level as i32);
        }
        for dx in 0..8 {
            for dz in 0..8 {
                let flags = normal.get(sx + dx, sz + dz, source.level as i32) & 0x80200000;
                if flags == 0 {
                    continue;
                }
                let (rx, rz) = region_rotation::translate(source.rotation, dx, dz, 8, 8, 1, 1)
                    .map_err(str::to_owned)?;
                built.collision.add(
                    destination.x() + rx,
                    destination.z() + rz,
                    level as i32,
                    flags,
                );
            }
        }
        for loc in locations.iter().filter(|loc| {
            loc.coords.x() >> 3 == source.x as i32
                && loc.coords.z() >> 3 == source.z as i32
                && loc.coords.level() == source.level as i32
        }) {
            let ty = definitions
                .locations
                .get(&loc.id)
                .ok_or_else(|| format!("Unknown copied location {}", loc.id))?;
            let (width, length) = if loc.angle & 1 == 0 {
                (ty.collision.width, ty.collision.length)
            } else {
                (ty.collision.length, ty.collision.width)
            };
            let (rx, rz) = region_rotation::translate(
                source.rotation,
                loc.coords.x() & 7,
                loc.coords.z() & 7,
                8,
                8,
                width as i32,
                length as i32,
            )
            .map_err(str::to_owned)?;
            let coords = destination
                .translate(rx, rz, 0)
                .map_err(|e| format!("{e:?}"))?;
            let angle = (loc.angle + source.rotation) & 3;
            loc_collision::toggle(
                &mut built.collision,
                coords,
                loc.shape,
                angle,
                ty.collision,
                true,
            )
            .map_err(str::to_owned)?;
            built.locations.push(StaticLocation {
                id: loc.id,
                coords,
                shape: loc.shape,
                angle,
            });
            if rx < 0 || rz < 0 {
                built.remapped.push((coords, loc.coords));
            }
        }
    }
    // RegionRegistry clears unmapped zones after loc footprints have been applied,
    // including collision allocated just outside the padded construction area.
    for level in 0..4 {
        for x in -1..=16 {
            for z in -1..=16 {
                if (x == -1 || x == 16) && (z == -1 || z == 16) {
                    continue;
                }
                if x < 0
                    || z < 0
                    || x == 16
                    || z == 16
                    || !template.contains_key(&(x as u8, z as u8, level))
                {
                    built.collision.deallocate(
                        origin.x() + x * 8,
                        origin.z() + z * 8,
                        level as i32,
                    );
                }
            }
        }
    }
    Ok(built)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locations::LocationType;
    use gielinor_world::tower_layout::Source;
    use std::collections::BTreeMap;
    fn definitions() -> Definitions {
        Definitions {
            inventories: BTreeMap::new(),
            items: BTreeMap::new(),
            locations: BTreeMap::from([(1, LocationType::decode(&[14, 3, 0]).unwrap())]),
            npcs: BTreeMap::new(),
            varps: BTreeMap::new(),
            varbits: BTreeMap::new(),
            stats: BTreeMap::new(),
        }
    }
    #[test]
    fn source_actor_flags_are_not_copied_and_sparse_terrain_stays_sparse() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let template = Template::from([(
            (6, 6, 1),
            Source {
                x: 404,
                z: 348,
                level: 0,
                rotation: 1,
            },
        )]);
        let mut normal = CollisionMap::default();
        normal.allocate(3232, 2784, 0);
        normal.set(3233, 2786, 0, 0x80200000 | 0x100 | 0x4000000);
        let built = build_small(origin, &template, &normal, &[], &definitions()).unwrap();
        assert_eq!(built.collision.get(10418, 118, 1), 0x80200000);
        assert_eq!(built.collision.get(10416, 112, 1), 0);
        assert!(!built.collision.allocated(10416, 112, 0));
        assert!(!built.collision.allocated(10408, 112, 1));
        assert_eq!(normal.get(3233, 2786, 0), 0x84200100);
    }
    #[test]
    fn rotated_oversize_loc_retains_origin_mapping_and_clears_unmapped_collision() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let template = Template::from([(
            (6, 6, 0),
            Source {
                x: 404,
                z: 348,
                level: 0,
                rotation: 1,
            },
        )]);
        let mut normal = CollisionMap::default();
        normal.allocate(3232, 2784, 0);
        let original = CoordGrid::new(3239, 2786, 0).unwrap();
        let loc = StaticLocation {
            id: 1,
            coords: original,
            shape: 10,
            angle: 0,
        };
        let built = build_small(origin, &template, &normal, &[loc], &definitions()).unwrap();
        let shifted = CoordGrid::new(10418, 110, 0).unwrap();
        assert_eq!(built.remapped, vec![(shifted, original)]);
        assert_eq!(built.locations[0].angle, 1);
        assert!(!built.collision.allocated(10418, 110, 0));
        assert_ne!(built.collision.get(10418, 112, 0) & 0x100, 0);
    }
}
