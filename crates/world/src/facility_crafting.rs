//! Ordinary grouped-material crafting at attested facilities. Selected inventory
//! exchange and precise XP staging are shared with assembly.
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
    pub input_count: u8,
    /// One additional ordinary material per product, captured as its own slot.
    pub secondary: Option<i32>,
    /// Every listed ordinary tool must remain carried; never consumed as material.
    pub required_inventory_tools: &'static [i32],
    /// One additional ordinary output credited in the same staged transaction.
    pub byproduct: Option<i32>,
    pub output: i32,
    pub output_count: u8,
    pub level: u8,
    pub xp_tenths: u16,
    pub members: bool,
    pub first_ticks: u8,
    pub cycle_ticks: u8,
    pub failure: Option<Failure>,
    pub label: &'static str,
}
#[derive(Clone, Debug)]
pub struct Config {
    /// None is material-use-only; it never admits a fabricated normal operation.
    pub locations: BTreeMap<u32, Option<u8>>,
    pub recipes: Arc<Vec<Recipe>>,
    pub action: &'static str,
    pub empty_message: &'static str,
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
    pub action: &'static str,
    pub animation: u16,
    pub location_animation: Option<u16>,
}
impl Recipe {
    pub fn accepts_input(self, id: i32) -> bool {
        id == self.input || self.secondary == Some(id)
    }
    pub fn available(self, inventory: &Inventory) -> usize {
        if !(1..=28).contains(&self.input_count)
            || self.secondary.is_some_and(|id| id < 0 || id == self.input)
        {
            return 0;
        }
        if self.required_inventory_tools.iter().any(|&id| {
            !inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == id && i.count == 1 && i.vars == 0)
        }) {
            return 0;
        }
        let owned = |id| {
            inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == id && i.count == 1 && i.vars == 0)
                .count()
        };
        let primary = owned(self.input) / usize::from(self.input_count);
        self.secondary
            .map_or(primary, |id| primary.min(owned(id)))
            .min(28)
    }
    // Only called after group length/policy validation; secondary occupies the last slot.
    fn inputs(self, slots: &[usize]) -> Vec<(i32, usize)> {
        let mut inputs: Vec<_> = slots[..usize::from(self.input_count)]
            .iter()
            .map(|&slot| (self.input, slot))
            .collect();
        if let Some(id) = self.secondary {
            inputs.push((id, slots[usize::from(self.input_count)]));
        }
        inputs
    }
}
impl Request {
    pub fn available(&self, inventory: &Inventory) -> usize {
        self.recipes
            .iter()
            .map(|r| r.available(inventory))
            .max()
            .unwrap_or(0)
    }
    pub(crate) fn groups(
        &self,
        recipe: Recipe,
        inventory: &Inventory,
        count: usize,
    ) -> Option<Vec<Vec<usize>>> {
        if recipe.available(inventory) == 0 {
            return None;
        }
        let matches = |slot, input| {
            inventory
                .get(slot)
                .is_some_and(|v| v.id == input && v.count == 1 && v.vars == 0)
        };
        if let Some((slot, input)) = self.selected {
            if !recipe.accepts_input(input) || !matches(slot, input) {
                return None;
            }
        }
        let gather = |input| {
            let mut slots = Vec::new();
            if let Some((slot, id)) = self.selected {
                if id == input {
                    slots.push(slot);
                }
            }
            for (slot, item) in inventory.slots().iter().enumerate() {
                if !slots.contains(&slot)
                    && item.is_some_and(|v| v.id == input && v.count == 1 && v.vars == 0)
                {
                    slots.push(slot);
                }
            }
            slots
        };
        let primary = gather(recipe.input);
        let mut groups: Vec<Vec<usize>> = primary
            .chunks_exact(usize::from(recipe.input_count))
            .take(count.min(28))
            .map(|s| s.to_vec())
            .collect();
        if let Some(input) = recipe.secondary {
            let secondary = gather(input);
            groups.truncate(secondary.len());
            for (group, slot) in groups.iter_mut().zip(secondary) {
                group.push(slot);
            }
        }
        (!groups.is_empty()).then_some(groups)
    }
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub request: Request,
    pub recipe: Recipe,
    pub groups: Arc<Vec<Vec<usize>>>,
    pub index: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Policy,
    Members,
    Level,
    Exchange(assembly::Outcome),
}
/// Original unary entry point remains useful to callers attesting one raw.
pub fn prepare(
    recipe: Recipe,
    slot: usize,
    inventory: &Inventory,
    catalog: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<(Inventory, Stats, Option<Advancement>), Error> {
    prepare_group(recipe, &[slot], inventory, catalog, stats, rate, member)
}
pub fn prepare_group(
    recipe: Recipe,
    slots: &[usize],
    inventory: &Inventory,
    catalog: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<(Inventory, Stats, Option<Advancement>), Error> {
    if !(1..=99).contains(&recipe.level)
        || !(1..=28).contains(&recipe.input_count)
        || !(1..=28).contains(&recipe.output_count)
        || slots.len() != usize::from(recipe.input_count) + usize::from(recipe.secondary.is_some())
        || recipe
            .secondary
            .is_some_and(|id| id < 0 || id == recipe.input)
        || recipe.byproduct.is_some_and(|id| id < 0)
        || recipe.first_ticks == 0
        || recipe.cycle_ticks == 0
        || inventory.stack != StackType::Normal
        || inventory.placeholders
        || recipe.input == recipe.output
        || recipe.input < 0
        || recipe.output < 0
        || (recipe.xp_tenths > 0 && rate <= 0)
    {
        return Err(Error::Policy);
    }
    if recipe.members && !member {
        return Err(Error::Members);
    }
    if (stats.current(12) as u8) < recipe.level {
        return Err(Error::Level);
    }
    if recipe.required_inventory_tools.iter().any(|id| {
        *id == recipe.input
            || recipe.secondary == Some(*id)
            || !assembly::ToolRequirement {
                inventory: std::slice::from_ref(id),
                worn: &[],
            }
            .available(inventory, inventory, catalog)
    }) {
        return Err(Error::Policy);
    }
    let inputs = recipe.inputs(slots);
    let mut outputs = vec![recipe.output; usize::from(recipe.output_count)];
    if let Some(id) = recipe.byproduct {
        outputs.push(id);
    }
    let next = assembly::exchange_slots(inventory, catalog, &inputs, &outputs, true)
        .map_err(Error::Exchange)?;
    let (next_stats, advanced) =
        assembly::prepare_reward_value(stats, 12, f64::from(recipe.xp_tenths) / 10.0, rate)
            .map_err(Error::Exchange)?;
    Ok((next, next_stats, advanced))
}

/// An explicit alternate transaction. None output means the raw is destroyed;
/// zero reward must preserve Stats and emit no fabricated advancement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Failure {
    pub low: i32,
    pub high: i32,
    pub output: Option<(i32, u8)>,
    pub xp_tenths: u16,
    pub message: &'static str,
}
pub type Prepared = (Inventory, Stats, Option<Advancement>);
pub fn resolve_group(
    recipe: Recipe,
    slots: &[usize],
    inventory: &Inventory,
    catalog: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
    random: &mut impl crate::npc::RandomSource,
) -> Result<(Prepared, bool), Error> {
    let success = prepare_group(recipe, slots, inventory, catalog, stats, rate, member)?;
    let Some(failure) = recipe.failure else {
        return Ok((success, true));
    };
    if failure.low < 0 || failure.low > failure.high || failure.high > u16::MAX as i32 {
        return Err(Error::Policy);
    }
    let outputs = match failure.output {
        None => Vec::new(),
        Some((id, count)) => {
            if id < 0 || !(1..=28).contains(&count) {
                return Err(Error::Policy);
            }
            vec![id; usize::from(count)]
        }
    };
    let inputs = recipe.inputs(slots);
    let failed_inventory = assembly::exchange_slots(inventory, catalog, &inputs, &outputs, true)
        .map_err(Error::Exchange)?;
    let (failed_stats, failed_advanced) =
        assembly::prepare_reward_value(stats, 12, f64::from(failure.xp_tenths) / 10.0, rate)
            .map_err(Error::Exchange)?;
    let level = (stats.current(12) as u8).clamp(1, 99) as i32;
    let threshold = (crate::skills::success_rate(failure.low, failure.high, level, 99) * 256.0)
        .clamp(0.0, 256.0) as u32;
    if random.below(256) < threshold {
        Ok((success, true))
    } else {
        Ok(((failed_inventory, failed_stats, failed_advanced), false))
    }
}
