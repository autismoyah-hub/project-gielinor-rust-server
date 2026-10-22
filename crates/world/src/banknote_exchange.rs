//! Selected banknote exchange, using the existing atomic inventory operations.
use crate::inventory::{Catalog, Delete, Insert, Inventory, StackType};

pub const QUESTION: &str = "Un-note the banknote?";
pub const OPTIONS: &[&str] = &["Yes", "No"];
pub const MESSAGE: &str = "The bank exchanges your banknote for an item.";
pub const FULL: &str = "You don't have any inventory space.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidSelection,
    NotNote,
    NoSpace,
    InvalidInventory,
    Transaction(crate::inventory::Error),
}

pub struct Prepared {
    pub inventory: Inventory,
    pub output: i32,
    pub count: i32,
}

/// Re-read the current selected stack after confirmation. The donor slot
/// replacement adds into the first empty slot; it is not replace-in-place.
pub fn prepare(
    inv: &Inventory,
    catalog: &Catalog,
    slot: usize,
    note: i32,
) -> Result<Prepared, Error> {
    if inv.stack != StackType::Normal || inv.placeholders {
        return Err(Error::InvalidInventory);
    }
    let selected = inv
        .get(slot)
        .filter(|v| v.id == note && v.count > 0)
        .ok_or(Error::InvalidSelection)?;
    let definition = catalog
        .notes
        .get(&note)
        .filter(|v| v.template > 0 && v.link != note)
        .ok_or(Error::NotNote)?;
    if selected.vars != 0 {
        return Err(Error::InvalidSelection);
    }
    let output = definition.link;
    if !(0..=u16::MAX as i32).contains(&note)
        || !(0..=u16::MAX as i32).contains(&output)
        || !catalog.ordinary_nonstackable(output)
        || catalog.dummy.contains(&note)
        || catalog
            .placeholders
            .get(&note)
            .is_some_and(|v| v.template != 0)
        || catalog
            .transforms
            .get(&note)
            .is_some_and(|v| v.template != 0)
        || !catalog
            .notes
            .get(&output)
            .is_some_and(|v| v.template == 0 && v.link == note)
    {
        return Err(Error::NotNote);
    }
    // Preserve the reference's pre-delete free-space cap, including full-pack
    // refusal even when deleting the last note would create a slot.
    let count = selected.count.min(inv.free_space() as i32);
    if count == 0 {
        return Err(Error::NoSpace);
    }
    let mut next = inv.clone();
    let mut debit = Delete::new(note, count);
    debit.strict_slot = Some(slot);
    next.delete(catalog, debit).map_err(Error::Transaction)?;
    next.insert(catalog, Insert::new(output, count))
        .map_err(Error::Transaction)?;
    Ok(Prepared {
        inventory: next,
        output,
        count,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::inventory::{Item, Template};
    pub(crate) fn catalog() -> Catalog {
        let mut c = Catalog::default();
        c.notes.insert(
            1512,
            Template {
                link: 1511,
                template: 799,
            },
        );
        c.notes.insert(
            1511,
            Template {
                link: 1512,
                template: 0,
            },
        );
        c
    }
    fn total(inv: &Inventory, id: i32) -> i64 {
        inv.slots()
            .iter()
            .flatten()
            .filter(|v| v.id == id)
            .map(|v| v.count as i64)
            .sum()
    }
    #[test]
    fn banknote_cap_selected_duplicate_and_quantities_preserve_all_semantic_items() {
        let c = catalog();
        let mut cases = 0;
        for slot in [0, 13, 27] {
            for quantity in [1, 2, 27, 28, i32::MAX] {
                for free in 0..28 {
                    let mut inv = Inventory::new(28, StackType::Normal, false);
                    for i in 0..28 {
                        inv.restore_slot(
                            i,
                            Some(Item {
                                id: 1513,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    for i in (0..28).filter(|&i| i != slot).take(free) {
                        inv.restore_slot(i, None).unwrap();
                    }
                    inv.restore_slot(
                        slot,
                        Some(Item {
                            id: 1512,
                            count: quantity,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                    let duplicate = (0..28).find(|&i| i != slot && inv.get(i).is_some());
                    if let Some(i) = duplicate {
                        inv.restore_slot(
                            i,
                            Some(Item {
                                id: 1512,
                                count: 9,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                    let before = inv.clone();
                    let prepared = prepare(&inv, &c, slot, 1512);
                    assert_eq!(inv, before);
                    if free == 0 {
                        assert!(matches!(prepared, Err(Error::NoSpace)));
                    } else {
                        let p = prepared.unwrap();
                        let converted = quantity.min(free as i32);
                        assert_eq!((p.output, p.count), (1511, converted));
                        assert_eq!(total(&p.inventory, 1511), converted as i64);
                        assert_eq!(
                            total(&p.inventory, 1512),
                            total(&inv, 1512) - converted as i64
                        );
                        assert_eq!(total(&p.inventory, 1513), total(&inv, 1513));
                        if let Some(i) = duplicate {
                            assert_eq!(p.inventory.get(i), inv.get(i));
                        }
                    }
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 420);
    }
    #[test]
    fn banknote_output_uses_first_empty_and_live_selected_count_with_strict_source_slot() {
        let c = catalog();
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for i in 0..28 {
            inv.restore_slot(
                i,
                Some(Item {
                    id: 1513,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        inv.restore_slot(0, None).unwrap();
        inv.restore_slot(27, None).unwrap();
        inv.restore_slot(
            13,
            Some(Item {
                id: 1512,
                count: 5,
                vars: 0,
            }),
        )
        .unwrap();
        let p = prepare(&inv, &c, 13, 1512).unwrap();
        assert_eq!(p.count, 2);
        assert_eq!(p.inventory.get(13).unwrap().count, 3);
        assert_eq!(p.inventory.get(0).unwrap().id, 1511);
        assert_eq!(p.inventory.get(27).unwrap().id, 1511);
        inv.restore_slot(
            13,
            Some(Item {
                id: 1512,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let p = prepare(&inv, &c, 13, 1512).unwrap();
        assert_eq!(p.count, 1);
        assert_eq!(p.inventory.get(0).unwrap().id, 1511);
        assert!(p.inventory.get(13).is_none());
    }
    #[test]
    fn banknote_invalid_note_graph_variables_selection_and_inventory_never_publish() {
        for mode in 0..11 {
            let mut c = catalog();
            let mut inv = Inventory::new(28, StackType::Normal, false);
            inv.restore_slot(
                13,
                Some(Item {
                    id: 1512,
                    count: 5,
                    vars: 0,
                }),
            )
            .unwrap();
            match mode {
                0 => {
                    c.notes.remove(&1512);
                }
                1 => {
                    c.notes.get_mut(&1511).unwrap().link = 100;
                }
                2 => {
                    c.stackable.insert(1511);
                }
                3 => {
                    c.dummy.insert(1512);
                }
                4 => {
                    c.placeholders.insert(
                        1512,
                        Template {
                            link: 1511,
                            template: 799,
                        },
                    );
                }
                5 => {
                    c.transforms.insert(
                        1511,
                        Template {
                            link: 1512,
                            template: 799,
                        },
                    );
                }
                6 => {
                    inv.restore_slot(
                        13,
                        Some(Item {
                            id: 1512,
                            count: 5,
                            vars: 1,
                        }),
                    )
                    .unwrap();
                }
                7 => {
                    inv.restore_slot(
                        13,
                        Some(Item {
                            id: 1513,
                            count: 5,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                8 => inv.stack = StackType::Always,
                9 => inv.placeholders = true,
                _ => {
                    c.notes.get_mut(&1511).unwrap().template = 799;
                }
            }
            let before = inv.clone();
            assert!(prepare(&inv, &c, 13, 1512).is_err(), "mode{mode}");
            assert_eq!(inv, before);
        }
    }
}
