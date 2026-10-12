use crate::CoordGrid;
use std::collections::HashMap;
const FLOOR: u32 = 0x240100;
const WEST: u32 = FLOOR | 8;
const EAST: u32 = FLOOR | 128;
const SOUTH: u32 = FLOOR | 2;
const NORTH: u32 = FLOOR | 32;
const SW: u32 = FLOOR | 2 | 4 | 8;
const SE: u32 = FLOOR | 1 | 2 | 128;
const NW: u32 = FLOOR | 8 | 16 | 32;
const NE: u32 = FLOOR | 32 | 64 | 128;
const V_WEST: u32 = FLOOR | 2 | 4 | 8 | 16 | 32;
const V_EAST: u32 = FLOOR | 1 | 2 | 32 | 64 | 128;
const H_SOUTH: u32 = FLOOR | 1 | 2 | 4 | 8 | 128;
const H_NORTH: u32 = FLOOR | 8 | 16 | 32 | 64 | 128;
#[derive(Debug, Clone, Copy)]
pub enum Strategy {
    Normal,
    Blocked,
    Indoors,
    Outdoors,
    LineOfSight,
}
impl Strategy {
    pub fn can_move(self, tile: u32, mask: u32) -> bool {
        match self {
            Self::Normal => tile & mask == 0,
            Self::Blocked => tile & (mask & !0x200000) == 0 && tile & 0x200000 != 0,
            Self::Indoors => tile & mask == 0 && tile & 0x80000000 != 0,
            Self::Outdoors => tile & (mask | 0x80000000) == 0,
            Self::LineOfSight => tile & (((mask & 0x1FF) << 9) | ((mask & 0x7FC00000) >> 13)) == 0,
        }
    }
}
#[derive(Default, Clone)]
pub struct CollisionMap {
    zones: HashMap<u32, [u32; 64]>,
}
impl CollisionMap {
    /// Publish an independently prepared region without erasing any existing
    /// map or actor occupancy. Validate every zone before making any change.
    pub fn append_disjoint(&mut self, other: CollisionMap) -> Result<(), &'static str> {
        if other.zones.keys().any(|key| self.zones.contains_key(key)) {
            return Err("Collision region overlaps allocated world zones");
        }
        self.zones.extend(other.zones);
        Ok(())
    }
    fn zone(x: i32, z: i32, level: i32) -> u32 {
        ((x as u32 >> 3) & 0x7FF) | (((z as u32 >> 3) & 0x7FF) << 11) | ((level as u32 & 3) << 22)
    }
    fn tile(x: i32, z: i32) -> usize {
        ((x & 7) | ((z & 7) << 3)) as usize
    }
    pub fn allocate(&mut self, x: i32, z: i32, level: i32) {
        self.zones.entry(Self::zone(x, z, level)).or_insert([0; 64]);
    }
    pub fn allocated(&self, x: i32, z: i32, level: i32) -> bool {
        self.zones.contains_key(&Self::zone(x, z, level))
    }
    pub fn get(&self, x: i32, z: i32, level: i32) -> u32 {
        self.zones
            .get(&Self::zone(x, z, level))
            .map_or(u32::MAX, |v| v[Self::tile(x, z)])
    }
    pub fn set(&mut self, x: i32, z: i32, level: i32, flags: u32) {
        self.zones.entry(Self::zone(x, z, level)).or_insert([0; 64])[Self::tile(x, z)] = flags;
    }
    pub fn add(&mut self, x: i32, z: i32, level: i32, flags: u32) {
        self.zones.entry(Self::zone(x, z, level)).or_insert([0; 64])[Self::tile(x, z)] |= flags;
    }
    pub fn remove(&mut self, x: i32, z: i32, level: i32, flags: u32) {
        self.set(x, z, level, self.get(x, z, level) & !flags);
    }
    pub fn deallocate(&mut self, x: i32, z: i32, level: i32) {
        self.zones.remove(&Self::zone(x, z, level));
    }
    /// All entity sizes follow the reference edge/corner masks. Unallocated zones block movement.
    pub fn can_step(
        &self,
        from: CoordGrid,
        dx: i32,
        dz: i32,
        size: i32,
        extra: u32,
        strategy: Strategy,
    ) -> bool {
        if size < 1
            || size > 64
            || !(-1..=1).contains(&dx)
            || !(-1..=1).contains(&dz)
            || (dx == 0 && dz == 0)
        {
            return false;
        }
        let (x, z, level) = (from.x(), from.z(), from.level());
        if x + dx < 0 || z + dz < 0 || x + dx + size > 16384 || z + dz + size > 16384 {
            return false;
        }
        let clear = |x, z, mask| strategy.can_move(self.get(x, z, level), mask | extra);
        if size == 1 {
            let mask = match (dx, dz) {
                (-1, 0) => WEST,
                (1, 0) => EAST,
                (0, -1) => SOUTH,
                (0, 1) => NORTH,
                (-1, -1) => SW,
                (-1, 1) => NW,
                (1, -1) => SE,
                (1, 1) => NE,
                _ => unreachable!(),
            };
            return clear(x + dx, z + dz, mask)
                && (dx == 0
                    || dz == 0
                    || (clear(x + dx, z, if dx < 0 { WEST } else { EAST })
                        && clear(x, z + dz, if dz < 0 { SOUTH } else { NORTH })));
        }
        if dx == 0 {
            let edge = z + if dz < 0 { -1 } else { size };
            return clear(x, edge, if dz < 0 { SW } else { NW })
                && clear(x + size - 1, edge, if dz < 0 { SE } else { NE })
                && (1..size - 1)
                    .all(|i| clear(x + i, edge, if dz < 0 { H_SOUTH } else { H_NORTH }));
        }
        if dz == 0 {
            let edge = x + if dx < 0 { -1 } else { size };
            return clear(edge, z, if dx < 0 { SW } else { SE })
                && clear(edge, z + size - 1, if dx < 0 { NW } else { NE })
                && (1..size - 1).all(|i| clear(edge, z + i, if dx < 0 { V_WEST } else { V_EAST }));
        }
        let ex = x + if dx < 0 { -1 } else { size };
        let ez = z + if dz < 0 { -1 } else { size };
        let mask = match (dx, dz) {
            (-1, -1) => SW,
            (-1, 1) => NW,
            (1, -1) => SE,
            _ => NE,
        };
        clear(ex, ez, mask)
            && (1..size).all(|i| {
                clear(
                    ex,
                    z + i + if dz < 0 { -1 } else { 0 },
                    if dx < 0 { V_WEST } else { V_EAST },
                ) && clear(
                    x + i + if dx < 0 { -1 } else { 0 },
                    ez,
                    if dz < 0 { H_SOUTH } else { H_NORTH },
                )
            })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detached_region_publication_rejects_overlap_without_partial_changes() {
        let mut normal = CollisionMap::default();
        normal.set(3200, 3200, 0, 0x80000);
        let mut region = CollisionMap::default();
        region.set(10424, 120, 0, 0x100);
        region.set(3207, 3207, 0, 0);
        assert!(normal.append_disjoint(region).is_err());
        assert_eq!(normal.get(3200, 3200, 0), 0x80000);
        assert!(!normal.allocated(10424, 120, 0));
        let mut region = CollisionMap::default();
        region.set(10424, 120, 0, 0x100);
        normal.append_disjoint(region).unwrap();
        assert_eq!(normal.get(10424, 120, 0), 0x100);
        assert_eq!(normal.get(3200, 3200, 0), 0x80000);
    }
    #[test]
    fn unknown_map_and_corners_block() {
        let p = CoordGrid::new(3203, 3203, 0).unwrap();
        let mut m = CollisionMap::default();
        assert!(!m.can_step(p, 1, 1, 1, 0, Strategy::Normal));
        m.allocate(3200, 3200, 0);
        assert!(m.can_step(p, 1, 1, 1, 0, Strategy::Normal));
        m.set(3204, 3203, 0, 128);
        assert!(!m.can_step(p, 1, 1, 1, 0, Strategy::Normal));
    }
    #[test]
    fn sparse_allocation_matches_reference_zone_semantics() {
        let mut m = CollisionMap::default();
        m.add(3200, 3200, 0, 1);
        assert_eq!(m.get(3201, 3201, 0), 0);
        assert_eq!(m.get(3208, 3200, 0), u32::MAX);
        m.deallocate(3200, 3200, 0);
        assert_eq!(m.get(3200, 3200, 0), u32::MAX);
    }
}
