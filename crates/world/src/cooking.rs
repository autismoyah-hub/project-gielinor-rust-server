//! Manual cooking on configured surfaces over existing inventory, skill and random systems.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    npc::RandomSource,
    stats::Stats,
};
use std::collections::BTreeMap;
pub(crate) const QUEUE: i32 = -2391012;
#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub raw: i32,
    pub cooked: i32,
    pub burnt: i32,
    pub level: u8,
    pub stop_burn: u8,
    pub low: i32,
    pub high: i32,
    pub xp: u16,
    pub ticks: u8,
}
#[derive(Clone, Debug)]
pub struct Surface {
    pub animation: u16,
    pub recipes: BTreeMap<i32, Recipe>,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub surfaces: BTreeMap<u32, Surface>,
    pub fire: u32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub recipes: BTreeMap<i32, Recipe>,
    pub tutorial: Option<crate::tutorial_progression::Config>,
}
impl Config {
    pub fn recipe(&self, source: u32, input: i32) -> Option<&Recipe> {
        if source == self.fire {
            self.recipes.get(&input)
        } else {
            self.surfaces.get(&source)?.recipes.get(&input)
        }
    }
    pub fn animation_for(&self, source: u32) -> Option<u16> {
        if source == self.fire {
            Some(self.animation)
        } else {
            self.surfaces.get(&source).map(|v| v.animation)
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub target: crate::locations::Location,
    pub revision: u64,
    pub origin: crate::CoordGrid,
    pub last_movement: i32,
    pub slot: usize,
    pub raw: i32,
    pub queued_at: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidPolicy,
    Level,
    StaleSlot,
    Inventory,
}
#[derive(Clone, Copy, Debug)]
pub struct Outcome {
    pub output: i32,
    pub burned: bool,
    pub advancement: Option<crate::skills::Advancement>,
}
impl Recipe {
    /// Prepare both possible credits and the successful XP result before drawing.
    /// An invalid output cannot consume input, advance XP or consume randomness.
    pub fn complete(
        &self,
        slot: usize,
        inv: &mut Inventory,
        items: &Catalog,
        stats: &mut Stats,
        skill: u8,
        max_level: u8,
        rate: i32,
        random: &mut impl RandomSource,
    ) -> Result<Outcome, Error> {
        let r = *self;
        if r.ticks == 0
            || r.level == 0
            || r.stop_burn < r.level
            || max_level < 2
            || r.low < 0
            || r.high < r.low
            || r.high > 65535
            || r.xp == 0
            || rate <= 0
            || r.raw == r.cooked
            || r.raw == r.burnt
            || r.cooked == r.burnt
        {
            return Err(Error::InvalidPolicy);
        }
        let ordinary = |id| {
            id >= 0
                && !items.stackable.contains(&id)
                && !items.dummy.contains(&id)
                && !items.notes.get(&id).is_some_and(|v| v.template != 0)
                && !items.placeholders.get(&id).is_some_and(|v| v.template != 0)
                && !items.transforms.get(&id).is_some_and(|v| v.template != 0)
        };
        if ![r.raw, r.cooked, r.burnt].into_iter().all(ordinary) {
            return Err(Error::InvalidPolicy);
        }
        let level = stats.current(skill) as u8;
        if level < r.level {
            return Err(Error::Level);
        }
        if inv
            .get(slot)
            .is_none_or(|v| v.id != r.raw || v.count != 1 || v.vars != 0)
        {
            return Err(Error::StaleSlot);
        }
        let prepare = |output| -> Result<Inventory, Error> {
            let mut next = inv.clone();
            let mut debit = Delete::new(r.raw, 1);
            debit.strict_slot = Some(slot);
            next.delete(items, debit).map_err(|_| Error::Inventory)?;
            let mut credit = Insert::new(output, 1);
            credit.strict_slot = Some(slot);
            next.insert(items, credit).map_err(|_| Error::Inventory)?;
            if next
                .get(slot)
                .is_none_or(|v| v.id != output || v.count != 1 || v.vars != 0)
            {
                return Err(Error::Inventory);
            }
            Ok(next)
        };
        let good = prepare(r.cooked)?;
        let bad = prepare(r.burnt)?;
        let mut next_stats = stats.clone();
        let advancement = crate::skills::advance(
            &mut next_stats,
            skill,
            f64::from(r.xp),
            f64::from(rate) / 100.0,
            max_level,
        )
        .map_err(|_| Error::InvalidPolicy)?;
        let burned = level < r.stop_burn
            && f64::from(random.below(256))
                >= crate::skills::success_rate(
                    r.low,
                    r.high,
                    i32::from(level),
                    i32::from(max_level),
                ) * 256.0;
        if burned {
            *inv = bad;
        } else {
            *inv = good;
            *stats = next_stats;
        }
        Ok(Outcome {
            output: if burned { r.burnt } else { r.cooked },
            burned,
            advancement: (!burned).then_some(advancement),
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Transition {
    pub job: Job,
    pub outcome: Outcome,
    pub committed_at: i32,
}
impl Transition {
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        format!("{{\"actor\":{},\"raw\":{},\"output\":{},\"burned\":{},\"slot\":{},\"fire\":{},\"queued_at\":{},\"committed_at\":{},\"xp\":{}}}",actor.slot,self.job.raw,self.outcome.output,self.outcome.burned,self.job.slot,self.job.target.id,self.job.queued_at,self.committed_at,self.outcome.advancement.map_or(0,|a|a.added_xp))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    struct Fixed {
        draw: u32,
        calls: u32,
    }
    impl RandomSource for Fixed {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.calls += 1;
            self.draw
        }
    }
    fn fixture() -> (Recipe, Inventory, Stats) {
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for slot in 0..2 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 10,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        (
            Recipe {
                raw: 10,
                cooked: 11,
                burnt: 12,
                level: 1,
                stop_burn: 34,
                low: 128,
                high: 512,
                xp: 30,
                ticks: 1,
            },
            inv,
            Stats::default(),
        )
    }
    #[test]
    fn threshold_selected_full_bag_burn_no_xp_and_guaranteed_no_draw() {
        for (draw, level, burn, calls) in
            [(128, 1, false, 1), (129, 1, true, 1), (255, 34, false, 0)]
        {
            let (r, mut inv, mut stats) = fixture();
            stats.set_current(7, level);
            let mut rng = Fixed { draw, calls: 0 };
            let out = r
                .complete(
                    1,
                    &mut inv,
                    &Catalog::default(),
                    &mut stats,
                    7,
                    99,
                    100,
                    &mut rng,
                )
                .unwrap();
            assert_eq!((out.burned, rng.calls), (burn, calls));
            assert_eq!(inv.get(0).unwrap().id, 10);
            assert_eq!(inv.get(1).unwrap().id, if burn { 12 } else { 11 });
            assert_eq!(stats.fine_xp(7), if burn { 0 } else { 300 });
            assert_eq!(
                r.complete(
                    1,
                    &mut inv,
                    &Catalog::default(),
                    &mut stats,
                    7,
                    99,
                    100,
                    &mut rng
                )
                .unwrap_err(),
                Error::StaleSlot
            );
        }
    }
    #[test]
    fn rates_and_failed_preparation_are_atomic_without_draw() {
        for rate in [100, 125, 500] {
            let (r, mut inv, mut stats) = fixture();
            r.complete(
                1,
                &mut inv,
                &Catalog::default(),
                &mut stats,
                7,
                99,
                rate,
                &mut Fixed { draw: 0, calls: 0 },
            )
            .unwrap();
            assert_eq!(stats.fine_xp(7), 3 * rate);
        }
        for mode in 0..6 {
            let (mut r, mut inv, mut stats) = fixture();
            let mut items = Catalog::default();
            let mut rng = Fixed { draw: 0, calls: 0 };
            let rate = if mode == 0 { 0 } else { 100 };
            match mode {
                1 => {
                    items.dummy.insert(12);
                }
                2 => stats.set_current(7, 0),
                3 => r.ticks = 0,
                4 => {
                    inv.restore_slot(
                        1,
                        Some(Item {
                            id: 10,
                            count: 1,
                            vars: 2,
                        }),
                    )
                    .unwrap();
                }
                5 => {
                    items.stackable.insert(11);
                }
                _ => {}
            }
            let before = (inv.clone(), stats.clone());
            assert!(r
                .complete(1, &mut inv, &items, &mut stats, 7, 99, rate, &mut rng)
                .is_err());
            assert_eq!((inv, stats), before);
            assert_eq!(rng.calls, 0);
        }
    }
}
