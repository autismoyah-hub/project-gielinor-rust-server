//! Rectangular interaction reach, including the original size-dependent wall checks.
use crate::{collision::CollisionMap, CoordGrid};
#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub coords: CoordGrid,
    pub width: i32,
    pub length: i32,
    pub angle: u8,
    pub blocked: u8,
    pub exclusive: bool,
}
impl Rectangle {
    pub fn valid(self) -> bool {
        self.coords != CoordGrid::NULL
            && (1..=64).contains(&self.width)
            && (1..=64).contains(&self.length)
            && self.angle < 4
            && self.blocked < 16
    }
    pub fn dimensions(self) -> (i32, i32) {
        if self.angle & 1 != 0 {
            (self.length, self.width)
        } else {
            (self.width, self.length)
        }
    }
    pub fn reached(self, map: &CollisionMap, source: CoordGrid, size: i32) -> bool {
        if !self.valid() || source.level() != self.coords.level() || !(1..=64).contains(&size) {
            return false;
        }
        let (w, l) = self.dimensions();
        let (sx, sz) = (source.x(), source.z());
        let (dx, dz) = (self.coords.x(), self.coords.z());
        let overlap = sx < dx + w && sx + size > dx && sz < dz + l && sz + size > dz;
        if overlap {
            return !self.exclusive;
        }
        let blocked = if self.angle == 0 {
            self.blocked
        } else {
            ((self.blocked << self.angle) & 15) | (self.blocked >> (4 - self.angle))
        };
        let clear = |x, z, mask| map.get(x, z, source.level()) & mask == 0;
        if size == 1 {
            return (sx == dx - 1
                && sz >= dz
                && sz < dz + l
                && clear(sx, sz, 8)
                && blocked & 8 == 0)
                || (sx == dx + w
                    && sz >= dz
                    && sz < dz + l
                    && clear(sx, sz, 128)
                    && blocked & 2 == 0)
                || (sz + 1 == dz
                    && sx >= dx
                    && sx < dx + w
                    && clear(sx, sz, 2)
                    && blocked & 4 == 0)
                || (sz == dz + l
                    && sx >= dx
                    && sx < dx + w
                    && clear(sx, sz, 32)
                    && blocked & 1 == 0);
        }
        if dx + w == sx && blocked & 2 == 0 {
            (sz.max(dz)..(sz + size).min(dz + l)).any(|z| clear(dx + w - 1, z, 8))
        } else if sx + size == dx && blocked & 8 == 0 {
            (sz.max(dz)..(sz + size).min(dz + l)).any(|z| clear(dx, z, 128))
        } else if sz == dz + l && blocked & 1 == 0 {
            (sx.max(dx)..(sx + size).min(dx + w)).any(|x| clear(x, dz + l - 1, 2))
        } else if dz == sz + size && blocked & 4 == 0 {
            (sx.max(dx)..(sx + size).min(dx + w)).any(|x| clear(x, dz, 32))
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exclusive_overlap_rotation_and_size_specific_wall_edges() {
        let c = |x, z| CoordGrid::new(x, z, 0).unwrap();
        let mut map = CollisionMap::default();
        map.allocate(3200, 3200, 0);
        let mut target = Rectangle {
            coords: c(3202, 3202),
            width: 2,
            length: 1,
            angle: 0,
            blocked: 0,
            exclusive: true,
        };
        assert!(!target.reached(&map, c(3202, 3202), 1));
        assert!(target.reached(&map, c(3201, 3202), 1));
        map.add(3201, 3202, 0, 8);
        assert!(!target.reached(&map, c(3201, 3202), 1));
        // Multi-tile sources inspect the destination edge, unlike size-one sources.
        assert!(target.reached(&map, c(3200, 3202), 2));
        map.add(3202, 3202, 0, 128);
        assert!(!target.reached(&map, c(3200, 3202), 2));
        target.angle = 1;
        target.blocked = 4; // south rotated once blocks west
        assert!(!target.reached(&map, c(3200, 3202), 2));
        target.exclusive = false;
        assert!(target.reached(&map, c(3202, 3203), 1));
    }
}
