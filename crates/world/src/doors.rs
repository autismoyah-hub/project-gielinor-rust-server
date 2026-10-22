//! Single/double door mutations from the generic-locs scripts. The caller owns
//! interaction reach, protected access, sound output and the optional two-cycle delay.
use crate::{locations::Location, simulation::World, CoordGrid};
use std::collections::BTreeMap;
#[cfg(test)]
#[path="picket_gate_tests.rs"]
mod picket_tests;
pub const DURATION: i32 = 500;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    ClosedSingle,
    OpenedSingle,
    ClosedLeft,
    ClosedRight,
    OpenedLeft,
    OpenedRight,
    GateClosedLeft,
    GateClosedRight,
    GateOpenedLeft,
    GateOpenedRight,
}
impl Kind {
    pub fn is_picket(self)->bool {matches!(self,Self::GateClosedLeft|Self::GateClosedRight|Self::GateOpenedLeft|Self::GateOpenedRight)}
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
    if door.kind.is_picket() {return begin_picket(source,door);}
    use Kind::*;
    use Translation::*;
    let (translation, rotation) = match door.kind {
        ClosedSingle | ClosedRight => (Open, 1),
        OpenedSingle | OpenedRight => (Close, 3),
        ClosedLeft => (Open, 3),
        OpenedLeft => (CloseOpposite, 1),
        _=>return Err("Invalid door kind"),
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
    if catalog.get(&pending.source.id).is_some_and(|d|d.kind.is_picket()) {return finish_picket(world,catalog,pending);}
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

/// Picket gates use a different hinge and pair geometry from double doors.
fn begin_picket(source:Location,door:Door)->Result<Started,&'static str> {
    use Kind::*;
    if source.shape!=0 || source.angle>3 {return Err("Picket gate requires a straight wall");}
    let left=matches!(door.kind,GateClosedLeft|GateOpenedLeft);
    let closed=matches!(door.kind,GateClosedLeft|GateClosedRight);
    let pair=[(0,1),(1,0),(0,-1),(-1,0)][source.angle as usize];
    let pair_coords=source.coords.translate(if left{pair.0}else{-pair.0},if left{pair.1}else{-pair.1},0).map_err(|_|"Gate pair outside world")?;
    let index=if closed{source.angle}else{(source.angle+3)&3} as usize;
    let(dx,dz)=if left{[(-1,0),(0,1),(1,0),(0,-1)][index]}else{[(-2,-1),(-1,2),(2,1),(1,-2)][index]};
    let coords=source.coords.translate(dx,dz,0).map_err(|_|"Gate destination outside world")?;
    // This migration covers normal-world gates; constructed regions retain
    // their existing registry rules and need separate gate acceptance.
    if source.coords.x()>=6432 || coords.x()>=6432 {return Err("Picket gate outside supported normal world");}
    let destination=Location::new(coords,door.next,source.shape,(source.angle+if closed{3}else{1})&3)?;
    let partner=match door.kind {GateClosedLeft=>GateClosedRight,GateClosedRight=>GateClosedLeft,GateOpenedLeft=>GateOpenedRight,GateOpenedRight=>GateOpenedLeft,_=>return Err("Invalid picket kind")};
    Ok(Started{sound:door.sound,step_away:None,pending:Pending{source,destination,pair:Some((pair_coords,partner))}})
}

/// Capture both halves before any mutation; an absent partner still permits a single half.
fn finish_picket(world:&mut World,catalog:&Catalog,pending:Pending)->Result<(),&'static str> {
    if world.clock.0<0 || world.clock.0.checked_add(DURATION).is_none(){return Err("Invalid gate restoration deadline");}
    let mut plans=vec![pending];
    if let Some((coords,kind))=pending.pair {
        if let Some(partner)=world.locations.find_shape(coords,pending.source.shape) {
            if partner.angle==pending.source.angle {
                if let Some(&door)=catalog.get(&partner.id).filter(|d|d.kind==kind) {plans.push(begin_picket(partner,door)?.pending);}
            }
        }
    }
    // Preflight geometry, normal-world bounds and collision before publication.
    // A hinge must not replace an unrelated wall on the same scene layer.
    for plan in &plans {
        if world.locations.find_type(plan.source.coords,plan.source.id)!=Some(plan.source)
            || !world.location_types.contains_key(&plan.source.id) || !world.location_types.contains_key(&plan.destination.id) {return Err("Stale or unknown gate stage");}
        if (0..=3).any(|shape|world.locations.find_shape(plan.destination.coords,shape).is_some_and(|l|!plans.iter().any(|p|p.source==l))) {return Err("Occupied gate destination");}
        let mut scratch=crate::collision::CollisionMap::default();
        crate::loc_collision::toggle(&mut scratch,plan.source.coords,0,plan.source.angle,world.location_types[&plan.source.id],false)?;
        crate::loc_collision::toggle(&mut scratch,plan.destination.coords,0,plan.destination.angle,world.location_types[&plan.destination.id],true)?;
    }
    for plan in &plans {world.change_location_for(plan.source,false,DURATION)?;}
    for plan in &plans {world.change_location_for(plan.destination,true,DURATION)?;}
    Ok(())
}
