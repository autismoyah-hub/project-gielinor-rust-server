//! CombatRangedAttributeCollector and the distinct accuracy/damage void rules.
use crate::{inventory::Inventory, ranged_modifiers::Ranged};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
pub struct Catalog {
    /// Canonical item attributes resolved by equipment slot at startup.
    pub items: BTreeMap<(usize, i32), u64>,
    pub chinchompas: BTreeSet<i32>,
    pub void_helm: BTreeSet<i32>,
    pub void_gloves: BTreeSet<i32>,
    pub regular_top: BTreeSet<i32>,
    pub regular_robe: BTreeSet<i32>,
    pub elite_top: BTreeSet<i32>,
    pub elite_robe: BTreeSet<i32>,
}
fn wearing(worn: &Inventory, slot: usize, ids: &BTreeSet<i32>) -> bool {
    worn.get(slot).is_some_and(|item| ids.contains(&item.id))
}
impl Catalog {
    /// Global AttackType Heavy=7; global AttackStyle Accurate/Rapid/Longrange=5/6/7.
    pub fn collect(
        &self,
        worn: &Inventory,
        kind: Option<u8>,
        style: Option<u8>,
        skull: i32,
    ) -> u64 {
        let mut attributes = 0;
        for slot in [0, 2, 3, 4, 7] {
            if let Some(item) = worn.get(slot) {
                attributes |= self.items.get(&(slot, item.id)).copied().unwrap_or(0);
            }
        }
        if kind == Some(7) {
            attributes |= 1 << Ranged::Heavy as u8;
        }
        if wearing(worn, 3, &self.chinchompas) {
            let fuse = match style {
                Some(5) => Some(Ranged::ShortFuse),
                Some(6) => Some(Ranged::MediumFuse),
                Some(7) => Some(Ranged::LongFuse),
                _ => None,
            };
            if let Some(fuse) = fuse {
                attributes |= 1 << fuse as u8;
            }
        }
        if skull == 3 {
            attributes |= 1 << Ranged::ForinthrySurge as u8;
        }
        attributes
    }
    pub fn void_multipliers(&self, worn: &Inventory) -> (f64, f64) {
        if !wearing(worn, 0, &self.void_helm) || !wearing(worn, 9, &self.void_gloves) {
            return (1.0, 1.0);
        }
        let regular_top = wearing(worn, 4, &self.regular_top);
        let regular_robe = wearing(worn, 7, &self.regular_robe);
        let elite_top = wearing(worn, 4, &self.elite_top);
        let elite_robe = wearing(worn, 7, &self.elite_robe);
        let accuracy = if (regular_top || elite_top) && (regular_robe || elite_robe) {
            1.1
        } else {
            1.0
        };
        let damage = if elite_top && elite_robe {
            1.125
        } else if regular_top && regular_robe {
            1.1
        } else {
            1.0
        };
        (accuracy, damage)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn worn(items: &[(usize, i32)]) -> Inventory {
        let mut inv = Inventory::new(14, StackType::Normal, false);
        for &(slot, id) in items {
            inv.restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        inv
    }
    #[test]
    fn mixed_void_is_accurate_but_has_no_damage_multiplier() {
        let c = Catalog {
            void_helm: [1].into(),
            void_gloves: [2].into(),
            regular_top: [3].into(),
            regular_robe: [4].into(),
            elite_top: [5].into(),
            elite_robe: [6].into(),
            ..Catalog::default()
        };
        for (top, robe, expected) in [
            (3, 4, (1.1, 1.1)),
            (5, 6, (1.1, 1.125)),
            (3, 6, (1.1, 1.0)),
            (5, 4, (1.1, 1.0)),
            (0, 6, (1.0, 1.0)),
        ] {
            assert_eq!(
                c.void_multipliers(&worn(&[(0, 1), (9, 2), (4, top), (7, robe)])),
                expected
            );
        }
        assert_eq!(
            c.void_multipliers(&worn(&[(0, 1), (4, 5), (7, 6)])),
            (1.0, 1.0)
        );
    }
    #[test]
    fn fuse_uses_weapon_category_and_style_while_attributes_use_equipment_slot() {
        let c = Catalog {
            chinchompas: [10].into(),
            items: [((0, 20), 1 << Ranged::CrystalHelm as u8)].into(),
            ..Catalog::default()
        };
        let heavy = 1 << Ranged::Heavy as u8;
        for (style, fuse) in [
            (None, 0),
            (Some(5), 1 << Ranged::ShortFuse as u8),
            (Some(6), 1 << Ranged::MediumFuse as u8),
            (Some(7), 1 << Ranged::LongFuse as u8),
        ] {
            assert_eq!(
                c.collect(&worn(&[(3, 10), (0, 20)]), Some(7), style, 3),
                heavy
                    | fuse
                    | (1 << Ranged::CrystalHelm as u8)
                    | (1 << Ranged::ForinthrySurge as u8)
            );
        }
        assert_eq!(
            c.collect(&worn(&[(2, 10), (4, 20)]), Some(5), Some(5), 0),
            0
        );
    }
}
