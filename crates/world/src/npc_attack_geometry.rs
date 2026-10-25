//! Pure NPC attack-range membership (issue147 engine framework only).
//!
//! Aggressiveness Wiki revision15373608 specifies current occupied tiles,
//! standard melee without corners, square halberd/projectile reach, and no
//! attacks into the NPC's own footprint. Geometry is an explicit input: scalar
//! range1, accuracy category and weapon appearance cannot select it.
//! This module has no runtime caller and does not decide collision, LOS,
//! aggression, hunting, routing, attack permission, damage or projectile effects.
use crate::CoordGrid;
use core::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    ZeroRange,
    InvalidSize,
    InvalidCoordinate,
    FootprintOutsideWorld,
}

/// Explicit positive square radius. The source does not establish one universal
/// NPC maximum; registration must validate the range for its actual caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SquareRange(NonZeroU32);
impl SquareRange {
    pub fn new(tiles: u32) -> Result<Self, Error> {
        NonZeroU32::new(tiles).map(Self).ok_or(Error::ZeroRange)
    }
    pub fn tiles(self) -> u32 { self.0.get() }
}

/// Deliberately has no Default or conversion from damage category/numeric range.
/// Halberd and Projectile remain distinct even though their pure shapes agree;
/// future callers must source their own collision/LOS/execution policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackGeometry {
    StandardMelee,
    Halberd(SquareRange),
    Projectile(SquareRange),
}

/// A snapshot of all occupied NPC tiles, anchored at the current southwest tile.
/// A future caller must construct a fresh snapshot when the NPC moves. Size1..64
/// matches the existing engine's Npc::new bounds, not a Wiki species claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OccupiedFootprint {
    southwest: CoordGrid,
    size: u32,
}
fn canonical(coords: CoordGrid) -> bool {
    coords != CoordGrid::NULL
        && CoordGrid::new(coords.x(), coords.z(), coords.level()).ok() == Some(coords)
}
impl OccupiedFootprint {
    pub fn new(southwest: CoordGrid, size: u32) -> Result<Self, Error> {
        if !canonical(southwest) { return Err(Error::InvalidCoordinate); }
        if !(1..=64).contains(&size) { return Err(Error::InvalidSize); }
        // Widen before addition; never wrap or clip an occupied world footprint.
        let end = i64::from(size) - 1;
        if i64::from(southwest.x()) + end > 16383
            || i64::from(southwest.z()) + end > 16383 {
            return Err(Error::FootprintOutsideWorld);
        }
        Ok(Self { southwest, size })
    }
    pub fn southwest(self) -> CoordGrid { self.southwest }
    pub fn size(self) -> u32 { self.size }

    fn axis_gaps(self, target: CoordGrid) -> (i64, i64) {
        let gap = |start: i32, point: i32| {
            let start = i64::from(start);
            let end = start + i64::from(self.size) - 1;
            let point = i64::from(point);
            (start - point).max(point - end).max(0)
        };
        (gap(self.southwest.x(), target.x()), gap(self.southwest.z(), target.z()))
    }
}
impl AttackGeometry {
    /// Geometric membership for a single target tile; true is not permission to
    /// attack. NULL/noncanonical coordinates, other planes and overlap fail.
    pub fn contains(self, current: OccupiedFootprint, target: CoordGrid) -> bool {
        if !canonical(target) || current.southwest.level() != target.level() { return false; }
        let (dx, dz) = current.axis_gaps(target);
        match self {
            Self::StandardMelee => dx + dz == 1,
            Self::Halberd(range) | Self::Projectile(range) => {
                let distance = dx.max(dz);
                distance > 0 && distance <= i64::from(range.tiles())
            }
        }
    }
}
