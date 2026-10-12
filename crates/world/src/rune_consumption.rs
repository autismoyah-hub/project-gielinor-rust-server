//! MagicRuneManager.delAll: strict inventory transaction followed by pouch writes.
use crate::{
    bank_actions::Binding,
    inventory::{Catalog, Delete, Inventory},
    runes::{Source, Validation},
    variables::{Update, Variables},
};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultKind {
    AllUnlimited,
    Consumed { sunfire: bool },
    NotEnoughVarbit,
    NotEnoughInventory,
    InvalidPouchResult,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub kind: ResultKind,
    pub updates: Vec<Update>,
}
/// Configured plans come from `runes::Catalog`. A stale inventory slot is skipped
/// by the source; strict deletion still validates the remaining aggregate plan.
pub fn consume(
    inventory: &mut Inventory,
    vars: &mut Variables,
    items: &Catalog,
    bits: &BTreeMap<u32, Binding>,
    validations: &[Validation],
    sunfire: i32,
    logged: bool,
) -> Result<Outcome, String> {
    if validations.is_empty() {
        return Err("Rune validations must not be empty".into());
    }
    if validations.iter().any(|v| !v.valid()) {
        return Err("Invalid rune consumption plan".into());
    }
    let sourced: Vec<_> = validations
        .iter()
        .filter_map(|v| match v {
            Validation::Enough(s) => Some(s),
            _ => None,
        })
        .collect();
    let outcome = |kind| Outcome {
        kind,
        updates: vec![],
    };
    if sourced.is_empty() {
        return Ok(outcome(ResultKind::AllUnlimited));
    }
    let sources: Vec<_> = sourced.iter().flat_map(|s| s.iter()).collect();
    let mut pouch = Vec::new();
    for source in &sources {
        if let Source::Varbit { id, count } = source {
            let binding = *bits.get(id).ok_or("Unknown rune pouch varbit")?;
            if vars.bit(binding.bit).map_err(|e| format!("{e:?}"))? < *count {
                return Ok(outcome(ResultKind::NotEnoughVarbit));
            }
            pouch.push((binding, *count));
        }
    }
    let consume: Vec<_> = sources
        .iter()
        .filter_map(|source| match source {
            Source::Inventory { item, slot, count } => inventory
                .get(*slot)
                .filter(|i| i.id == *item)
                .map(|_| (*item, *count)),
            _ => None,
        })
        .collect();
    let mut next = inventory.clone();
    // The reference transaction result is unsuccessful when it contains no
    // queries. A pouch-only plan therefore fails here without reducing vars.
    if consume.is_empty() {
        return Ok(outcome(ResultKind::NotEnoughInventory));
    }
    for &(item, count) in &consume {
        if next.delete(items, Delete::new(item, count)).is_err() {
            return Ok(outcome(ResultKind::NotEnoughInventory));
        }
    }
    *inventory = next;
    let mut updates = Vec::new();
    for (binding, count) in pouch {
        let value = vars
            .bit(binding.bit)
            .map_err(|e| format!("{e:?}"))?
            .wrapping_sub(count);
        // Matches the reference assertion after the inventory has committed;
        // duplicate malformed sources may fail after earlier pouch writes.
        if value < 0 {
            return Ok(Outcome {
                kind: ResultKind::InvalidPouchResult,
                updates,
            });
        }
        updates.extend(
            vars.set_bit(binding.base, binding.bit, value, logged)
                .map_err(|e| format!("{e:?}"))?,
        );
    }
    Ok(Outcome {
        kind: ResultKind::Consumed {
            sunfire: consume.iter().any(|(id, _)| *id == sunfire),
        },
        updates,
    })
}
