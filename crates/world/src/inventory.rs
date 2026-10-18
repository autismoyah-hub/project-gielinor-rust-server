use std::collections::{BTreeSet, HashMap, HashSet};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    pub id: i32,
    pub count: i32,
    pub vars: i32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackType {
    Normal,
    Always,
    Never,
}
#[derive(Debug, Clone, Copy)]
pub struct Template {
    pub link: i32,
    pub template: i32,
}
#[derive(Default)]
pub struct Catalog {
    pub stackable: HashSet<i32>,
    pub notes: HashMap<i32, Template>,
    pub placeholders: HashMap<i32, Template>,
    pub transforms: HashMap<i32, Template>,
    pub dummy: HashSet<i32>,
}
impl Catalog {
    /// Ordinary type identity; item variables remain the caller's policy.
    pub fn ordinary_nonstackable(&self, id:i32)->bool {
        id>=0 && !self.stackable.contains(&id) && !self.dummy.contains(&id)
            && !self.notes.get(&id).is_some_and(|t|t.template!=0)
            && !self.placeholders.get(&id).is_some_and(|t|t.template!=0)
            && !self.transforms.get(&id).is_some_and(|t|t.template!=0)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidCount,
    InvalidSlot,
    InvalidCapacity,
    VarItemHasNote,
    NotEnoughSpace,
    ItemNotFound,
    NotEnoughItems,
    StrictSlotTaken,
    RestrictedDummy,
    InvalidVarStack,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Success {
    pub requested: i32,
    pub completed: i32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    pub stack: StackType,
    pub placeholders: bool,
    slots: Vec<Option<Item>>,
    modified: BTreeSet<usize>,
}
#[derive(Debug, Clone)]
pub struct Insert {
    pub id: i32,
    pub count: i32,
    pub strict: bool,
    pub slot: usize,
    pub strict_slot: Option<usize>,
    pub capacity: Option<usize>,
    pub vars: i32,
    pub note: bool,
    pub unnote: bool,
    pub transform: bool,
    pub untransform: bool,
}
impl Insert {
    pub fn new(id: i32, count: i32) -> Self {
        Self {
            id,
            count,
            strict: true,
            slot: 0,
            strict_slot: None,
            capacity: None,
            vars: 0,
            note: false,
            unnote: false,
            transform: false,
            untransform: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Delete {
    pub id: i32,
    pub count: i32,
    pub strict: bool,
    pub slot: usize,
    pub strict_slot: Option<usize>,
    pub placehold: bool,
}
impl Delete {
    pub fn new(id: i32, count: i32) -> Self {
        Self {
            id,
            count,
            strict: true,
            slot: 0,
            strict_slot: None,
            placehold: false,
        }
    }
}
impl Inventory {
    pub fn new(size: usize, stack: StackType, placeholders: bool) -> Self {
        Self {
            stack,
            placeholders,
            slots: vec![None; size],
            modified: BTreeSet::new(),
        }
    }
    pub fn slots(&self) -> &[Option<Item>] {
        &self.slots
    }
    pub fn get(&self, slot: usize) -> Option<Item> {
        self.slots.get(slot).copied().flatten()
    }
    pub fn modified(&self) -> &BTreeSet<usize> {
        &self.modified
    }
    pub fn clear_modified(&mut self) {
        self.modified.clear();
    }
    pub fn free_space(&self) -> usize {
        self.slots.iter().filter(|v| v.is_none()).count()
    }
    pub fn last_occupied_slot(&self) -> usize {
        self.slots
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |i| i + 1)
    }
    pub fn restore_slot(&mut self, slot: usize, item: Option<Item>) -> Result<(), Error> {
        if slot >= self.slots.len() {
            return Err(Error::InvalidSlot);
        }
        self.set(slot, item);
        Ok(())
    }
    fn set(&mut self, slot: usize, item: Option<Item>) {
        self.slots[slot] = item;
        self.modified.insert(slot);
    }
    fn indexes(&self, id: i32, start: usize) -> Vec<usize> {
        if self.slots.is_empty() {
            return Vec::new();
        }
        (0..self.slots.len())
            .map(|i| (start + i) % self.slots.len())
            .filter(|i| self.slots[*i].is_some_and(|v| v.id == id))
            .collect()
    }
    fn empty(&self, start: usize, capacity: usize) -> Option<usize> {
        (0..self.slots.len())
            .map(|i| (start + i) % capacity)
            .find(|i| self.slots[*i].is_none())
    }
    pub fn insert(&mut self, catalog: &Catalog, query: Insert) -> Result<Success, Error> {
        let mut image = self.clone();
        let result = image.insert_image(catalog, &query)?;
        *self = image;
        Ok(result)
    }
    fn insert_image(&mut self, c: &Catalog, q: &Insert) -> Result<Success, Error> {
        if q.count < 1 {
            return Err(Error::InvalidCount);
        }
        let capacity = q.capacity.unwrap_or(self.slots.len());
        if capacity == 0 || capacity > self.slots.len() {
            return Err(Error::InvalidCapacity);
        }
        if q.strict_slot.is_some_and(|s| s >= self.slots.len()) {
            return Err(Error::InvalidSlot);
        }
        let mut id = q.id;
        if let Some(t) = c.transforms.get(&id) {
            if (q.transform && t.template == 0) || (q.untransform && t.template != 0) {
                id = t.link;
            }
        }
        let base_id = id;
        let note = c.notes.get(&id);
        if note.is_some() && q.vars > 0 {
            return Err(Error::VarItemHasNote);
        }
        let mut stack = false;
        if q.vars <= 0 {
            if let Some(t) = note.filter(|t| t.template != 0) {
                if q.unnote || self.stack == StackType::Always {
                    id = t.link;
                }
                stack = self.stack != StackType::Never
                    && (id == base_id || self.stack == StackType::Always);
            } else if self.stack == StackType::Always {
                stack = true;
            } else if self.stack != StackType::Never {
                if let Some(t) = note.filter(|t| q.note && t.template == 0) {
                    id = t.link;
                    stack = true;
                } else {
                    stack = c.stackable.contains(&id);
                }
            }
        }
        if stack {
            if c.dummy.contains(&id) {
                return Err(Error::RestrictedDummy);
            }
            if self.placeholders {
                if let Some(t) = c.placeholders.get(&id).filter(|t| t.template == 0) {
                    if let Some(slot) = self.indexes(t.link, 0).first().copied() {
                        self.set(
                            slot,
                            Some(Item {
                                id,
                                count: q.count,
                                vars: 0,
                            }),
                        );
                        return Ok(Success {
                            requested: q.count,
                            completed: q.count,
                        });
                    }
                }
            }
            if let Some(slot) = self.indexes(id, 0).first().copied() {
                let previous = self.slots[slot].unwrap().count;
                let completed = q.count.min(i32::MAX - previous);
                if q.strict && completed != q.count {
                    return Err(Error::NotEnoughSpace);
                }
                self.set(
                    slot,
                    Some(Item {
                        id,
                        count: previous + completed,
                        vars: 0,
                    }),
                );
                return Ok(Success {
                    requested: q.count,
                    completed,
                });
            }
            if q.strict_slot.is_some_and(|s| self.slots[s].is_some()) {
                return Err(Error::StrictSlotTaken);
            }
            let slot = self
                .empty(q.strict_slot.unwrap_or(q.slot), capacity)
                .ok_or(Error::NotEnoughSpace)?;
            if capacity != self.slots.len() && self.slots.len() - self.free_space() >= capacity {
                return Err(Error::NotEnoughSpace);
            }
            self.set(
                slot,
                Some(Item {
                    id,
                    count: q.count,
                    vars: 0,
                }),
            );
            return Ok(Success {
                requested: q.count,
                completed: q.count,
            });
        }
        if c.dummy.contains(&id) {
            return Err(Error::RestrictedDummy);
        }
        if self.placeholders {
            if let Some(t) = c.placeholders.get(&id).filter(|t| t.template == 0) {
                if let Some(slot) = self.indexes(t.link, 0).first().copied() {
                    self.set(
                        slot,
                        Some(Item {
                            id,
                            count: q.count,
                            vars: q.vars,
                        }),
                    );
                    return Ok(Success {
                        requested: q.count,
                        completed: q.count,
                    });
                }
            }
        }
        if q.strict_slot.is_some_and(|s| self.slots[s].is_some()) {
            return Err(Error::StrictSlotTaken);
        }
        let free = self.free_space() as i64 - (self.slots.len() - capacity) as i64;
        if q.strict && q.count as i64 > free {
            return Err(Error::NotEnoughSpace);
        }
        let completed = (q.count as i64).min(free).max(0) as i32;
        for i in 0..completed {
            let slot = self
                .empty(q.strict_slot.unwrap_or(q.slot) + i as usize, capacity)
                .ok_or(Error::NotEnoughSpace)?;
            self.set(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: q.vars,
                }),
            );
        }
        Ok(Success {
            requested: q.count,
            completed,
        })
    }
    pub fn delete(&mut self, catalog: &Catalog, query: Delete) -> Result<Success, Error> {
        let mut image = self.clone();
        let result = image.delete_image(catalog, &query)?;
        *self = image;
        Ok(result)
    }
    fn decrement(&self, c: &Catalog, id: i32, count: i32, placehold: bool) -> Option<Item> {
        if count > 0 {
            Some(Item { id, count, vars: 0 })
        } else if placehold {
            c.placeholders
                .get(&id)
                .filter(|t| t.template == 0)
                .map(|t| Item {
                    id: t.link,
                    count: 1,
                    vars: 0,
                })
        } else {
            None
        }
    }
    fn delete_image(&mut self, c: &Catalog, q: &Delete) -> Result<Success, Error> {
        if q.count < 1 {
            return Err(Error::InvalidCount);
        }
        let slots = self.indexes(q.id, q.strict_slot.unwrap_or(q.slot));
        if slots.is_empty() && q.strict {
            return Err(Error::ItemNotFound);
        }
        if let Some(slot) = q.strict_slot {
            if slots.first() != Some(&slot) {
                return Err(Error::ItemNotFound);
            }
        }
        let stack = self.stack == StackType::Always
            || c.stackable.contains(&q.id)
            || c.notes.get(&q.id).is_some_and(|t| t.template != 0);
        if stack && !slots.is_empty() && self.slots[slots[0]].unwrap().vars <= 0 {
            let slot = slots[0];
            let previous = self.slots[slot].unwrap().count;
            let count = previous.saturating_sub(q.count).max(0);
            let completed = previous - count;
            if q.strict && completed != q.count {
                return Err(Error::NotEnoughItems);
            }
            self.set(slot, self.decrement(c, q.id, count, q.placehold));
            return Ok(Success {
                requested: q.count,
                completed,
            });
        }
        if q.strict && q.count as usize > slots.len() {
            return Err(Error::NotEnoughItems);
        }
        let completed = (q.count as usize).min(slots.len());
        for slot in slots.into_iter().take(completed) {
            let count = self.slots[slot].unwrap().count - 1;
            self.set(slot, self.decrement(c, q.id, count, q.placehold));
        }
        Ok(Success {
            requested: q.count,
            completed: completed as i32,
        })
    }
    pub fn swap_slots(&mut self, a: usize, b: usize) -> Result<(), Error> {
        if a >= self.slots.len() || b >= self.slots.len() {
            return Err(Error::InvalidSlot);
        }
        self.slots.swap(a, b);
        self.modified.extend([a, b]);
        Ok(())
    }
    /// Kotlin invMoveToSlot uses a transaction even within one inventory. This
    /// differs from swapping two slots when stacks merge, counts are capped, or
    /// both indices refer to the same slot.
    pub fn move_slot(&mut self, c: &Catalog, from: usize, into: usize) -> Result<Success, Error> {
        self.swap_slot(c, from, into, true)
    }
    /// Raw SwapQuery defaults to merge=false (bank dragging), unlike invMoveToSlot.
    pub fn swap_slot(
        &mut self,
        c: &Catalog,
        from: usize,
        into: usize,
        merge: bool,
    ) -> Result<Success, Error> {
        if from >= self.slots.len() || into >= self.slots.len() {
            return Err(Error::InvalidSlot);
        }
        let item = self.get(from).ok_or(Error::ItemNotFound)?;
        if item.count < 1 {
            return Err(Error::NotEnoughItems);
        }
        let can_stack = self.stack != StackType::Never
            && (self.stack == StackType::Always
                || c.stackable.contains(&item.id)
                || c.notes.get(&item.id).is_some_and(|t| t.template != 0));
        let count = if can_stack { item.count } else { 1 };
        let other = self.get(into);
        let mut image = self.clone();
        if let Some(other) = other.filter(|other| {
            merge && other.id == item.id && can_stack && item.vars <= 0 && other.vars <= 0
        }) {
            let add = count.min(i32::MAX - other.count);
            if add == 0 {
                return Err(Error::NotEnoughSpace);
            }
            image.set(
                into,
                Some(Item {
                    count: other.count + add,
                    ..other
                }),
            );
            image.set(
                from,
                if count > add {
                    Some(Item {
                        count: count - add,
                        ..item
                    })
                } else {
                    None
                },
            );
            *self = image;
            return Ok(Success {
                requested: count,
                completed: add,
            });
        }
        if let Some(other) = other {
            let mut delete = Delete::new(other.id, other.count);
            delete.strict_slot = Some(into);
            image.delete_image(c, &delete)?;
        }
        let mut delete = Delete::new(item.id, count);
        delete.strict_slot = Some(from);
        image.delete_image(c, &delete)?;
        if let Some(other) = other {
            let mut insert = Insert::new(other.id, other.count);
            insert.vars = other.vars;
            insert.strict_slot = Some(from);
            image.insert_image(c, &insert)?;
        }
        let mut insert = Insert::new(item.id, count);
        insert.vars = item.vars;
        insert.strict_slot = Some(into);
        image.insert_image(c, &insert)?;
        *self = image;
        Ok(Success {
            requested: count,
            completed: count,
        })
    }
    pub fn compact(&mut self) {
        let packed: Vec<_> = self.slots.iter().filter_map(|i| *i).collect();
        for slot in 0..self.slots.len() {
            let value = packed.get(slot).copied();
            if self.slots[slot] != value {
                self.set(slot, value);
            }
        }
    }
}

/// Transfers are atomic across both inventories, including note conversion and placeholders.
/// DumpQuery may skip individual insertion failures, but an invalid variable stack
/// or failed deletion rolls back the entire batch. Counts use JVM Int overflow.
pub fn dump(
    from: &mut Inventory,
    into: &mut Inventory,
    c: &Catalog,
    mut insert: Insert,
    keep_slots: &HashSet<usize>,
    placehold: bool,
) -> Result<Success, Error> {
    let mut source = from.clone();
    let mut target = into.clone();
    let mut requested = 0i32;
    let mut completed = 0i32;
    let mut error = None;
    for slot in 0..source.slots.len() {
        let Some(item) = source.get(slot) else {
            continue;
        };
        if keep_slots.contains(&slot) {
            continue;
        }
        if item.count > 1 && item.vars > 0 {
            return Err(Error::InvalidVarStack);
        }
        insert.id = item.id;
        insert.count = item.count;
        insert.vars = item.vars;
        insert.strict = false;
        insert.strict_slot = None;
        let added = match target.insert_image(c, &insert) {
            Ok(result) => result,
            Err(failure) => {
                if error.is_none() {
                    error = Some(failure);
                }
                continue;
            }
        };
        if added.completed == 0 {
            error = Some(Error::NotEnoughSpace);
            continue;
        }
        let mut delete = Delete::new(item.id, added.completed);
        delete.strict_slot = Some(slot);
        delete.placehold = placehold;
        source.delete_image(c, &delete)?;
        requested = requested.wrapping_add(added.requested);
        completed = completed.wrapping_add(added.completed);
    }
    if completed == 0 {
        if let Some(error) = error {
            return Err(error);
        }
    }
    *from = source;
    *into = target;
    Ok(Success {
        requested,
        completed,
    })
}

/// Transfers are atomic across both inventories, including note conversion and placeholders.
pub fn transfer(
    from: &mut Inventory,
    into: &mut Inventory,
    c: &Catalog,
    from_slot: usize,
    count: i32,
    mut insert: Insert,
    placehold: bool,
) -> Result<Success, Error> {
    if count < 1 {
        return Err(Error::InvalidCount);
    }
    let item = from.get(from_slot).ok_or(Error::ItemNotFound)?;
    if item.count < 1 {
        return Err(Error::NotEnoughItems);
    }
    let mut source = from.clone();
    let mut target = into.clone();
    let stack_count = source.stack != StackType::Never
        && (source.stack == StackType::Always
            || c.stackable.contains(&item.id)
            || c.notes.get(&item.id).is_some_and(|t| t.template != 0));
    let total: i64 = if count == 1 {
        1
    } else if item.vars > 0 || !stack_count {
        source.indexes(item.id, 0).len() as i64
    } else {
        source.get(source.indexes(item.id, 0)[0]).unwrap().count as i64
    };
    let requested = (count as i64).min(total) as i32;
    let completed = if item.vars > 0 && requested > 1 {
        let slots = source.indexes(item.id, from_slot);
        let mut completed = 0;
        for slot in slots.into_iter().take(requested as usize) {
            let item = source.get(slot).ok_or(Error::InvalidVarStack)?;
            completed += item.count;
            if completed > requested {
                return Err(Error::InvalidVarStack);
            }
            let mut delete = Delete::new(item.id, item.count);
            delete.slot = slot;
            delete.placehold = placehold;
            source.delete_image(c, &delete)?;
            insert.id = item.id;
            insert.vars = item.vars;
            insert.count = item.count;
            insert.strict = true;
            target.insert_image(c, &insert)?;
        }
        count
    } else {
        insert.id = item.id;
        insert.vars = item.vars;
        insert.count = requested;
        let result = target.insert_image(c, &insert)?;
        if result.completed == 0 {
            return Err(Error::NotEnoughSpace);
        }
        let mut delete = Delete::new(item.id, result.completed);
        delete.slot = from_slot;
        delete.placehold = placehold;
        source.delete_image(c, &delete)?.completed
    };
    let requested = if item.vars > 0 && requested > 1 {
        count
    } else {
        requested
    };
    *from = source;
    *into = target;
    Ok(Success {
        requested,
        completed,
    })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Swap {
    pub merge: bool,
    pub relaxed_source_slot: bool,
    pub note: bool,
    pub unnote: bool,
    pub transform: bool,
    pub untransform: bool,
}
/// Atomic worn/held exchange. Equipment eligibility and blocked wear slots are checked above this layer.
pub fn swap(
    from: &mut Inventory,
    into: &mut Inventory,
    c: &Catalog,
    from_slot: usize,
    into_slot: usize,
    q: Swap,
) -> Result<Success, Error> {
    if from_slot >= from.slots.len() || into_slot >= into.slots.len() {
        return Err(Error::InvalidSlot);
    }
    let item = from.get(from_slot).ok_or(Error::ItemNotFound)?;
    if item.count < 1 {
        return Err(Error::NotEnoughItems);
    }
    let can_stack = into.stack != StackType::Never
        && (into.stack == StackType::Always
            || c.stackable.contains(&item.id)
            || c.notes.get(&item.id).is_some_and(|t| t.template != 0));
    let count = if can_stack { item.count } else { 1 };
    let other = into.get(into_slot);
    let mut source = from.clone();
    let mut target = into.clone();
    if let Some(other) =
        other.filter(|v| q.merge && v.id == item.id && v.vars <= 0 && item.vars <= 0 && can_stack)
    {
        let add = count.min(i32::MAX - other.count);
        if add == 0 {
            return Err(Error::NotEnoughSpace);
        }
        target.set(
            into_slot,
            Some(Item {
                count: other.count + add,
                ..other
            }),
        );
        source.set(
            from_slot,
            if count > add {
                Some(Item {
                    count: count - add,
                    ..item
                })
            } else {
                None
            },
        );
        *from = source;
        *into = target;
        return Ok(Success {
            requested: count,
            completed: add,
        });
    }
    if let Some(other) = other {
        let mut delete = Delete::new(other.id, other.count);
        delete.strict_slot = Some(into_slot);
        target.delete_image(c, &delete)?;
    }
    let mut delete = Delete::new(item.id, count);
    delete.strict_slot = Some(from_slot);
    source.delete_image(c, &delete)?;
    if let Some(other) = other {
        let mut insert = Insert::new(other.id, other.count);
        insert.vars = other.vars;
        insert.slot = from_slot;
        insert.strict_slot = if q.relaxed_source_slot {
            None
        } else {
            Some(from_slot)
        };
        insert.note = q.note;
        insert.unnote = q.unnote;
        insert.transform = q.untransform;
        insert.untransform = q.transform;
        source.insert_image(c, &insert)?;
    }
    let mut insert = Insert::new(item.id, count);
    insert.vars = item.vars;
    insert.strict_slot = Some(into_slot);
    insert.note = q.note;
    insert.unnote = q.unnote;
    insert.transform = q.transform;
    insert.untransform = q.untransform;
    target.insert_image(c, &insert)?;
    *from = source;
    *into = target;
    Ok(Success {
        requested: count,
        completed: count,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn catalog() -> Catalog {
        let mut c = Catalog::default();
        c.stackable.insert(995);
        c.notes.insert(
            1038,
            Template {
                link: 1039,
                template: 0,
            },
        );
        c.notes.insert(
            1039,
            Template {
                link: 1038,
                template: 799,
            },
        );
        c.placeholders.insert(
            1038,
            Template {
                link: 14387,
                template: 0,
            },
        );
        c
    }
    #[test]
    fn overflow_and_failed_transfer_do_not_lose_items() {
        let c = catalog();
        let mut a = Inventory::new(2, StackType::Normal, false);
        a.insert(&c, Insert::new(995, i32::MAX)).unwrap();
        let before = a.clone();
        assert_eq!(
            a.insert(&c, Insert::new(995, 1)),
            Err(Error::NotEnoughSpace)
        );
        assert_eq!(a, before);
        let mut b = Inventory::new(1, StackType::Normal, false);
        b.insert(&c, Insert::new(1, 1)).unwrap();
        let before_b = b.clone();
        assert!(transfer(&mut a, &mut b, &c, 0, 10, Insert::new(995, 10), false).is_err());
        assert_eq!(a, before);
        assert_eq!(b, before_b);
    }
    #[test]
    fn bank_converts_notes_and_replaces_placeholders() {
        let c = catalog();
        let mut b = Inventory::new(5, StackType::Always, true);
        b.insert(&c, Insert::new(1039, 3)).unwrap();
        assert_eq!(b.get(0).unwrap().id, 1038);
        let mut q = Delete::new(1038, 3);
        q.placehold = true;
        b.delete(&c, q).unwrap();
        assert_eq!(b.get(0).unwrap().id, 14387);
        b.insert(&c, Insert::new(1038, 2)).unwrap();
        assert_eq!(
            b.get(0),
            Some(Item {
                id: 1038,
                count: 2,
                vars: 0
            })
        );
    }
}
