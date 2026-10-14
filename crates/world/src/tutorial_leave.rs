//! TutorialLeave's ordered inventory grants; worn equipment is not cleared by the source.
use crate::{
    inventory::{Catalog, Insert, Inventory},
    smithing_make::total,
    tutorial_progression, CoordGrid,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: tutorial_progression::Config,
    pub items: [i32; 18],
    pub coins: i32,
}
pub const ITEMS: [(&str, i32); 18] = [
    ("bronze_axe", 1),
    ("bronze_pickaxe", 1),
    ("tinderbox", 1),
    ("net", 1),
    ("shrimp", 1),
    ("bronze_dagger", 1),
    ("bronze_sword", 1),
    ("wooden_shield", 1),
    ("shortbow", 1),
    ("bronze_arrow", 25),
    ("airrune", 25),
    ("mindrune", 15),
    ("waterrune", 6),
    ("earthrune", 4),
    ("bodyrune", 2),
    ("bucket_empty", 1),
    ("pot_empty", 1),
    ("bread", 1),
];
impl Config {
    pub fn destination() -> CoordGrid {
        CoordGrid::new(3221, 3218, 0).unwrap()
    }
    pub fn grant(self, inv: &mut Inventory, bank: &mut Inventory, catalog: &Catalog) {
        // Inventory.fillNulls marks occupied slots only. Each invAdd commits independently;
        // the reference ignores a failed transaction and continues to the next item.
        for slot in 0..inv.slots().len() {
            if inv.get(slot).is_some() {
                inv.restore_slot(slot, None).unwrap();
            }
        }
        for (id, (_, count)) in self.items.into_iter().zip(ITEMS) {
            let _ = inv.insert(catalog, Insert::new(id, count));
        }
        let current = total(bank, catalog, self.coins);
        if current < 25 {
            let _ = bank.insert(
                catalog,
                Insert::new(self.coins, 25i32.wrapping_sub(current)),
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Item, StackType},
        variables::{Transmit, Varp},
    };
    pub fn config() -> Config {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        Config {
            progression: tutorial_progression::Config {
                step: varp(1),
                completed: varp(2),
            },
            items: std::array::from_fn(|i| i as i32 + 100),
            coins: 995,
        }
    }
    pub fn catalog() -> Catalog {
        let mut c = Catalog::default();
        c.stackable.extend(109..=114);
        c.stackable.insert(995);
        c
    }
    #[test]
    fn ordered_kit_clears_backpack_and_only_tops_up_bank_coins() {
        let c = config();
        let types = catalog();
        for coins in [0, 1, 24, 25, 100, i32::MAX] {
            let mut inv = Inventory::new(28, StackType::Normal, false);
            inv.restore_slot(
                27,
                Some(Item {
                    id: 42,
                    count: 1,
                    vars: 9,
                }),
            )
            .unwrap();
            inv.clear_modified();
            let mut bank = Inventory::new(2, StackType::Always, false);
            bank.insert(&types, Insert::new(42, 100)).unwrap();
            if coins > 0 {
                bank.insert(&types, Insert::new(995, coins)).unwrap();
            }
            c.grant(&mut inv, &mut bank, &types);
            for (i, (_, amount)) in ITEMS.into_iter().enumerate() {
                assert_eq!(
                    inv.get(i),
                    Some(Item {
                        id: c.items[i],
                        count: amount,
                        vars: 0
                    })
                );
            }
            assert!(inv.get(27).is_none());
            assert_eq!(inv.free_space(), 10);
            assert_eq!(total(&bank, &types, 995), coins.max(25));
            assert_eq!(total(&bank, &types, 42), 100);
            assert!(!inv.modified().contains(&26));
        }
    }
    #[test]
    fn failed_bank_topup_does_not_rollback_backpack_grants() {
        let c = config();
        let types = catalog();
        let mut inv = Inventory::new(2, StackType::Normal, false);
        let mut bank = Inventory::new(1, StackType::Always, false);
        bank.insert(&types, Insert::new(42, 1)).unwrap();
        c.grant(&mut inv, &mut bank, &types);
        assert_eq!((inv.get(0).unwrap().id, inv.get(1).unwrap().id), (100, 101));
        assert_eq!(total(&bank, &types, 995), 0);
    }
}
