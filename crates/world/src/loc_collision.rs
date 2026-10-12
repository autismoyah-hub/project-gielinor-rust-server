//! Static and dynamic location collision from CollisionFlagMapExtensions.toggleLoc.
use crate::{collision::CollisionMap, CoordGrid};
#[derive(Clone, Copy, Debug)]
pub struct LocCollision {
    pub width: u8,
    pub length: u8,
    pub block_walk: u8,
    pub block_range: bool,
    pub break_route_finding: bool,
}
pub fn toggle(
    map: &mut CollisionMap,
    coords: CoordGrid,
    shape: u8,
    angle: u8,
    ty: LocCollision,
    add: bool,
) -> Result<(), &'static str> {
    if shape > 22 || angle > 3 {
        return Err("Invalid location shape or angle");
    }
    let mut updates = vec![];
    let mut flag = |dx: i32, dz: i32, mask: u32| -> Result<(), &'static str> {
        let p = coords
            .translate(dx, dz, 0)
            .map_err(|_| "Location extends outside world")?;
        updates.push((p, mask));
        Ok(())
    };
    if shape == 22 {
        if ty.block_walk == 1 {
            flag(0, 0, 0x40000)?;
        }
    } else if ty.block_walk != 0 {
        if shape == 9 || shape >= 10 {
            let (w, l) = if angle & 1 != 0 {
                (ty.length, ty.width)
            } else {
                (ty.width, ty.length)
            };
            let mask = 0x100
                | if ty.block_range { 0x20000 } else { 0 }
                | if ty.break_route_finding {
                    0x40000000
                } else {
                    0
                };
            for x in 0..w {
                for z in 0..l {
                    flag(x as i32, z as i32, mask)?;
                }
            }
        } else if shape <= 3 {
            let cardinal = [
                (-1, 0, 0x80u32, 0x8u32),
                (0, 1, 0x2, 0x20),
                (1, 0, 0x8, 0x80),
                (0, -1, 0x20, 0x2),
            ];
            let diagonal = [
                (-1, 1, 0x1u32, 0x10u32),
                (1, 1, 0x4, 0x40),
                (1, -1, 0x10, 0x1),
                (-1, -1, 0x40, 0x4),
            ];
            let mut side = |entry: (i32, i32, u32, u32)| -> Result<(), &'static str> {
                let (x, z, own, other) = entry;
                flag(0, 0, own | if ty.block_range { own << 9 } else { 0 })?;
                flag(x, z, other | if ty.block_range { other << 9 } else { 0 })
            };
            match shape {
                0 => side(cardinal[angle as usize])?,
                1 | 3 => side(diagonal[angle as usize])?,
                2 => {
                    side(cardinal[angle as usize])?;
                    side(cardinal[((angle + 1) & 3) as usize])?;
                }
                _ => unreachable!(),
            }
        }
    }
    for (p, mask) in updates {
        if add {
            map.add(p.x(), p.z(), p.level(), mask)
        } else {
            map.remove(p.x(), p.z(), p.level(), mask)
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walls_are_symmetric_and_removal_preserves_floor() {
        let p = CoordGrid::new(3203, 3203, 0).unwrap();
        let mut m = CollisionMap::default();
        m.set(3203, 3203, 0, 0x200000);
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        toggle(&mut m, p, 2, 0, ty, true).unwrap();
        assert_eq!(m.get(3203, 3203, 0), 0x200000 | 0x82 | 0x10400);
        assert_eq!(m.get(3202, 3203, 0), 8 | 4096);
        assert_eq!(m.get(3203, 3204, 0), 32 | 16384);
        toggle(&mut m, p, 2, 0, ty, false).unwrap();
        assert_eq!(m.get(3203, 3203, 0), 0x200000);
        assert_eq!(m.get(3202, 3203, 0), 0);
    }
}
