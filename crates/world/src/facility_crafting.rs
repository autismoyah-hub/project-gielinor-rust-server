//! Ordinary unary crafting at an attested facility. Inventory and XP transactions
//! are shared with assembly; this module owns only recipes and captured choices.
use crate::{
    assembly,
    inventory::{Catalog, Inventory, StackType},
    locations::Location,
    skills::Advancement,
    stats::Stats,
    CoordGrid,
};
use std::{collections::BTreeMap, sync::Arc};
pub(crate) const QUEUE: i32 = -2391016;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub input: i32,
    pub output: i32,
    pub level: u8,
    pub xp_tenths: u16,
    pub members: bool,
    pub label: &'static str,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub locations: BTreeMap<u32, u8>,
    pub recipes: Arc<Vec<Recipe>>,
    pub ticks: u8,
    pub animation: u16,
    pub location_animation: Option<u16>,
}
#[derive(Clone, Debug)]
pub struct Request {
    pub recipes: Arc<Vec<Recipe>>,
    pub location: Location,
    pub revision: u64,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub selected: Option<(usize, i32)>,
    pub ticks: u8,
    pub animation: u16,
    pub location_animation: Option<u16>,
}
impl Request {
    pub fn available(&self, inventory: &Inventory) -> usize {
        self.recipes
            .iter()
            .map(|r| {
                inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == r.input && i.count == 1 && i.vars == 0)
                    .count()
            })
            .max()
            .unwrap_or(0)
            .min(28)
    }
    pub(crate) fn slots(
        &self,
        recipe: Recipe,
        inventory: &Inventory,
        count: usize,
    ) -> Option<Vec<usize>> {
        let matches = |slot| {
            inventory
                .get(slot)
                .is_some_and(|v| v.id == recipe.input && v.count == 1 && v.vars == 0)
        };
        let mut slots = Vec::new();
        if let Some((slot, input)) = self.selected {
            if input != recipe.input || !matches(slot) {
                return None;
            }
            slots.push(slot);
        }
        for (slot, item) in inventory.slots().iter().enumerate() {
            if !slots.contains(&slot)
                && item.is_some_and(|v| v.id == recipe.input && v.count == 1 && v.vars == 0)
            {
                slots.push(slot);
            }
        }
        slots.truncate(count.min(28));
        (!slots.is_empty()).then_some(slots)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub request: Request,
    pub recipe: Recipe,
    pub slots: Arc<Vec<usize>>,
    pub index: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Policy,
    Members,
    Level,
    Exchange(assembly::Outcome),
}
pub fn prepare(
    recipe: Recipe,
    slot: usize,
    inventory: &Inventory,
    catalog: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<(Inventory, Stats, Option<Advancement>), Error> {
    if !(1..=99).contains(&recipe.level)
        || inventory.stack != StackType::Normal
        || inventory.placeholders
        || recipe.input == recipe.output
        || recipe.input < 0
        || recipe.output < 0
        || recipe.xp_tenths == 0
        || rate <= 0
    {
        return Err(Error::Policy);
    }
    if recipe.members && !member {
        return Err(Error::Members);
    }
    if (stats.current(12) as u8) < recipe.level {
        return Err(Error::Level);
    }
    let next = assembly::exchange_slots(
        inventory,
        catalog,
        &[(recipe.input, slot)],
        &[recipe.output],
        true,
    )
    .map_err(Error::Exchange)?;
    let (next_stats, advanced) =
        assembly::prepare_reward_value(stats, 12, f64::from(recipe.xp_tenths) / 10.0, rate)
            .map_err(Error::Exchange)?;
    Ok((next, next_stats, advanced))
}
