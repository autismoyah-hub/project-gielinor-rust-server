//! Coordinate routes with Kotlin's search bounds, neighbor order and waypoint compression.
//! Location/entity reach strategies are separate from this coordinate-only entry point.
use crate::{
    collision::{CollisionMap, Strategy},
    CoordGrid,
};
use std::collections::VecDeque;
const WIDTH: i32 = 128;
const NEIGHBORS: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub waypoints: Vec<CoordGrid>,
    pub alternative: bool,
    pub success: bool,
}
impl Route {
    fn failed() -> Self {
        Self {
            waypoints: vec![],
            alternative: false,
            success: false,
        }
    }
}
pub fn coordinate_route(
    map: &CollisionMap,
    source: CoordGrid,
    destination: CoordGrid,
    size: i32,
    move_near: bool,
    max_waypoints: usize,
    strategy: Strategy,
) -> Route {
    find_route(
        map,
        source,
        destination,
        size,
        move_near,
        max_waypoints,
        strategy,
        (1, 1),
        |p| p == destination,
    )
}
/// Entity and rectangular-location interaction routes. Wall/decor shapes require their own reach strategy.
pub fn rectangle_route(
    map: &CollisionMap,
    source: CoordGrid,
    target: crate::reach::Rectangle,
    size: i32,
    move_near: bool,
    max_waypoints: usize,
    strategy: Strategy,
) -> Route {
    if !target.valid() {
        return Route::failed();
    }
    find_route(
        map,
        source,
        target.coords,
        size,
        move_near,
        max_waypoints,
        strategy,
        target.dimensions(),
        |p| target.reached(map, p, size),
    )
}
pub fn location_route(
    map: &CollisionMap,
    source: CoordGrid,
    target: crate::loc_reach::Target,
    size: i32,
    move_near: bool,
    max_waypoints: usize,
    strategy: Strategy,
) -> Route {
    if !target.valid() {
        return Route::failed();
    }
    find_route(
        map,
        source,
        target.rectangle.coords,
        size,
        move_near,
        max_waypoints,
        strategy,
        target.rectangle.dimensions(),
        |p| target.reached(map, p, size),
    )
}
fn find_route(
    map: &CollisionMap,
    source: CoordGrid,
    destination: CoordGrid,
    size: i32,
    move_near: bool,
    max_waypoints: usize,
    strategy: Strategy,
    dimensions: (i32, i32),
    reached: impl Fn(CoordGrid) -> bool,
) -> Route {
    if source.level() != destination.level() || !(1..=64).contains(&size) || max_waypoints == 0 {
        return Route::failed();
    }
    let base_x = source.x() - 64;
    let base_z = source.z() - 64;
    let target = (destination.x() - base_x, destination.z() - base_z);
    let index = |x: i32, z: i32| (x * WIDTH + z) as usize;
    let mut directions = vec![0u8; 16384];
    let mut distances = vec![99_999_999i32; 16384];
    let mut queue = [(0i32, 0i32); 4096];
    let mut read = 0;
    let mut write = 1;
    queue[0] = (64, 64);
    directions[index(64, 64)] = 99;
    distances[index(64, 64)] = 0;
    let mut found = None;
    while read != write {
        let (x, z) = queue[read];
        read = (read + 1) & 4095;
        if CoordGrid::new(base_x + x, base_z + z, source.level()).is_ok_and(&reached) {
            found = Some((x, z));
            break;
        }
        let distance = distances[index(x, z)] + 1;
        let Ok(from) = CoordGrid::new(base_x + x, base_z + z, source.level()) else {
            continue;
        };
        for (direction, (dx, dz)) in NEIGHBORS.iter().copied().enumerate() {
            let nx = x + dx;
            let nz = z + dz;
            if nx < 0
                || nz < 0
                || nx > WIDTH - size
                || nz > WIDTH - size
                || directions[index(nx, nz)] != 0
            {
                continue;
            }
            if !map.can_step(from, dx, dz, size, 0, strategy) {
                continue;
            }
            directions[index(nx, nz)] = direction as u8 + 1;
            distances[index(nx, nz)] = distance;
            queue[write] = (nx, nz);
            write = (write + 1) & 4095;
        }
    }
    let alternative = found.is_none();
    if alternative {
        if !move_near {
            return Route::failed();
        }
        let mut cost = 1000;
        let mut path = 100;
        for x in target.0 - 10..=target.0 + 10 {
            for z in target.1 - 10..=target.1 + 10 {
                if !(0..128).contains(&x) || !(0..128).contains(&z) || distances[index(x, z)] >= 100
                {
                    continue;
                }
                let distance = distances[index(x, z)];
                let dx = if x < target.0 {
                    target.0 - x
                } else {
                    (x - (target.0 + dimensions.0 - 1)).max(0)
                };
                let dz = if z < target.1 {
                    target.1 - z
                } else {
                    (z - (target.1 + dimensions.1 - 1)).max(0)
                };
                let next = dx.pow(2) + dz.pow(2);
                if next < cost || (next == cost && path > distance) {
                    found = Some((x, z));
                    cost = next;
                    path = distance;
                }
            }
        }
    }
    let Some((mut x, mut z)) = found else {
        return Route::failed();
    };
    let mut waypoints = VecDeque::new();
    let mut previous = 0;
    for _ in 0..16384 {
        if (x, z) == (64, 64) {
            break;
        }
        let direction = directions[index(x, z)];
        if direction == 0 || direction > 8 {
            return Route::failed();
        }
        if previous != direction {
            previous = direction;
            if waypoints.len() >= max_waypoints {
                waypoints.pop_back();
            }
            let Ok(coord) = CoordGrid::new(base_x + x, base_z + z, source.level()) else {
                return Route::failed();
            };
            waypoints.push_front(coord);
        }
        let (dx, dz) = NEIGHBORS[direction as usize - 1];
        x -= dx;
        z -= dz;
    }
    Route {
        waypoints: waypoints.into_iter().collect(),
        alternative,
        success: true,
    }
}
