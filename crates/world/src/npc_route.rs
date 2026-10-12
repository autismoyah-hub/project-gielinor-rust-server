//! Reference NPC combat mode boundaries and entity approach destinations.
use crate::CoordGrid;
pub fn can_retaliate(action_delay: i32, clock: i32, combat_mode: bool) -> bool {
    action_delay.wrapping_add(8) < clock || !combat_mode
}
pub fn retreat_destination(
    source: CoordGrid,
    spawn: CoordGrid,
    target: CoordGrid,
    range: i32,
) -> Option<CoordGrid> {
    if range < 0 {
        return None;
    }
    let dx = if source.x() <= target.x() { -1 } else { 1 };
    let dz = if source.z() <= target.z() { -1 } else { 1 };
    CoordGrid::new(
        (source.x() + dx).clamp(spawn.x() - range, spawn.x() + range),
        (source.z() + dz).clamp(spawn.z() - range, spawn.z() + range),
        source.level(),
    )
    .ok()
}
/// RouteFinding.naiveDestination; returns the adjacent perimeter waypoint for entity routing.
pub fn entity_destination(
    source: CoordGrid,
    source_size: (i32, i32),
    target: CoordGrid,
    target_size: (i32, i32),
    angle: u8,
) -> Option<CoordGrid> {
    let (sw, sl) = source_size;
    let (tw, tl) = if angle & 1 == 0 {
        target_size
    } else {
        (target_size.1, target_size.0)
    };
    if sw <= 0 || sl <= 0 || tw <= 0 || tl <= 0 {
        return None;
    }
    let diagonal = source.x() - target.x() + source.z() - target.z();
    let anti = source.x() - target.x() - (source.z() - target.z());
    let southwest = anti < 0;
    let northwest = diagonal >= tl - sw;
    let northeast = anti > sw - sl;
    let southeast = diagonal <= tw - sl;
    let (x, z) = if southwest && !northwest {
        (
            -sw,
            if diagonal >= -sw {
                (diagonal + sw).min(tl - 1)
            } else if anti > -sw {
                -(sw + anti)
            } else {
                0
            },
        )
    } else if northwest && !northeast {
        (
            if anti >= -tl {
                (anti + tl).min(tw - 1)
            } else if diagonal < tl {
                (diagonal - tl).max(-(sw - 1))
            } else {
                0
            },
            tl,
        )
    } else if northeast && !southeast {
        (
            tw,
            if anti <= tw {
                tl - anti
            } else if diagonal < tw {
                (diagonal - tw).max(-(sl - 1))
            } else {
                0
            },
        )
    } else {
        if !southeast || southwest {
            return None;
        }
        (
            if diagonal > -sl {
                (diagonal + sl).min(tw - 1)
            } else if anti < sl {
                (anti - sl).max(-(sl - 1))
            } else {
                0
            },
            -sl,
        )
    };
    CoordGrid::new(target.x() + x, target.z() + z, source.level()).ok()
}
