//! Authoritative location handlers after reach, plane, protection and target validation.
use crate::{
    inventory::{Catalog, Error, Insert},
    simulation::PlayerData,
    CoordGrid,
};
#[derive(Clone, Copy, Debug)]
pub enum Handler {
    VialCrate { origin: CoordGrid, item: i32 },
    PrayerAltar { origin: CoordGrid, prayer: u8 },
    BankBooth { at: CoordGrid },
}
impl Handler {
    pub fn arrival_delay(self) -> bool {
        matches!(self, Self::PrayerAltar { .. })
    }
    pub fn apply(
        self,
        player: &mut PlayerData,
        catalog: &Catalog,
    ) -> Result<Option<&'static str>, Error> {
        let origin = match self {
            Self::VialCrate { origin, .. } | Self::PrayerAltar { origin, .. } => origin,
            Self::BankBooth { .. } => return Ok(None),
        };
        // TowerMap.contains: the same cached types elsewhere do not run floor hooks.
        if player.coords.level() > 1
            || !(0..128).contains(&(player.coords.x() - origin.x()))
            || !(0..128).contains(&(player.coords.z() - origin.z()))
        {
            return Ok(None);
        }
        match self {
            Self::BankBooth { .. } => Ok(None),
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
