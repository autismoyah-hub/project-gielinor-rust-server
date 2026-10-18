//! Worn skill-cape boosts stage the existing stat operation; no new boost engine.
use crate::{
    inventory::{Catalog, Inventory},
    stat_ops::{Operation, Update},
    stats::Stats,
};
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub skill: u8,
    pub slot: usize,
    pub button_operation: u8,
    pub members: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Policy,
    Members,
    Level,
    Item,
}
pub struct Prepared {
    pub stats: Stats,
    pub update: Update,
}
impl Rule {
    pub fn prepare(
        self,
        item: i32,
        slot: usize,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        member: bool,
    ) -> Result<Prepared, Error> {
        if self.skill >= 24 || self.slot >= 14 || self.button_operation != 2 {
            return Err(Error::Policy);
        }
        if self.members && !member {
            return Err(Error::Members);
        }
        if slot != self.slot
            || !items.ordinary_nonstackable(item)
            || worn
                .get(slot)
                .is_none_or(|i| i.id != item || i.count != 1 || i.vars != 0)
        {
            return Err(Error::Item);
        }
        if stats.base(self.skill) as u8 != 99 {
            return Err(Error::Level);
        }
        let mut next = stats.clone();
        let update = next
            .apply(self.skill, Operation::Boost, 1, 0, false)
            .map_err(|_| Error::Policy)?;
        Ok(Prepared {
            stats: next,
            update,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn fixture() -> (Rule, Inventory, Catalog, Stats) {
        let rule = Rule {
            skill: 12,
            slot: 1,
            button_operation: 2,
            members: true,
        };
        let mut worn = Inventory::new(15, StackType::Normal, false);
        worn.restore_slot(
            1,
            Some(Item {
                id: 9780,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut stats = Stats::default();
        stats.set_base(12, 99);
        stats.set_current(12, 99);
        stats
            .set_xp(12, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        (rule, worn, Catalog::default(), stats)
    }
    #[test]
    fn worn_boost_uses_shared_plus_one_caps_no_stacking_or_xp_and_preserves_higher_boost() {
        for current in [50, 98, 99, 100, 104] {
            let (rule, worn, items, mut stats) = fixture();
            stats.set_current(12, current);
            let before = stats.clone();
            let p = rule.prepare(9780, 1, &worn, &items, &stats, true).unwrap();
            assert_eq!(
                p.stats.current(12),
                if current < 100 { current + 1 } else { current }
            );
            assert_eq!(p.stats.base(12), 99);
            assert_eq!(p.stats.fine_xp(12), stats.fine_xp(12));
            assert_eq!(p.update.notify, current < 100);
            assert_eq!(p.update.changed, current < 100);
            assert_eq!(stats, before);
        }
    }
    #[test]
    fn worn_boost_member_base_and_exact_gear_or_policy_failures_do_not_change_stats() {
        for case in 0..9 {
            let (mut rule, mut worn, mut items, mut stats) = fixture();
            let mut slot = 1;
            let mut member = true;
            match case {
                0 => member = false,
                1 => stats.set_base(12, 98),
                2 => slot = 4,
                3 => {
                    let mut i = worn.get(1).unwrap();
                    i.vars = 9;
                    worn.restore_slot(1, Some(i)).unwrap();
                }
                4 => {
                    let mut i = worn.get(1).unwrap();
                    i.count = 2;
                    worn.restore_slot(1, Some(i)).unwrap();
                }
                5 => {
                    items.dummy.insert(9780);
                }
                6 => {
                    worn.restore_slot(1, None).unwrap();
                }
                7 => rule.skill = 24,
                8 => rule.button_operation = 3,
                _ => unreachable!(),
            }
            let before = (worn.clone(), stats.clone());
            assert!(
                rule.prepare(9780, slot, &worn, &items, &stats, member)
                    .is_err(),
                "case{case}"
            );
            assert_eq!((worn, stats), before);
        }
    }
}
