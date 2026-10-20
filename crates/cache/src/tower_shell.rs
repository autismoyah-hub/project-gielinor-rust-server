//! Detached FloorOneScript startup stage through ensureHubWalkable.
//! Removed cache locations are retained for later client zone-delete messages.
use crate::{
    constructed::Built,
    game::{Definitions, StaticLocation},
};
use gielinor_world::{loc_collision, tower_layout, CoordGrid};
use std::collections::BTreeMap;

pub struct Shell {
    pub map: Built,
    /// Cache template before script mutations, needed to derive zone deltas.
    pub base: Vec<StaticLocation>,
    pub removed: Vec<StaticLocation>,
}

impl Shell {
    /// FloorOneScript.loc uses centrepiece layer 2 and west orientation. Keep
    /// collision removal/addition in script order, including repeated stamps.
    pub fn stamp(
        &mut self,
        coords: CoordGrid,
        id: u32,
        definitions: &Definitions,
    ) -> Result<(), String> {
        self.stamp_shaped(coords, id, 10, 0, definitions)
    }
    /// Place the cache-supported scene shape and replace only its own layer.
    pub fn stamp_shaped(
        &mut self,
        coords: CoordGrid,
        id: u32,
        shape: u8,
        angle: u8,
        definitions: &Definitions,
    ) -> Result<(), String> {
        if shape > 22 || angle > 3 {
            return Err("Invalid scene shape/orientation".into());
        }
        let layer = |shape: u8| match shape {
            0..=3 => 0,
            4..=8 => 1,
            9..=21 => 2,
            _ => 3,
        };
        let ty = definitions
            .locations
            .get(&id)
            .ok_or_else(|| format!("Missing tower location {id}"))?;
        if let Some(index) = self
            .map
            .locations
            .iter()
            .position(|loc| loc.coords == coords && layer(loc.shape) == layer(shape))
        {
            let old = &self.map.locations[index];
            let old_type = definitions
                .locations
                .get(&old.id)
                .ok_or_else(|| format!("Missing replaced tower location {}", old.id))?;
            loc_collision::toggle(
                &mut self.map.collision,
                coords,
                old.shape,
                old.angle,
                old_type.collision,
                false,
            )?;
            self.map.locations.remove(index);
        }
        loc_collision::toggle(
            &mut self.map.collision,
            coords,
            shape,
            angle,
            ty.collision,
            true,
        )?;
        self.map.locations.push(StaticLocation {
            id,
            coords,
            shape,
            angle,
        });
        Ok(())
    }
}

pub fn prepare(
    mut map: Built,
    origin: CoordGrid,
    wall: u32,
    definitions: &Definitions,
) -> Result<Shell, String> {
    let base = map.locations.clone();
    let wall_type = definitions
        .locations
        .get(&wall)
        .ok_or("Missing tower room-wall definition")?;
    // Work on detached state: a failure never partially publishes world collision.
    let mut retained = Vec::new();
    let mut removed = Vec::new();
    for loc in map.locations.drain(..) {
        let x = loc.coords.x() - origin.x();
        let z = loc.coords.z() - origin.z();
        if loc.coords.level() <= 1 && (32..80).contains(&x) && (32..80).contains(&z) {
            let ty = definitions
                .locations
                .get(&loc.id)
                .ok_or_else(|| format!("Missing copied location {}", loc.id))?;
            loc_collision::toggle(
                &mut map.collision,
                loc.coords,
                loc.shape,
                loc.angle,
                ty.collision,
                false,
            )?;
            removed.push(loc);
        } else {
            retained.push(loc);
        }
    }
    map.locations = retained;
    // LocRepository replaces an existing loc in the same tile/layer. In
    // particular corner wall orientations follow the script's insertion order.
    let mut stamped = BTreeMap::new();
    for entry in tower_layout::shell() {
        let coords = origin
            .translate(entry.x as i32, entry.z as i32, 0)
            .map_err(|e| format!("{e:?}"))?;
        let key = (entry.x, entry.z, entry.shape);
        if let Some(previous) = stamped.insert(
            key,
            StaticLocation {
                id: wall,
                coords,
                shape: entry.shape,
                angle: entry.angle,
            },
        ) {
            loc_collision::toggle(
                &mut map.collision,
                coords,
                previous.shape,
                previous.angle,
                wall_type.collision,
                false,
            )?;
        }
        loc_collision::toggle(
            &mut map.collision,
            coords,
            entry.shape,
            entry.angle,
            wall_type.collision,
            true,
        )?;
    }
    map.locations.extend(stamped.into_values());
    tower_layout::prepare_collision(&mut map.collision, origin)?;
    Ok(Shell { map, base, removed })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locations::LocationType;
    use gielinor_world::collision::CollisionMap;
    #[test]
    fn strips_copied_layers_and_preserves_deletions_and_doorways() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let at = |x, z, l| origin.translate(x, z, l).unwrap();
        let definitions = Definitions {
            inventories: Default::default(),
            items: Default::default(),
            npcs: Default::default(),
            varps: Default::default(),
            varbits: Default::default(),
            stats: Default::default(),
            locations: BTreeMap::from([(1, LocationType::decode(&[0]).unwrap())]),
        };
        let mut collision = CollisionMap::default();
        collision.set(at(75, 55, 0).x(), at(75, 55, 0).z(), 0, 0x200000);
        let map = Built {
            collision,
            remapped: vec![],
            locations: vec![
                StaticLocation {
                    id: 1,
                    coords: at(55, 55, 0),
                    shape: 10,
                    angle: 0,
                },
                StaticLocation {
                    id: 1,
                    coords: at(55, 55, 1),
                    shape: 22,
                    angle: 0,
                },
                StaticLocation {
                    id: 1,
                    coords: at(31, 55, 0),
                    shape: 10,
                    angle: 0,
                },
            ],
        };
        let shell = prepare(map, origin, 1, &definitions).unwrap();
        assert_eq!(shell.removed.len(), 2);
        assert!(shell
            .map
            .locations
            .iter()
            .any(|l| l.coords == at(31, 55, 0)));
        assert!(!shell
            .map
            .locations
            .iter()
            .any(|l| l.coords == at(55, 55, 0)));
        for (x, z) in tower_layout::openings() {
            let p = at(x as i32, z as i32, 0);
            assert!(!shell.map.locations.iter().any(|l| l.coords == p));
            assert_eq!(shell.map.collision.get(p.x(), p.z(), 0), 0);
        }
        assert_ne!(
            shell
                .map
                .collision
                .get(at(32, 40, 0).x(), at(32, 40, 0).z(), 0)
                & 0x100,
            0
        );
        assert_eq!(
            shell
                .map
                .collision
                .get(at(75, 55, 0).x(), at(75, 55, 0).z(), 0)
                & 0x200000,
            0x200000
        );
        let corner = shell
            .map
            .locations
            .iter()
            .filter(|l| l.coords == at(32, 32, 0))
            .collect::<Vec<_>>();
        assert_eq!(corner.len(), 2);
        assert_eq!(corner.iter().find(|l| l.shape == 0).unwrap().angle, 0);
    }
    #[test]
    fn wall_decoration_keeps_supporting_masonry_and_its_real_collision() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let coords = origin.translate(54, 51, 0).unwrap();
        let definitions = Definitions {
            inventories: Default::default(),
            items: Default::default(),
            npcs: Default::default(),
            varps: Default::default(),
            varbits: Default::default(),
            stats: Default::default(),
            locations: BTreeMap::from([
                (1, LocationType::decode(&[0]).unwrap()),
                (2, LocationType::decode(&[0]).unwrap()),
            ]),
        };
        let mut shell = Shell {
            map: Built {
                collision: CollisionMap::default(),
                locations: vec![],
                remapped: vec![],
            },
            base: vec![],
            removed: vec![],
        };
        shell.stamp_shaped(coords, 1, 0, 3, &definitions).unwrap();
        let before = shell.map.collision.get(coords.x(), coords.z(), 0);
        assert!(shell.stamp_shaped(coords, 2, 23, 0, &definitions).is_err());
        assert!(shell.stamp_shaped(coords, 2, 10, 4, &definitions).is_err());
        assert_eq!(shell.map.locations.len(), 1);
        shell.stamp_shaped(coords, 2, 4, 3, &definitions).unwrap();
        assert_eq!(shell.map.locations.len(), 2);
        assert_eq!(shell.map.collision.get(coords.x(), coords.z(), 0), before);
        assert_ne!(before & 32, 0);
        shell.stamp_shaped(coords, 1, 4, 2, &definitions).unwrap();
        assert_eq!(shell.map.locations.len(), 2);
        assert!(shell
            .map
            .locations
            .iter()
            .any(|l| l.shape == 0 && l.angle == 3));
    }
}
