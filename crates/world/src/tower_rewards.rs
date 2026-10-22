//! Reusable staging through the ordinary inventory engine. No economy policy.
//! Callers still own eligibility, durable consumption and save acknowledgement.
use crate::inventory::{Catalog, Error, Insert, Inventory};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant { pub id: i32, pub count: i32 }

pub fn stage(inventory: &Inventory, catalog: &Catalog, grants: &[Grant]) -> Result<Inventory, Error> {
    let mut proposed = inventory.clone();
    for grant in grants {
        if grant.id < 0 || grant.count <= 0 { return Err(Error::InvalidCount); }
        proposed.insert(catalog, Insert::new(grant.id, grant.count))?;
    }
    Ok(proposed)
}

/// Exact tool identity across carried, worn and banked possessions. Duplicate
/// configured tools yield one recovery grant. This never restores money or gear.
pub fn missing_tools(tools: &[i32], possessions: &[&Inventory]) -> Result<Vec<Grant>, Error> {
    if tools.iter().any(|id| *id < 0) { return Err(Error::InvalidCount); }
    let owned: BTreeSet<_> = possessions.iter().flat_map(|inv| inv.slots().iter().flatten())
        .filter(|item| item.count > 0).map(|item| item.id).collect();
    let mut seen = BTreeSet::new();
    Ok(tools.iter().copied().filter(|id| !owned.contains(id) && seen.insert(*id))
        .map(|id| Grant { id, count: 1 }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn late_capacity_and_stack_overflow_leave_all_original_slots_unchanged() {
        let mut catalog = Catalog::default(); catalog.stackable.insert(995);
        let mut pack = Inventory::new(1, StackType::Normal, false);
        pack.restore_slot(0, Some(Item { id: 995, count: 50, vars: 0 })).unwrap();
        let before = pack.clone();
        assert_eq!(stage(&pack, &catalog, &[Grant { id: 995, count: 25 }, Grant { id: 1917, count: 1 }]), Err(Error::NotEnoughSpace));
        assert_eq!(pack, before);
        assert!(stage(&pack, &catalog, &[Grant { id: 995, count: i32::MAX }]).is_err());
        assert_eq!(pack, before);
        assert!(stage(&pack, &catalog, &[Grant { id: -1, count: 1 }]).is_err());
        assert!(stage(&pack, &catalog, &[Grant { id: 995, count: 0 }]).is_err());
    }
    #[test]
    fn tool_recovery_checks_every_container_and_deduplicates_without_granting_other_items() {
        let mut pack = Inventory::new(2, StackType::Normal, false);
        let mut worn = Inventory::new(15, StackType::Normal, false);
        let mut bank = Inventory::new(5, StackType::Always, true);
        pack.restore_slot(0, Some(Item { id: 10, count: 1, vars: 0 })).unwrap();
        worn.restore_slot(3, Some(Item { id: 11, count: 1, vars: 0 })).unwrap();
        bank.restore_slot(4, Some(Item { id: 12, count: 1, vars: 0 })).unwrap();
        let missing = missing_tools(&[10, 11, 12, 13, 13], &[&pack, &worn, &bank]).unwrap();
        assert_eq!(missing, [Grant { id: 13, count: 1 }]);
        let staged = stage(&pack, &Catalog::default(), &missing).unwrap();
        assert_eq!(staged.get(1).unwrap().id, 13);
        assert_eq!(pack.get(1), None);
    }
}
