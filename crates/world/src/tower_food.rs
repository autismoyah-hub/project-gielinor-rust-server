//! FloorOneScript.restoreFightFood. Withdraws existing shrimp; never creates food.
use crate::{
    inventory::{self, Catalog, Insert},
    simulation::PlayerData,
};
use std::collections::BTreeSet;
pub const SPACE_NAMES: [&str; 9] = [
    "Pure essence",
    "Rune essence",
    "Bones",
    "Big bones",
    "Raw beef",
    "Raw chicken",
    "Cowhide",
    "Cow hide",
    "Leather",
];
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub cooked: BTreeSet<i32>,
    /// Name equals Shrimps, with certificate aliases included as in source.
    pub bank_shrimp: BTreeSet<i32>,
    /// ownsNamed-style noncertificate type sets in SPACE_NAMES order.
    pub space: [BTreeSet<i32>; 9],
}
pub fn cooked_name(name: &str) -> bool {
    let name = name.to_lowercase();
    !name.contains("burnt")
        && !name.starts_with("raw ")
        && (name == "bread"
            || name == "cooked meat"
            || name.contains("chicken")
            || name.contains("shrimp"))
}
impl Config {
    pub fn restore(&self, data: &mut PlayerData, catalog: &Catalog) -> Option<&'static str> {
        if data
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| self.cooked.contains(&i.id))
        {
            return None;
        }
        for ids in &self.space {
            if data.inventory.free_space() > 0 {
                break;
            }
            crate::tower_kit::bank_named_extras(
                &mut data.inventory,
                &data.equipment,
                &mut data.bank,
                catalog,
                ids,
                0,
            );
        }
        if data.inventory.free_space() == 0 {
            return None;
        }
        let slot = data
            .bank
            .slots()
            .iter()
            .position(|i| i.is_some_and(|i| self.bank_shrimp.contains(&i.id)))?;
        let amount = data.bank.get(slot)?.count.min(3);
        let mut insert = Insert::new(0, amount);
        insert.unnote = true;
        insert.strict = false;
        inventory::transfer(
            &mut data.bank,
            &mut data.inventory,
            catalog,
            slot,
            amount,
            insert,
            false,
        )
        .ok()?;
        Some("You keep cooked shrimp for the fight.")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Inventory, Item, StackType, Template},
        stats::Stats,
        CoordGrid,
    };
    fn setup(size: usize) -> (Config, PlayerData, Catalog) {
        let mut c = Config {
            cooked: [315, 2309].into(),
            bank_shrimp: [315, 316].into(),
            ..Default::default()
        };
        c.space[0].insert(7936);
        c.space[2].insert(526);
        let mut p = PlayerData::new(
            1,
            "Food".into(),
            CoordGrid::new(10424, 120, 0).unwrap(),
            Stats::default(),
        );
        p.inventory = Inventory::new(size, StackType::Normal, false);
        let mut cat = Catalog::default();
        cat.notes.insert(
            316,
            Template {
                link: 315,
                template: 799,
            },
        );
        (c, p, cat)
    }
    fn put(inv: &mut Inventory, slot: usize, id: i32, count: i32) {
        inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
            .unwrap();
    }
    #[test]
    fn takes_at_most_three_existing_shrimp_and_unnotes() {
        let (c, mut p, cat) = setup(4);
        put(&mut p.bank, 0, 316, 5);
        assert_eq!(
            c.restore(&mut p, &cat),
            Some("You keep cooked shrimp for the fight.")
        );
        assert_eq!(
            p.inventory
                .slots()
                .iter()
                .flatten()
                .map(|i| i.id)
                .collect::<Vec<_>>(),
            [315; 3]
        );
        assert_eq!(p.bank.get(0).unwrap().count, 2);
        assert_eq!(c.restore(&mut p, &cat), None);
        assert_eq!(p.bank.get(0).unwrap().count, 2);
    }
    #[test]
    fn banks_loot_before_withdrawal_and_allows_partial_food_transfer() {
        let (c, mut p, cat) = setup(1);
        put(&mut p.inventory, 0, 7936, 1);
        put(&mut p.bank, 0, 315, 5);
        assert!(c.restore(&mut p, &cat).is_some());
        assert_eq!(p.inventory.get(0).unwrap().id, 315);
        assert_eq!(p.bank.get(0).unwrap().count, 4);
        assert_eq!(p.bank.get(1).unwrap().id, 7936);
    }
    #[test]
    fn no_food_does_not_mint_but_source_still_banks_loot_first() {
        let (c, mut p, cat) = setup(1);
        put(&mut p.inventory, 0, 526, 1);
        assert_eq!(c.restore(&mut p, &cat), None);
        assert!(p.inventory.get(0).is_none());
        assert_eq!(p.bank.get(0).unwrap().id, 526);
    }
    #[test]
    fn existing_bread_prevents_banking_or_withdrawal_and_kit_is_never_banked() {
        let (c, mut p, cat) = setup(1);
        put(&mut p.inventory, 0, 2309, 1);
        put(&mut p.bank, 0, 315, 5);
        assert_eq!(c.restore(&mut p, &cat), None);
        assert_eq!(p.bank.get(0).unwrap().count, 5);
        put(&mut p.inventory, 0, 1381, 1);
        assert_eq!(c.restore(&mut p, &cat), None);
        assert_eq!(p.inventory.get(0).unwrap().id, 1381);
    }
    #[test]
    fn cooked_classifier_preserves_substring_behavior() {
        for name in [
            "Bread",
            "Cooked meat",
            "Cooked chicken",
            "Shrimps",
            "Shrimp fragments",
        ] {
            assert!(cooked_name(name));
        }
        for name in ["Raw shrimps", "Burnt chicken", "Raw beef", "Bones"] {
            assert!(!cooked_name(name));
        }
    }
}
