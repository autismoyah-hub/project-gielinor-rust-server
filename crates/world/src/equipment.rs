use crate::{
    inventory::{self, Catalog, Insert, Inventory, Swap},
    stats::Stats,
};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct WearType {
    pub primary: Option<usize>,
    pub secondary: [Option<usize>; 2],
    pub requirements: Vec<(u8, i32)>,
}
impl WearType {
    fn two_handed(&self) -> bool {
        self.secondary.contains(&Some(5))
    }
}
#[derive(Default)]
pub struct WearCatalog {
    pub types: HashMap<i32, WearType>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidItem,
    Requirements(Vec<(u8, i32)>),
    NotEnoughWornSpace,
    NotEnoughInventorySpace,
    Transaction(inventory::Error),
}
impl From<inventory::Error> for Error {
    fn from(value: inventory::Error) -> Self {
        Self::Transaction(value)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Changed {
        slot: usize,
        id: i32,
    },
    Unequipped {
        slot: usize,
        id: i32,
    },
    Equipped {
        inventory_slot: usize,
        slot: usize,
        id: i32,
    },
}
pub fn equip(
    held: &mut Inventory,
    worn: &mut Inventory,
    slot: usize,
    items: &Catalog,
    wear: &WearCatalog,
    stats: &Stats,
) -> Result<Vec<Event>, Error> {
    let item = held.get(slot).ok_or(Error::InvalidItem)?;
    let ty = wear.types.get(&item.id).ok_or(Error::InvalidItem)?;
    let missing: Vec<_> = ty
        .requirements
        .iter()
        .copied()
        .filter(|(skill, level)| (stats.base(*skill) as u8 as i32) < *level)
        .collect();
    if !missing.is_empty() {
        return Err(Error::Requirements(missing));
    }
    let primary = ty.primary.filter(|s| *s < 14).ok_or(Error::InvalidItem)?;
    let mut blocked: Vec<_> = ty
        .secondary
        .iter()
        .flatten()
        .copied()
        .filter(|s| *s < 14 && ![6, 8, 11].contains(s))
        .collect();
    if primary == 5
        && worn
            .get(3)
            .and_then(|v| wear.types.get(&v.id))
            .is_some_and(WearType::two_handed)
        && !blocked.contains(&3)
    {
        blocked.push(3);
    }
    let all: Vec<_> = blocked.iter().copied().chain([primary]).collect();
    let old: Vec<_> = all.iter().map(|s| (*s, worn.get(*s))).collect();
    let primary_occupied = worn.get(primary).is_some();
    let mut held_image = held.clone();
    let mut worn_image = worn.clone();
    inventory::swap(
        &mut held_image,
        &mut worn_image,
        items,
        slot,
        primary,
        Swap {
            merge: true,
            transform: true,
            ..Default::default()
        },
    )
    .map_err(|e| {
        if e == inventory::Error::NotEnoughSpace {
            Error::NotEnoughWornSpace
        } else {
            e.into()
        }
    })?;
    for blocked_slot in blocked {
        let Some(old_item) = worn.get(blocked_slot) else {
            continue;
        };
        let mut insert = Insert::new(old_item.id, old_item.count);
        insert.untransform = true;
        insert.slot = if primary_occupied { 0 } else { slot };
        inventory::transfer(
            &mut worn_image,
            &mut held_image,
            items,
            blocked_slot,
            old_item.count,
            insert,
            false,
        )
        .map_err(|e| {
            if e == inventory::Error::NotEnoughSpace {
                Error::NotEnoughInventorySpace
            } else {
                e.into()
            }
        })?;
    }
    *held = held_image;
    *worn = worn_image;
    let mut events = Vec::new();
    for (wear_slot, old_item) in &old {
        if *wear_slot == primary {
            events.push(Event::Changed {
                slot: primary,
                id: item.id,
            });
        } else if let Some(item) = old_item {
            events.push(Event::Changed {
                slot: *wear_slot,
                id: item.id,
            });
        }
    }
    for (wear_slot, old_item) in old {
        if let Some(item) = old_item {
            events.push(Event::Unequipped {
                slot: wear_slot,
                id: item.id,
            });
        }
    }
    events.push(Event::Equipped {
        inventory_slot: slot,
        slot: primary,
        id: item.id,
    });
    Ok(events)
}
pub fn unequip(
    worn: &mut Inventory,
    held: &mut Inventory,
    slot: usize,
    items: &Catalog,
) -> Result<Vec<Event>, Error> {
    let item = worn.get(slot).ok_or(Error::InvalidItem)?;
    if slot >= 14 {
        return Err(Error::InvalidItem);
    }
    let mut insert = Insert::new(item.id, item.count);
    insert.untransform = true;
    inventory::transfer(worn, held, items, slot, item.count, insert, false).map_err(|e| {
        if e == inventory::Error::NotEnoughSpace {
            Error::NotEnoughInventorySpace
        } else {
            e.into()
        }
    })?;
    Ok(vec![
        Event::Changed { slot, id: item.id },
        Event::Unequipped { slot, id: item.id },
    ])
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn two_handed_equipping_is_atomic_and_checks_base_stats() {
        let items = Catalog::default();
        let mut wear = WearCatalog::default();
        wear.types.insert(
            1,
            WearType {
                primary: Some(3),
                secondary: [Some(5), None],
                requirements: vec![(0, 50)],
            },
        );
        wear.types.insert(
            2,
            WearType {
                primary: Some(5),
                secondary: [None, None],
                requirements: vec![],
            },
        );
        let mut held = Inventory::new(1, StackType::Normal, false);
        held.restore_slot(
            0,
            Some(Item {
                id: 1,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut worn = Inventory::new(15, StackType::Normal, false);
        worn.restore_slot(
            5,
            Some(Item {
                id: 2,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut stats = Stats::default();
        stats.set_current(0, 99);
        assert!(matches!(
            equip(&mut held, &mut worn, 0, &items, &wear, &stats),
            Err(Error::Requirements(_))
        ));
        stats.set_base(0, 50);
        equip(&mut held, &mut worn, 0, &items, &wear, &stats).unwrap();
        assert_eq!(held.get(0).unwrap().id, 2);
        assert_eq!(worn.get(3).unwrap().id, 1);
        assert!(worn.get(5).is_none());
    }
    #[test]
    fn full_inventory_keeps_both_weapon_and_shield_when_swap_cannot_fit() {
        let items = Catalog::default();
        let mut wear = WearCatalog::default();
        wear.types.insert(
            1,
            WearType {
                primary: Some(3),
                secondary: [Some(5), None],
                requirements: vec![],
            },
        );
        let mut held = Inventory::new(1, StackType::Normal, false);
        held.restore_slot(
            0,
            Some(Item {
                id: 1,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut worn = Inventory::new(15, StackType::Normal, false);
        for slot in [3, 5] {
            worn.restore_slot(
                slot,
                Some(Item {
                    id: slot as i32 + 10,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let before = (held.clone(), worn.clone());
        assert!(equip(&mut held, &mut worn, 0, &items, &wear, &Stats::default()).is_err());
        assert_eq!((held, worn), before);
    }
}
