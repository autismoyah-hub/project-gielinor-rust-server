//! CombatMeleeAttributeCollector and effective-level equipment state.
use crate::{inventory::Inventory, melee_modifiers::Melee};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
pub struct Catalog {
    /// Pre-resolved attributes for each item in a specific equipment slot.
    pub items: BTreeMap<(usize, i32), u64>,
    pub obsidian: [BTreeSet<i32>; 3],
    pub dharok: [BTreeSet<i32>; 4],
    pub void: [BTreeSet<i32>; 4],
    pub corpbane: BTreeSet<i32>,
    pub soulreaper: BTreeSet<i32>,
}
impl Catalog {
    fn set<const N: usize>(worn: &Inventory, slots: [usize; N], sets: &[BTreeSet<i32>; N]) -> bool {
        slots
            .into_iter()
            .zip(sets)
            .all(|(slot, ids)| worn.get(slot).is_some_and(|item| ids.contains(&item.id)))
    }
    pub fn collect(&self, worn: &Inventory, kind: Option<u8>, skull: i32) -> u64 {
        let mut bits = 0;
        for slot in [0, 2, 3, 4, 7] {
            if let Some(item) = worn.get(slot) {
                bits |= self.items.get(&(slot, item.id)).copied().unwrap_or(0);
            }
        }
        let mut add = |attr: Melee| {
            bits |= 1u64 << attr as u8;
        };
        if kind == Some(3) {
            add(Melee::Crush);
        } else if kind == Some(1) {
            add(Melee::Stab);
        }
        if skull == 3 {
            add(Melee::ForinthrySurge);
        }
        if Self::set(worn, [0, 4, 7], &self.obsidian) {
            add(Melee::Obsidian);
        }
        if Self::set(worn, [0, 4, 7, 3], &self.dharok) {
            add(Melee::Dharoks);
        }
        if kind == Some(1)
            && worn
                .get(3)
                .is_some_and(|item| self.corpbane.contains(&item.id))
        {
            add(Melee::CorpBaneWeapon);
        }
        bits
    }
    pub fn void_multiplier(&self, worn: &Inventory) -> f64 {
        if Self::set(worn, [0, 4, 7, 9], &self.void) {
            1.1
        } else {
            1.0
        }
    }
    pub fn soul_multiplier(&self, worn: &Inventory, souls: i32) -> f64 {
        if worn
            .get(3)
            .is_some_and(|item| self.soulreaper.contains(&item.id))
        {
            1.0 + souls as f64 * 0.06
        } else {
            1.0
        }
    }
}
