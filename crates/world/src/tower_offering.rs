//! FloorOneScript's named-bones selection and one-bone bank fallback.
//! Selection intentionally includes certificates; the bank move unnotes them.
use crate::inventory::{self, Catalog, Insert, Inventory};
use std::collections::BTreeSet;
#[derive(Clone, Debug)]
pub struct Bones {
    pub named: BTreeSet<i32>,
}
pub fn is_bones(name: &str) -> bool {
    let name = name.to_lowercase();
    name.contains("bones") && !name.contains("burnt")
}
impl Bones {
    pub fn slot(&self, inventory: &Inventory) -> Option<usize> {
        inventory
            .slots()
            .iter()
            .position(|item| item.is_some_and(|item| self.named.contains(&item.id)))
    }
    pub fn count(&self, inventory: &Inventory) -> i32 {
        inventory
            .slots()
            .iter()
            .flatten()
            .filter(|item| self.named.contains(&item.id))
            .fold(0i32, |count, item| count.wrapping_add(item.count))
    }
    /// Existing inventory wins. An empty slot is required even for a stack;
    /// only the first matching bank slot is tried and only one bone moves.
    pub fn take_from_bank(
        &self,
        inventory: &mut Inventory,
        bank: &mut Inventory,
        catalog: &Catalog,
    ) -> bool {
        if self.slot(inventory).is_some() {
            return true;
        }
        if inventory.free_space() == 0 {
            return false;
        }
        let Some(slot) = self.slot(bank) else {
            return false;
        };
        let mut insert = Insert::new(0, 1);
        insert.unnote = true;
        insert.strict = false;
        inventory::transfer(bank, inventory, catalog, slot, 1, insert, false).is_ok()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType, Template};
    fn containers() -> (Inventory, Inventory) {
        (
            Inventory::new(3, StackType::Normal, false),
            Inventory::new(5, StackType::Always, true),
        )
    }
    #[test]
    fn named_filter_and_integer_count_match_source() {
        assert!(is_bones("BIG BoNeS"));
        assert!(is_bones("Noted bones"));
        assert!(!is_bones("Burnt bones"));
        assert!(!is_bones("Bone fragments"));
        let (mut inv, _) = containers();
        for (slot, id, count) in [(0, 527, i32::MAX), (1, 526, 1), (2, 999, 10)] {
            inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
                .unwrap();
        }
        let bones = Bones {
            named: [526, 527].into(),
        };
        assert_eq!(bones.slot(&inv), Some(0));
        assert_eq!(bones.count(&inv), i32::MIN);
    }
    #[test]
    fn fallback_unnotes_exactly_one_and_preserves_other_slots() {
        let (mut inv, mut bank) = containers();
        let mut catalog = Catalog::default();
        catalog.notes.insert(
            527,
            Template {
                link: 526,
                template: 799,
            },
        );
        bank.restore_slot(
            1,
            Some(Item {
                id: 527,
                count: 9,
                vars: 0,
            }),
        )
        .unwrap();
        bank.restore_slot(
            2,
            Some(Item {
                id: 532,
                count: 4,
                vars: 0,
            }),
        )
        .unwrap();
        let bones = Bones {
            named: [526, 527, 532].into(),
        };
        assert!(bones.take_from_bank(&mut inv, &mut bank, &catalog));
        assert_eq!(
            inv.get(0),
            Some(Item {
                id: 526,
                count: 1,
                vars: 0
            })
        );
        assert_eq!(bank.get(1).unwrap().count, 8);
        assert_eq!(bank.get(2).unwrap().count, 4);
        assert!(bones.take_from_bank(&mut inv, &mut bank, &catalog));
        assert_eq!(bank.get(1).unwrap().count, 8);
    }
    #[test]
    fn full_inventory_and_failed_first_bone_never_skip_to_later_bank_items() {
        let (mut inv, mut bank) = containers();
        let mut catalog = Catalog::default();
        bank.restore_slot(
            0,
            Some(Item {
                id: 527,
                count: 2,
                vars: 1,
            }),
        )
        .unwrap();
        bank.restore_slot(
            1,
            Some(Item {
                id: 526,
                count: 3,
                vars: 0,
            }),
        )
        .unwrap();
        catalog.notes.insert(
            527,
            Template {
                link: 526,
                template: 799,
            },
        );
        let bones = Bones {
            named: [526, 527].into(),
        };
        let original = bank.clone();
        assert!(!bones.take_from_bank(&mut inv, &mut bank, &catalog));
        assert_eq!(bank, original);
        assert_eq!(inv.free_space(), 3);
        inv.insert(&catalog, Insert::new(995, 3)).unwrap();
        assert!(!bones.take_from_bank(&mut inv, &mut bank, &catalog));
        assert_eq!(bank, original);
    }
}

/// Captured by offerBones before entering the prayer coroutine. Cancellation
/// discards this continuation, even if the prayer loop has consumed some bones.
#[derive(Clone, Copy, Debug)]
pub struct Continuation {
    pub before: i32,
}
#[derive(Debug)]
pub struct Completion {
    pub sync: crate::tower_diary::Sync,
    pub summary: String,
    pub crate_message: bool,
    pub visit_crate: bool,
}
impl Continuation {
    /// Source completion decisions. The caller publishes sync first, then the
    /// summary and optional crate message, and finally runs visitCrate.
    pub fn complete(
        self,
        player: &mut crate::simulation::PlayerData,
        bones: &Bones,
        staff: &crate::tower_staff::Config,
        altar: crate::CoordGrid,
        skills: [u8; 3],
        offered: bool,
    ) -> Result<Option<Completion>, crate::simulation::ActionError> {
        if !offered {
            return Ok(None);
        }
        let progress = player
            .tower_diary
            .ok_or(crate::simulation::ActionError::UnknownTowerDiary)?;
        let n = self
            .before
            .wrapping_sub(bones.count(&player.inventory))
            .max(1);
        let sync = progress
            .marks(
                skills.map(|id| player.stats.base(id) as i32),
                player.tower_arrival.staff_taken,
            )
            .add(2, n)
            .session_sync();
        player.tower_diary = Some(sync.marks.into());
        player.tower_arrival.diary_progress |= player.tower_diary.unwrap().has_progress();
        let near_altar = player.coords.chebyshev_distance(altar) <= 2;
        if near_altar {
            player.tower_arrival.bones_offered = Some(true);
            player.tower_arrival.diary_progress = true;
        }
        let crate_message = near_altar && staff.needs_search(player);
        Ok(Some(Completion {
            summary: format!(
                "Bones offered {}/5. Hides tanned {}/5.",
                sync.marks.counts[1], sync.marks.counts[0]
            ),
            crate_message,
            visit_crate: crate_message && sync.marks.counts[1] >= 5,
            sync,
        }))
    }
}
#[cfg(test)]
mod completion_tests {
    use super::*;
    use crate::{
        simulation::PlayerData, stats::Stats, tower_diary::Progress, tower_staff, CoordGrid,
    };
    fn setup() -> (PlayerData, Bones, tower_staff::Config, CoordGrid) {
        let altar = CoordGrid::new(10428, 116, 0).unwrap();
        let mut p = PlayerData::new(
            1,
            "Offering completion".into(),
            altar.translate(1, 0, 0).unwrap(),
            Stats::default(),
        );
        p.tower_diary = Some(Progress {
            schema11: true,
            ..Default::default()
        });
        let staff = tower_staff::Config {
            staff: 1381,
            talisman: 1448,
            staff_names: [1381].into(),
            staff_owned: [1381].into(),
            talisman_owned: [1448].into(),
        };
        (
            p,
            Bones {
                named: [526, 532].into(),
            },
            staff,
            altar,
        )
    }
    #[test]
    fn offering_flag_is_independent_of_diary_row_eligibility() {
        let (mut p, bones, staff, altar) = setup();
        let outcome = Continuation { before: 5 }
            .complete(&mut p, &bones, &staff, altar, [12, 13, 11], true)
            .unwrap()
            .unwrap();
        assert_eq!(outcome.sync.marks.counts[1], 0); // Row1 still current.
        assert_eq!(p.tower_arrival.bones_offered, Some(true));
        assert!(outcome.crate_message);
        assert!(!outcome.visit_crate);
        assert_eq!(outcome.summary, "Bones offered 0/5. Hides tanned 0/5.");
    }
    #[test]
    fn fifth_eligible_offering_queues_crate_after_diary_and_summary() {
        let (mut p, bones, staff, altar) = setup();
        let progress = p.tower_diary.as_mut().unwrap();
        progress.done[0] = true;
        progress.counts[1] = 4;
        let outcome = Continuation { before: 1 }
            .complete(&mut p, &bones, &staff, altar, [12, 13, 11], true)
            .unwrap()
            .unwrap();
        assert_eq!(outcome.sync.marks.bounty, 2);
        assert_eq!(outcome.sync.marks.counts[1], 5);
        assert!(outcome.crate_message && outcome.visit_crate);
        assert_eq!(
            outcome.sync.message,
            Some("The chest holds a needle and thread.")
        );
        p.tower_arrival.staff_taken = true;
        let repeat = Continuation { before: 1 }
            .complete(&mut p, &bones, &staff, altar, [12, 13, 11], true)
            .unwrap()
            .unwrap();
        assert!(!repeat.crate_message && !repeat.visit_crate);
    }
    #[test]
    fn false_return_and_remote_completion_do_not_set_offering_flag() {
        let (mut p, bones, staff, altar) = setup();
        let before = p.tower_diary;
        assert!(Continuation { before: 5 }
            .complete(&mut p, &bones, &staff, altar, [12, 13, 11], false)
            .unwrap()
            .is_none());
        assert_eq!(p.tower_diary, before);
        assert_eq!(p.tower_arrival.bones_offered, None);
        p.coords = altar.translate(3, 0, 0).unwrap();
        p.tower_diary.as_mut().unwrap().done[0] = true;
        let outcome = Continuation { before: 2 }
            .complete(&mut p, &bones, &staff, altar, [12, 13, 11], true)
            .unwrap()
            .unwrap();
        assert_eq!(outcome.sync.marks.counts[1], 2);
        assert!(!outcome.crate_message);
        assert_eq!(p.tower_arrival.bones_offered, None);
    }
}

#[derive(Clone, Debug)]
pub struct HandlerConfig {
    pub location: u32,
    pub origin: crate::CoordGrid,
    pub animation: u16,
    pub accepted: [i32; 2],
    pub bones: Bones,
    pub xp: std::collections::BTreeMap<i32, Option<f64>>,
    pub skills: [u8; 3],
    pub drop_bones: u16,
}
