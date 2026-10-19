//! Authoritative location handlers after reach, plane, protection and target validation.
use crate::{
    inventory::{Catalog, Error, Insert},
    simulation::PlayerData,
    CoordGrid,
};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Stairs {pub dx:i32,pub dz:i32,pub delta_level:i32}
impl Stairs {
    /// Straight flights use raw model width/length, including offsets outside it.
    pub fn destination(self,location:crate::locations::Location,width:u8,length:u8)->Result<CoordGrid,&'static str>{
        if width==0||length==0||location.coords==CoordGrid::NULL{return Err("Invalid staircase geometry");}
        let w=width as i64;let l=length as i64;let x=self.dx as i64;let z=self.dz as i64;
        let(dx,dz)=match location.angle{0=>(x,z),1=>(z,w-1-x),2=>(w-1-x,l-1-z),3=>(l-1-z,x),_=>return Err("Invalid staircase angle")};
        let dx=i32::try_from(dx).map_err(|_|"Staircase landing overflow")?;let dz=i32::try_from(dz).map_err(|_|"Staircase landing overflow")?;
        location.coords.translate(dx,dz,self.delta_level).map_err(|_|"Staircase landing outside world")
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Climb {pub delta_z:i32,pub delta_level:i32,pub animation:u16,pub delay:i32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ClimbTransition {pub tick:i32,pub location:crate::locations::Location,pub from:CoordGrid,pub to:CoordGrid}
impl ClimbTransition {
    pub fn json(self,actor:crate::simulation::ActorId)->String{
        let l=self.location;let coords=|c:CoordGrid|format!("[{},{},{}]",c.x(),c.z(),c.level());
        format!("{{\"schema\":1,\"actor_slot\":{},\"actor_generation\":{},\"tick\":{},\"location\":[{},{},{},{},{},{}],\"from\":{},\"to\":{}}}",actor.slot,actor.generation,self.tick,l.coords.x(),l.coords.z(),l.coords.level(),l.id,l.shape,l.angle,coords(self.from),coords(self.to))
    }
}

/// Timed native scenery stages reuse the existing location transaction and world timer.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Swap {pub next:u32,pub animation:Option<u16>,pub sound:Option<u16>,pub duration:i32,pub arrival_delay:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct SwapTransition {pub tick:i32,pub from:crate::locations::Location,pub to:crate::locations::Location,pub restore:Option<crate::locations::Location>,pub restore_at:Option<i32>}
impl SwapTransition {
    pub fn json(self,actor:crate::simulation::ActorId)->String{
        let loc=|l:crate::locations::Location|format!("[{},{},{},{},{},{}]",l.coords.x(),l.coords.z(),l.coords.level(),l.id,l.shape,l.angle);
        format!("{{\"schema\":1,\"actor_slot\":{},\"actor_generation\":{},\"tick\":{},\"from\":{},\"to\":{},\"restore\":{},\"restore_tick\":{}}}",actor.slot,actor.generation,self.tick,loc(self.from),loc(self.to),self.restore.map(loc).unwrap_or_else(||"null".into()),self.restore_at.map(|v|v.to_string()).unwrap_or_else(||"null".into()))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Handler {
    Pickable(crate::pickables::Rule),
    PickableStage {
        rule: crate::pickables::Rule,
        step: crate::pickables::Step,
    },
    PickableChance {rule:crate::pickables::Rule,depletion:crate::pickables::Depletion},
    Nettles {rule:crate::pickables::Rule,excluded_glove:i32,mark:crate::hit_display::Mark},
    Stairs(Stairs),
    Climb(Climb),
    DelayedSwap {rule:Swap,delay:i32},
    TimedSwap(Swap),
    Bookcase,
    ArrivedMessage(&'static str),
    Message(&'static str),
    EmptyPickable(&'static str),
    VialCrate {
        origin: CoordGrid,
        item: i32,
    },
    PrayerAltar {
        origin: CoordGrid,
        prayer: u8,
    },
}
impl Handler {
    pub fn is_pickable(self) -> bool {
        matches!(
            self,
            Self::Pickable(_) | Self::PickableStage { .. } | Self::PickableChance { .. } | Self::Nettles { .. } | Self::EmptyPickable(_)
        )
    }
    pub fn arrival_delay(self) -> bool {
        matches!(self, Self::PrayerAltar { .. } | Self::Stairs(_) | Self::Climb(_) | Self::Bookcase | Self::ArrivedMessage(_)) || self.is_pickable() || matches!(self,Self::TimedSwap(rule)|Self::DelayedSwap{rule,..}if rule.arrival_delay)
    }
    pub fn perform(
        self,
        player: &mut PlayerData,
        catalog: &Catalog,
        member: bool,
    ) -> Result<crate::pickables::Outcome, Error> {
        match self {
            Self::Pickable(rule) => rule.perform(player, catalog, member),
            Self::Stairs(_) | Self::Climb(_) | Self::TimedSwap(_) | Self::DelayedSwap {..} | Self::Bookcase | Self::PickableStage { .. } | Self::PickableChance { .. } | Self::Nettles { .. } => Err(Error::InvalidCapacity),
            Self::ArrivedMessage(message) | Self::Message(message) | Self::EmptyPickable(message) => Ok(crate::pickables::Outcome::Message(message)),
            _ => self.apply(player, catalog).map(|m| {
                m.map_or(
                    crate::pickables::Outcome::Ignored,
                    crate::pickables::Outcome::Message,
                )
            }),
        }
    }
    pub fn apply(
        self,
        player: &mut PlayerData,
        catalog: &Catalog,
    ) -> Result<Option<&'static str>, Error> {
        if matches!(self, Self::Stairs(_) | Self::Climb(_) | Self::TimedSwap(_) | Self::DelayedSwap {..} | Self::Bookcase | Self::PickableStage { .. } | Self::PickableChance { .. } | Self::Nettles { .. }) {
            return Err(Error::InvalidCapacity);
        }
        if let Self::ArrivedMessage(message) | Self::Message(message) | Self::EmptyPickable(message) = self {
            return Ok(Some(message));
        }
        if let Self::Pickable(rule) = self {
            return rule
                .perform(player, catalog, true)
                .map(crate::pickables::Outcome::message);
        }
        let origin = match self {
            Self::Pickable(_) | Self::PickableStage { .. } | Self::PickableChance { .. } | Self::Nettles { .. } | Self::Stairs(_) | Self::Climb(_) | Self::TimedSwap(_) | Self::DelayedSwap {..} | Self::Bookcase | Self::ArrivedMessage(_) | Self::Message(_) | Self::EmptyPickable(_) => {
                unreachable!("handled above")
            }
            Self::VialCrate { origin, .. } | Self::PrayerAltar { origin, .. } => origin,
        };
        // TowerMap.contains: the same cached types elsewhere do not run floor hooks.
        if player.coords.level() > 1
            || !(0..128).contains(&(player.coords.x() - origin.x()))
            || !(0..128).contains(&(player.coords.z() - origin.z()))
        {
            return Ok(None);
        }
        match self {
            Self::Pickable(_) | Self::PickableStage { .. } | Self::PickableChance { .. } | Self::Nettles { .. } | Self::Stairs(_) | Self::Climb(_) | Self::TimedSwap(_) | Self::DelayedSwap {..} | Self::Bookcase | Self::ArrivedMessage(_) | Self::Message(_) | Self::EmptyPickable(_) => {
                unreachable!("handled above")
            }
            Self::VialCrate { item, .. } => {
                let count = player.inventory.free_space().min(5) as i32;
                if count == 0 {
                    return Ok(Some("You don't have enough inventory space."));
                }
                player.inventory.insert(catalog, Insert::new(item, count))?;
                Ok(Some(
                    "You take water-filled vials. The crate still holds more.",
                ))
            }
            Self::PrayerAltar { prayer, .. } => {
                // statRestore resets boosted as well as drained levels; no XP is awarded.
                player.stats.set_current(prayer, player.stats.base(prayer));
                Ok(Some("You recharge your Prayer points."))
            }
        }
    }
}
