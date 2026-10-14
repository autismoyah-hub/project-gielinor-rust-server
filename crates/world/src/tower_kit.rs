//! FloorOneScript.bankExtraKitTools and extraTake transaction decisions.
//! The login/ash-tick scheduler is separate; callers must supply known diary marks.
use crate::{
    inventory::{self, Catalog, Insert, Inventory},
    tower_diary::Progress,
};
use std::collections::BTreeSet;

pub const NAMES: [&str; 9] = [
    "Bronze pickaxe",
    "Bronze axe",
    "Hammer",
    "Tinderbox",
    "Small fishing net",
    "Knife",
    "Bow string",
    "Wooden shield",
    "Bronze sword",
];

#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Packed types matching each name, excluding certificates as ownsNamed does.
    pub named: [BTreeSet<i32>; 9],
}
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Number of successful transfer transactions, not number of objects.
    pub moved: i32,
    pub message: Option<&'static str>,
}
fn count(inv: &Inventory, ids: &BTreeSet<i32>) -> i32 {
    inv.slots()
        .iter()
        .flatten()
        .filter(|i| ids.contains(&i.id))
        .fold(0i32, |n, i| n.wrapping_add(i.count))
}
/// Preserve source slot selection: skip packKeep *slots*, falling back to the
/// first matching slot when retained quantity spans only a single stack.
pub fn extra_take(
    inv: &Inventory,
    worn: &Inventory,
    ids: &BTreeSet<i32>,
    keep: i32,
) -> Option<(usize, i32)> {
    let pack_keep = keep.wrapping_sub(count(worn, ids)).max(0);
    let slots: Vec<_> = inv
        .slots()
        .iter()
        .enumerate()
        .filter_map(|(s, i)| i.filter(|i| ids.contains(&i.id)).map(|_| s))
        .collect();
    let surplus = count(inv, ids).wrapping_sub(pack_keep);
    if surplus <= 0 {
        return None;
    }
    let slot = slots
        .get(pack_keep as usize)
        .or_else(|| slots.first())
        .copied()?;
    let take = inv.get(slot)?.count.min(surplus);
    (take > 0).then_some((slot, take))
}
impl Config {
    pub fn bank_extras(
        &self,
        inv: &mut Inventory,
        worn: &Inventory,
        bank: &mut Inventory,
        catalog: &Catalog,
        diary: Progress,
    ) -> Outcome {
        let d = diary.done;
        let keeps = [
            i32::from(d[2]),
            i32::from(d[0]),
            i32::from(d[6] || !d[3]),
            i32::from(d[3]),
            i32::from(d[4]),
            i32::from(d[5]),
            0,
            0,
            0,
        ];
        let mut moved = 0i32;
        for (ids, keep) in self.named.iter().zip(keeps) {
            moved = moved.wrapping_add(bank_named_extras(inv, worn, bank, catalog, ids, keep));
        }
        Outcome {
            moved,
            message: if moved <= 0 {
                None
            } else if d[0] {
                Some("Diary tools wait in the bank.")
            } else {
                Some("Diary tools wait in the bank. Tan 5 hides for the axe.")
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn inv(size: usize, stack: StackType) -> Inventory {
        Inventory::new(size, stack, false)
    }
    fn put(inv: &mut Inventory, slot: usize, id: i32, count: i32) {
        inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
            .unwrap();
    }
    #[test]
    fn worn_counts_and_stack_surplus_use_original_slot_priority() {
        let mut pack = inv(4, StackType::Always);
        let mut worn = inv(4, StackType::Normal);
        let ids = [10, 11].into();
        put(&mut pack, 0, 10, 2);
        assert_eq!(extra_take(&pack, &worn, &ids, 1), Some((0, 1)));
        put(&mut pack, 2, 11, 1);
        assert_eq!(extra_take(&pack, &worn, &ids, 1), Some((2, 1)));
        put(&mut worn, 3, 10, 1);
        assert_eq!(extra_take(&pack, &worn, &ids, 1), Some((0, 2)));
    }
    #[test]
    fn paid_rewards_keep_one_and_banked_equipment_does_not_count_as_worn() {
        let mut c = Config::default();
        c.named[1].insert(10);
        c.named[2].insert(11);
        let mut pack = inv(5, StackType::Normal);
        let worn = inv(4, StackType::Normal);
        let mut bank = inv(5, StackType::Always);
        put(&mut pack, 0, 10, 1);
        put(&mut pack, 1, 10, 1);
        put(&mut pack, 2, 11, 1);
        put(&mut bank, 0, 10, 1);
        let mut diary = Progress::default();
        diary.done[0] = true;
        let out = c.bank_extras(&mut pack, &worn, &mut bank, &Catalog::default(), diary);
        assert_eq!(out.moved, 1);
        assert_eq!(out.message, Some("Diary tools wait in the bank."));
        assert_eq!(pack.get(0).unwrap().id, 10);
        assert!(pack.get(1).is_none());
        assert_eq!(pack.get(2).unwrap().id, 11); // Hammer kept before scimitar reward.
        assert_eq!(bank.get(0).unwrap().count, 2);
    }
    #[test]
    fn full_bank_failure_does_not_drop_items_and_later_names_still_merge() {
        let mut c = Config::default();
        c.named[0].insert(10);
        c.named[1].insert(11);
        let mut pack = inv(3, StackType::Normal);
        let worn = inv(4, StackType::Normal);
        let mut bank = inv(1, StackType::Always);
        put(&mut pack, 0, 10, 1);
        put(&mut pack, 1, 11, 1);
        put(&mut bank, 0, 11, 1);
        let out = c.bank_extras(
            &mut pack,
            &worn,
            &mut bank,
            &Catalog::default(),
            Progress::default(),
        );
        assert_eq!(out.moved, 1);
        assert_eq!(pack.get(0).unwrap().id, 10);
        assert!(pack.get(1).is_none());
        assert_eq!(bank.get(0).unwrap().count, 2);
    }
    #[test]
    fn original_wrapping_total_can_suppress_surplus() {
        let mut pack = inv(2, StackType::Always);
        let worn = inv(1, StackType::Normal);
        put(&mut pack, 0, 10, i32::MAX);
        put(&mut pack, 1, 11, 1);
        assert_eq!(extra_take(&pack, &worn, &[10, 11].into(), 0), None);
    }
}

/// Result from reconcileTansFromLeather; publish/apply sync before its message.
#[derive(Debug)]
pub struct Reconcile {
    pub sync: Option<crate::tower_diary::Sync>,
    pub message: Option<String>,
}
pub fn reconcile_tans(
    diary: Progress,
    inv: &Inventory,
    bank: &Inventory,
    leather: &BTreeSet<i32>,
    levels: [i32; 3],
    staff_taken: bool,
) -> Reconcile {
    if diary.done[0] {
        return Reconcile {
            sync: None,
            message: None,
        };
    }
    let held = count(inv, leather).wrapping_add(count(bank, leather));
    if held <= diary.counts[0] {
        return Reconcile {
            sync: None,
            message: (held == 0 && diary.counts[0] == 0)
                .then(|| "No leather in pack or bank to count toward 20/20.".into()),
        };
    }
    let mut marks = diary.marks(levels, staff_taken);
    marks.counts[0] = held.min(crate::tower_diary::CAPS[0]);
    let sync = marks.session_sync();
    Reconcile {
        message: Some(format!("Hides tanned {}/5.", sync.marks.counts[0])),
        sync: Some(sync),
    }
}

#[cfg(test)]
mod reconcile_tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn pack(items: &[(i32, i32)]) -> Inventory {
        let mut inv = Inventory::new(items.len(), StackType::Always, false);
        for (slot, &(id, count)) in items.iter().enumerate() {
            inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
                .unwrap();
        }
        inv
    }
    #[test]
    fn known_leather_reconciles_to_cap_then_runs_original_diary_sync() {
        let diary = Progress {
            schema11: true,
            ..Default::default()
        };
        let result = reconcile_tans(
            diary,
            &pack(&[(1741, 2), (1742, 100)]),
            &pack(&[(1741, 4)]),
            &[1741].into(),
            [1; 3],
            false,
        );
        let sync = result.sync.unwrap();
        assert_eq!(sync.marks.counts[0], 5);
        assert_eq!(sync.marks.bounty, 1);
        assert_eq!(sync.message, Some("The chest holds a bronze axe."));
        assert_eq!(result.message.as_deref(), Some("Hides tanned 5/5."));
    }
    #[test]
    fn paid_row_and_lower_holdings_never_rewind_existing_tans() {
        let mut diary = Progress::default();
        diary.counts[0] = 4;
        assert!(
            reconcile_tans(diary, &pack(&[]), &pack(&[]), &[1741].into(), [1; 3], false)
                .message
                .is_none()
        );
        diary.done[0] = true;
        assert!(reconcile_tans(
            diary,
            &pack(&[(1741, 10)]),
            &pack(&[]),
            &[1741].into(),
            [1; 3],
            false
        )
        .sync
        .is_none());
    }
    #[test]
    fn empty_pack_keeps_historical_twenty_text_and_overflow_does_not_add_tans() {
        let diary = Progress::default();
        let result = reconcile_tans(diary, &pack(&[]), &pack(&[]), &[1741].into(), [1; 3], false);
        assert_eq!(
            result.message.as_deref(),
            Some("No leather in pack or bank to count toward 20/20.")
        );
        let result = reconcile_tans(
            diary,
            &pack(&[(1741, i32::MAX)]),
            &pack(&[(1741, 1)]),
            &[1741].into(),
            [1; 3],
            false,
        );
        assert!(result.sync.is_none());
        assert!(result.message.is_none());
    }
}

pub fn bank_named_extras(
    inv: &mut Inventory,
    worn: &Inventory,
    bank: &mut Inventory,
    catalog: &Catalog,
    ids: &BTreeSet<i32>,
    keep: i32,
) -> i32 {
    let mut moved = 0i32;
    while let Some((slot, amount)) = extra_take(inv, worn, ids, keep) {
        let mut insert = Insert::new(0, amount);
        insert.unnote = true;
        insert.strict = false;
        if inventory::transfer(inv, bank, catalog, slot, amount, insert, false).is_err() {
            break;
        }
        moved = moved.wrapping_add(1);
    }
    moved
}
