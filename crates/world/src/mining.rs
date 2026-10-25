//! Ordinary ore profiles. Pickaxe controls cadence; rock controls success odds.
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Pickaxe {
    pub level: i32,
    pub animation: u16,
    pub interval: i32,
    pub members: bool,
    /// One-in-N chance to shorten this action by one tick; None draws no RNG.
    pub fast_one_in: Option<u32>,
}
impl Pickaxe {
    pub fn action_delay(self, random: &mut impl crate::npc::RandomSource) -> i32 {
        if self
            .fast_one_in
            .is_some_and(|n| n >= 2 && random.below(n) == 0)
        {
            self.interval - 1
        } else {
            self.interval
        }
    }
}
#[derive(Clone, Debug)]
pub struct Rock {
    pub members: bool,
    /// Staged resources restore this canonical full node and continue into live next stages.
    pub restore: Option<u32>,
    pub level: i32,
    pub product: i32,
    pub product_name: String,
    pub xp: f64,
    pub depleted: u32,
    /// Zero marks an infinite rock (OpenRune deplete mechanic 3): it never depletes and
    /// `depleted` must be the rock itself.
    pub respawn: i32,
    pub rates: (i32, i32),
}
#[derive(Clone, Debug)]
pub struct Config {
    pub skill: u8,
    pub picks: BTreeMap<i32, Pickaxe>,
    pub rocks: BTreeMap<u32, Rock>,
    pub full_sound: u16,
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.picks.is_empty()
            || self.rocks.is_empty()
            || self.picks.iter().any(|(&id, p)| {
                id < 0
                    || !(1..=99).contains(&p.level)
                    || !(2..=8).contains(&p.interval)
                    || p.fast_one_in
                        .is_some_and(|n| !(2..=65535).contains(&n) || p.interval < 3)
            })
        {
            return Err("Invalid mining tools");
        }
        for (&id, r) in &self.rocks {
            let infinite = r.respawn == 0;
            if id >= crate::locations::DELETED_ID
                || r.depleted >= crate::locations::DELETED_ID
                || (id == r.depleted) != infinite
                || (infinite && r.restore.is_some())
                || r.product < 0
                || r.product_name.is_empty()
                || !(1..=99).contains(&r.level)
                || !r.xp.is_finite()
                || r.xp <= 0.0
                || r.respawn < 0
                || r.rates.0 < 0
                || r.rates.1 < r.rates.0
                || r.rates.1 > 65535
            {
                return Err("Invalid mining rock");
            }
        }
        for (&start, rock) in &self.rocks {
            if let Some(root) = rock.restore {
                if root >= crate::locations::DELETED_ID || !self.rocks.contains_key(&root) {
                    return Err("Invalid mining restoration root");
                }
                let mut seen = std::collections::BTreeSet::new();
                let mut id = start;
                loop {
                    if !seen.insert(id) {
                        return Err("Cyclic mining stages");
                    }
                    let node = &self.rocks[&id];
                    if node.restore != Some(root)
                        || node.members != rock.members
                        || node.product != rock.product
                    {
                        return Err("Incompatible mining stage");
                    }
                    if !self.rocks.contains_key(&node.depleted) {
                        break;
                    }
                    id = node.depleted;
                }
                if self.rocks[&root].restore != Some(root) {
                    return Err("Invalid mining restoration root");
                }
            }
        }
        Ok(())
    }
    pub fn pick(
        &self,
        inv: &crate::inventory::Inventory,
        worn: &crate::inventory::Inventory,
        level: i32,
    ) -> Option<(i32, Pickaxe)> {
        self.pick_for_member(inv, worn, level, false)
    }
    pub fn pick_for_member(
        &self,
        inv: &crate::inventory::Inventory,
        worn: &crate::inventory::Inventory,
        level: i32,
        member: bool,
    ) -> Option<(i32, Pickaxe)> {
        crate::gathering::best_tool(inv, worn, level, |id| {
            self.picks
                .get(&id)
                .filter(|p| !p.members || member)
                .map(|p| p.level)
        })
        .map(|id| (id, self.picks[&id]))
    }
}
impl Rock {
    pub fn infinite(&self) -> bool {
        self.respawn == 0
    }
}
/// Essence rocks: members at or above `pure_level` mine pure essence instead of the
/// rock's rune essence (Wiki Pure essence / Rune essence; OpenRune has no such rule).
#[derive(Clone, Debug)]
pub struct Essence {
    pub rocks: std::collections::BTreeSet<u32>,
    pub pure: i32,
    pub pure_name: String,
    pub pure_level: i32,
}
impl Essence {
    pub fn validate(&self, rocks: &BTreeMap<u32, Rock>) -> Result<(), &'static str> {
        if self.rocks.is_empty()
            || self.pure < 0
            || self.pure_name.is_empty()
            || !(1..=99).contains(&self.pure_level)
            || self.rocks.iter().any(|id| {
                rocks.get(id).is_none_or(|r| !r.infinite() || r.members || r.product == self.pure)
            })
        {
            return Err("Invalid essence rocks");
        }
        Ok(())
    }
    /// The product for this rock and miner: pure essence for members at `pure_level`.
    pub fn product<'a>(&'a self, rock: &'a Rock, member: bool, level: i32) -> (i32, &'a str) {
        if member && level >= self.pure_level {
            (self.pure, &self.pure_name)
        } else {
            (rock.product, &rock.product_name)
        }
    }
}
/// OpenRune `MiningGemDropTables` and `Mining.kt` rollGemPreTable: on eligible rocks
/// every attempt first draws the gem pre-roll, before the ore success roll.
pub const PRE_ROLL_TOTAL: u32 = 32768;
/// Drop table "Mining gem drop table": a 1/256 roll plus a 3/256 retry with a charged
/// glory (1021/65536, about 1/64.2), which OpenRune folds into one /8216 table.
pub const PRE_ROLL_GLORY_TOTAL: u32 = 8216;
/// The pre-roll's nothing rows: the attempt yields no ore (OpenRune SkipOre).
pub const PRE_ROLL_NOTHING: u32 = 70;
pub const GEM_ROCK_TOTAL: u32 = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreRoll {
    KeepOre,
    SkipOre,
    Gem(i32),
}
#[derive(Clone, Debug, Default)]
pub struct Gems {
    /// Rocks that draw the pre-roll on every attempt.
    pub pre_roll_rocks: std::collections::BTreeSet<u32>,
    /// Weighted gems after the nothing rows (sapphire, emerald, ruby, diamond).
    pub pre_roll_gems: Vec<(u32, i32)>,
    /// Gem rocks: the product is drawn from `gem_rock_table`, and a charged glory
    /// triples the success rates.
    pub gem_rocks: std::collections::BTreeSet<u32>,
    pub gem_rock_table: Vec<(u32, i32)>,
    pub names: BTreeMap<i32, String>,
    pub charged_glory: std::collections::BTreeSet<i32>,
}
impl Gems {
    pub fn validate(&self, rocks: &BTreeMap<u32, Rock>) -> Result<(), &'static str> {
        let sum = |rows: &[(u32, i32)]| rows.iter().map(|r| r.0).sum::<u32>();
        if self.pre_roll_gems.is_empty()
            || PRE_ROLL_NOTHING + sum(&self.pre_roll_gems) != 128
            || sum(&self.gem_rock_table) != GEM_ROCK_TOTAL
            || self
                .pre_roll_gems
                .iter()
                .chain(&self.gem_rock_table)
                .any(|&(weight, item)| weight == 0 || !self.names.contains_key(&item))
        {
            return Err("Invalid mining gem tables");
        }
        if self
            .pre_roll_rocks
            .iter()
            .chain(&self.gem_rocks)
            .any(|id| !rocks.contains_key(id))
            || self.pre_roll_rocks.iter().any(|id| self.gem_rocks.contains(id))
        {
            return Err("Invalid mining gem rocks");
        }
        Ok(())
    }
    pub fn glory(&self, worn: &crate::inventory::Inventory) -> bool {
        worn.get(2).is_some_and(|item| self.charged_glory.contains(&item.id))
    }
    pub fn pre_roll_total(glory: bool) -> u32 {
        if glory {
            PRE_ROLL_GLORY_TOTAL
        } else {
            PRE_ROLL_TOTAL
        }
    }
    /// One draw below [`Gems::pre_roll_total`], in OpenRune row order: nothing,
    /// gems, then keep the ore for the rest.
    pub fn pre_roll(&self, draw: u32) -> PreRoll {
        if draw < PRE_ROLL_NOTHING {
            return PreRoll::SkipOre;
        }
        let mut at = draw - PRE_ROLL_NOTHING;
        for &(weight, item) in &self.pre_roll_gems {
            if at < weight {
                return PreRoll::Gem(item);
            }
            at -= weight;
        }
        PreRoll::KeepOre
    }
    /// One draw below [`GEM_ROCK_TOTAL`].
    pub fn gem_rock(&self, draw: u32) -> Option<i32> {
        let mut at = draw;
        for &(weight, item) in &self.gem_rock_table {
            if at < weight {
                return Some(item);
            }
            at -= weight;
        }
        None
    }
}
