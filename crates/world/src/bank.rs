//! Bank slot operations, preserving Transaction.kt and BankSlots.kt ordering.
use crate::inventory::{transfer, Catalog, Error, Insert, Inventory, Success};
use std::ops::Range;

impl Inventory {
    pub fn shift_insert(&mut self, from: usize, into: usize) -> Result<Success, Error> {
        if from >= self.slots().len() || into >= self.slots().len() {
            return Err(Error::InvalidSlot);
        }
        let item = self.get(from).ok_or(Error::ItemNotFound)?;
        if from == into {
            return Ok(Success {
                requested: item.count,
                completed: 0,
            });
        }
        if from < into {
            for slot in from + 1..=into {
                self.restore_slot(slot - 1, self.get(slot))?;
            }
        } else {
            for slot in (into..from).rev() {
                self.restore_slot(slot + 1, self.get(slot))?;
            }
        }
        self.restore_slot(into, Some(item))?;
        Ok(Success {
            requested: item.count,
            completed: item.count,
        })
    }
    pub fn shift_left(
        &mut self,
        start: usize,
        into: usize,
        end: usize,
        strict: bool,
    ) -> Result<Success, Error> {
        let len = self.slots().len();
        if start >= len || into >= len || into > start || end > len || end < start {
            return Err(Error::InvalidSlot);
        }
        if strict && self.slots()[into..start].iter().any(Option::is_some) {
            return Err(Error::StrictSlotTaken);
        }
        let copy = self.slots()[start..end].to_vec();
        for (i, item) in copy.iter().enumerate() {
            self.restore_slot(into + i, *item)?;
        }
        for slot in into + copy.len()..end {
            if self.get(slot).is_some() {
                self.restore_slot(slot, None)?;
            }
        }
        let requested = (end - into) as i32;
        Ok(Success {
            requested,
            completed: requested,
        })
    }
    pub fn shift_right(&mut self, start: usize, count: usize) -> Result<Success, Error> {
        let len = self.slots().len();
        if start >= len {
            return Err(Error::InvalidSlot);
        }
        if count == 0 || count > len - start {
            return Err(Error::InvalidCount);
        }
        if self.slots()[len - count..].iter().any(Option::is_some) {
            return Err(Error::NotEnoughSpace);
        }
        for slot in (start + count..len).rev() {
            let item = self.get(slot - count);
            if item.is_some() || self.get(slot).is_some() {
                self.restore_slot(slot, item)?;
            }
        }
        for slot in start..start + count {
            self.restore_slot(slot, None)?;
        }
        let requested = (len - start) as i32;
        Ok(Success {
            requested,
            completed: requested,
        })
    }
    pub fn shift_bulk(&mut self, range: Range<usize>, into: usize) -> Result<Success, Error> {
        let len = self.slots().len();
        if range.is_empty() || range.end > len || into >= len || range.contains(&into) {
            return Err(Error::InvalidSlot);
        }
        let copy = self.slots()[range.clone()].to_vec();
        let count = copy.len();
        let target = if into >= range.end {
            for slot in range.end..=into {
                self.restore_slot(slot - count, self.get(slot))?;
            }
            into + 1 - count
        } else {
            for slot in (into..range.start).rev() {
                self.restore_slot(slot + count, self.get(slot))?;
            }
            into
        };
        for (i, item) in copy.into_iter().enumerate() {
            self.restore_slot(target + i, item)?;
        }
        Ok(Success {
            requested: count as i32,
            completed: count as i32,
        })
    }
    pub fn compact_slots(&mut self, range: Range<usize>) -> Result<Success, Error> {
        if range.start >= range.end || range.len() < 2 || range.end > self.slots().len() {
            return Err(Error::InvalidSlot);
        }
        let mut write = range.start;
        for read in range.clone() {
            if let Some(item) = self.get(read) {
                if read != write {
                    self.restore_slot(write, Some(item))?;
                    self.restore_slot(read, None)?;
                }
                write += 1;
            }
        }
        // The reference reports the difference between inclusive endpoints, not the slot count.
        let requested = range.len() as i32 - 1;
        Ok(Success {
            requested,
            completed: requested,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GapShift {
    pub leading: usize,
    pub compact: usize,
    pub trailing: usize,
}
pub fn shift_leading_gaps(bank: &mut Inventory, range: Range<usize>) -> Result<GapShift, Error> {
    if range.start > range.end || range.end > bank.slots().len() {
        return Err(Error::InvalidSlot);
    }
    let Some(first) = range.clone().find(|&i| bank.get(i).is_some()) else {
        return Ok(GapShift {
            leading: range.len(),
            compact: 0,
            trailing: range.len(),
        });
    };
    let Some(gap) = range.clone().find(|&i| bank.get(i).is_none()) else {
        return Ok(GapShift {
            leading: 0,
            compact: 0,
            trailing: 0,
        });
    };
    if gap == range.end - 1 {
        return Ok(GapShift {
            leading: 0,
            compact: range.len() - 1,
            trailing: 1,
        });
    }
    let last = range
        .clone()
        .rev()
        .find(|&i| bank.get(i).is_some())
        .unwrap();
    let mut write = range.start;
    for read in first..=last {
        if bank.get(write) != bank.get(read) {
            bank.restore_slot(write, bank.get(read))?;
        }
        if write != read && bank.get(read).is_some() {
            bank.restore_slot(read, None)?;
        }
        write += 1;
    }
    for slot in write..range.end {
        if bank.get(slot).is_some() {
            bank.restore_slot(slot, None)?;
        }
    }
    // Preserve Kotlin's absolute write-index result, including for a nonzero start.
    Ok(GapShift {
        leading: first - range.start,
        compact: write,
        trailing: range.end - write,
    })
}

/// Numbered tabs occupy the prefix; the main tab occupies the final range.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tabs {
    sizes: [usize; 10],
}
impl Tabs {
    pub const MAIN: usize = 9;
    pub fn restore(sizes: [usize; 10], capacity: usize) -> Result<Self, Error> {
        let sum = sizes
            .iter()
            .try_fold(0usize, |sum, size| sum.checked_add(*size))
            .ok_or(Error::InvalidCapacity)?;
        if sum > capacity {
            return Err(Error::InvalidCapacity);
        }
        Ok(Self { sizes })
    }
    pub fn sizes(&self) -> &[usize; 10] {
        &self.sizes
    }
    pub fn range(&self, tab: usize) -> Result<Range<usize>, Error> {
        if tab >= 10 {
            return Err(Error::InvalidSlot);
        }
        let start: usize = self.sizes[..tab].iter().sum();
        Ok(start..start + self.sizes[tab])
    }
    pub fn for_slot(&self, slot: usize) -> Option<usize> {
        (0..10).find(|&tab| self.range(tab).unwrap().contains(&slot))
    }
}

#[derive(Clone, Debug)]
pub struct Banking {
    pub tabs: Tabs,
    pub selected: usize,
    pub capacity: usize,
    pub withdraw_notes: bool,
    pub always_placehold: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BankError {
    Inventory(Error),
    RestrictedItem,
    InvalidTabs,
}
impl From<Error> for BankError {
    fn from(value: Error) -> Self {
        Self::Inventory(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Withdrawal {
    RemovedPlaceholder,
    Moved { result: Success, cannot_note: bool },
}
#[derive(Clone, Copy, Debug)]
pub struct DepositSummary {
    pub any_completed: bool,
    pub restricted: bool,
    pub capacity_error: bool,
    pub refresh_side: bool,
}
impl Banking {
    pub fn deposit_all(
        &mut self,
        from: &mut Inventory,
        bank: &mut Inventory,
        catalog: &Catalog,
        restricted: &std::collections::HashSet<i32>,
    ) -> Result<DepositSummary, BankError> {
        self.validate(bank)?;
        let keep: std::collections::HashSet<_> = from
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| {
                item.filter(|item| restricted.contains(&item.id))
                    .map(|_| slot)
            })
            .collect();
        let restricted_any = !keep.is_empty();
        let tab = self.selected;
        if tab == Tabs::MAIN {
            let before = bank.last_occupied_slot();
            let mut insert = Insert::new(0, 0);
            insert.untransform = true;
            insert.slot = self.tabs.range(tab)?.start;
            insert.capacity = Some(self.capacity);
            let result = crate::inventory::dump(from, bank, catalog, insert, &keep, false);
            if result == Err(Error::NotEnoughSpace) {
                return Ok(DepositSummary {
                    any_completed: false,
                    restricted: restricted_any,
                    capacity_error: true,
                    refresh_side: false,
                });
            }
            let increase = bank
                .last_occupied_slot()
                .checked_sub(before)
                .ok_or(BankError::InvalidTabs)?;
            self.tabs.sizes[tab] += increase;
            return Ok(DepositSummary {
                any_completed: result.is_ok_and(|r| r.completed != 0),
                restricted: restricted_any,
                capacity_error: false,
                refresh_side: true,
            });
        }
        let bank_ids: std::collections::HashSet<_> =
            bank.slots().iter().flatten().map(|item| item.id).collect();
        let unique: std::collections::HashSet<_> = from
            .slots()
            .iter()
            .flatten()
            .map(|item| {
                catalog
                    .notes
                    .get(&item.id)
                    .filter(|ty| ty.template != 0)
                    .map_or(item.id, |ty| ty.link)
            })
            .filter(|id| !restricted.contains(id))
            .collect();
        let contained = unique
            .iter()
            .filter(|&&id| {
                let id = catalog
                    .transforms
                    .get(&id)
                    .filter(|ty| ty.template != 0)
                    .map_or(id, |ty| ty.link);
                bank_ids.contains(&id)
                    || catalog.placeholders.get(&id).is_some_and(|ty| {
                        ty.template == 0 && ty.link > 0 && bank_ids.contains(&ty.link)
                    })
            })
            .count();
        let empty = self
            .tabs
            .range(tab)?
            .filter(|&slot| bank.get(slot).is_none())
            .count();
        let new_slots = unique
            .len()
            .checked_sub(contained + empty)
            .ok_or(BankError::InvalidTabs)?;
        if new_slots > 0 {
            for other in 0..10 {
                if other != tab {
                    let range = self.tabs.range(other)?;
                    self.compact_tab(bank, other)?;
                    self.trim(bank, range)?;
                }
            }
            if bank
                .shift_right(self.tabs.range(tab)?.end, new_slots)
                .is_err()
            {
                return Ok(DepositSummary {
                    any_completed: false,
                    restricted: restricted_any,
                    capacity_error: true,
                    refresh_side: false,
                });
            }
        }
        let mut seen = std::collections::HashSet::new();
        let slots: Vec<_> = from
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| {
                item.filter(|item| seen.insert(item.id))
                    .filter(|_| !keep.contains(&slot))
                    .map(|_| slot)
            })
            .collect();
        let mut source = from.clone();
        let mut target = bank.clone();
        let mut success = true;
        let mut any_completed = false;
        for &slot in &slots {
            let mut insert = Insert::new(0, 0);
            insert.slot = self.tabs.range(tab)?.start;
            insert.capacity = Some(self.capacity);
            insert.unnote = true;
            insert.untransform = true;
            insert.strict = false;
            match transfer(
                &mut source,
                &mut target,
                catalog,
                slot,
                i32::MAX,
                insert,
                false,
            ) {
                Ok(result) => any_completed |= result.completed > 0,
                Err(_) => {
                    success = false;
                    break;
                }
            }
        }
        if success {
            *from = source;
            *bank = target;
        }
        if !any_completed {
            return Ok(DepositSummary {
                any_completed: false,
                restricted: restricted_any,
                capacity_error: !slots.is_empty(),
                refresh_side: false,
            });
        }
        // Kotlin inspects completed queries even when a later query rolls the transaction back.
        self.tabs.sizes[tab] += new_slots;
        Ok(DepositSummary {
            any_completed,
            restricted: restricted_any,
            capacity_error: false,
            refresh_side: false,
        })
    }
    pub fn reorder_tabs(
        &mut self,
        bank: &mut Inventory,
        from: usize,
        into: usize,
    ) -> Result<(), BankError> {
        self.validate(bank)?;
        if from >= 9 || into >= 9 || self.tabs.sizes[from] == 0 || self.tabs.sizes[into] == 0 {
            return Err(Error::InvalidSlot.into());
        }
        let source = self.tabs.range(from)?;
        let target = self.tabs.range(into)?;
        let target = if from > into {
            target.start
        } else {
            target.end - 1
        };
        bank.shift_bulk(source, target)?;
        let size = self.tabs.sizes[from];
        if from > into {
            for i in (into..from).rev() {
                self.tabs.sizes[i + 1] = self.tabs.sizes[i];
            }
        } else {
            for i in from..into {
                self.tabs.sizes[i] = self.tabs.sizes[i + 1];
            }
        }
        self.tabs.sizes[into] = size;
        Ok(())
    }
    pub fn collapse_tab(
        &mut self,
        bank: &mut Inventory,
        catalog: &Catalog,
        tab: usize,
    ) -> Result<(), BankError> {
        self.validate(bank)?;
        if tab >= 9 {
            return Err(Error::InvalidSlot.into());
        }
        for tab in 0..9 {
            self.compact_tab(bank, tab)?;
        }
        let count = self.tabs.sizes[tab];
        // Kotlin's empty transaction fails after the preceding compression writes.
        if count == 0 {
            return Err(BankError::InvalidTabs);
        }
        let source = self.tabs.range(tab)?;
        let main = self.tabs.range(Tabs::MAIN)?;
        let empty: Vec<_> = main
            .clone()
            .filter(|&slot| bank.get(slot).is_none())
            .take(count)
            .collect();
        let mut image = bank.clone();
        for (i, &slot) in empty.iter().enumerate() {
            image.swap_slot(catalog, source.start + i, slot, false)?;
        }
        let new = count - empty.len();
        if new > 0 {
            image.shift_bulk(
                source.start + empty.len()..source.end,
                main.end.checked_sub(1).ok_or(Error::InvalidSlot)?,
            )?;
        }
        *bank = image;
        self.tabs.sizes[Tabs::MAIN] += new;
        let trailing = self.trim(bank, self.tabs.range(tab)?)?;
        self.tabs.sizes[tab] = self.tabs.sizes[tab].saturating_sub(trailing);
        if count > trailing {
            self.tabs.sizes[tab] = self.tabs.sizes[tab].saturating_sub(count - trailing);
        }
        self.compact_empty_tabs(tab);
        if self.selected == tab {
            self.selected = Tabs::MAIN;
        }
        Ok(())
    }
    pub fn release_tab_placeholders(
        &mut self,
        bank: &mut Inventory,
        catalog: &Catalog,
        tab: usize,
    ) -> Result<usize, BankError> {
        self.validate(bank)?;
        let mut removed = 0;
        for slot in self.tabs.range(tab)? {
            if bank.get(slot).is_some_and(|item| {
                catalog
                    .placeholders
                    .get(&item.id)
                    .is_some_and(|ty| ty.template != 0)
            }) {
                bank.restore_slot(slot, None)?;
                removed += 1;
            }
        }
        if removed > 0 {
            self.notify(bank, tab)?;
        }
        Ok(removed)
    }
    /// BankInvScript's ordinary grid drag, including the synthetic extended-tab slots.
    pub fn drag(
        &mut self,
        bank: &mut Inventory,
        catalog: &Catalog,
        from: usize,
        into: usize,
        insert: bool,
    ) -> Result<(), BankError> {
        self.validate(bank)?;
        let from_tab = self.tabs.for_slot(from).ok_or(Error::InvalidSlot)?;
        let extended = bank.slots().len() + 18;
        if (extended..extended + 10).contains(&into) {
            let offset = into - extended;
            let tab = if offset == 0 { Tabs::MAIN } else { offset - 1 };
            if tab == from_tab {
                return Err(Error::InvalidSlot.into());
            }
            return self.drag_extended(bank, catalog, from_tab, from, tab);
        }
        if into >= bank.slots().len() {
            return Err(Error::InvalidSlot.into());
        }
        let into_tab = self.tabs.for_slot(into).unwrap_or(Tabs::MAIN);
        let same = from_tab == into_tab;
        let swap = !insert || same && from.abs_diff(into) == 1;
        let empty = bank.get(into).is_none();
        let main_empty = same && from_tab == Tabs::MAIN && empty;
        let mut image = bank.clone();
        if swap {
            image.swap_slot(catalog, from, into, false)?;
        } else if !empty {
            let target = into - usize::from(from < into && !same);
            image.shift_insert(from, target)?;
        } else {
            image.swap_slot(catalog, from, into, false)?;
            if !main_empty {
                image.shift_left(from + 1, from, image.slots().len(), true)?;
            }
        }
        *bank = image;
        if !swap && !main_empty {
            self.tabs.sizes[from_tab] = self.tabs.sizes[from_tab].saturating_sub(1);
        }
        if !swap && !empty {
            self.tabs.sizes[into_tab] += 1;
        } else {
            let end = self.tabs.range(into_tab)?.end;
            if into >= end {
                self.tabs.sizes[into_tab] += into + 1 - end;
            }
        }
        // Swap recomputes the source tab by slot after expansion; shift uses its captured tab.
        let notify_tab = if swap {
            self.tabs.for_slot(from).ok_or(Error::InvalidSlot)?
        } else {
            from_tab
        };
        self.notify(bank, notify_tab)?;
        if self.tabs.sizes[from_tab] == 0 {
            self.compact_empty_tabs(from_tab);
        }
        Ok(())
    }
    fn drag_extended(
        &mut self,
        bank: &mut Inventory,
        catalog: &Catalog,
        from_tab: usize,
        from: usize,
        into_tab: usize,
    ) -> Result<(), BankError> {
        let range = self.tabs.range(into_tab)?;
        let empty = range.clone().find(|&slot| bank.get(slot).is_none());
        let last = range.end as i64 - 1;
        let into = empty.map_or(last, |slot| slot as i64);
        let target = into + i64::from(empty.is_none() && from as i64 > into);
        let target = usize::try_from(target).map_err(|_| Error::InvalidSlot)?;
        let mut image = bank.clone();
        if image.get(target).is_some() {
            image.shift_insert(from, target)?;
        } else {
            image.swap_slot(catalog, from, target, false)?;
            image.shift_left(from + 1, from, image.slots().len(), true)?;
        }
        *bank = image;
        self.tabs.sizes[from_tab] = self.tabs.sizes[from_tab].saturating_sub(1);
        if empty.is_none() {
            self.tabs.sizes[into_tab] += 1;
        }
        self.notify(bank, from_tab)?;
        if self.tabs.sizes[from_tab] == 0 {
            self.compact_empty_tabs(from_tab);
        }
        Ok(())
    }
    pub fn validate(&self, bank: &Inventory) -> Result<(), BankError> {
        if self.selected >= 10
            || self.capacity > bank.slots().len()
            || bank.slots().is_empty()
            || self.tabs.sizes.iter().sum::<usize>() > bank.slots().len()
        {
            return Err(BankError::InvalidTabs);
        }
        Ok(())
    }
    fn trim(&self, bank: &mut Inventory, range: Range<usize>) -> Result<usize, BankError> {
        let trailing = shift_leading_gaps(bank, range.clone())?.trailing;
        if trailing == 0 {
            return Ok(0);
        }
        let len = bank.slots().len();
        let start = range.end.min(len - 1);
        let into = (range.end - trailing).min(len - 1);
        let end = (bank.last_occupied_slot() + trailing).min(len);
        bank.shift_left(start, into, end, true)?;
        Ok(trailing)
    }
    fn compact_empty_tabs(&mut self, start: usize) {
        for tab in start..9 {
            self.tabs.sizes[tab] = if tab < 8 { self.tabs.sizes[tab + 1] } else { 0 };
        }
    }
    fn notify(&mut self, bank: &mut Inventory, tab: usize) -> Result<(), BankError> {
        let trailing = self.trim(bank, self.tabs.range(tab)?)?;
        if trailing == 0 {
            return Ok(());
        }
        self.tabs.sizes[tab] = self.tabs.sizes[tab].saturating_sub(trailing);
        if self.tabs.sizes[tab] == 0 && tab != Tabs::MAIN {
            self.compact_empty_tabs(tab);
            if self.selected == tab {
                self.selected = Tabs::MAIN;
            }
        }
        Ok(())
    }
    fn compact_tab(&mut self, bank: &mut Inventory, tab: usize) -> Result<(), BankError> {
        let range = self.tabs.range(tab)?;
        if range.is_empty() {
            return Ok(());
        }
        if range.len() > 1 {
            bank.compact_slots(range.clone())?;
        }
        let empty = range.filter(|&slot| bank.get(slot).is_none()).count();
        self.tabs.sizes[tab] = self.tabs.sizes[tab].saturating_sub(empty);
        Ok(())
    }
    pub fn compact(&mut self, bank: &mut Inventory) -> Result<(), BankError> {
        self.validate(bank)?;
        for tab in 0..10 {
            let range = self.tabs.range(tab)?;
            self.compact_tab(bank, tab)?;
            self.trim(bank, range)?;
        }
        Ok(())
    }
    /// `restricted` is the source item's resolved `no_bank` cache parameter.
    pub fn deposit(
        &mut self,
        source: &mut Inventory,
        bank: &mut Inventory,
        catalog: &Catalog,
        slot: usize,
        count: i32,
        restricted: bool,
    ) -> Result<Success, BankError> {
        self.validate(bank)?;
        let item = source.get(slot).ok_or(Error::ItemNotFound)?;
        if restricted {
            return Err(BankError::RestrictedItem);
        }
        let tab = self.selected;
        let placeholder = catalog
            .placeholders
            .get(&item.id)
            .map_or(item.id, |p| p.link);
        let contained = bank
            .slots()
            .iter()
            .position(|obj| obj.is_some_and(|obj| obj.id == item.id || obj.id == placeholder));
        let priority = contained.or_else(|| {
            self.tabs
                .range(tab)
                .unwrap()
                .find(|&slot| bank.get(slot).is_none())
        });
        // The original performs this compression outside the transaction, even if deposit fails.
        if priority.is_none() && tab != Tabs::MAIN {
            for other in 0..10 {
                if other == tab {
                    continue;
                }
                let range = self.tabs.range(other)?;
                self.compact_tab(bank, other)?;
                self.trim(bank, range)?;
            }
        }
        let range = self.tabs.range(tab)?;
        let mut target = bank.clone();
        let mut from = source.clone();
        if priority.is_none() && tab != Tabs::MAIN {
            target.shift_right(range.end, 1)?;
        }
        let mut query = Insert::new(item.id, count);
        query.slot = priority.unwrap_or(range.start);
        query.capacity = Some(self.capacity);
        query.unnote = true;
        query.strict = false;
        let result = transfer(&mut from, &mut target, catalog, slot, count, query, false)?;
        *source = from;
        *bank = target;
        let unnoted = catalog
            .notes
            .get(&item.id)
            .filter(|t| t.template != 0)
            .map_or(item.id, |t| t.link);
        if bank.get(range.end).is_some_and(|obj| obj.id == unnoted) {
            self.tabs.sizes[tab] += 1;
        }
        Ok(result)
    }
    pub fn withdraw(
        &mut self,
        bank: &mut Inventory,
        target: &mut Inventory,
        catalog: &Catalog,
        slot: usize,
        count: i32,
        force_placeholder: bool,
    ) -> Result<Withdrawal, BankError> {
        self.validate(bank)?;
        let item = bank.get(slot).ok_or(Error::ItemNotFound)?;
        let tab = self.tabs.for_slot(slot).ok_or(BankError::InvalidTabs)?;
        if catalog
            .placeholders
            .get(&item.id)
            .is_some_and(|p| p.template != 0)
        {
            bank.restore_slot(slot, None)?;
            self.notify(bank, tab)?;
            return Ok(Withdrawal::RemovedPlaceholder);
        }
        let mut query = Insert::new(item.id, count);
        query.strict = false;
        query.note = self.withdraw_notes;
        let result = transfer(
            bank,
            target,
            catalog,
            slot,
            count,
            query,
            self.always_placehold || force_placeholder,
        )?;
        if bank.get(slot).is_none() {
            self.notify(bank, tab)?;
        }
        let cannot_note = self.withdraw_notes
            && !catalog
                .notes
                .get(&item.id)
                .is_some_and(|t| t.template == 0 && t.link != 0);
        Ok(Withdrawal::Moved {
            result,
            cannot_note,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn tab_reorder_collapse_and_placeholder_release_preserve_item_order() {
        let mut bank = Inventory::new(12, StackType::Always, true);
        for id in 1..=5 {
            bank.restore_slot(
                id as usize - 1,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut banking = Banking {
            tabs: Tabs::restore([2, 1, 0, 0, 0, 0, 0, 0, 0, 2], 12).unwrap(),
            selected: 0,
            capacity: 12,
            withdraw_notes: false,
            always_placehold: false,
        };
        let mut catalog = Catalog::default();
        let ids = |bank: &Inventory| {
            bank.slots()
                .iter()
                .flatten()
                .map(|item| item.id)
                .collect::<Vec<_>>()
        };
        banking.reorder_tabs(&mut bank, 0, 1).unwrap();
        assert_eq!(ids(&bank), [3, 1, 2, 4, 5]);
        assert_eq!(banking.tabs.sizes(), &[1, 2, 0, 0, 0, 0, 0, 0, 0, 2]);
        banking.collapse_tab(&mut bank, &catalog, 0).unwrap();
        assert_eq!(ids(&bank), [1, 2, 4, 5, 3]);
        assert_eq!(banking.tabs.sizes(), &[2, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
        assert_eq!(banking.selected, Tabs::MAIN);
        catalog.placeholders.insert(
            4,
            crate::inventory::Template {
                link: 40,
                template: 1,
            },
        );
        assert_eq!(
            banking
                .release_tab_placeholders(&mut bank, &catalog, Tabs::MAIN)
                .unwrap(),
            1
        );
        assert_eq!(ids(&bank), [1, 2, 5, 3]);
        assert_eq!(banking.tabs.sizes(), &[2, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
    }
    #[test]
    fn numbered_tab_deposit_shifts_main_and_placeholder_release_trims_gap() {
        use crate::inventory::Template;
        let mut catalog = Catalog::default();
        catalog.placeholders.insert(
            100,
            Template {
                link: 102,
                template: 0,
            },
        );
        catalog.placeholders.insert(
            102,
            Template {
                link: 100,
                template: 1,
            },
        );
        let mut bank = Inventory::new(8, StackType::Always, true);
        bank.restore_slot(
            0,
            Some(Item {
                id: 100,
                count: 5,
                vars: 0,
            }),
        )
        .unwrap();
        bank.restore_slot(
            1,
            Some(Item {
                id: 995,
                count: 20,
                vars: 0,
            }),
        )
        .unwrap();
        let mut source = Inventory::new(28, StackType::Normal, false);
        source
            .restore_slot(
                0,
                Some(Item {
                    id: 200,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let mut banking = Banking {
            tabs: Tabs::restore([1, 0, 0, 0, 0, 0, 0, 0, 0, 1], 8).unwrap(),
            selected: 0,
            capacity: 8,
            withdraw_notes: false,
            always_placehold: true,
        };
        banking
            .deposit(&mut source, &mut bank, &catalog, 0, 1, false)
            .unwrap();
        assert_eq!(bank.get(1).unwrap().id, 200);
        assert_eq!(bank.get(2).unwrap().id, 995);
        assert_eq!(banking.tabs.range(0).unwrap(), 0..2);
        banking
            .withdraw(&mut bank, &mut source, &catalog, 0, 5, false)
            .unwrap();
        assert_eq!(bank.get(0).unwrap().id, 102);
        assert_eq!(
            banking
                .withdraw(&mut bank, &mut source, &catalog, 0, 1, false)
                .unwrap(),
            Withdrawal::RemovedPlaceholder
        );
        assert_eq!(bank.get(0).unwrap().id, 200);
        assert_eq!(bank.get(1).unwrap().id, 995);
        assert_eq!(banking.tabs.sizes()[0], 1);
        assert_eq!(banking.tabs.range(Tabs::MAIN).unwrap(), 1..2);
    }
    #[test]
    fn note_deposit_and_withdraw_and_restricted_deposit() {
        use crate::inventory::Template;
        let mut catalog = Catalog::default();
        catalog.notes.insert(
            100,
            Template {
                link: 101,
                template: 0,
            },
        );
        catalog.notes.insert(
            101,
            Template {
                link: 100,
                template: 799,
            },
        );
        let mut bank = Inventory::new(8, StackType::Always, true);
        let mut held = Inventory::new(2, StackType::Normal, false);
        held.restore_slot(
            0,
            Some(Item {
                id: 101,
                count: 50,
                vars: 0,
            }),
        )
        .unwrap();
        let mut banking = Banking {
            tabs: Tabs::default(),
            selected: Tabs::MAIN,
            capacity: 8,
            withdraw_notes: true,
            always_placehold: false,
        };
        let before = held.clone();
        assert_eq!(
            banking.deposit(&mut held, &mut bank, &catalog, 0, 50, true),
            Err(BankError::RestrictedItem)
        );
        assert_eq!(held, before);
        banking
            .deposit(&mut held, &mut bank, &catalog, 0, 50, false)
            .unwrap();
        assert_eq!(
            bank.get(0),
            Some(Item {
                id: 100,
                count: 50,
                vars: 0
            })
        );
        let result = banking
            .withdraw(&mut bank, &mut held, &catalog, 0, 50, false)
            .unwrap();
        assert!(matches!(
            result,
            Withdrawal::Moved {
                cannot_note: false,
                ..
            }
        ));
        assert_eq!(
            held.get(0),
            Some(Item {
                id: 101,
                count: 50,
                vars: 0
            })
        );
        assert_eq!(banking.tabs.sizes()[9], 0);
    }
    fn inv() -> Inventory {
        let mut bank = Inventory::new(8, StackType::Always, true);
        for slot in [1, 3, 4] {
            bank.restore_slot(
                slot,
                Some(Item {
                    id: slot as i32,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        bank.clear_modified();
        bank
    }
    #[test]
    fn failed_shift_is_atomic_and_tabs_keep_main_last() {
        let mut bank = inv();
        let before = bank.clone();
        assert_eq!(bank.shift_left(4, 1, 8, true), Err(Error::StrictSlotTaken));
        assert_eq!(bank, before);
        assert_eq!(bank.shift_right(1, 5), Err(Error::NotEnoughSpace));
        assert_eq!(bank, before);
        let tabs = Tabs::restore([2, 0, 1, 0, 0, 0, 0, 0, 0, 3], 8).unwrap();
        assert_eq!(tabs.range(Tabs::MAIN).unwrap(), 3..6);
        assert_eq!(tabs.for_slot(2), Some(2));
        assert_eq!(tabs.for_slot(6), None);
    }
    #[test]
    fn gap_shift_preserves_interior_gaps_then_compaction_removes_them() {
        let mut bank = inv();
        assert_eq!(
            shift_leading_gaps(&mut bank, 0..8).unwrap(),
            GapShift {
                leading: 1,
                compact: 4,
                trailing: 4
            }
        );
        assert!(bank.get(1).is_none());
        bank.compact_slots(0..8).unwrap();
        assert_eq!(
            bank.slots()[..3]
                .iter()
                .map(|v| v.unwrap().id)
                .collect::<Vec<_>>(),
            [1, 3, 4]
        );
    }
}
