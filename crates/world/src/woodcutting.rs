//! Ordinary woodcutting profiles. Tutorial trees retain their independent handler.
use crate::{inventory::Inventory, locations::Location};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Axe {
    pub level: i32,
    pub animation: u16,
}
#[derive(Clone, Debug)]
pub struct Tree {
    pub level: i32,
    pub product: i32,
    pub product_name: String,
    pub xp: f64,
    pub stump: u32,
    pub deplete_chance: u8,
    /// Shared active chopping ticks before the next successful log fells the tree.
    pub chopping_ticks: Option<i32>,
    pub respawn: [i32; 2],
    pub rates: BTreeMap<i32, (i32, i32)>,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub skill: u8,
    pub axes: BTreeMap<i32, Axe>,
    pub trees: BTreeMap<u32, Tree>,
    pub full_sound: u16,
    pub fall_sound: u16,
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.axes.is_empty() || self.trees.is_empty() {
            return Err("Empty woodcutting profile");
        }
        if self
            .axes
            .iter()
            .any(|(&id, axe)| id < 0 || !(1..=99).contains(&axe.level))
        {
            return Err("Invalid woodcutting axe");
        }
        for (&id, tree) in &self.trees {
            if id >= crate::locations::DELETED_ID
                || tree.stump >= crate::locations::DELETED_ID
                || id == tree.stump
                || tree.product < 0
                || tree.product_name.is_empty()
                || !(1..=99).contains(&tree.level)
                || !tree.xp.is_finite()
                || tree.xp <= 0.0
                || tree.respawn[0] <= 0
                || tree.respawn[1] < tree.respawn[0]
                || tree.chopping_ticks.is_some_and(|ticks| ticks <= 0)
                || self.axes.keys().any(|axe| !tree.rates.contains_key(axe))
                || tree
                    .rates
                    .values()
                    .any(|&(low, high)| low < 0 || high < low)
            {
                return Err("Invalid woodcutting tree");
            }
        }
        Ok(())
    }
    pub fn axe(&self, inv: &Inventory, worn: &Inventory, level: i32) -> Option<(i32, Axe)> {
        let eligible = |id: i32| {
            self.axes
                .get(&id)
                .copied()
                .filter(|a| level >= a.level)
                .map(|a| (id, a))
        };
        // Inventory maxByOrNull keeps the first equal requirement; worn wins a tie.
        let mut carried = None;
        for item in inv.slots().iter().flatten() {
            if let Some(candidate) = eligible(item.id) {
                if carried.is_none_or(|(_, a): (i32, Axe)| candidate.1.level > a.level) {
                    carried = Some(candidate);
                }
            }
        }
        let worn = worn.get(3).and_then(|i| eligible(i.id));
        match (worn, carried) {
            (Some(w), Some(c)) => Some(if w.1.level >= c.1.level { w } else { c }),
            (w, c) => w.or(c),
        }
    }
}
/// Per-location state, advanced once in ControllerMain, never once per player.
#[derive(Default)]
pub(crate) struct ActiveTrees {
    trees: std::collections::HashMap<(crate::CoordGrid, u8), ActiveTree>,
}
struct ActiveTree {
    location: Location,
    revision: u64,
    last_swing: i32,
    active_ticks: i32,
}
impl ActiveTrees {
    pub fn swing(&mut self, location: Location, revision: u64, clock: i32) -> i32 {
        let tree = self
            .trees
            .entry((location.coords, location.layer))
            .or_insert(ActiveTree {
                location,
                revision,
                last_swing: clock,
                active_ticks: 0,
            });
        if tree.revision != revision || tree.location != location {
            *tree = ActiveTree {
                location,
                revision,
                last_swing: clock,
                active_ticks: 0,
            };
        }
        tree.last_swing = clock;
        tree.active_ticks
    }
    pub fn tick(&mut self, clock: i32, locations: &crate::locations::Locations) {
        self.trees.retain(|_, tree| {
            if locations.revision(tree.location) != Some(tree.revision) {
                return false;
            }
            tree.active_ticks = if tree.last_swing == clock - 1 {
                tree.active_ticks.saturating_add(1)
            } else {
                tree.active_ticks - 1
            };
            tree.active_ticks > 0
        });
    }
}
#[derive(Clone, Debug)]
pub struct Trace {
    pub tick: i32,
    pub location: Location,
    pub revision: u64,
    pub action_delay: i32,
    pub result: &'static str,
    pub roll: Option<u32>,
    pub threshold: Option<f64>,
    pub product: i32,
    pub product_delta: i32,
    pub fine_xp_delta: i32,
    pub level: i32,
    pub required_level: i32,
    pub axe: Option<i32>,
    pub free_slots: usize,
    pub product_before: i64,
    pub fine_xp_before: i32,
}
impl Trace {
    /// Numeric and closed-vocabulary gameplay data only; no account/session secrets.
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        let optional = |value: Option<String>| value.unwrap_or_else(|| "null".into());
        format!(
            concat!(
                "{{\"schema\":1,\"actor_slot\":{},\"actor_generation\":{},\"tick\":{},",
                "\"location\":[{},{},{},{},{},{}],\"revision\":{},\"scheduled_tick\":{},",
                "\"result\":\"{}\",\"roll\":{},\"threshold\":{},\"product\":{},",
                "\"product_delta\":{},\"fine_xp_delta\":{},\"level\":{},\"required_level\":{},",
                "\"axe\":{},\"free_slots\":{},\"product_before\":{},\"fine_xp_before\":{}}}"
            ),
            actor.slot,
            actor.generation,
            self.tick,
            self.location.coords.x(),
            self.location.coords.z(),
            self.location.coords.level(),
            self.location.id,
            self.location.shape,
            self.location.angle,
            self.revision,
            self.action_delay,
            self.result,
            optional(self.roll.map(|v| v.to_string())),
            optional(self.threshold.map(|v| v.to_string())),
            self.product,
            self.product_delta,
            self.fine_xp_delta,
            self.level,
            self.required_level,
            optional(self.axe.map(|v| v.to_string())),
            self.free_slots,
            self.product_before,
            self.fine_xp_before
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_timer_counts_once_per_tick_recovers_when_idle_and_resets_on_replacement() {
        let loc =
            Location::new(crate::CoordGrid::new(3200, 3200, 0).unwrap(), 1281, 10, 0).unwrap();
        let mut locations = crate::locations::Locations::default();
        locations.load(loc);
        let rev = locations.revision(loc).unwrap();
        let mut timers = ActiveTrees::default();
        assert_eq!(timers.swing(loc, rev, 10), 0);
        assert_eq!(timers.swing(loc, rev, 10), 0); // second player
        timers.tick(11, &locations);
        assert_eq!(timers.swing(loc, rev, 11), 1);
        timers.tick(12, &locations);
        assert_eq!(timers.swing(loc, rev, 12), 2);
        timers.tick(13, &locations);
        timers.tick(14, &locations); // one idle tick heals one active tick
        assert_eq!(timers.swing(loc, rev, 14), 2);
        timers.tick(15, &locations);
        timers.tick(16, &locations);
        timers.tick(17, &locations);
        timers.tick(18, &locations);
        assert!(timers.trees.is_empty());
        timers.swing(loc, rev, 18);
        locations.load(loc);
        let new_rev = locations.revision(loc).unwrap();
        assert_ne!(rev, new_rev);
        assert_eq!(timers.swing(loc, new_rev, 19), 0);
    }
    #[test]
    fn axe_level_ties_prefer_worn_and_highest_eligible_requirement() {
        use crate::inventory::{Item, StackType};
        let mut inv = Inventory::new(28, StackType::Normal, false);
        let mut worn = Inventory::new(14, StackType::Normal, false);
        let mut c = Config {
            skill: 8,
            axes: [
                (
                    1,
                    Axe {
                        level: 1,
                        animation: 1,
                    },
                ),
                (
                    2,
                    Axe {
                        level: 1,
                        animation: 2,
                    },
                ),
                (
                    3,
                    Axe {
                        level: 6,
                        animation: 3,
                    },
                ),
            ]
            .into(),
            trees: BTreeMap::new(),
            full_sound: 0,
            fall_sound: 0,
        };
        inv.restore_slot(
            0,
            Some(Item {
                id: 1,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 3,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        worn.restore_slot(
            3,
            Some(Item {
                id: 2,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(c.axe(&inv, &worn, 1).unwrap().0, 2);
        assert_eq!(c.axe(&inv, &worn, 6).unwrap().0, 3);
        c.axes.remove(&2);
        assert_eq!(c.axe(&inv, &worn, 1).unwrap().0, 1);
    }
}
