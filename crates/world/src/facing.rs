//! PathingEntity.calculateAngle and Direction.angleBetween, in client angle units.
use crate::CoordGrid;
pub fn square_angle(
    source: CoordGrid,
    size: i32,
    target: CoordGrid,
    width: i32,
    length: i32,
) -> Option<u16> {
    if target.x() == 0 && target.z() == 0 && target.level() == 0 {
        return Some(0);
    }
    if source == target {
        return None;
    }
    let dx = (source.x() * 2 + size - target.x() * 2 - width) as f64;
    let dz = (source.z() * 2 + size - target.z() * 2 - length) as f64;
    Some(((dx.atan2(dz) * 325.94932345220167) as i32 & 2047) as u16)
}
