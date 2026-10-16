//! TowerFloor's copied zones and shell, independent of allocation and transport.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub x: u16,
    pub z: u16,
    pub level: u8,
    pub rotation: u8,
}
pub type Template = BTreeMap<(u8, u8, u8), Source>;

pub fn template() -> Template {
    let mut zones = Template::new();
    for (x, z, level, sx, sz) in [
        (4, 4, 0, 407, 408),
        (6, 4, 0, 388, 395),
        (8, 4, 0, 406, 408),
        (4, 6, 0, 404, 348),
        (6, 6, 0, 404, 348),
        (4, 8, 0, 407, 408),
        (6, 8, 0, 404, 348),
        (8, 8, 0, 404, 348),
        (6, 6, 1, 404, 348),
    ] {
        for dx in 0..2 {
            for dz in 0..2 {
                zones.insert(
                    (x + dx, z + dz, level),
                    Source {
                        x: sx,
                        z: sz,
                        level: 0,
                        rotation: 0,
                    },
                );
            }
        }
    }
    for (x, z, sx, sz) in [
        (8, 6, 404, 393),
        (8, 7, 404, 394),
        (9, 6, 405, 393),
        (9, 7, 405, 394),
    ] {
        zones.insert(
            (x, z, 0),
            Source {
                x: sx,
                z: sz,
                level: 0,
                rotation: 0,
            },
        );
    }
    zones
}

pub fn openings() -> BTreeSet<(u8, u8)> {
    [
        (55, 48),
        (56, 48),
        (55, 64),
        (56, 64),
        (48, 55),
        (48, 56),
        (64, 55),
        (64, 56),
        (39, 64),
        (40, 64),
        (39, 48),
        (40, 48),
        (71, 64),
        (72, 64),
        (71, 48),
        (72, 48),
        (48, 71),
        (48, 72),
        (64, 71),
        (64, 72),
        (48, 39),
        (48, 40),
        (64, 39),
        (64, 40),
    ]
    .into_iter()
    .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wall {
    pub x: u8,
    pub z: u8,
    pub shape: u8,
    pub angle: u8,
}
/// Preserve script insertion order: overlapping edges can replace the wall layer,
/// while each centrepiece is inserted just once to block the whole tile.
pub fn shell() -> Vec<Wall> {
    let gaps = openings();
    let mut blocked = BTreeSet::new();
    let mut result = Vec::new();
    let mut edge = |x, z, angle| {
        if gaps.contains(&(x, z)) {
            return;
        }
        result.push(Wall {
            x,
            z,
            shape: 0,
            angle,
        });
        if blocked.insert((x, z)) {
            result.push(Wall {
                x,
                z,
                shape: 10,
                angle: 0,
            });
        }
    };
    for x in 32..80 {
        edge(x, 32, 3);
        edge(x, 79, 1);
    }
    for z in 32..80 {
        edge(32, z, 0);
        edge(79, z, 2);
    }
    // The Cinderworks is an open sanctuary with visible service branches.
    // Keep the exterior boundary, but remove the old nine-room partition:
    // its full-tile wall models hid the player and every nearby landmark.
    result
}

/// FloorOneScript.ensureHubWalkable after copying/stripping locs and stamping
/// the shell. Preserve river clipping and wall flags while opening doorway lanes.
pub fn prepare_collision(
    map: &mut crate::collision::CollisionMap,
    origin: crate::CoordGrid,
) -> Result<(), &'static str> {
    if origin.level() != 0 || origin.x() + 79 >= 16384 || origin.z() + 79 >= 16384 {
        return Err("Invalid floor origin");
    }
    let gaps = openings();
    let walls: BTreeSet<_> = shell().into_iter()
        .filter(|wall| wall.shape == 10)
        .map(|wall| (wall.x as i32, wall.z as i32))
        .collect();
    for x in 32..80 {
        for z in 32..80 {
            for level in 0..=1 {
                map.allocate(origin.x() + x, origin.z() + z, level);
            }
        }
    }
    for x in 32..80 {
        for z in 32..80 {
            if x >= 72 && (48..=63).contains(&z) {
                continue;
            }
            let wall = walls.contains(&(x, z));
            if wall && !gaps.contains(&(x as u8, z as u8)) {
                map.remove(origin.x() + x, origin.z() + z, 0, 0x200000);
            } else {
                map.set(origin.x() + x, origin.z() + z, 0, 0);
            }
            if (48..=63).contains(&x) && (48..=63).contains(&z) {
                map.set(origin.x() + x, origin.z() + z, 1, 0);
            }
        }
    }
    for (x, z) in gaps {
        let (x, z) = (x as i32, z as i32);
        map.set(origin.x() + x, origin.z() + z, 0, 0);
        if x == 48 || x == 64 {
            for dx in [-1, 1] {
                map.set(origin.x() + x + dx, origin.z() + z, 0, 0);
            }
        }
        if z == 48 || z == 64 {
            for dz in [-1, 1] {
                map.set(origin.x() + x, origin.z() + z + dz, 0, 0);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floor_walkability_keeps_river_and_shell_but_clears_hub_and_doorway_lanes() {
        let origin = crate::CoordGrid::new(10368, 64, 0).unwrap();
        let mut map = crate::collision::CollisionMap::default();
        for level in 0..=1 {
            for x in 32..80 {
                for z in 32..80 {
                    map.set(origin.x() + x, origin.z() + z, level, 0x80200100);
                }
            }
        }
        prepare_collision(&mut map, origin).unwrap();
        let get = |x, z, l| map.get(origin.x() + x, origin.z() + z, l);
        assert_eq!(get(75, 55, 0), 0x80200100);
        assert_eq!(get(32, 40, 0), 0x80000100);
        assert_eq!(get(56, 56, 0), 0);
        assert_eq!(get(56, 56, 1), 0);
        assert_eq!(get(40, 47, 0), 0);
        assert_eq!(get(40, 48, 0), 0);
        assert_eq!(get(40, 49, 0), 0);
    }
    #[test]
    fn all_nine_rooms_are_connected_through_openings_and_outer_shell_is_closed() {
        let walls: BTreeSet<_> = shell()
            .into_iter()
            .filter(|w| w.shape == 10)
            .map(|w| (w.x, w.z))
            .collect();
        let mut reached = BTreeSet::from([(56u8, 56u8)]);
        let mut queue = std::collections::VecDeque::from([(56u8, 56u8)]);
        while let Some((x, z)) = queue.pop_front() {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let p = ((x as i32 + dx) as u8, (z as i32 + dz) as u8);
                assert!(
                    (32..80).contains(&p.0) && (32..80).contains(&p.1),
                    "shell leaked at {p:?}"
                );
                if !walls.contains(&p) && reached.insert(p) {
                    queue.push_back(p);
                }
            }
        }
        for x in [40, 56, 72] {
            for z in [40, 56, 72] {
                assert!(reached.contains(&(x, z)));
            }
        }
        assert!(openings().iter().all(|p| reached.contains(p)));
        assert_eq!(template().len(), 40);
    }
}

/// TowerMap.contains: the whole allocated square on planes zero and one.
pub fn contains(origin: crate::CoordGrid, coords: crate::CoordGrid) -> bool {
    (0..128).contains(&(coords.x() - origin.x()))
        && (0..128).contains(&(coords.z() - origin.z()))
        && coords.level() <= 1
}
