//! FloorOneScript.tanHides inventory core. The host owns floor/reach checks,
//! diary synchronization, persistence, and the resulting journal refresh.
use crate::inventory::{Catalog, Delete, Insert, Inventory};
use std::collections::BTreeSet;

/// FloorOneRules.isCowhide accepts the canonical ID or a case-insensitive name,
/// but excludes certificates before either check.
pub fn is_cowhide(id: i32, name: &str, certificate: bool) -> bool {
    if certificate {
        return false;
    }
    if id == 1739 {
        return true;
    }
    let name = name.to_lowercase();
    name.contains("cowhide") || name == "cow hide"
}

pub struct HandlerConfig {
    pub origin: crate::CoordGrid,
    pub inventory: Config,
    pub skills: [u8; 3],
    pub bones: u16,
}
impl HandlerConfig {
    pub fn contains(&self, coords: crate::CoordGrid) -> bool {
        crate::tower_layout::contains(self.origin, coords)
    }
}
pub struct Config {
    pub hides: BTreeSet<i32>,
    pub leather: i32,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub found_hides: bool,
    pub converted: i32,
    pub no_space: bool,
}
impl Config {
    pub fn tan(&self, inventory: &mut Inventory, catalog: &Catalog) -> Outcome {
        // Capture slots before any mutation. Kotlin processes one hide per slot,
        // descending, even if an unusual cache type stacks multiple cowhides.
        let slots: Vec<_> = inventory
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| item.filter(|i| self.hides.contains(&i.id)).map(|_| slot))
            .collect();
        let mut outcome = Outcome {
            found_hides: !slots.is_empty(),
            converted: 0,
            no_space: false,
        };
        for slot in slots.into_iter().rev() {
            let Some(item) = inventory.get(slot) else {
                continue;
            };
            let mut query = Delete::new(item.id, 1);
            query.strict_slot = Some(slot);
            if inventory.delete(catalog, query).is_err() {
                continue;
            }
            if inventory
                .insert(catalog, Insert::new(self.leather, 1))
                .is_err()
            {
                // Separate commits: refund through normal insertion. Original
                // code neither restores the old slot nor preserves item vars.
                let _ = inventory.insert(catalog, Insert::new(item.id, 1));
                outcome.no_space = true;
                break;
            }
            outcome.converted += 1;
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn recognized_names_exclude_notes_even_for_canonical_id() {
        assert!(is_cowhide(1739, "changed cache name", false));
        assert!(is_cowhide(900, "COW HIDE", false));
        assert!(is_cowhide(901, "Heavy cowhide", false));
        assert!(!is_cowhide(1739, "Cowhide", true));
        assert!(!is_cowhide(902, "Leather body", false));
    }
    #[test]
    fn descending_slots_and_one_hide_per_stack_match_original() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(900);
        let config = Config {
            hides: [1739, 900].into(),
            leather: 1741,
        };
        let mut inv = Inventory::new(4, StackType::Normal, false);
        for (slot, id, count) in [(0, 1739, 1), (2, 900, 3), (3, 1739, 1)] {
            inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
                .unwrap();
        }
        assert_eq!(
            config.tan(&mut inv, &catalog),
            Outcome {
                found_hides: true,
                converted: 3,
                no_space: false
            }
        );
        assert_eq!(inv.get(2).unwrap().count, 2);
        assert_eq!(inv.get(2).unwrap().id, 900);
        assert_eq!([0, 1, 3].map(|slot| inv.get(slot).unwrap().id), [1741; 3]);
    }
    #[test]
    fn full_stacked_hide_refunds_and_stops_without_converting() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(900);
        let mut inv = Inventory::new(1, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 900,
                count: 3,
                vars: 0,
            }),
        )
        .unwrap();
        let config = Config {
            hides: [900].into(),
            leather: 1741,
        };
        assert_eq!(
            config.tan(&mut inv, &catalog),
            Outcome {
                found_hides: true,
                converted: 0,
                no_space: true
            }
        );
        assert_eq!(inv.get(0).unwrap().count, 3);
    }
}
