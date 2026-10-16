//! Atomic pack opening. Rules are supplied by the host's attested definitions.
use crate::inventory::{Catalog, Delete, Error, Insert, Inventory};

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub output: i32,
    pub count: i32,
    pub noted: bool,
}

pub fn open(
    inv: &mut Inventory,
    catalog: &Catalog,
    slot: usize,
    pack: i32,
    rule: Rule,
) -> Result<(), Error> {
    if inv
        .get(slot)
        .is_none_or(|item| item.id != pack || item.count <= 0)
    {
        return Err(Error::ItemNotFound);
    }
    if rule.count <= 0 || rule.output == pack {
        return Err(Error::InvalidCount);
    }
    let mut next = inv.clone();
    let mut debit = Delete::new(pack, 1);
    debit.strict_slot = Some(slot);
    next.delete(catalog, debit)?;
    let mut credit = Insert::new(rule.output, rule.count);
    credit.note = rule.noted;
    next.insert(catalog, credit)?;
    *inv = next;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn selected_pack_and_overflow_are_atomic() {
        let mut catalog = Catalog::default();
        catalog.stackable.insert(314);
        let mut inv = Inventory::new(3, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 11881,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 314,
                count: i32::MAX - 50,
                vars: 0,
            }),
        )
        .unwrap();
        let before = inv.clone();
        let rule = Rule {
            output: 314,
            count: 100,
            noted: false,
        };
        assert!(open(&mut inv, &catalog, 2, 11881, rule).is_err());
        assert_eq!(inv, before);
        assert!(open(&mut inv, &catalog, 0, 11881, rule).is_err());
        assert_eq!(inv, before);
        inv.restore_slot(
            1,
            Some(Item {
                id: 314,
                count: 50,
                vars: 0,
            }),
        )
        .unwrap();
        open(&mut inv, &catalog, 0, 11881, rule).unwrap();
        assert!(inv.get(0).is_none());
        assert_eq!(inv.get(1).unwrap().count, 150);
    }
}
