//! Ordinary dairy-cow harvesting. The host supplies attested definitions.
use crate::{
    inventory::{Catalog, Delete, Error, Insert, Inventory},
    locations::Location,
    CoordGrid,
};
use std::collections::BTreeMap;
pub(crate) const QUEUE: i32 = -2391003;
#[derive(Clone, Debug)]
pub struct Config {
    pub cows: BTreeMap<u32, u8>,
    pub bucket: i32,
    pub milk: i32,
    pub animation: u16,
    pub sound: u16,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Job {
    pub target: Location,
    pub revision: u64,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub slot: usize,
}
impl Config {
    pub fn fill(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        slot: usize,
    ) -> Result<(), Error> {
        if inventory
            .get(slot)
            .is_none_or(|i| i.id != self.bucket || i.count != 1 || i.vars != 0)
        {
            return Err(Error::ItemNotFound);
        }
        let mut next = inventory.clone();
        let mut debit = Delete::new(self.bucket, 1);
        debit.strict_slot = Some(slot);
        next.delete(catalog, debit)?;
        let mut credit = Insert::new(self.milk, 1);
        credit.strict_slot = Some(slot);
        next.insert(catalog, credit)?;
        *inventory = next;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn selected_bucket_replacement_is_atomic_even_when_full() {
        let config = Config {
            cows: BTreeMap::new(),
            bucket: 1,
            milk: 2,
            animation: 0,
            sound: 0,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for slot in 0..2 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut catalog = Catalog::default();
        catalog.dummy.insert(2);
        let before = inv.clone();
        assert!(config.fill(&mut inv, &catalog, 1).is_err());
        assert_eq!(inv, before);
        catalog.dummy.remove(&2);
        config.fill(&mut inv, &catalog, 1).unwrap();
        assert_eq!(inv.get(0).unwrap().id, 1);
        assert_eq!(inv.get(1).unwrap().id, 2);
        let after = inv.clone();
        assert!(config.fill(&mut inv, &catalog, 1).is_err());
        assert_eq!(inv, after);
    }
}
