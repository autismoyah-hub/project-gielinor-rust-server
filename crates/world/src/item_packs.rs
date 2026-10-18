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
    open_count(inv,catalog,slot,pack,rule,1)
}

/// Open a positive quantity from the captured stack as one transaction.
pub fn open_count(inv:&mut Inventory,catalog:&Catalog,slot:usize,pack:i32,rule:Rule,count:i32)->Result<(),Error>{
    if count<=0 || (count>1 && !catalog.stackable.contains(&pack)){return Err(Error::InvalidCount);}
    if inv
        .get(slot)
        .is_none_or(|item| item.id != pack || item.count < count)
    {
        return Err(Error::ItemNotFound);
    }
    if rule.count <= 0 || rule.output == pack {
        return Err(Error::InvalidCount);
    }
    let total=rule.count.checked_mul(count).ok_or(Error::InvalidCount)?;
    let mut next = inv.clone();
    let mut debit = Delete::new(pack, count);
    debit.strict_slot = Some(slot);
    next.delete(catalog, debit)?;
    let mut credit = Insert::new(rule.output, total);
    credit.note = rule.noted;
    next.insert(catalog, credit)?;
    *inv = next;
    Ok(())
}

/// Immutable host-attested coin-pouch policy; currency uses the same pack rule.
#[derive(Clone,Copy,Debug)]
pub struct PouchRule {pub rule:Rule,pub members:bool}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn noted_outputs_use_freed_pack_slot_and_overflow_restores_pack() {
        use crate::inventory::Template;
        let mut catalog = Catalog::default();
        catalog.notes.insert(
            1925,
            Template {
                link: 1926,
                template: 0,
            },
        );
        catalog.notes.insert(
            1926,
            Template {
                link: 1925,
                template: 799,
            },
        );
        let rule = Rule {
            output: 1925,
            count: 100,
            noted: true,
        };
        let mut inv = Inventory::new(2, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 22660,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 99,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        open(&mut inv, &catalog, 0, 22660, rule).unwrap();
        assert_eq!(
            inv.get(0),
            Some(Item {
                id: 1926,
                count: 100,
                vars: 0
            })
        );
        assert_eq!(inv.get(1).unwrap().id, 99);
        inv.restore_slot(
            0,
            Some(Item {
                id: 22660,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 1926,
                count: i32::MAX - 99,
                vars: 0,
            }),
        )
        .unwrap();
        let before = inv.clone();
        assert!(open(&mut inv, &catalog, 0, 22660, rule).is_err());
        assert_eq!(inv, before);
    }
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

#[cfg(test)]
mod count_tests {
    use super::*;
    use crate::inventory::{Item,StackType};
    #[test]
    fn selected_batch_pack_exchange_checks_products_capacity_and_currency_overflow_atomically(){
        let mut catalog=Catalog::default();catalog.stackable.extend([22521,995]);let rule=Rule{output:995,count:3,noted:false};
        for count in [1,3]{let mut inv=Inventory::new(2,StackType::Normal,false);inv.restore_slot(0,Some(Item{id:22521,count:3,vars:0})).unwrap();inv.restore_slot(1,Some(Item{id:995,count:10,vars:0})).unwrap();open_count(&mut inv,&catalog,0,22521,rule,count).unwrap();assert_eq!(inv.get(0).map(|i|i.count),if count==3{None}else{Some(2)});assert_eq!(inv.get(1).unwrap().count,10+3*count);}
        for mode in 0..5{let mut inv=Inventory::new(2,StackType::Normal,false);inv.restore_slot(0,Some(Item{id:22521,count:if mode==4{i32::MAX}else{3},vars:0})).unwrap();inv.restore_slot(1,Some(Item{id:if mode==2{1511}else{995},count:if mode==3{i32::MAX-8}else{1},vars:0})).unwrap();let before=inv.clone();let count=match mode{0=>0,1=>4,2=>1,3=>3,_=>i32::MAX};assert!(open_count(&mut inv,&catalog,0,22521,rule,count).is_err());assert_eq!(inv,before);}
    }
}
