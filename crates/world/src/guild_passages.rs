//! Guild entry decisions; world movement, collision and timers remain shared.
use crate::{
    doors,
    inventory::{Catalog, Inventory},
    locations::Location,
    stats::Stats,
    CoordGrid,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Crafting,
    Cooking,
}
#[derive(Clone, Copy, Debug)]
pub struct Outfit {
    pub id: i32,
    pub slot: usize,
    pub members: bool,
}
#[derive(Clone, Debug)]
pub struct Gate {
    pub kind: Kind,
    pub opened: u32,
    pub outfits: Vec<Outfit>,
}
#[derive(Clone, Copy, Debug)]
pub struct Passage {
    pub opened: Location,
    pub destination: CoordGrid,
    pub delay: i32,
    pub entering: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Location,
    Level,
    Outfit,
}
impl Gate {
    pub fn prepare(
        &self,
        door: Location,
        source: CoordGrid,
        stats: &Stats,
        worn: &Inventory,
        items: &Catalog,
        member: bool,
    ) -> Result<Passage, Error> {
        if !matches!(door.shape, 0 | 9)
            || source.level() != door.coords.level()
            || source.x().abs_diff(door.coords.x()) > 1
            || source.z().abs_diff(door.coords.z()) > 1
        {
            return Err(Error::Location);
        }
        let (entering, skill, level, delta, delay) = match self.kind {
            Kind::Crafting => {
                let entering = source.z() >= door.coords.z();
                (entering, 12, 40, if entering { -1 } else { 0 }, 2)
            }
            Kind::Cooking => {
                let entering = source.z() <= door.coords.z();
                (entering, 7, 32, if entering { 1 } else { -1 }, 0)
            }
        };
        if entering {
            if i32::from(stats.current(skill)) < level {
                return Err(Error::Level);
            }
            if !self.outfits.iter().any(|o| {
                o.slot < 15
                    && (!o.members || member)
                    && items.ordinary_nonstackable(o.id)
                    && worn
                        .get(o.slot)
                        .is_some_and(|i| i.id == o.id && i.count == 1 && i.vars == 0)
            }) {
                return Err(Error::Outfit);
            }
        }
        let destination = door
            .coords
            .translate(0, delta, 0)
            .map_err(|_| Error::Location)?;
        let opened = Location::new(
            doors::translate(
                door.coords,
                door.shape,
                door.angle,
                doors::Translation::Open,
            )
            .map_err(|_| Error::Location)?,
            self.opened,
            door.shape,
            (door.angle + 1) & 3,
        )
        .map_err(|_| Error::Location)?;
        Ok(Passage {
            opened,
            destination,
            delay,
            entering,
        })
    }
    pub fn message(&self, error: Error) -> &'static str {
        match (self.kind, error) {
            (Kind::Crafting, Error::Level) => "You need level 40 Crafting to enter the guild.",
            (Kind::Cooking, Error::Level) => "You need level 32 Cooking to enter the guild.",
            (Kind::Crafting, Error::Outfit) => {
                "Wear a brown or golden apron, a Crafting cape or a max cape to enter."
            }
            (Kind::Cooking, Error::Outfit) => {
                "Wear a chef's hat or another qualifying guild outfit to enter."
            }
            _ => "You can't pass through that guild door right now.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn guild_passages_visible_levels_actual_slots_and_free_exit_ignore_held_clothes() {
        for kind in [Kind::Crafting, Kind::Cooking] {
            let (skill, level, slot, id) = if kind == Kind::Crafting {
                (12, 40, 4, 1757)
            } else {
                (7, 32, 0, 1949)
            };
            let g = Gate {
                kind,
                opened: 1539,
                outfits: vec![Outfit {
                    id,
                    slot,
                    members: false,
                }],
            };
            let door = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 14910, 0, 0).unwrap();
            let outside = door
                .coords
                .translate(0, if kind == Kind::Crafting { 1 } else { -1 }, 0)
                .unwrap();
            let inside = door
                .coords
                .translate(0, if kind == Kind::Crafting { -1 } else { 1 }, 0)
                .unwrap();
            let mut stats = Stats::default();
            stats.set_base(skill, level - 1);
            stats.set_current(skill, level);
            let mut worn = Inventory::new(15, StackType::Normal, false);
            let item = Item {
                id,
                count: 1,
                vars: 0,
            };
            worn.restore_slot(slot, Some(item)).unwrap();
            let cat = Catalog::default();
            assert!(
                g.prepare(door, outside, &stats, &worn, &cat, false)
                    .unwrap()
                    .entering
            );
            stats.set_current(skill, level - 1);
            assert!(matches!(
                g.prepare(door, outside, &stats, &worn, &cat, false),
                Err(Error::Level)
            ));
            stats.set_current(skill, level);
            worn.restore_slot(slot, None).unwrap();
            worn.restore_slot((slot + 1) % 15, Some(item)).unwrap();
            assert!(matches!(
                g.prepare(door, outside, &stats, &worn, &cat, false),
                Err(Error::Outfit)
            ));
            worn.restore_slot((slot + 1) % 15, None).unwrap();
            stats.set_current(skill, 1);
            assert!(
                !g.prepare(door, inside, &stats, &worn, &cat, false)
                    .unwrap()
                    .entering
            );
        }
    }
}
