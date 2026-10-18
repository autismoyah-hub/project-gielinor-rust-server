//! SmithingActions.prepareAnvil/smithSlot selection rules. The host supplies
//! original product grids and cache enums and closes the modal before queueing.
use crate::{
    inventory::{Catalog, Inventory},
    smithing_make::{self, Request},
    stats::Stats,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct Config {
    pub hammer: i32,
    pub skill: u8,
    pub animation: u16,
    pub bar_by_tier: BTreeMap<i32, i32>,
    pub products: BTreeMap<i32, BTreeMap<u16, i32>>,
    pub quantity: BTreeMap<i32, i32>,
    pub bars: BTreeMap<i32, i32>,
    pub level: BTreeMap<i32, i32>,
    pub xp_per_bar: BTreeMap<i32, f64>,
    pub names: BTreeMap<i32, String>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Open {
    Denied(&'static str),
    Tier(i32),
}
#[derive(Debug)]
pub enum Selection {
    Ignore,
    Denied(String),
    /// Close the interface, then read the remembered quantity and start the job.
    /// Kotlin reads that varp after ifClose callbacks, so it is not captured here.
    Start {
        request: Arc<Request>,
    },
}
/// Shared make-quantity mirror for literal buttons and the remembered value.
pub fn quantity(value: i32) -> i32 {
    value.clamp(1, 28)
}
/// countDialog applies JVM Int.absoluteValue before askQuantity clamps it.
pub fn count_input(value: i32) -> i32 {
    quantity(value.wrapping_abs())
}

impl Config {
    fn level_for(&self, product: i32) -> i32 {
        self.level.get(&product).copied().unwrap_or(i32::MAX)
    }
    pub fn prepare(
        &self,
        inv: &Inventory,
        catalog: &Catalog,
        stats: &Stats,
        preferred: Option<i32>,
    ) -> Open {
        if smithing_make::total(inv, catalog, self.hammer) <= 0 {
            return Open::Denied("You need a hammer to work the metal with.");
        }
        // A recognised preferred bar bypasses the best-tier possession/level
        // search, exactly as the original item-on-anvil handler does.
        let preferred = preferred.and_then(|id| {
            self.bar_by_tier
                .iter()
                .find_map(|(&tier, &bar)| (bar == id).then_some(tier))
        });
        let best = || {
            self.bar_by_tier.iter().rev().find_map(|(&tier, &bar)| {
                (self.names.contains_key(&bar)
                    && smithing_make::total(inv, catalog, bar) > 0
                    && self.products.get(&bar).is_some_and(|slots| {
                        slots.values().any(|&product| {
                            self.level_for(product) <= stats.current(self.skill) as u8 as i32
                        })
                    }))
                .then_some(tier)
            })
        };
        preferred.or_else(best).map_or(
            Open::Denied("You don't have any bars to smith with."),
            Open::Tier,
        )
    }
    pub fn select(
        &self,
        inv: &Inventory,
        catalog: &Catalog,
        stats: &Stats,
        tier: i32,
        component: u16,
    ) -> Selection {
        let Some(&bar) = self.bar_by_tier.get(&tier) else {
            return Selection::Ignore;
        };
        let Some(bar_name) = self.names.get(&bar) else {
            return Selection::Ignore;
        };
        let Some(&product) = self
            .products
            .get(&bar)
            .and_then(|slots| slots.get(&component))
        else {
            return Selection::Ignore;
        };
        let Some(product_name) = self.names.get(&product) else {
            return Selection::Ignore;
        };
        if smithing_make::total(inv, catalog, self.hammer) <= 0 {
            return Selection::Denied("You need a hammer to work the metal with.".into());
        }
        let level = self.level_for(product);
        if (stats.current(self.skill) as u8 as i32) < level {
            return Selection::Denied(format!(
                "You need a Smithing level of {level} to make {}.",
                product_name.to_lowercase()
            ));
        }
        let bars = self.bars.get(&product).copied().unwrap_or(1);
        if smithing_make::total(inv, catalog, bar) < bars {
            return Selection::Denied(format!(
                "You need {bars} {} {} to make that.",
                bar_name.to_lowercase(),
                if bars == 1 { "bar" } else { "bars" }
            ));
        }
        let Some(&xp) = self.xp_per_bar.get(&bar) else {
            return Selection::Ignore;
        };
        Selection::Start {
            request: Arc::new(Request {
                product,
                produced: self.quantity.get(&product).copied().unwrap_or(1),
                consumed: vec![(bar, bars)],
                xp: xp * bars as f64,
                animation: self.animation,
                cycles: 5,
                impurity: false,
                members: false,
                repeat_delay: None,
                required_level: level,
                required_tools: vec![self.hammer],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn config() -> Config {
        Config {
            hammer: 2347,
            skill: 13,
            animation: 898,
            bar_by_tier: BTreeMap::from([(1, 2349), (6, 2363)]),
            products: BTreeMap::from([
                (2349, BTreeMap::from([(9, 1205), (22, 1117)])),
                (2363, BTreeMap::from([(9, 1213)])),
            ]),
            quantity: BTreeMap::from([(1117, 2)]),
            bars: BTreeMap::from([(1117, 5)]),
            level: BTreeMap::from([(1205, 0), (1117, 18), (1213, 85)]),
            xp_per_bar: BTreeMap::from([(2349, 12.5), (2363, 75.0)]),
            names: [
                (2349, "Bronze bar"),
                (2363, "Runite bar"),
                (1205, "Bronze dagger"),
                (1117, "Bronze platebody"),
                (1213, "Rune dagger"),
            ]
            .into_iter()
            .map(|(id, n)| (id, n.into()))
            .collect(),
        }
    }
    fn inventory(ids: &[i32]) -> Inventory {
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for (slot, &id) in ids.iter().enumerate() {
            inv.restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        inv
    }
    #[test]
    fn best_tier_uses_current_level_and_preferred_tier_bypasses_search() {
        let c = config();
        let cat = Catalog::default();
        let inv = inventory(&[2347, 2349, 2363]);
        let mut stats = Stats::default();
        assert_eq!(c.prepare(&inv, &cat, &stats, None), Open::Tier(1));
        stats.set_current(13, 85);
        assert_eq!(c.prepare(&inv, &cat, &stats, None), Open::Tier(6));
        assert_eq!(
            c.prepare(&inventory(&[2347]), &cat, &Stats::default(), Some(2363)),
            Open::Tier(6)
        );
        assert_eq!(
            c.prepare(&inv, &cat, &Stats::default(), Some(999)),
            Open::Tier(1)
        );
    }
    #[test]
    fn hammer_precedes_empty_tier_but_empty_product_click_is_silent() {
        let c = config();
        let cat = Catalog::default();
        let stats = Stats::default();
        assert_eq!(
            c.prepare(&inventory(&[]), &cat, &stats, None),
            Open::Denied("You need a hammer to work the metal with.")
        );
        assert_eq!(
            c.prepare(&inventory(&[2347]), &cat, &stats, None),
            Open::Denied("You don't have any bars to smith with.")
        );
        assert!(matches!(
            c.select(&inventory(&[]), &cat, &stats, 1, 99),
            Selection::Ignore
        ));
        assert!(
            matches!(c.select(&inventory(&[]), &cat, &stats, 1, 9), Selection::Denied(s) if s == "You need a hammer to work the metal with.")
        );
    }
    #[test]
    fn missing_requirement_is_uncraftable_and_missing_type_is_silent() {
        let mut c = config();
        let cat = Catalog::default();
        let inv = inventory(&[2347, 2349]);
        let stats = Stats::default();
        c.level.remove(&1205);
        assert!(
            matches!(c.select(&inv, &cat, &stats, 1, 9), Selection::Denied(s) if s == "You need a Smithing level of 2147483647 to make bronze dagger.")
        );
        c.names.remove(&1205);
        assert!(matches!(
            c.select(&inv, &cat, &stats, 1, 9),
            Selection::Ignore
        ));
    }
    #[test]
    fn levels_precede_bars_and_jobs_preserve_requested_count_and_per_bar_xp() {
        let c = config();
        let cat = Catalog::default();
        let mut stats = Stats::default();
        let inv = inventory(&[2347, 2349, 2349, 2349, 2349, 2349]);
        assert!(
            matches!(c.select(&inv, &cat, &stats, 1, 22), Selection::Denied(s) if s.contains("level of 18"))
        );
        stats.set_current(13, 18);
        assert!(
            matches!(c.select(&inventory(&[2347]), &cat, &stats, 1, 22), Selection::Denied(s) if s == "You need 5 bronze bar bars to make that.")
        );
        let Selection::Start { request } = c.select(&inv, &cat, &stats, 1, 22) else {
            panic!("expected smithing job")
        };
        assert_eq!(
            (
                request.product,
                request.produced,
                request.xp,
                request.cycles,
                quantity(99)
            ),
            (1117, 2, 62.5, 5, 28)
        );
        assert_eq!(request.consumed, vec![(2349, 5)]);
        let Selection::Start { request } = c.select(&inv, &cat, &stats, 1, 9) else {
            panic!("expected dagger job")
        };
        assert_eq!((request.produced, quantity(-1)), (1, 1));
    }
}

#[cfg(test)]
mod count_tests {
    #[test]
    fn count_dialog_absolute_value_preserves_jvm_minimum_edge() {
        for (input, expected) in [
            (i32::MIN, 1),
            (-29, 28),
            (-5, 5),
            (0, 1),
            (7, 7),
            (i32::MAX, 28),
        ] {
            assert_eq!(super::count_input(input), expected);
        }
    }
}
