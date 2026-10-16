//! Ordinary two-input finished-potion transactions and shared-runtime configuration.
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
    /// Prescription goggles for the currently supported ordinary single-secondary
    /// recipes. The caller resolves live equipment and recipe eligibility.
    /// Validate the ordinary exchange before drawing; publish inventory and XP
    /// together, including when the secondary is retained in its selected slot.
    pub fn complete_with_goggles(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        member: bool,
        quest_complete: bool,
        rate_hundreds: i32,
        eligible_goggles: bool,
        random: &mut impl crate::npc::RandomSource,
    ) -> (Outcome, bool) {
        let mut next_inventory = inventory.clone();
        let mut next_stats = stats.clone();
        let outcome = self.complete(&mut next_inventory, catalog, &mut next_stats, member, quest_complete, rate_hundreds);
        if !matches!(outcome, Outcome::Made(_)) { return (outcome, false); }
        let saved = eligible_goggles && random.below(10) == 0;
        if saved {
            // Removing only the unfinished vial leaves the selected secondary
            // untouched even when it precedes the vial in a full backpack.
            next_inventory = inventory.clone();
            let mut debit = crate::inventory::Delete::new(self.recipe.unfinished, 1);
            debit.strict_slot = Some(self.unfinished_slot);
            if next_inventory.delete(catalog, debit).is_err()
                || next_inventory.insert(catalog, crate::inventory::Insert::new(self.recipe.output, 1)).is_err() {
                return (Outcome::Capacity, false);
            }
        }
        *inventory = next_inventory;
        *stats = next_stats;
        (outcome, saved)
    }

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
    fn profiles() -> [Recipe; 29] {
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
            Recipe { unfinished: 97, secondary: 223, output: 127, level: 22, boostable: true, base_fine_xp: 625 },
            Recipe { unfinished: 97, secondary: 1975, output: 3010, level: 26, boostable: true, base_fine_xp: 675 },
            Recipe { unfinished: 99, secondary: 239, output: 133, level: 30, boostable: true, base_fine_xp: 750 },
            Recipe { unfinished: 3002, secondary: 2152, output: 3034, level: 34, boostable: true, base_fine_xp: 800 },
            Recipe { unfinished: 97, secondary: 9736, output: 9741, level: 36, boostable: true, base_fine_xp: 840 },
            Recipe { unfinished: 99, secondary: 231, output: 139, level: 38, boostable: true, base_fine_xp: 875 },
            Recipe { unfinished: 101, secondary: 221, output: 145, level: 45, boostable: true, base_fine_xp: 1000 },
            Recipe { unfinished: 101, secondary: 235, output: 181, level: 48, boostable: true, base_fine_xp: 1063 },
            Recipe { unfinished: 103, secondary: 231, output: 151, level: 50, boostable: true, base_fine_xp: 1125 },
            Recipe { unfinished: 103, secondary: 2970, output: 3018, level: 52, boostable: true, base_fine_xp: 1175 },
            Recipe { unfinished: 103, secondary: 10111, output: 10000, level: 53, boostable: true, base_fine_xp: 1200 },
            Recipe { unfinished: 105, secondary: 225, output: 157, level: 55, boostable: true, base_fine_xp: 1250 },
            Recipe { unfinished: 5942, secondary: 6049, output: 5943, level: 68, boostable: true, base_fine_xp: 1550 },
            Recipe { unfinished: 5951, secondary: 6051, output: 5952, level: 79, boostable: true, base_fine_xp: 1775 },
            Recipe { unfinished: 22443, secondary: 245, output: 22464, level: 80, boostable: true, base_fine_xp: 1550 },
            Recipe { unfinished: 22443, secondary: 3138, output: 22452, level: 80, boostable: true, base_fine_xp: 1550 },
            Recipe { unfinished: 3004, secondary: 223, output: 3026, level: 63, boostable: true, base_fine_xp: 1425 },
            Recipe { unfinished: 107, secondary: 239, output: 163, level: 66, boostable: true, base_fine_xp: 1500 },
            Recipe { unfinished: 2483, secondary: 241, output: 2454, level: 69, boostable: true, base_fine_xp: 1575 },
            Recipe { unfinished: 109, secondary: 245, output: 169, level: 72, boostable: true, base_fine_xp: 1625 },
            Recipe { unfinished: 2483, secondary: 3138, output: 3042, level: 76, boostable: true, base_fine_xp: 1725 },
            Recipe { unfinished: 111, secondary: 247, output: 189, level: 78, boostable: true, base_fine_xp: 1750 },
            Recipe { unfinished: 3002, secondary: 6693, output: 6687, level: 81, boostable: true, base_fine_xp: 1800 },
            Recipe { unfinished: 30100, secondary: 29993, output: 30128, level: 58, boostable: true, base_fine_xp: 1320 },
            Recipe { unfinished: 97, secondary: 29993, output: 30140, level: 54, boostable: true, base_fine_xp: 1320 },
            Recipe { unfinished: 105, secondary: 241, output: 187, level: 60, boostable: true, base_fine_xp: 1375 },
        ]
    }
    #[test]
    fn goggles_keep_selected_secondary_and_full_reward_for_every_profile_and_order() {
        struct Draw { value:u32, calls:usize }
        impl crate::npc::RandomSource for Draw {
            fn below(&mut self, limit:u32)->u32 {assert_eq!(limit,10);self.calls+=1;self.value}
        }
        for recipe in profiles() {
            for reverse in [false,true] {
                for draw in 0..10 {
                    let (job,mut inventory,catalog,mut stats)=fixture(recipe,reverse);
                    let before=stats.fine_xp(15);let mut rng=Draw{value:draw,calls:0};
                    let (outcome,saved)=job.complete_with_goggles(&mut inventory,&catalog,&mut stats,true,true,500,true,&mut rng);
                    assert!(matches!(outcome,Outcome::Made(_)));assert_eq!(saved,draw==0);assert_eq!(rng.calls,1);
                    assert_eq!(stats.fine_xp(15)-before,i32::from(recipe.base_fine_xp)*5);
                    if saved {
                        assert_eq!(inventory.get(job.secondary_slot),Some(Item{id:recipe.secondary,count:1,vars:0}));
                        assert_eq!(inventory.get(job.unfinished_slot),Some(Item{id:recipe.output,count:1,vars:0}));
                    } else {
                        assert_eq!(inventory.get(0),Some(Item{id:recipe.output,count:1,vars:0}));assert!(inventory.get(1).is_none());
                    }
                    let final_state=(inventory.clone(),stats.clone());
                    assert_eq!(job.complete_with_goggles(&mut inventory,&catalog,&mut stats,true,true,500,true,&mut rng),(Outcome::Stale,false));
                    assert_eq!((inventory,stats),final_state);assert_eq!(rng.calls,1);
                }
            }
        }
        for mode in 0..6 {
            let (job,mut inv,mut catalog,mut stats)=fixture(profiles()[1],false);
            if mode==3 {inv.restore_slot(1,None).unwrap();}
            if mode==4 {catalog.dummy.insert(job.recipe.output);}
            if mode==5 {stats.set_current(15,1);}
            let before=(inv.clone(),stats.clone());let mut rng=Draw{value:0,calls:0};
            let (outcome,saved)=job.complete_with_goggles(&mut inv,&catalog,&mut stats,mode!=0,mode!=1,if mode==2{0}else{500},true,&mut rng);
            assert!(!matches!(outcome,Outcome::Made(_)));assert!(!saved);assert_eq!(rng.calls,0);assert_eq!((inv,stats),before);
        }
        let (job,mut inv,catalog,mut stats)=fixture(profiles()[1],true);let mut rng=Draw{value:0,calls:0};
        let (outcome,saved)=job.complete_with_goggles(&mut inv,&catalog,&mut stats,true,true,500,false,&mut rng);
        assert!(matches!(outcome,Outcome::Made(_)));assert!(!saved);assert_eq!(rng.calls,0);
    }

    #[test]
    fn ingredient_pairs_preserve_shared_bases_and_reverse_selected_slots() {
        let config=Config{goggles:[29974,29976],recipes:profiles().into_iter().map(|r|((r.unfinished,r.secondary),r)).collect(),quest_progress:80,quest_complete:4,animation:363};
        assert_eq!(config.recipes.len(),29);
        let origin=crate::CoordGrid::new(3200,3200,0).unwrap();
        for recipe in profiles() {
            for reverse in [false,true] {
                let held=crate::held_use::Request{selected_id:if reverse {recipe.secondary}else{recipe.unfinished},target_id:if reverse{recipe.unfinished}else{recipe.secondary},selected_slot:7,target_slot:19};
                let request=config.request(held,origin,41).unwrap();
                assert_eq!((request.unfinished,request.secondary),(recipe.unfinished,recipe.secondary));
                assert_eq!((request.unfinished_slot,request.secondary_slot),if reverse{(19,7)}else{(7,19)});
                assert_eq!(config.recipes[&(request.unfinished,request.secondary)].output,recipe.output);
                assert_eq!(request.origin,origin);assert_eq!(request.last_movement,41);
            }
        }
        for (a,b) in [(97,231),(99,1975),(97,97),(995,221)] {
            assert!(config.request(crate::held_use::Request{selected_id:a,target_id:b,selected_slot:0,target_slot:1},origin,0).is_none());
        }
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

// Shared world scheduler content adapter; no independent tick engine.
pub(crate) const QUEUE: i32 = -2391008;
#[derive(Clone, Debug)]
pub struct Config {
    pub goggles: [i32;2],
    pub recipes: std::collections::BTreeMap<(i32, i32), Recipe>,
    pub quest_progress: u16,
    pub quest_complete: i32,
    pub animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub unfinished: i32,
    pub secondary: i32,
    pub unfinished_slot: usize,
    pub secondary_slot: usize,
    pub origin: crate::CoordGrid,
    pub last_movement: i32,
}
impl Config {
    pub fn request(
        &self,
        held: crate::held_use::Request,
        origin: crate::CoordGrid,
        last_movement: i32,
    ) -> Option<Request> {
        if held.selected_slot < 0 || held.target_slot < 0 || held.selected_slot == held.target_slot
        {
            return None;
        }
        let (unfinished, unfinished_slot, secondary, secondary_slot) =
            if self.recipes.contains_key(&(held.selected_id, held.target_id)) {
                (
                    held.selected_id,
                    held.selected_slot,
                    held.target_id,
                    held.target_slot,
                )
            } else {
                (
                    held.target_id,
                    held.target_slot,
                    held.selected_id,
                    held.selected_slot,
                )
            };
        self.recipes.get(&(unfinished, secondary))?;
        Some(Request {
            unfinished,
            secondary,
            unfinished_slot: unfinished_slot as usize,
            secondary_slot: secondary_slot as usize,
            origin,
            last_movement,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub goggles: [i32;2],
    pub job: Job,
    pub request: Request,
    pub remaining: i32,
    pub queued_at: i32,
    pub index: u32,
    pub quest_progress: u16,
    pub quest_complete: i32,
    pub animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub secondary_saved: bool,
    pub job: Job,
    pub queued_at: i32,
    pub committed_at: i32,
    pub index: u32,
    pub fine_before: i32,
    pub fine_after: i32,
    pub rate_hundreds: i32,
}
impl Transition {
    pub fn json(self, actor: crate::simulation::ActorId) -> String {
        format!(
            "{{\"actor_slot\":{},\"actor_generation\":{},\"unfinished_slot\":{},\"secondary_slot\":{},\"unfinished\":{},\"secondary\":{},\"output\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{},\"index\":{},\"base_fine_xp\":{},\"rate_hundreds\":{},\"fine_before\":{},\"fine_after\":{},\"secondary_saved\":{}}}",
            actor.slot,
            actor.generation,
            self.job.unfinished_slot,
            self.job.secondary_slot,
            self.job.recipe.unfinished,
            self.job.recipe.secondary,
            self.job.recipe.output,
            self.queued_at,
            self.committed_at,
            self.index,
            self.job.recipe.base_fine_xp,
            self.rate_hundreds,
            self.fine_before,
            self.fine_after,
            self.secondary_saved
        )
    }
}
