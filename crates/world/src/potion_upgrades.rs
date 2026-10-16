//! Dose-scaled potion upgrades. Runtime registration is intentionally separate.
//! Wiki Stamina potion15281677 and Prescription goggles15240969.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    skills,
    stats::Stats,
};

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub base: i32,
    pub secondary: i32,
    pub output: i32,
    pub doses: u8,
    pub secondary_per_dose: u8,
    pub fine_xp_per_dose: u16,
    pub level: u8,
    pub ticks: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub recipe: Recipe,
    pub base_slot: usize,
    pub secondary_slot: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Made(skills::Advancement),
    InvalidRecipe,
    Members,
    Quest,
    Level,
    Stale,
    Capacity,
    InvalidReward,
}
impl Job {
    /// Prepare both normal and goggles outcomes before drawing. One proc saves
    /// the entire selected secondary requirement; XP depends on input doses.
    pub fn complete(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        member: bool,
        quest: bool,
        rate: i32,
        goggles: bool,
        random: &mut impl crate::npc::RandomSource,
    ) -> (Outcome, bool) {
        let r = self.recipe;
        let fail = |outcome| (outcome, false);
        let ordinary = |id| {
            id >= 0
                && !catalog.dummy.contains(&id)
                && !catalog.notes.get(&id).is_some_and(|v| v.template != 0)
                && !catalog
                    .placeholders
                    .get(&id)
                    .is_some_and(|v| v.template != 0)
                && !catalog.transforms.get(&id).is_some_and(|v| v.template != 0)
        };
        if !matches!(r.ticks,2|3) || !(1..=4).contains(&r.doses)
            || r.secondary_per_dose == 0
            || r.fine_xp_per_dose == 0
            || !(1..=99).contains(&r.level)
            || r.base == r.secondary
            || r.base == r.output
            || r.secondary == r.output
            || ![r.base, r.secondary, r.output].into_iter().all(ordinary)
            || catalog.stackable.contains(&r.base)
            || catalog.stackable.contains(&r.output)
            || !catalog.stackable.contains(&r.secondary)
        {
            return fail(Outcome::InvalidRecipe);
        }
        if !member {
            return fail(Outcome::Members);
        }
        if !quest {
            return fail(Outcome::Quest);
        }
        if stats.current(15) < r.level as i8 {
            return fail(Outcome::Level);
        }
        if rate <= 0 {
            return fail(Outcome::InvalidReward);
        }
        let count = i32::from(r.doses) * i32::from(r.secondary_per_dose);
        if self.base_slot == self.secondary_slot
            || inventory
                .get(self.base_slot)
                .is_none_or(|i| i.id != r.base || i.count != 1 || i.vars != 0)
            || inventory
                .get(self.secondary_slot)
                .is_none_or(|i| i.id != r.secondary || i.count < count || i.vars != 0)
        {
            return fail(Outcome::Stale);
        }
        let candidate = |saved: bool| {
            let mut next = inventory.clone();
            let mut debit = Delete::new(r.base, 1);
            debit.strict_slot = Some(self.base_slot);
            if next.delete(catalog, debit).ok()?.completed != 1 {
                return None;
            }
            if !saved {
                let mut debit = Delete::new(r.secondary, count);
                debit.strict_slot = Some(self.secondary_slot);
                if next.delete(catalog, debit).ok()?.completed != count {
                    return None;
                }
            }
            if next
                .insert(catalog, Insert::new(r.output, 1))
                .ok()?
                .completed
                != 1
            {
                return None;
            }
            Some(next)
        };
        let Some(normal) = candidate(false) else {
            return fail(Outcome::Capacity);
        };
        let Some(saved) = (if goggles {
            candidate(true)
        } else {
            Some(normal.clone())
        }) else {
            return fail(Outcome::Capacity);
        };
        let mut next_stats = stats.clone();
        let advancement = match skills::advance(
            &mut next_stats,
            15,
            f64::from(r.fine_xp_per_dose) * f64::from(r.doses) / 10.0,
            f64::from(rate) / 100.0,
            99,
        ) {
            Ok(a) => a,
            Err(_) => return fail(Outcome::InvalidReward),
        };
        let save = goggles && random.below(10) == 0;
        *inventory = if save { saved } else { normal };
        *stats = next_stats;
        (Outcome::Made(advancement), save)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modified {
    pub outcome: Outcome,
    pub saved: bool,
    pub chemistry: crate::chemistry::Proc,
    pub output: i32,
}
impl Job {
    /// Extra output is an attested next-dose item. Input doses still determine
    /// secondary cost and XP; four-dose upgrades never consume an amulet charge.
    pub fn complete_with_modifiers(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        stats: &mut Stats,
        equipment: &mut Inventory,
        state: &mut crate::chemistry::State,
        member: bool,
        quest: bool,
        rate: i32,
        goggles: bool,
        binding: Option<crate::chemistry_modifier::Binding>,
        random: &mut impl crate::npc::RandomSource,
    ) -> Modified {
        let none = crate::chemistry::Proc {
            extra_dose: false,
            crumbled: false,
            stop: false,
        };
        let refused = |outcome| Modified {
            outcome,
            saved: false,
            chemistry: none,
            output: self.recipe.output,
        };
        // This deterministic preparation source never touches the gameplay RNG.
        struct Save;
        impl crate::npc::RandomSource for Save {
            fn below(&mut self, n: u32) -> u32 {
                debug_assert_eq!(n, 10);
                0
            }
        }
        let prepare = |output, saved| {
            let mut job = *self;
            job.recipe.output = output;
            let mut inv = inventory.clone();
            let mut stats = stats.clone();
            let (outcome, _) = job.complete(
                &mut inv, catalog, &mut stats, member, quest, rate, saved, &mut Save,
            );
            match outcome {
                Outcome::Made(_) => Ok((inv, stats, outcome)),
                other => Err(other),
            }
        };
        let normal = match prepare(self.recipe.output, false) {
            Ok(v) => v,
            Err(e) => return refused(e),
        };
        let retained = if goggles {
            match prepare(self.recipe.output, true) {
                Ok(v) => v,
                Err(e) => return refused(e),
            }
        } else {
            normal.clone()
        };
        let binding = binding.filter(|b| {
            self.recipe.doses < 4
                && equipment
                    .get(2)
                    .is_some_and(|i| i.id == b.amulet && i.count == 1 && i.vars == 0)
        });
        let mut next_equipment = equipment.clone();
        let extra = if let Some(b) = binding {
            if b.amulet < 0
                || [
                    self.recipe.base,
                    self.recipe.secondary,
                    self.recipe.output,
                    b.extra_output,
                ]
                .contains(&b.amulet)
                || b.extra_output == self.recipe.output
                || catalog.stackable.contains(&b.amulet)
                || catalog.dummy.contains(&b.amulet)
                || catalog
                    .notes
                    .get(&b.amulet)
                    .is_some_and(|v| v.template != 0)
                || catalog
                    .placeholders
                    .get(&b.amulet)
                    .is_some_and(|v| v.template != 0)
                || catalog
                    .transforms
                    .get(&b.amulet)
                    .is_some_and(|v| v.template != 0)
            {
                return refused(Outcome::InvalidRecipe);
            }
            let ordinary = match prepare(b.extra_output, false) {
                Ok(v) => v,
                Err(e) => return refused(e),
            };
            let saved = if goggles {
                match prepare(b.extra_output, true) {
                    Ok(v) => v,
                    Err(e) => return refused(e),
                }
            } else {
                ordinary.clone()
            };
            let mut debit = Delete::new(b.amulet, 1);
            debit.strict_slot = Some(2);
            if next_equipment.delete(catalog, debit).is_err() {
                return refused(Outcome::Stale);
            }
            Some((ordinary, saved, b.extra_output))
        } else {
            None
        };
        let saved = goggles && random.below(10) == 0;
        let mut next_state = *state;
        let proc = next_state.roll(binding.is_some(), random);
        let (selected, output) = if proc.extra_dose {
            let (normal, retained, output) = extra.expect("prepared extra dose");
            (if saved { retained } else { normal }, output)
        } else {
            (if saved { retained } else { normal }, self.recipe.output)
        };
        let (next_inventory, next_stats, outcome) = selected;
        *inventory = next_inventory;
        *stats = next_stats;
        *state = next_state;
        if proc.crumbled {
            *equipment = next_equipment;
        }
        Modified {
            outcome,
            saved,
            chemistry: proc,
            output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    struct Draw {
        value: u32,
        calls: usize,
    }
    impl crate::npc::RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 10);
            self.calls += 1;
            self.value
        }
    }
    fn fixture(doses: u8, reverse: bool) -> (Job, Inventory, Catalog, Stats) {
        let job = Job {
            recipe: Recipe {
                base: 100,
                secondary: 101,
                output: 102,
                doses,
                secondary_per_dose: 1,
                fine_xp_per_dose: 255,
                level: 77,
                ticks: 2,
            },
            base_slot: if reverse { 1 } else { 0 },
            secondary_slot: if reverse { 0 } else { 1 },
        };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: if slot == job.base_slot {
                        100
                    } else if slot == job.secondary_slot {
                        101
                    } else {
                        1931
                    },
                    count: if slot == job.secondary_slot { 9 } else { 1 },
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut catalog = Catalog::default();
        catalog.stackable.insert(101);
        let mut stats = Stats::default();
        stats.set_base(15, 76);
        stats.set_current(15, 77);
        stats
            .set_fine_xp(15, skills::xp_from_level(76).unwrap() * 10 + 1)
            .unwrap();
        (job, inv, catalog, stats)
    }
    #[test]
    fn all_doses_fractional_xp_full_bag_boosted_and_one_goggles_draw() {
        for doses in 1..=4 {
            for reverse in [false, true] {
                for goggles in [false, true] {
                    for proc in [false, true] {
                        let (job, mut inv, catalog, mut stats) = fixture(doses, reverse);
                        let before = stats.fine_xp(15);
                        let pots = inv.slots()[2..].to_vec();
                        let mut rng = Draw {
                            value: if proc { 0 } else { 1 },
                            calls: 0,
                        };
                        let (outcome, saved) = job.complete(
                            &mut inv, &catalog, &mut stats, true, true, 500, goggles, &mut rng,
                        );
                        assert!(matches!(outcome, Outcome::Made(_)));
                        assert_eq!(saved, goggles && proc);
                        assert_eq!(rng.calls, usize::from(goggles));
                        assert_eq!(stats.fine_xp(15) - before, 1275 * i32::from(doses));
                        assert_eq!(inv.slots()[2..], pots);
                        assert_eq!(
                            inv.get(job.secondary_slot).unwrap().count,
                            9 - if saved { 0 } else { i32::from(doses) }
                        );
                        assert_eq!(
                            inv.slots().iter().flatten().filter(|i| i.id == 102).count(),
                            1
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn refused_upgrade_preserves_inventory_stats_and_random_stream() {
        for mode in 0..10 {
            let (mut job, mut inv, mut catalog, mut stats) = fixture(4, false);
            match mode {
                3 => stats.set_current(15, 76),
                4 => {
                    inv.restore_slot(
                        job.secondary_slot,
                        Some(Item {
                            id: 101,
                            count: 3,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                5 => {
                    catalog.dummy.insert(102);
                }
                6 => job.recipe.doses = 0,
                7 => job.secondary_slot = job.base_slot,
                8 => {
                    catalog.stackable.remove(&101);
                }
                9 => job.recipe.output = 101,
                _ => (),
            }
            let before = (inv.clone(), stats.clone());
            let mut rng = Draw { value: 0, calls: 0 };
            let (outcome, _) = job.complete(
                &mut inv,
                &catalog,
                &mut stats,
                mode != 0,
                mode != 1,
                if mode == 2 { 0 } else { 500 },
                true,
                &mut rng,
            );
            assert!(!matches!(outcome, Outcome::Made(_)), "mode{mode}");
            assert_eq!((inv, stats), before);
            assert_eq!(rng.calls, 0);
        }
    }
    struct Sequence(std::collections::VecDeque<(u32, u32)>);
    impl crate::npc::RandomSource for Sequence {
        fn below(&mut self, n: u32) -> u32 {
            let (bound, value) = self.0.pop_front().expect("unexpected draw");
            assert_eq!(n, bound);
            value
        }
    }
    #[test]
    fn chemistry_adds_one_dose_without_extra_debit_or_xp_and_four_doses_are_ineligible() {
        for doses in 1..=4 {
            for saved in [false, true] {
                for extra in [false, true] {
                    let (job, mut inv, catalog, mut stats) = fixture(doses, true);
                    let fine = stats.fine_xp(15);
                    let mut gear = Inventory::new(14, StackType::Normal, false);
                    gear.restore_slot(
                        2,
                        Some(Item {
                            id: 21163,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                    let mut state = crate::chemistry::State::restore(1, 1).unwrap();
                    let mut draws =
                        std::collections::VecDeque::from([(10, if saved { 0 } else { 1 })]);
                    if doses < 4 {
                        draws.push_back((100, if extra { 4 } else { 5 }));
                    }
                    let mut rng = Sequence(draws);
                    let result = job.complete_with_modifiers(
                        &mut inv,
                        &catalog,
                        &mut stats,
                        &mut gear,
                        &mut state,
                        true,
                        true,
                        500,
                        true,
                        Some(crate::chemistry_modifier::Binding {
                            amulet: 21163,
                            extra_output: 103,
                        }),
                        &mut rng,
                    );
                    let procced = doses < 4 && extra;
                    assert!(matches!(result.outcome, Outcome::Made(_)));
                    assert_eq!(result.saved, saved);
                    assert_eq!(result.chemistry.extra_dose, procced);
                    assert_eq!(result.chemistry.stop, procced);
                    assert_eq!(result.output, if procced { 103 } else { 102 });
                    assert_eq!(gear.get(2).is_none(), procced);
                    assert_eq!(state.remaining(), if procced { 5 } else { 1 });
                    assert_eq!(stats.fine_xp(15) - fine, 1275 * i32::from(doses));
                    assert_eq!(
                        inv.get(job.secondary_slot).unwrap().count,
                        9 - if saved { 0 } else { i32::from(doses) }
                    );
                    assert!(rng.0.is_empty());
                }
            }
        }
    }
    #[test]
    fn invalid_extra_output_cannot_consume_randomness_or_partial_state() {
        let (job, mut inv, mut catalog, mut stats) = fixture(3, false);
        catalog.dummy.insert(103);
        let mut gear = Inventory::new(14, StackType::Normal, false);
        gear.restore_slot(
            2,
            Some(Item {
                id: 21163,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut state = crate::chemistry::State::restore(1, 1).unwrap();
        let before = (inv.clone(), stats.clone(), gear.clone(), state);
        let mut rng = Sequence(Default::default());
        let result = job.complete_with_modifiers(
            &mut inv,
            &catalog,
            &mut stats,
            &mut gear,
            &mut state,
            true,
            true,
            500,
            true,
            Some(crate::chemistry_modifier::Binding {
                amulet: 21163,
                extra_output: 103,
            }),
            &mut rng,
        );
        assert_eq!(result.outcome, Outcome::InvalidRecipe);
        assert_eq!((inv, stats, gear, state), before);
    }
}

// Shared world scheduler content adapter; no independent tick engine.
pub(crate) const QUEUE: i32 = -2391010;
#[derive(Clone, Debug)]
pub struct Config {
    pub chemistry: std::collections::BTreeMap<i32, crate::chemistry_modifier::Binding>,
    pub goggles: [i32;2],
    pub recipes: std::collections::BTreeMap<(i32, i32), Recipe>,
    pub quest_progress: u16,
    pub quest_complete: i32,
    pub animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub base: i32,
    pub secondary: i32,
    pub base_slot: usize,
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
        let (base, base_slot, secondary, secondary_slot) =
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
        self.recipes.get(&(base, secondary))?;
        Some(Request {
            base,
            secondary,
            base_slot: base_slot as usize,
            secondary_slot: secondary_slot as usize,
            origin,
            last_movement,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub chemistry: Option<crate::chemistry_modifier::Binding>,
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
    pub output: i32,
    pub chemistry: crate::chemistry::Proc,
    pub charges_remaining: u8,
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
            "{{\"actor_slot\":{},\"actor_generation\":{},\"base_slot\":{},\"secondary_slot\":{},\"base\":{},\"secondary\":{},\"output\":{},\"count\":1,\"queued_at\":{},\"committed_at\":{},\"index\":{},\"base_fine_xp\":{},\"rate_hundreds\":{},\"fine_before\":{},\"fine_after\":{},\"secondary_saved\":{},\"extra_dose\":{},\"amulet_crumbled\":{},\"stop_on_crumble\":{},\"charges_remaining\":{}}}",
            actor.slot,
            actor.generation,
            self.job.base_slot,
            self.job.secondary_slot,
            self.job.recipe.base,
            self.job.recipe.secondary,
            self.output,
            self.queued_at,
            self.committed_at,
            self.index,
            u32::from(self.job.recipe.fine_xp_per_dose) * u32::from(self.job.recipe.doses),
            self.rate_hundreds,
            self.fine_before,
            self.fine_after,
            self.secondary_saved, self.chemistry.extra_dose, self.chemistry.crumbled, self.chemistry.stop, self.charges_remaining
        )
    }
}
