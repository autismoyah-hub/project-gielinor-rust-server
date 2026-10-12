//! Full location reach dispatch: straight/corner/diagonal walls, decoration,
//! rectangular locs, exclusive rectangles and coordinate-only shapes.
use crate::{collision::CollisionMap, reach::Rectangle, CoordGrid};
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub rectangle: Rectangle,
    pub shape: i8,
}
impl Target {
    pub fn valid(self) -> bool {
        self.rectangle.valid() && (-2..=22).contains(&self.shape)
    }
    pub fn reached(self, map: &CollisionMap, source: CoordGrid, size: i32) -> bool {
        let r = self.rectangle;
        if !self.valid()
            || source == CoordGrid::NULL
            || source.level() != r.coords.level()
            || !(1..=64).contains(&size)
        {
            return false;
        }
        if self.shape != -2 && source == r.coords {
            return true;
        }
        if matches!(self.shape, -2 | 10 | 11 | 22) {
            return Rectangle {
                exclusive: self.shape == -2,
                ..r
            }
            .reached(map, source, size);
        }
        if !matches!(self.shape, 0..=9) {
            return false;
        }
        let (sx, sz, dx, dz) = (source.x(), source.z(), r.coords.x(), r.coords.z());
        let (east, north) = (sx + size - 1, sz + size - 1);
        if (sx..=east).contains(&dx) && (sz..=north).contains(&dz) {
            return true;
        }
        let touching = [
            sx == dx - size && (sz..=north).contains(&dz),
            sz == dz + 1 && (sx..=east).contains(&dx),
            sx == dx + 1 && (sz..=north).contains(&dz),
            sz == dz - size && (sx..=east).contains(&dx),
        ];
        let edges = [(east, dz), (dx, sz), (sx, dz), (dx, north)];
        let walls = [8, 32, 128, 2];
        let angle = r.angle as usize;
        for side in 0..4 {
            if !touching[side] {
                continue;
            }
            let (allowed, unconditional, full_mask) = match self.shape {
                0 => (side == angle || side % 2 != angle % 2, side == angle, true),
                2 => (true, side == angle || side == (angle + 1) % 4, true),
                9 => (true, false, size != 1),
                6 | 7 => {
                    let a = (angle + if self.shape == 7 { 2 } else { 0 }) % 4;
                    let sides = [[2, 3], [0, 3], [0, 1], [2, 1]][a];
                    (sides.contains(&side), false, false)
                }
                8 => (true, false, false),
                _ => (false, false, false),
            };
            if allowed
                && (unconditional
                    || map.get(edges[side].0, edges[side].1, source.level())
                        & (walls[side] | if full_mask { 0x240100 } else { 0 })
                        == 0)
            {
                return true;
            }
        }
        false
    }
}
