//! Single/double door mutations from the generic-locs scripts. The caller owns
//! interaction reach, protected access, sound output and the optional two-cycle delay.
use crate::{locations::Location, simulation::World, CoordGrid};
use std::collections::BTreeMap;
pub const DURATION: i32 = 500;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ClosedSingle,
    OpenedSingle,
    ClosedLeft,
    ClosedRight,
    OpenedLeft,
    OpenedRight,
}
#[derive(Clone, Copy, Debug)]
pub struct Door {
    pub kind: Kind,
    pub next: u32,
    pub sound: u16,
}
pub type Catalog = BTreeMap<u32, Door>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Translation {
    Open,
    OpenOpposite,
    Close,
    CloseOpposite,
}
pub fn translate(
    coords: CoordGrid,
    shape: u8,
    angle: u8,
    mode: Translation,
) -> Result<CoordGrid, &'static str> {
    if angle > 3 {
        return Err("Invalid door angle");
    }
    let open = [(-1, 0), (0, 1), (1, 0), (0, -1)];
    let close = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    let (open_mode, opposite) = match mode {
        Translation::Open => (true, false),
        Translation::OpenOpposite => (true, true),
        Translation::Close => (false, false),
        Translation::CloseOpposite => (false, true),
    };
    let (mut x, mut z) = match (shape, open_mode) {
        (0, true) | (9, false) => open[angle as usize],
        (9, true) | (0, false) => close[angle as usize],
        _ => return Ok(coords),
    };
    // Diagonal close uses the opposite of straight open.
    if shape == 9 && !open_mode {
        x = -x;
        z = -z;
    }
    if opposite {
        x = -x;
        z = -z;
    }
    coords
        .translate(x, z, 0)
        .map_err(|_| "Door translation outside world")
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub source: Location,
    pub destination: Location,
    pub pair: Option<(CoordGrid, Kind)>,
}
#[derive(Clone, Copy, Debug)]
pub struct Started {
    pub sound: u16,
    pub step_away: Option<CoordGrid>,
    pub pending: Pending,
}
/// Capture the same door metadata the Kotlin coroutine retains across delay(2).
pub fn begin(source: Location, door: Door, actor: CoordGrid) -> Result<Started, &'static str> {
    use Kind::*;
    use Translation::*;
    let (translation, rotation) = match door.kind {
        ClosedSingle | ClosedRight => (Open, 1),
        OpenedSingle | OpenedRight => (Close, 3),
        ClosedLeft => (Open, 3),
        OpenedLeft => (CloseOpposite, 1),
    };
    let coords = translate(source.coords, source.shape, source.angle, translation)?;
    let destination = Location::new(
        coords,
        door.next,
        source.shape,
        (source.angle + rotation) & 3,
    )?;
    let step_away =
        if source.shape == 9 && actor == coords && matches!(door.kind, ClosedSingle | OpenedSingle)
        {
            let (x, z) = if door.kind == ClosedSingle {
                [(-1, 0), (0, 1), (1, 0), (0, -1)][source.angle as usize]
            } else {
                [(1, 1), (1, -1), (-1, -1), (-1, 1)][source.angle as usize]
            };
            Some(
                coords
                    .translate(x, z, 0)
                    .map_err(|_| "Door step away outside world")?,
            )
        } else {
            None
        };
    let pair = match door.kind {
        ClosedLeft => Some((
            translate(source.coords, source.shape, source.angle, Close)?,
            ClosedRight,
        )),
        ClosedRight => Some((
            translate(source.coords, source.shape, source.angle, CloseOpposite)?,
            ClosedLeft,
        )),
        OpenedLeft => Some((
            translate(source.coords, source.shape, source.angle, OpenOpposite)?,
            OpenedRight,
        )),
        OpenedRight => Some((
            translate(source.coords, source.shape, source.angle, OpenOpposite)?,
            OpenedLeft,
        )),
        _ => None,
    };
    Ok(Started {
        sound: door.sound,
        step_away,
        pending: Pending {
            source,
            destination,
            pair,
        },
    })
}
/// Finish immediately unless begin requested step-away, then after the protected
/// two-cycle delay. Partner lookup intentionally occurs after the first mutation.
pub fn finish(world: &mut World, catalog: &Catalog, pending: Pending) -> Result<(), &'static str> {
    world.change_location_for(pending.source, false, DURATION)?;
    world.change_location_for(pending.destination, true, DURATION)?;
    if let Some((coords, kind)) = pending.pair {
        if let Some(partner) = world.locations.find_shape(coords, pending.source.shape) {
            if let Some(&door) = catalog.get(&partner.id).filter(|d| d.kind == kind) {
                let secondary = begin(partner, door, CoordGrid::NULL)?.pending;
                world.change_location_for(secondary.source, false, DURATION)?;
                world.change_location_for(secondary.destination, true, DURATION)?;
            }
        }
    }
    Ok(())
}
