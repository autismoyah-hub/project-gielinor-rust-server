//! Ordinary two-input finished-potion transactions, not runtime registration.
//! Wiki Attack potion15247702, Antipoison15292478, Strength potion15183998.
//! Equipment modifiers and alternate recipes must be resolved by the caller.
use crate::{
    assembly,
    inventory::{Catalog, Inventory},
    skills::{self, Advancement},
    stats::Stats,
};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub unfinished: i32,
    pub secondary: i32,
    pub output: i32,
    pub level: u8,
    pub boostable: bool,
    pub base_fine_xp: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub unfinished_slot: usize,
    pub secondary_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made(Advancement),
    InvalidRecipe,
    Members,
    Quest,
    Level,
    Stale,
    Capacity,
    InvalidReward,
}
impl Job {
    pub fn complete(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        member: bool,
        quest_complete: bool,
        rate_hundreds: i32,
    ) -> Outcome {
        let r = self.recipe;
        if r.unfinished < 0
            || r.secondary < 0
            || r.output < 0
            || r.unfinished == r.secondary
            || r.unfinished == r.output
            || r.secondary == r.output
            || !(1..=99).contains(&r.level)
            || r.base_fine_xp == 0
        {
            return Outcome::InvalidRecipe;
        }
        if !member {
            return Outcome::Members;
        }
        if !quest_complete {
            return Outcome::Quest;
        }
        if rate_hundreds <= 0 {
            return Outcome::InvalidReward;
        }
        if i32::from(stats.current(15)) < i32::from(r.level)
            || (!r.boostable && i32::from(stats.base(15)) < i32::from(r.level))
        {
            return Outcome::Level;
        }
        if self.unfinished_slot == self.secondary_slot
            || [
                (self.unfinished_slot, r.unfinished),
                (self.secondary_slot, r.secondary),
            ]
            .iter()
            .any(|&(s, id)| {
                inventory
                    .get(s)
                    .is_none_or(|i| i.id != id || i.count != 1 || i.vars != 0)
            })
        {
            return Outcome::Stale;
        }
        let mut next_inventory = inventory.clone();
        let mut next_stats = stats.clone();
        // The explicit non-boostable gate supplements the existing live-current
        // requirement. Reuse the shared strict-slot exchange and level check.
        let exchange = assembly::Job {
            recipe: assembly::Recipe {
                first: r.unfinished,
                second: r.secondary,
                output: r.output,
                skill: 15,
                level: r.level,
                base_xp: 0,
                members: true,
            },
            first_slot: self.unfinished_slot,
            second_slot: self.secondary_slot,
        };
        match exchange
            .complete_awarding(
                &mut next_inventory,
                catalog,
                &mut next_stats,
                rate_hundreds,
                member,
            )
            .0
        {
            assembly::Outcome::Made => (),
            assembly::Outcome::Capacity => return Outcome::Capacity,
            assembly::Outcome::Stale => return Outcome::Stale,
            assembly::Outcome::Level => return Outcome::Level,
            assembly::Outcome::Members => return Outcome::Members,
            assembly::Outcome::InvalidReward => return Outcome::InvalidReward,
        }
        let advancement = match skills::advance(
            &mut next_stats,
            15,
            f64::from(r.base_fine_xp) / 10.0,
            f64::from(rate_hundreds) / 100.0,
            99,
        ) {
            Ok(a) => a,
            Err(_) => return Outcome::InvalidReward,
        };
        *inventory = next_inventory;
        *stats = next_stats;
        Outcome::Made(advancement)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn fixture(r: Recipe, reverse: bool) -> (Job, Inventory, Catalog, Stats) {
        let (a, b) = if reverse { (1, 0) } else { (0, 1) };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for s in 0..28 {
            inv.restore_slot(
                s,
                Some(Item {
                    id: if s == a {
                        r.unfinished
                    } else if s == b {
                        r.secondary
                    } else {
                        1931
                    },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut stats = Stats::default();
        stats.set_base(15, r.level as i8);
        stats.set_current(15, r.level as i8);
        stats
            .set_fine_xp(15, skills::xp_from_level(r.level).unwrap() * 10 + 1)
            .unwrap();
        (
            Job {
                recipe: r,
                unfinished_slot: a,
                secondary_slot: b,
            },
            inv,
            Catalog::default(),
            stats,
        )
    }
    fn profiles() -> [Recipe; 3] {
        [
            Recipe {
                unfinished: 91,
                secondary: 221,
                output: 121,
                level: 3,
                boostable: false,
                base_fine_xp: 250,
            },
            Recipe {
                unfinished: 93,
                secondary: 235,
                output: 175,
                level: 5,
                boostable: true,
                base_fine_xp: 375,
            },
            Recipe {
                unfinished: 95,
                secondary: 225,
                output: 115,
                level: 12,
                boostable: true,
                base_fine_xp: 500,
            },
        ]
    }
    #[test]
    fn full_backpack_rewards_preserve_fractions_and_no_replay() {
        for recipe in profiles() {
            for reverse in [false, true] {
                for rate in [100, 500, 501] {
                    let (job, mut inv, catalog, mut stats) = fixture(recipe, reverse);
                    let before = stats.clone();
                    let Outcome::Made(award) =
                        job.complete(&mut inv, &catalog, &mut stats, true, true, rate)
                    else {
                        panic!("valid profile refused")
                    };
                    let expected = i32::from(recipe.base_fine_xp) * rate / 100;
                    assert_eq!(stats.fine_xp(15) - before.fine_xp(15), expected);
                    assert_eq!(award.added_xp, expected / 10);
                    assert_eq!(
                        inv.get(0),
                        Some(Item {
                            id: recipe.output,
                            count: 1,
                            vars: 0
                        })
                    );
                    assert!(inv.get(1).is_none());
                    for slot in 2..28 {
                        assert_eq!(
                            inv.get(slot),
                            Some(Item {
                                id: 1931,
                                count: 1,
                                vars: 0
                            })
                        );
                    }
                    for skill in 0..25 {
                        if skill != 15 {
                            assert_eq!(stats.fine_xp(skill), before.fine_xp(skill));
                            assert_eq!(stats.base(skill), before.base(skill));
                            assert_eq!(stats.current(skill), before.current(skill));
                        }
                    }
                    let state = (inv.clone(), stats.clone());
                    assert_eq!(
                        job.complete(&mut inv, &catalog, &mut stats, true, true, rate),
                        Outcome::Stale
                    );
                    assert_eq!((inv, stats), state);
                }
            }
        }
    }
    #[test]
    fn explicit_boost_policy_and_live_refusals_conserve_state() {
        for recipe in profiles() {
            let (job, mut inv, catalog, mut stats) = fixture(recipe, false);
            stats.set_base(15, 1);
            let state = (inv.clone(), stats.clone());
            let outcome = job.complete(&mut inv, &catalog, &mut stats, true, true, 500);
            if recipe.boostable {
                assert!(matches!(outcome, Outcome::Made(_)));
            } else {
                assert_eq!(outcome, Outcome::Level);
                assert_eq!((inv, stats), state);
            }
            for mode in 0..8 {
                let (job, mut inv, mut catalog, mut stats) = fixture(recipe, false);
                match mode {
                    3 => {
                        stats.set_base(15, 1);
                        stats.set_current(15, 1);
                    }
                    4 => {
                        inv.restore_slot(1, None).unwrap();
                    }
                    5 => {
                        catalog.dummy.insert(recipe.output);
                    }
                    6 => {
                        inv.restore_slot(
                            1,
                            Some(Item {
                                id: recipe.secondary,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap();
                    }
                    7 => stats.set_current(15, recipe.level as i8 - 1),
                    _ => (),
                }
                let state = (inv.clone(), stats.clone());
                let result = job.complete(
                    &mut inv,
                    &catalog,
                    &mut stats,
                    mode != 0,
                    mode != 1,
                    if mode == 2 { 0 } else { 500 },
                );
                assert!(!matches!(result, Outcome::Made(_)));
                assert_eq!((inv, stats), state);
            }
        }
    }
    #[test]
    fn experience_cap_does_not_prevent_valid_inventory_exchange() {
        for remaining in [0, 5, 1875, 1876] {
            let (job, mut inv, catalog, mut stats) = fixture(profiles()[1], false);
            stats.set_base(15, 99);
            stats.set_current(15, 99);
            stats
                .set_fine_xp(15, Stats::MAX_FINE_XP - remaining)
                .unwrap();
            let before = stats.clone();
            let Outcome::Made(award) =
                job.complete(&mut inv, &catalog, &mut stats, true, true, 500)
            else {
                panic!("cap rejected inventory exchange")
            };
            let delta = remaining.min(1875);
            assert_eq!(stats.fine_xp(15) - before.fine_xp(15), delta);
            assert_eq!(award.added_xp, delta / 10);
            assert_eq!((stats.base(15), stats.current(15)), (99, 99));
            assert_eq!(inv.get(0).unwrap().id, 175);
        }
    }
}
