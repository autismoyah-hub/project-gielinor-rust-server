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
    // A cache-authored, flat charcoal stone tile with a matching ninth border.
    // Repeating the old overlay-rich zones produced conspicuous gold seams.
    let stone = Source {
        x: 364,
        z: 1245,
        level: 0,
        rotation: 0,
    };
    for x in 4..10 {
        for z in 4..10 {
            zones.insert((x, z, 0), stone);
        }
    }
    // Equal-height, unpatterned materials distinguish the service courts without
    // restoring the prototype's internal walls or repeated overlay seams.
    let earth = Source {
        x: 321,
        z: 649,
        ..stone
    };
    let moss = Source {
        x: 212,
        z: 571,
        ..stone
    };
    let ash = Source {
        x: 242,
        z: 620,
        ..stone
    };
    for x in 4..6 {
        for z in 4..6 {
            zones.insert((x, z, 0), moss);
        }
        for z in 6..8 {
            zones.insert((x, z, 0), earth);
        }
        for z in 8..10 {
            zones.insert((x, z, 0), moss);
        }
    }
    for x in 6..8 {
        for z in 8..10 {
            zones.insert((x, z, 0), ash);
        }
    }
    // Retain the actual river and its bank for net fishing.
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
impl Wall {
    /// Lower southern room edges keep the normal north-facing traversal camera
    /// readable. Exterior, northern back walls and plaque supports remain high.
    pub fn parapet(&self) -> bool {
        [48, 50, 62, 64, 67].contains(&self.z)
            && (self.angle == 3 || (self.shape == 2 && ((self.angle + 1) & 3) == 3))
    }
}

/// Cache-shaped masonry edges. Each wall occupies the boundary between tiles,
/// never an extra full-tile copy of a wall model. Doorways are genuinely absent
/// edges, so the same architecture drives scene placement and collision.
pub fn shell() -> Vec<Wall> {
    let mut edges: BTreeMap<(u8, u8), BTreeSet<u8>> = BTreeMap::new();
    let mut edge = |x, z, angle| {
        edges.entry((x, z)).or_default().insert(angle);
    };
    // Closed exterior; corner pairs become the cache's L-shaped wall (shape 2).
    for x in 32..80 {
        edge(x, 32, 3);
        edge(x, 79, 1);
    }
    for z in 32..80 {
        edge(32, z, 0);
        edge(79, z, 2);
    }
    // Southern processional hall, with broad side passages to the supply yards.
    for z in 33..48 {
        if !(39..=42).contains(&z) {
            edge(50, z, 0);
            edge(62, z, 0);
        }
    }
    // Service wings and northern courts. Three separate thresholds lead north.
    for x in 33..79 {
        if ![39, 40, 55, 56, 71, 72].contains(&x) {
            edge(x, 64, 3);
        }
        if x < 48 || x >= 64 {
            if ![39, 40, 71, 72].contains(&x) {
                edge(x, 48, 3);
            }
        }
    }
    for z in 50..62 {
        if ![55, 56].contains(&z) {
            edge(48, z, 0);
            edge(64, z, 0);
        }
    }
    for z in 65..79 {
        if ![71, 72].contains(&z) {
            edge(48, z, 0);
            edge(64, z, 0);
        }
    }
    // Cut-corner sanctuary: eight-sided circulation instead of a nine-box grid.
    for x in 50..62 {
        if ![55, 56].contains(&x) {
            edge(x, 48, 3);
        }
    }
    for (x, z, a) in [
        (50, 48, 0),
        (50, 49, 0),
        (48, 50, 3),
        (49, 50, 3),
        (62, 48, 0),
        (62, 49, 0),
        (62, 50, 3),
        (63, 50, 3),
        (48, 62, 3),
        (49, 62, 3),
        (50, 62, 0),
        (50, 63, 0),
        (62, 62, 3),
        (63, 62, 3),
        (62, 62, 0),
        (62, 63, 0),
    ] {
        edge(x, z, a);
    }
    // The foundry is a tighter fortified chamber behind a three-tile vestibule.
    // Side thresholds preserve circulation; combat bounds remain unchanged.
    for z in 67..78 {
        if ![71, 72].contains(&z) {
            edge(50, z, 0);
            edge(63, z, 0);
        }
    }
    for x in 50..63 {
        edge(x, 78, 3);
        if ![55, 56].contains(&x) {
            edge(x, 67, 3);
        }
    }
    // A masonry information alcove supports the plaque's wall-decoration model.
    edge(53, 51, 3);
    edge(54, 51, 3);
    edges
        .into_iter()
        .map(|((x, z), angles)| {
            let (shape, angle) = if angles.len() == 1 {
                (0, *angles.first().unwrap())
            } else {
                assert_eq!(angles.len(), 2, "Overlapping Tower wall layers at {x},{z}");
                let angle = (0..4)
                    .find(|a| angles.contains(a) && angles.contains(&((a + 1) & 3)))
                    .expect("opposite wall edges");
                (2, angle)
            };
            Wall { x, z, shape, angle }
        })
        .collect()
}

/// Clear obsolete source clipping before authored masonry is installed. River
/// terrain keeps its real banks; walls must be applied AFTER this preparation.
pub fn prepare_collision(
    map: &mut crate::collision::CollisionMap,
    origin: crate::CoordGrid,
) -> Result<(), &'static str> {
    if origin.level() != 0 || origin.x() + 80 >= 16384 || origin.z() + 80 >= 16384 {
        return Err("Invalid floor origin");
    }
    for x in 32..80 {
        for z in 32..80 {
            map.allocate(origin.x() + x, origin.z() + z, 0);
            if !(x >= 72 && (48..=63).contains(&z)) {
                map.set(origin.x() + x, origin.z() + z, 0, 0);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        collision::CollisionMap,
        loc_collision::{self, LocCollision},
        CoordGrid,
    };
    #[test]
    fn authored_edges_enclose_the_floor_but_every_play_space_is_reachable() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let mut map = CollisionMap::default();
        map.set(origin.x() + 75, origin.z() + 55, 0, 0x200000);
        prepare_collision(&mut map, origin).unwrap();
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        for w in shell() {
            assert!(w.shape == 0 || w.shape == 2);
            loc_collision::toggle(
                &mut map,
                origin.translate(w.x as i32, w.z as i32, 0).unwrap(),
                w.shape,
                w.angle,
                ty,
                true,
            )
            .unwrap();
        }
        assert_eq!(map.get(origin.x() + 75, origin.z() + 55, 0), 0x200000);
        let mut seen = BTreeSet::from([(56i32, 56i32)]);
        let mut queue = std::collections::VecDeque::from([(56, 56)]);
        while let Some((x, z)) = queue.pop_front() {
            let flags = map.get(origin.x() + x, origin.z() + z, 0);
            for (dx, dz, mask) in [(-1, 0, 0x80), (1, 0, 8), (0, -1, 32), (0, 1, 2)] {
                if flags & mask != 0 {
                    continue;
                }
                let p = (x + dx, z + dz);
                assert!(
                    (32..80).contains(&p.0) && (32..80).contains(&p.1),
                    "Exterior leak {p:?}"
                );
                if seen.insert(p) {
                    queue.push_back(p);
                }
            }
        }
        for p in [
            (40, 40),
            (56, 40),
            (72, 40),
            (38, 54),
            (55, 56),
            (71, 52),
            (40, 72),
            (55, 72),
            (71, 72),
            (60, 60),
        ] {
            assert!(seen.contains(&p), "Unreachable required space {p:?}");
        }
        assert_eq!(template().len(), 36);
    }
}

/// TowerMap.contains: the whole allocated square on planes zero and one.
pub fn contains(origin: crate::CoordGrid, coords: crate::CoordGrid) -> bool {
    (0..128).contains(&(coords.x() - origin.x()))
        && (0..128).contains(&(coords.z() - origin.z()))
        && coords.level() <= 1
}
