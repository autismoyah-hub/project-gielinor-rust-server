//! Ordinary NPC fishing spots: OpenRune FishingConfig/FishingTable/FishingCatchLogic.
//! The host binds every spot, tool, bait and catch against the target cache.
use std::collections::BTreeMap;
pub(crate) const QUEUE: i32 = -2391017;
/// Ticks between catch rolls (OpenRune CATCH_CYCLE).
pub const CATCH_CYCLE: i32 = 5;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tool {
    pub id: i32,
    pub level: i32,
    pub members: bool,
    /// Percentage added to both catch-rate bounds.
    pub bonus: i32,
    pub animation: Option<u16>,
    /// Lower-case name for the level refusal ("dragon harpoon").
    pub name: &'static str,
}
#[derive(Clone, Debug)]
pub struct Method {
    /// Any carried or wielded entry satisfies the method; the best bonus is used.
    pub tools: Vec<Tool>,
    pub bait: Option<i32>,
    pub animation: u16,
    pub missing_tool: &'static str,
    pub cast: &'static [&'static str],
}
#[derive(Clone, Debug)]
pub struct Catch {
    pub method: u8,
    pub product: i32,
    pub level: i32,
    pub xp_tenths: i32,
    pub rates: (i32, i32),
    pub members: bool,
    /// Replaces the method bait for this row only (rainbow fish).
    pub bait: Option<i32>,
    pub message: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub spot: u8,
    pub method: u8,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub skill: u8,
    pub full_sound: u16,
    pub methods: BTreeMap<u8, Method>,
    /// (spot kind, catch); spot kinds follow OpenRune FishingSpotDef ids.
    pub catches: Vec<(u8, Catch)>,
    /// (npc type, one-based operation).
    pub spots: BTreeMap<(u16, u8), Binding>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Tool,
    /// Only a higher-level tool is carried (dragon harpoon).
    ToolLevel(i32, &'static str),
    Bait,
    Level(i32),
    Members,
    CorrectBait,
    Capacity,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Attempt {
    pub tool: i32,
    pub bonus: i32,
    pub animation: u16,
    /// Indexes into `catches`, highest level first.
    pub eligible: Vec<usize>,
}
fn carried(inv: &crate::inventory::Inventory, worn: &crate::inventory::Inventory, id: i32) -> bool {
    inv.slots().iter().flatten().any(|i| i.id == id && i.count > 0)
        || worn.get(3).is_some_and(|i| i.id == id)
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.methods.is_empty() || self.catches.is_empty() || self.spots.is_empty() {
            return Err("Empty fishing configuration");
        }
        for m in self.methods.values() {
            if m.tools.is_empty()
                || m.tools
                    .iter()
                    .any(|t| t.id < 0 || !(1..=99).contains(&t.level) || !(0..=100).contains(&t.bonus))
                || m.bait.is_some_and(|b| b < 0)
            {
                return Err("Invalid fishing method");
            }
        }
        for (_, c) in &self.catches {
            if !self.methods.contains_key(&c.method)
                || c.product < 0
                || !(1..=99).contains(&c.level)
                || c.xp_tenths <= 0
                || c.rates.0 < 0
                || c.rates.1 < c.rates.0
                || c.rates.1 > 256
                || c.message.is_empty()
            {
                return Err("Invalid fishing catch");
            }
        }
        for b in self.spots.values() {
            if !self
                .catches
                .iter()
                .any(|(s, c)| *s == b.spot && c.method == b.method)
            {
                return Err("Fishing spot without catches");
            }
        }
        Ok(())
    }
    /// OpenRune `prepare`: tool, bait, level, row eligibility, then capacity.
    pub fn prepare(
        &self,
        binding: Binding,
        inv: &crate::inventory::Inventory,
        worn: &crate::inventory::Inventory,
        level: i32,
        member: bool,
    ) -> Result<Attempt, Refusal> {
        let method = self.methods.get(&binding.method).ok_or(Refusal::Tool)?;
        let usable = |t: &&Tool| (!t.members || member) && carried(inv, worn, t.id);
        let tool = method
            .tools
            .iter()
            .filter(usable)
            .filter(|t| t.level <= level)
            .max_by_key(|t| t.bonus);
        let Some(tool) = tool else {
            return Err(match method.tools.iter().filter(usable).min_by_key(|t| t.level) {
                Some(t) => Refusal::ToolLevel(t.level, t.name),
                None => Refusal::Tool,
            });
        };
        let rows: Vec<usize> = self
            .catches
            .iter()
            .enumerate()
            .filter(|(_, (s, c))| *s == binding.spot && c.method == binding.method)
            .map(|(i, _)| i)
            .collect();
        let mut unlocked: Vec<usize> = rows
            .iter()
            .copied()
            .filter(|&i| level >= self.catches[i].1.level)
            .collect();
        unlocked.sort_by_key(|&i| std::cmp::Reverse(self.catches[i].1.level));
        let bait_for = |i: usize| self.catches[i].1.bait.or(method.bait);
        // Wiki: a row-specific bait (stripy feathers) also admits the spot alone.
        if method.bait.is_some()
            && !unlocked
                .iter()
                .any(|&i| bait_for(i).is_some_and(|b| carried(inv, worn, b)))
            && !method.bait.is_some_and(|b| carried(inv, worn, b))
        {
            return Err(Refusal::Bait);
        }
        if unlocked.is_empty() {
            let min = rows.iter().map(|&i| self.catches[i].1.level).min().unwrap_or(1);
            return Err(Refusal::Level(min));
        }
        let paid: Vec<usize> = unlocked
            .into_iter()
            .filter(|&i| bait_for(i).is_none_or(|b| carried(inv, worn, b)))
            .collect();
        let eligible: Vec<usize> = paid
            .iter()
            .copied()
            .filter(|&i| !self.catches[i].1.members || member)
            .collect();
        if eligible.is_empty() {
            return Err(if paid.is_empty() {
                Refusal::CorrectBait
            } else {
                Refusal::Members
            });
        }
        if inv.free_space() == 0 {
            return Err(Refusal::Capacity);
        }
        Ok(Attempt {
            tool: tool.id,
            bonus: tool.bonus,
            animation: tool.animation.unwrap_or(method.animation),
            eligible,
        })
    }
    /// OpenRune `rollCatch`: one 256-roll per eligible row, highest level first.
    pub fn roll(
        &self,
        attempt: &Attempt,
        level: i32,
        random: &mut impl crate::npc::RandomSource,
    ) -> Option<usize> {
        attempt.eligible.iter().copied().find(|&i| {
            let (low, high) = self.catches[i].1.rates;
            let scale = |v: i32| v * (100 + attempt.bonus) / 100;
            let threshold =
                crate::skills::success_rate(scale(low), scale(high), level.clamp(1, 99), 99) * 256.0;
            (random.below(256) as f64) < threshold
        })
    }
    pub fn bait(&self, binding: Binding, row: usize) -> Option<i32> {
        self.catches[row]
            .1
            .bait
            .or_else(|| self.methods.get(&binding.method).and_then(|m| m.bait))
    }
    pub fn refusal_text(&self, binding: Binding, refusal: Refusal) -> String {
        match refusal {
            Refusal::Tool => self
                .methods
                .get(&binding.method)
                .map_or("You need a tool to fish here.", |m| m.missing_tool)
                .into(),
            Refusal::ToolLevel(level, name) => {
                format!("You need a Fishing level of {level} to use the {name}.")
            }
            Refusal::Bait => "You don't have any bait left.".into(),
            Refusal::Level(level) => format!("You need a Fishing level of {level} to fish here."),
            Refusal::Members => "You need to be a member to fish here.".into(),
            Refusal::CorrectBait => "You don't have the correct bait to catch these fish.".into(),
            Refusal::Capacity => "Your inventory is too full to hold any more fish.".into(),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::inventory::{Inventory, Item, StackType};
    pub(crate) const NET: i32 = 303;
    pub(crate) const ROD: i32 = 307;
    pub(crate) const FLY: i32 = 309;
    pub(crate) const BAIT: i32 = 313;
    pub(crate) const FEATHER: i32 = 314;
    pub(crate) const STRIPY: i32 = 10087;
    pub(crate) const SHRIMP: i32 = 317;
    pub(crate) const ANCHOVY: i32 = 321;
    pub(crate) const SARDINE: i32 = 327;
    pub(crate) const HERRING: i32 = 345;
    pub(crate) const TROUT: i32 = 335;
    pub(crate) const RAINBOW: i32 = 10138;
    pub(crate) const SPOT: u16 = 1530;
    pub(crate) const LURE_SPOT: u16 = 1526;
    fn tool(id: i32) -> Tool {
        Tool {
            id,
            level: 1,
            members: false,
            bonus: 0,
            animation: None,
            name: "tool",
        }
    }
    fn row(method: u8, product: i32, level: i32, xp: i32, rates: (i32, i32), message: &str) -> Catch {
        Catch {
            method,
            product,
            level,
            xp_tenths: xp,
            rates,
            members: false,
            bait: None,
            message: message.into(),
        }
    }
    /// Lumbridge-style net/bait spot plus a lure/bait spot, OpenRune values.
    pub(crate) fn config() -> Config {
        let mut rainbow = row(2, RAINBOW, 38, 800, (8, 64), "You catch a raw rainbow fish");
        rainbow.members = true;
        rainbow.bait = Some(STRIPY);
        Config {
            skill: 10,
            full_sound: 2277,
            methods: [
                (
                    0,
                    Method {
                        tools: vec![tool(NET)],
                        bait: None,
                        animation: 621,
                        missing_tool: "You need a small fishing net to fish here.",
                        cast: &["You cast out your net..."],
                    },
                ),
                (
                    1,
                    Method {
                        tools: vec![tool(ROD)],
                        bait: Some(BAIT),
                        animation: 622,
                        missing_tool: "You need a fishing rod to fish here.",
                        cast: &["You cast out your line..."],
                    },
                ),
                (
                    2,
                    Method {
                        tools: vec![tool(FLY)],
                        bait: Some(FEATHER),
                        animation: 622,
                        missing_tool: "You need a fly fishing rod to lure these fish.",
                        cast: &["You cast out your line..."],
                    },
                ),
            ]
            .into(),
            catches: vec![
                (0, row(0, SHRIMP, 1, 100, (48, 256), "You catch some raw shrimps.")),
                (0, row(0, ANCHOVY, 15, 400, (24, 128), "You catch some raw anchovies.")),
                (0, row(1, SARDINE, 5, 200, (32, 192), "You catch a raw sardine.")),
                (0, row(1, HERRING, 10, 300, (24, 128), "You catch a raw herring.")),
                (1, row(2, TROUT, 20, 500, (32, 192), "You catch a raw trout")),
                (1, rainbow),
            ],
            spots: [
                ((SPOT, 1), Binding { spot: 0, method: 0 }),
                ((SPOT, 3), Binding { spot: 0, method: 1 }),
                ((LURE_SPOT, 1), Binding { spot: 1, method: 2 }),
            ]
            .into(),
        }
    }
    fn inv(items: &[(i32, i32)]) -> Inventory {
        let mut i = Inventory::new(28, StackType::Normal, false);
        for (slot, &(id, count)) in items.iter().enumerate() {
            i.restore_slot(slot, Some(Item { id, count, vars: 0 })).unwrap();
        }
        i
    }
    fn worn() -> Inventory {
        Inventory::new(15, StackType::Normal, false)
    }
    const NETS: Binding = Binding { spot: 0, method: 0 };
    const BAITS: Binding = Binding { spot: 0, method: 1 };
    const LURES: Binding = Binding { spot: 1, method: 2 };
    #[test]
    fn gates_follow_tool_bait_level_order() {
        let c = config();
        c.validate().unwrap();
        assert_eq!(c.prepare(NETS, &inv(&[]), &worn(), 1, false), Err(Refusal::Tool));
        assert_eq!(c.prepare(BAITS, &inv(&[(ROD, 1)]), &worn(), 1, false), Err(Refusal::Bait));
        assert_eq!(
            c.prepare(BAITS, &inv(&[(ROD, 1), (BAIT, 5)]), &worn(), 4, false),
            Err(Refusal::Level(5))
        );
        // Missing tool is reported before missing bait, as in FishingCatchLogic.attemptGate.
        assert_eq!(c.prepare(BAITS, &inv(&[]), &worn(), 1, false), Err(Refusal::Tool));
        let a = c.prepare(NETS, &inv(&[(NET, 1)]), &worn(), 20, false).unwrap();
        assert_eq!(a.animation, 621);
        assert_eq!(
            a.eligible.iter().map(|&i| c.catches[i].1.product).collect::<Vec<_>>(),
            vec![ANCHOVY, SHRIMP]
        );
    }
    #[test]
    fn row_bait_override_and_membership_shape_eligibility() {
        let c = config();
        // Feathers only: trout. Rainbow fish needs stripy feathers and membership.
        let a = c.prepare(LURES, &inv(&[(FLY, 1), (FEATHER, 9)]), &worn(), 50, true).unwrap();
        assert_eq!(a.eligible.iter().map(|&i| c.catches[i].1.product).collect::<Vec<_>>(), vec![TROUT]);
        let a = c.prepare(LURES, &inv(&[(FLY, 1), (STRIPY, 9)]), &worn(), 50, true).unwrap();
        assert_eq!(a.eligible.iter().map(|&i| c.catches[i].1.product).collect::<Vec<_>>(), vec![RAINBOW]);
        assert_eq!(c.bait(LURES, a.eligible[0]), Some(STRIPY));
        assert_eq!(
            c.prepare(LURES, &inv(&[(FLY, 1), (STRIPY, 9)]), &worn(), 50, false),
            Err(Refusal::Members)
        );
        // Below rainbow fish level, stripy feathers are not bait for trout.
        assert_eq!(
            c.prepare(LURES, &inv(&[(FLY, 1), (STRIPY, 9)]), &worn(), 30, true),
            Err(Refusal::Bait)
        );
    }
    #[test]
    fn wielded_tool_counts_and_full_inventory_refuses() {
        let c = config();
        let mut w = worn();
        w.restore_slot(3, Some(Item { id: NET, count: 1, vars: 0 })).unwrap();
        assert!(c.prepare(NETS, &inv(&[]), &w, 1, false).is_ok());
        let full: Vec<(i32, i32)> = (0..28).map(|_| (SHRIMP, 1)).collect();
        assert_eq!(c.prepare(NETS, &inv(&full), &w, 1, false), Err(Refusal::Capacity));
    }
    #[test]
    fn roll_tries_highest_level_first_against_interpolated_threshold() {
        struct Fixed(Vec<u32>);
        impl crate::npc::RandomSource for Fixed {
            fn below(&mut self, _: u32) -> u32 {
                self.0.remove(0)
            }
        }
        let c = config();
        let a = c.prepare(NETS, &inv(&[(NET, 1)]), &worn(), 15, false).unwrap();
        // Level 15 anchovy threshold: 1 + floor((24*84 + 128*14)/98 + 0.5) = 40.
        assert_eq!(c.roll(&a, 15, &mut Fixed(vec![39])), Some(a.eligible[0]));
        assert_eq!(c.roll(&a, 15, &mut Fixed(vec![40, 0])), Some(a.eligible[1]));
        assert_eq!(c.roll(&a, 15, &mut Fixed(vec![255, 255])), None);
    }
}
