//! Prayer burial and chapel offerings: delayed consumption and source XP modifiers.
use crate::{
    inventory::{Catalog, Delete, Inventory},
    simulation::PlayerData,
    skills, stat_ops, CoordGrid,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Config {
    pub skill: u8,
    pub animation: u16,
    pub bones: BTreeMap<i32, f64>,
    pub necklace: Option<i32>,
    pub restore: BTreeMap<i32, i32>,
    /// WornXpModifiers values for the Prayer stat, summed before dividing by100.
    pub worn_xp_percent: BTreeMap<i32, i32>,
    pub chaos: Option<[i32; 4]>,
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub bone: i32,
    pub restore: i32,
}
#[derive(Debug)]
pub enum Error {
    MissingBone,
    Inventory,
    Restore,
    Xp,
}
#[derive(Debug)]
pub struct Completed {
    pub restore: Option<stat_ops::Update>,
    pub advancement: skills::Advancement,
}
#[derive(Clone, Copy, Debug)]
pub enum State {
    Confirm {
        token: u64,
        bone: i32,
    },
    Bury(Pending),
    Offer {
        bone: i32,
        animation: u16,
        offered: bool,
        xp: Option<f64>,
        continuation: Option<crate::tower_offering::Continuation>,
    },
    CrateWalk {
        step: u8,
        attempts: u8,
        pending: Option<CoordGrid>,
    },
}
#[derive(Debug)]
pub enum Event {
    Confirm {
        token: u64,
    },
    Closed,
    Animation(u16),
    Completed {
        skill: u8,
        result: Completed,
    },
    Message(&'static str),
    /// Coroutine return value, emitted only on normal completion, never cancellation.
    OfferingFinished {
        offered: bool,
    },
}
pub(crate) fn contains(inv: &Inventory, bone: i32) -> bool {
    inv.slots()
        .iter()
        .flatten()
        .any(|i| i.id == bone && i.count > 0)
}
impl Config {
    pub fn requires_confirmation(&self, coords: CoordGrid) -> bool {
        self.chaos.is_some_and(|[x0, z0, x1, z1]| {
            (x0..=x1).contains(&coords.x()) && (z0..=z1).contains(&coords.z())
        })
    }
    pub fn prepare(&self, player: &PlayerData, bone: i32) -> Result<Pending, Error> {
        if !self.bones.contains_key(&bone) || !contains(&player.inventory, bone) {
            return Err(Error::MissingBone);
        }
        let necklace = self
            .necklace
            .is_some_and(|id| player.equipment.get(2).is_some_and(|i| i.id == id));
        Ok(Pending {
            bone,
            restore: if necklace {
                self.restore.get(&bone).copied().unwrap_or(0)
            } else {
                0
            },
        })
    }
    pub fn finish(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        pending: Pending,
    ) -> Result<Option<Completed>, Error> {
        self.finish_with_multiplier(
            player,
            catalog,
            pending,
            1.0,
            self.bones.get(&pending.bone).copied(),
        )
    }
    /// Prayer.offerOnAltar omits necklace restoration and applies the chapel
    /// multiplier before the worn modifier and account XP rate.
    pub fn finish_offering(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        bone: i32,
    ) -> Result<Option<Completed>, Error> {
        self.finish_offering_xp(player, catalog, bone, self.bones.get(&bone).copied())
    }
    pub(crate) fn finish_offering_xp(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        bone: i32,
        xp: Option<f64>,
    ) -> Result<Option<Completed>, Error> {
        self.finish_with_multiplier(player, catalog, Pending { bone, restore: 0 }, 2.0, xp)
    }
    fn finish_with_multiplier(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        pending: Pending,
        multiplier: f64,
        xp: Option<f64>,
    ) -> Result<Option<Completed>, Error> {
        if !contains(&player.inventory, pending.bone) {
            return Ok(None);
        }
        // Kotlin invDel searches from slot0, not the originally clicked slot.
        player
            .inventory
            .delete(catalog, Delete::new(pending.bone, 1))
            .map_err(|_| Error::Inventory)?;
        let restore = if pending.restore > 0 {
            Some(
                player
                    .stats
                    .apply(
                        self.skill,
                        stat_ops::Operation::Heal,
                        pending.restore,
                        0,
                        false,
                    )
                    .map_err(|_| Error::Restore)?,
            )
        } else {
            None
        };
        // Source resolves the XP parameter after deleting the bone.
        let xp = xp.ok_or(Error::MissingBone)?;
        let percent = player
            .equipment
            .slots()
            .iter()
            .flatten()
            .fold(0i32, |sum, item| {
                sum.wrapping_add(self.worn_xp_percent.get(&item.id).copied().unwrap_or(0))
            });
        let xp = xp * multiplier * (1.0 + percent as f64 / 100.0);
        let advancement = skills::advance(
            &mut player.stats,
            self.skill,
            xp,
            player.xp_rate_hundreds as f64 / 100.0,
            99,
        )
        .map_err(|_| Error::Xp)?;
        Ok(Some(Completed {
            restore,
            advancement,
        }))
    }
}
