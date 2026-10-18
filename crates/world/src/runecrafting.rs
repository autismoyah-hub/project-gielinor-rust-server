//! Ordinary altar batches stage existing inventory and XP transactions together.
use crate::{
    assembly,
    inventory::{Catalog, Delete, Insert, Inventory, StackType},
    stats::Stats,
    CoordGrid,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
pub(crate) const QUEUE: i32 = -2391050;
pub const RUNE_ESSENCE: i32 = 1436;
pub const PURE_ESSENCE: i32 = 7936;
pub const DAEYALT_ESSENCE: i32 = 24704;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub output: i32,
    pub level: u8,
    pub xp_tenths: u16,
    pub interval: u8,
    pub name: &'static str,
    pub members: bool,
    pub pure_only: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct OutfitBinding {
    pub group: u8,
    pub slot: usize,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub altars: BTreeMap<u32, Recipe>,
    pub blocked_held: BTreeSet<i32>,
    pub blocked_worn: BTreeSet<i32>,
    pub outfit: BTreeMap<i32, OutfitBinding>,
    pub animation: u16,
    pub spot: u16,
}
#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub location: crate::locations::Location,
    pub revision: u64,
    pub origin: CoordGrid,
    pub last_movement: i32,
    pub recipe: Recipe,
    pub materials: Arc<Vec<(i32, usize)>>,
}
pub struct Prepared {
    pub inventory: Inventory,
    pub stats: Stats,
    pub advancement: Option<crate::skills::Advancement>,
}
// Both complete transaction images are prepared before any random draw.
pub struct Plan {
    low: Prepared,
    high: Option<Prepared>,
    threshold: u32,
}
impl Plan {
    pub fn resolve(self, random: &mut impl crate::npc::RandomSource) -> Prepared {
        match self.high {
            Some(high) if random.below(10) < self.threshold => high,
            _ => self.low,
        }
    }
}
struct PreparedBase {
    debited: Inventory,
    stats: Stats,
    advancement: Option<crate::skills::Advancement>,
    output: i32,
    produced: i32,
}
impl PreparedBase {
    fn credit(&self, items: &Catalog, count: i32) -> Result<Prepared, Error> {
        let mut inventory = self.debited.clone();
        inventory
            .insert(items, Insert::new(self.output, count))
            .map_err(|_| Error::Capacity)?;
        Ok(Prepared {
            inventory,
            stats: self.stats.clone(),
            advancement: self.advancement,
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Policy,
    Members,
    Level,
    Empty,
    Unsupported,
    Stale,
    Capacity,
    Reward,
}
impl Config {
    pub fn capture(&self, inventory: &Inventory) -> Result<Arc<Vec<(i32, usize)>>, Error> {
        let daeyalt = inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| i.id == DAEYALT_ESSENCE);
        let parts = inventory
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| {
                item.filter(|i| {
                    if daeyalt {
                        i.id == DAEYALT_ESSENCE
                    } else {
                        [RUNE_ESSENCE, PURE_ESSENCE].contains(&i.id)
                    }
                })
                .map(|i| (i.id, slot))
            })
            .collect::<Vec<_>>();
        if parts.is_empty() {
            return Err(Error::Empty);
        }
        if parts.len() > 28 {
            return Err(Error::Policy);
        }
        Ok(Arc::new(parts))
    }
    /// Preserve ineligible Rune essence at pure-only altars; Daeyalt keeps priority.
    pub fn capture_for(
        &self,
        recipe: Recipe,
        inventory: &Inventory,
    ) -> Result<Arc<Vec<(i32, usize)>>, Error> {
        let captured = self.capture(inventory)?;
        if !recipe.pure_only {
            return Ok(captured);
        }
        let parts = captured
            .iter()
            .copied()
            .filter(|v| v.0 != RUNE_ESSENCE)
            .collect::<Vec<_>>();
        if parts.is_empty() {
            return Err(Error::Empty);
        }
        Ok(Arc::new(parts))
    }
    /// Legacy callers have no member authority for special essence.
    pub fn prepare(
        &self,
        recipe: Recipe,
        parts: &[(i32, usize)],
        inventory: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        rate: i32,
    ) -> Result<Prepared, Error> {
        self.prepare_member(recipe, parts, inventory, worn, items, stats, rate, false)
    }
    pub fn prepare_member(
        &self,
        recipe: Recipe,
        parts: &[(i32, usize)],
        inventory: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        rate: i32,
        member: bool,
    ) -> Result<Prepared, Error> {
        let (base, multiplier) =
            self.prepare_base(recipe, parts, inventory, worn, items, stats, rate, member)?;
        // Callers without RNG must not silently discard a worn outfit bonus.
        if multiplier != 0 {
            return Err(Error::Unsupported);
        }
        base.credit(items, base.produced)
    }
    pub fn prepare_outcomes(
        &self,
        recipe: Recipe,
        parts: &[(i32, usize)],
        inventory: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        rate: i32,
        member: bool,
    ) -> Result<Plan, Error> {
        let (base, multiplier) =
            self.prepare_base(recipe, parts, inventory, worn, items, stats, rate, member)?;
        // Wiki total runes per action: mixed Rune/Pure share this one rounding decision.
        let count = base.produced + base.produced / 10 * multiplier;
        let low = base.credit(items, count)?;
        let threshold = (base.produced % 10) as u32;
        let high = if multiplier > 0 && threshold > 0 {
            Some(base.credit(items, count + multiplier)?)
        } else {
            None
        };
        Ok(Plan {
            low,
            high,
            threshold,
        })
    }
    pub fn resolve(
        &self,
        recipe: Recipe,
        parts: &[(i32, usize)],
        inventory: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        rate: i32,
        member: bool,
        random: &mut impl crate::npc::RandomSource,
    ) -> Result<Prepared, Error> {
        Ok(self
            .prepare_outcomes(recipe, parts, inventory, worn, items, stats, rate, member)?
            .resolve(random))
    }
    fn outfit_multiplier(
        &self,
        worn: &Inventory,
        items: &Catalog,
        member: bool,
    ) -> Result<i32, Error> {
        let mut groups = 0u8;
        for (slot, item) in worn
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(s, i)| i.map(|i| (s, i)))
        {
            let Some(binding) = self.outfit.get(&item.id) else {
                continue;
            };
            if !member {
                return Err(Error::Members);
            }
            if binding.group >= 4
                || binding.slot >= 15
                || binding.slot != slot
                || item.count != 1
                || item.vars != 0
                || !items.ordinary_nonstackable(item.id)
                || groups & (1 << binding.group) != 0
            {
                return Err(Error::Policy);
            }
            groups |= 1 << binding.group;
        }
        Ok(match groups.count_ones() {
            4 => 6,
            n => n as i32,
        })
    }
    fn prepare_base(
        &self,
        recipe: Recipe,
        parts: &[(i32, usize)],
        inventory: &Inventory,
        worn: &Inventory,
        items: &Catalog,
        stats: &Stats,
        rate: i32,
        member: bool,
    ) -> Result<(PreparedBase, i32), Error> {
        if !self.altars.values().any(|&v| v == recipe)
            || recipe.level == 0
            || recipe.level > 99
            || recipe.interval == 0
            || recipe.xp_tenths == 0
            || inventory.stack != StackType::Normal
            || inventory.placeholders
            || parts.is_empty()
            || parts.len() > 28
            || parts.iter().map(|v| v.1).collect::<BTreeSet<_>>().len() != parts.len()
            || parts.iter().any(|&(id, _)| {
                ![RUNE_ESSENCE, PURE_ESSENCE, DAEYALT_ESSENCE].contains(&id)
                    || !items.ordinary_nonstackable(id)
            })
            || !items.stackable.contains(&recipe.output)
            || items.dummy.contains(&recipe.output)
            || items
                .notes
                .get(&recipe.output)
                .is_some_and(|t| t.template != 0)
            || items
                .placeholders
                .get(&recipe.output)
                .is_some_and(|t| t.template != 0)
            || items
                .transforms
                .get(&recipe.output)
                .is_some_and(|t| t.template != 0)
        {
            return Err(Error::Policy);
        }
        if recipe.members && !member {
            return Err(Error::Members);
        }
        if recipe.pure_only && parts.iter().any(|v| v.0 == RUNE_ESSENCE) {
            return Err(Error::Policy);
        }
        let mut rune_stacks = 0;
        for item in inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == recipe.output)
        {
            rune_stacks += 1;
            if rune_stacks > 1 || item.count <= 0 || item.vars != 0 {
                return Err(Error::Policy);
            }
        }
        let daeyalt = parts[0].0 == DAEYALT_ESSENCE;
        if parts
            .iter()
            .any(|&(id, _)| (id == DAEYALT_ESSENCE) != daeyalt)
        {
            return Err(Error::Policy);
        }
        if daeyalt && !member {
            return Err(Error::Members);
        }
        if inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| self.blocked_held.contains(&i.id))
            || worn
                .slots()
                .iter()
                .flatten()
                .any(|i| self.blocked_worn.contains(&i.id))
        {
            return Err(Error::Unsupported);
        }
        let multiplier = self.outfit_multiplier(worn, items, member)?;
        let level = stats.current(20) as u8;
        if level < recipe.level {
            return Err(Error::Level);
        }
        if level > 104 {
            return Err(Error::Policy);
        }
        if parts.iter().any(|&(id, slot)| {
            !inventory
                .get(slot)
                .is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0)
        }) {
            return Err(Error::Stale);
        }
        // Yield uses the visible level before this action's XP; extra runes give no extra XP.
        let produced = (parts.len() as i32)
            .checked_mul(i32::from(level / recipe.interval) + 1)
            .ok_or(Error::Policy)?;
        let mut next = inventory.clone();
        for &(id, slot) in parts {
            let mut debit = Delete::new(id, 1);
            debit.strict_slot = Some(slot);
            next.delete(items, debit).map_err(|_| Error::Stale)?;
        }
        let xp = parts.len() as f64 * f64::from(recipe.xp_tenths) / 10.0
            * if daeyalt { 1.5 } else { 1.0 };
        let (stats, advancement) =
            assembly::prepare_reward_value(stats, 20, xp, rate).map_err(|_| Error::Reward)?;
        Ok((
            PreparedBase {
                debited: next,
                stats,
                advancement,
                output: recipe.output,
                produced,
            },
            multiplier,
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    fn fixture() -> (Config, Recipe, Inventory, Inventory, Catalog, Stats) {
        let recipe = Recipe {
            output: 556,
            level: 1,
            xp_tenths: 50,
            interval: 11,
            name: "Air rune",
            members: false,
            pure_only: false,
        };
        let c = Config {
            altars: [(34760, recipe)].into(),
            blocked_held: [26879].into(),
            blocked_worn: [26850].into(),
            outfit: BTreeMap::new(),
            animation: 791,
            spot: 186,
        };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: if slot % 2 == 0 { 1436 } else { 7936 },
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let worn = Inventory::new(15, StackType::Normal, false);
        let mut items = Catalog::default();
        items.stackable.insert(556);
        let mut stats = Stats::default();
        stats.set_base(20, 10);
        stats.set_current(20, 11);
        stats
            .set_xp(20, crate::skills::xp_from_level(10).unwrap())
            .unwrap();
        (c, recipe, inv, worn, items, stats)
    }
    #[test]
    fn ordinary_altar_extra_runes_do_not_multiply_xp_and_boosted104_earth_is_not_clamped99() {
        let (mut c, r, inv, worn, mut items, mut stats) = fixture();
        let parts = c.capture(&inv).unwrap();
        let p = c
            .prepare(r, &parts, &inv, &worn, &items, &stats, 125)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 56);
        assert_eq!(p.inventory.free_space(), 27);
        assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 1750);
        let earth = Recipe {
            output: 557,
            level: 9,
            xp_tenths: 65,
            interval: 26,
            name: "Earth rune",
            members: false,
            pure_only: false,
        };
        c.altars.insert(34763, earth);
        items.stackable.insert(557);
        stats.set_base(20, 99);
        stats.set_current(20, 104);
        stats
            .set_xp(20, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        let p = c
            .prepare(earth, &parts, &inv, &worn, &items, &stats, 100)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 140);
        assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 1820);
        stats.set_base(20, 10);
        stats.set_current(20, 10);
        stats
            .set_xp(20, crate::skills::xp_from_level(11).unwrap() - 1)
            .unwrap();
        let p = c
            .prepare(r, &parts, &inv, &worn, &items, &stats, 100)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 28);
        assert!(p.stats.base(20) >= 11);
    }
    #[test]
    fn ordinary_altar_all_slot_credit_reward_and_unsupported_failures_leave_both_states_unchanged()
    {
        for case in 0..10 {
            let (c, r, mut inv, mut worn, mut items, mut stats) = fixture();
            let mut parts = c.capture(&inv).unwrap().as_ref().clone();
            match case {
                0 => parts[1].1 = 0,
                1 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 7936,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap();
                }
                2 => {
                    items.dummy.insert(556);
                }
                3 => {
                    stats.set_current(20, 0);
                }
                4 => {
                    inv.restore_slot(1, None).unwrap();
                }
                5 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 26879,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                6 => {
                    worn.restore_slot(
                        0,
                        Some(Item {
                            id: 26850,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                7 => {
                    items.stackable.insert(1436);
                }
                9 => {
                    stats.set_current(20, 105);
                }
                _ => {}
            }
            let before = (inv.clone(), worn.clone(), stats.clone());
            assert!(
                c.prepare(
                    r,
                    &parts,
                    &inv,
                    &worn,
                    &items,
                    &stats,
                    if case == 8 { 0 } else { 100 }
                )
                .is_err(),
                "case{case}"
            );
            assert_eq!((inv, worn, stats), before);
        }
    }
    #[test]
    fn daeyalt_priority_member_authority_aggregate_half_bonus_and_full_stack_credit_are_atomic() {
        let (mut c, _, mut inv, worn, mut items, mut stats) = fixture();
        let r = Recipe {
            output: 558,
            level: 2,
            xp_tenths: 55,
            interval: 14,
            name: "Mind rune",
            members: false,
            pure_only: false,
        };
        c.altars.insert(34761, r);
        items.stackable.insert(558);
        stats.set_base(20, 13);
        stats.set_current(20, 14);
        stats
            .set_xp(20, crate::skills::xp_from_level(13).unwrap())
            .unwrap();
        for slot in 0..7 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: DAEYALT_ESSENCE,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let parts = c.capture(&inv).unwrap();
        assert_eq!(parts.len(), 7);
        assert!(parts.iter().all(|v| v.0 == DAEYALT_ESSENCE));
        let before = (inv.clone(), stats.clone());
        assert!(matches!(
            c.prepare(r, &parts, &inv, &worn, &items, &stats, 125),
            Err(Error::Members)
        ));
        assert!(matches!(
            c.prepare_member(r, &parts, &inv, &worn, &items, &stats, 125, false),
            Err(Error::Members)
        ));
        let p = c
            .prepare_member(r, &parts, &inv, &worn, &items, &stats, 125, true)
            .unwrap();
        assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 721);
        assert_eq!(p.inventory.get(0).unwrap().count, 14);
        assert_eq!(
            p.inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| [1436, 7936].contains(&i.id))
                .count(),
            21
        );
        assert_eq!((inv.clone(), stats.clone()), before);
        let mut mixed = parts.as_ref().clone();
        mixed.push((7936, 7));
        assert!(matches!(
            c.prepare_member(r, &mixed, &inv, &worn, &items, &stats, 125, true),
            Err(Error::Policy)
        ));
        inv.restore_slot(
            27,
            Some(Item {
                id: 558,
                count: i32::MAX,
                vars: 0,
            }),
        )
        .unwrap();
        let before = (inv.clone(), stats.clone());
        assert!(matches!(
            c.prepare_member(r, &parts, &inv, &worn, &items, &stats, 125, true),
            Err(Error::Capacity)
        ));
        assert_eq!((inv, stats), before);
    }
    #[test]
    fn existing_rune_stacks_preserve_metadata_or_refuse_and_valid_credit_keeps_prior_count() {
        for case in 0..4 {
            let (c, r, mut inv, worn, items, stats) = fixture();
            inv.restore_slot(
                27,
                Some(Item {
                    id: 556,
                    count: if case == 1 { 0 } else { 7 },
                    vars: if case == 0 { 9 } else { 0 },
                }),
            )
            .unwrap();
            if case == 2 {
                inv.restore_slot(
                    26,
                    Some(Item {
                        id: 556,
                        count: 11,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let parts = c.capture(&inv).unwrap();
            let before = (inv.clone(), stats.clone());
            let result = c.prepare(r, &parts, &inv, &worn, &items, &stats, 125);
            if case == 3 {
                let p = result.unwrap();
                assert_eq!(
                    p.inventory.get(27).unwrap(),
                    Item {
                        id: 556,
                        count: 61,
                        vars: 0
                    }
                );
                assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 1687);
            } else {
                assert!(matches!(result, Err(Error::Policy)), "case{case}");
            }
            assert_eq!((inv, stats), before);
        }
    }
    fn raiments(c: &mut Config, worn: &mut Inventory, pieces: usize) {
        c.blocked_worn.clear();
        for (group, (id, slot)) in [(26858, 0), (26866, 4), (26874, 7), (26856, 10)]
            .into_iter()
            .enumerate()
        {
            c.outfit.insert(
                id,
                OutfitBinding {
                    group: group as u8,
                    slot,
                },
            );
            if group < pieces {
                worn.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
        }
    }
    struct Draw {
        value: u32,
        calls: usize,
    }
    impl crate::npc::RandomSource for Draw {
        fn below(&mut self, bound: u32) -> u32 {
            assert_eq!(bound, 10);
            self.calls += 1;
            self.value
        }
    }
    struct NoDraw;
    impl crate::npc::RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Invalid or deterministic rune batch must not draw");
        }
    }
    #[test]
    fn rune_raiments_whole_mixed_batch_single_roll_all_piece_counts_exact_tens_and_daeyalt_xp() {
        for pieces in 0..=4 {
            for value in [0, 5, 6, 9] {
                let (mut c, r, inv, mut worn, items, stats) = fixture();
                raiments(&mut c, &mut worn, pieces);
                let before = (inv.clone(), worn.clone(), stats.clone());
                let mut rng = Draw { value, calls: 0 };
                let parts = c.capture(&inv).unwrap();
                let p = c
                    .resolve(r, &parts, &inv, &worn, &items, &stats, 125, true, &mut rng)
                    .unwrap();
                let k = if pieces == 4 { 6 } else { pieces as i32 };
                assert_eq!(
                    p.inventory.get(0).unwrap().count,
                    56 + 5 * k + if value < 6 { k } else { 0 }
                );
                assert_eq!(rng.calls, usize::from(pieces > 0));
                assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 1750);
                assert_eq!((inv.clone(), worn.clone(), stats.clone()), before);
                if pieces > 0 {
                    assert!(matches!(
                        c.prepare_member(r, &parts, &inv, &worn, &items, &stats, 125, true),
                        Err(Error::Unsupported)
                    ));
                }
            }
        }
        let (mut c, r, mut inv, mut worn, items, mut stats) = fixture();
        raiments(&mut c, &mut worn, 4);
        stats.set_base(20, 99);
        stats.set_current(20, 99);
        stats
            .set_xp(20, crate::skills::xp_from_level(99).unwrap())
            .unwrap();
        let parts = c.capture(&inv).unwrap();
        let p = c
            .resolve(
                r,
                &parts,
                &inv,
                &worn,
                &items,
                &stats,
                100,
                true,
                &mut NoDraw,
            )
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 448);
        stats.set_base(20, 10);
        stats.set_current(20, 11);
        stats
            .set_xp(20, crate::skills::xp_from_level(10).unwrap())
            .unwrap();
        for slot in 0..7 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: DAEYALT_ESSENCE,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let parts = c.capture(&inv).unwrap();
        let mut rng = Draw { value: 0, calls: 0 };
        let p = c
            .resolve(r, &parts, &inv, &worn, &items, &stats, 125, true, &mut rng)
            .unwrap();
        assert_eq!(p.inventory.get(0).unwrap().count, 26);
        assert_eq!(p.stats.fine_xp(20) - stats.fine_xp(20), 656);
        assert_eq!(
            p.inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| [RUNE_ESSENCE, PURE_ESSENCE].contains(&i.id))
                .count(),
            21
        );
        assert_eq!(rng.calls, 1);
    }
    #[test]
    fn rune_raiments_all_credit_outcomes_and_live_gear_validate_before_rng_without_mutation() {
        for case in 0..9 {
            let (mut c, r, mut inv, mut worn, mut items, stats) = fixture();
            raiments(&mut c, &mut worn, 4);
            let mut member = true;
            let mut rate = 125;
            match case {
                0 => member = false,
                1 => {
                    let mut i = worn.get(0).unwrap();
                    i.vars = 9;
                    worn.restore_slot(0, Some(i)).unwrap();
                }
                2 => {
                    let i = worn.get(0);
                    worn.restore_slot(0, None).unwrap();
                    worn.restore_slot(1, i).unwrap();
                }
                3 => {
                    let mut i = worn.get(10).unwrap();
                    i.count = 2;
                    worn.restore_slot(10, Some(i)).unwrap();
                }
                4 => {
                    items.stackable.insert(26858);
                }
                5 => rate = 0,
                6 => {
                    // Low credit fits exactly, possible high credit cannot: refuse before drawing.
                    inv.restore_slot(
                        27,
                        Some(Item {
                            id: 556,
                            count: i32::MAX - 84,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                7 => {
                    // Malformed config cannot shift by an out-of-range bit count.
                    c.outfit.get_mut(&26858).unwrap().group = 8;
                }
                8 => {
                    c.outfit.insert(26850, OutfitBinding { group: 0, slot: 1 });
                    worn.restore_slot(
                        1,
                        Some(Item {
                            id: 26850,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                _ => unreachable!(),
            }
            let before = (inv.clone(), worn.clone(), stats.clone());
            let parts = c.capture(&inv).unwrap();
            assert!(
                c.resolve(
                    r,
                    &parts,
                    &inv,
                    &worn,
                    &items,
                    &stats,
                    rate,
                    member,
                    &mut NoDraw
                )
                .is_err(),
                "case{case}"
            );
            assert_eq!((inv, worn, stats), before);
        }
    }
    #[test]
    fn rune_member_recipes_filter_rune_essence_and_recheck_membership_without_fallback() {
        for (output, level, xp, interval) in [(562, 35, 85, 74), (561, 44, 90, 91)] {
            let (mut c, _, mut inv, worn, mut items, mut stats) = fixture();
            let recipe = Recipe {
                output,
                level,
                xp_tenths: xp,
                interval,
                name: "member rune",
                members: true,
                pure_only: true,
            };
            c.altars.insert(99, recipe);
            items.stackable.insert(output);
            stats.set_base(20, interval as i8 - 1);
            stats.set_current(20, interval as i8);
            stats
                .set_xp(20, crate::skills::xp_from_level(interval - 1).unwrap())
                .unwrap();
            let parts = c.capture_for(recipe, &inv).unwrap();
            assert_eq!(parts.len(), 14);
            assert!(parts.iter().all(|v| v.0 == PURE_ESSENCE));
            let before = (inv.clone(), stats.clone());
            assert!(matches!(
                c.resolve(
                    recipe,
                    &parts,
                    &inv,
                    &worn,
                    &items,
                    &stats,
                    125,
                    false,
                    &mut NoDraw
                ),
                Err(Error::Members)
            ));
            let forced = c.capture(&inv).unwrap();
            assert!(matches!(
                c.resolve(
                    recipe,
                    &forced,
                    &inv,
                    &worn,
                    &items,
                    &stats,
                    125,
                    true,
                    &mut NoDraw
                ),
                Err(Error::Policy)
            ));
            let p = c
                .resolve(
                    recipe,
                    &parts,
                    &inv,
                    &worn,
                    &items,
                    &stats,
                    125,
                    true,
                    &mut NoDraw,
                )
                .unwrap();
            assert_eq!(
                p.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .find(|i| i.id == output)
                    .unwrap()
                    .count,
                28
            );
            assert_eq!(
                p.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == RUNE_ESSENCE)
                    .count(),
                14
            );
            assert_eq!(
                p.stats.fine_xp(20) - stats.fine_xp(20),
                14 * i32::from(xp) * 125 / 100
            );
            assert_eq!((inv.clone(), stats.clone()), before);
            for slot in 0..28 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: RUNE_ESSENCE,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            assert!(matches!(c.capture_for(recipe, &inv), Err(Error::Empty)));
        }
    }
}
