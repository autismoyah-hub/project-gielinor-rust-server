//! MagicRunes requirement validation. Source order is significant: this returns
//! a consumption plan without reserving or consuming inventory or pouch items.
use crate::inventory::Item;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Inventory { item: i32, slot: usize, count: i32 },
    Varbit { id: u32, count: i32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Validation {
    Unlimited,
    Enough(Vec<Source>),
    NotEnough(i32),
    NotWearing(i32),
}
impl Validation {
    pub fn valid(&self) -> bool {
        matches!(self, Self::Unlimited | Self::Enough(_))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PouchSlot {
    pub compact: i32,
    pub varbit: Option<u32>,
    pub count: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Requirement {
    pub item: i32,
    pub count: i32,
    pub worn_slot: Option<usize>,
}
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub compact: BTreeMap<i32, i32>,
    pub high: BTreeMap<i32, BTreeSet<i32>>,
    pub low: BTreeMap<i32, BTreeSet<i32>>,
    pub fake: BTreeMap<i32, i32>,
    pub substitutes: BTreeMap<i32, Vec<i32>>,
    pub staff_substitutes: BTreeMap<i32, BTreeSet<i32>>,
    /// Config iteration order, not sorted by item id.
    pub combos: Vec<(i32, i32, i32)>,
}
pub struct Context<'a> {
    pub inventory: &'a [Option<Item>],
    pub worn: &'a [Option<Item>],
    pub pouch: &'a [PouchSlot],
    pub fake_runes: bool,
    pub fountain: bool,
}
impl Context<'_> {
    fn first(&self, id: i32) -> Option<(usize, Item)> {
        self.inventory
            .iter()
            .enumerate()
            .find_map(|(slot, item)| item.filter(|i| i.id == id).map(|i| (slot, i)))
    }
    fn unlimited(&self, map: &BTreeMap<i32, BTreeSet<i32>>, rune: i32) -> bool {
        map.get(&rune).is_some_and(|sources| {
            [3, 5].into_iter().any(|slot| {
                self.worn
                    .get(slot)
                    .and_then(|i| *i)
                    .is_some_and(|i| sources.contains(&i.id))
            })
        })
    }
}
impl Catalog {
    pub fn rune(&self, ctx: &Context<'_>, rune: i32, required: i32) -> Result<Validation, String> {
        self.rune_inner(ctx, rune, required, &mut BTreeSet::new())
    }
    fn rune_inner(
        &self,
        ctx: &Context<'_>,
        rune: i32,
        required: i32,
        visited: &mut BTreeSet<i32>,
    ) -> Result<Validation, String> {
        if required <= 0 {
            return Err("Rune requirement must be positive".into());
        }
        // Reject malformed configuration cycles rather than overflowing the server stack.
        if !visited.insert(rune) {
            return Err("Cyclic rune substitutes".into());
        }
        let result = self.rune_sources(ctx, rune, required, visited);
        visited.remove(&rune);
        result
    }
    fn rune_sources(
        &self,
        ctx: &Context<'_>,
        rune: i32,
        required: i32,
        visited: &mut BTreeSet<i32>,
    ) -> Result<Validation, String> {
        let compact = self.compact.get(&rune);
        if (compact.is_some() && ctx.fountain) || ctx.unlimited(&self.high, rune) {
            return Ok(Validation::Unlimited);
        }
        if ctx.fake_runes {
            if let Some(&fake) = self.fake.get(&rune) {
                return Ok(match ctx.first(fake) {
                    Some((slot, item)) if item.count >= required => {
                        Validation::Enough(vec![Source::Inventory {
                            item: fake,
                            slot,
                            count: required,
                        }])
                    }
                    _ => Validation::NotEnough(rune),
                });
            }
        }
        if ctx.unlimited(&self.low, rune) {
            return Ok(Validation::Unlimited);
        }
        let mut sources = Vec::new();
        let mut remaining = required;
        if let Some((slot, item)) = ctx.first(rune) {
            if item.count >= required {
                return Ok(Validation::Enough(vec![Source::Inventory {
                    item: rune,
                    slot,
                    count: required,
                }]));
            }
            remaining = remaining.wrapping_sub(item.count);
            sources.push(Source::Inventory {
                item: rune,
                slot,
                count: item.count,
            });
        }
        // Only the first matching pouch compartment is considered. A partial
        // pouch count is not combined with a substitute rune in Kotlin.
        if let Some(slot) = ctx.pouch.iter().find(|s| Some(&s.compact) == compact) {
            if let Some(id) = slot.varbit {
                if slot.count >= remaining {
                    sources.push(Source::Varbit {
                        id,
                        count: remaining,
                    });
                    return Ok(Validation::Enough(sources));
                }
            }
        }
        if let Some(subs) = self.substitutes.get(&rune) {
            for &sub in subs {
                match self.rune_inner(ctx, sub, remaining, visited)? {
                    Validation::Unlimited => return Ok(Validation::Unlimited),
                    Validation::Enough(extra) => {
                        sources.extend(extra);
                        return Ok(Validation::Enough(sources));
                    }
                    _ => (),
                }
            }
        }
        Ok(Validation::NotEnough(rune))
    }
    pub fn worn(&self, ctx: &Context<'_>, item: i32, slot: usize) -> Validation {
        if ctx.worn.get(slot).and_then(|i| *i).is_some_and(|w| {
            w.id == item
                || self
                    .staff_substitutes
                    .get(&item)
                    .is_some_and(|subs| subs.contains(&w.id))
        }) {
            Validation::Unlimited
        } else {
            Validation::NotWearing(item)
        }
    }
    pub fn requirements(
        &self,
        ctx: &Context<'_>,
        requirements: &[Requirement],
    ) -> Result<Vec<Validation>, String> {
        let mut reqs = requirements.to_vec();
        let mut output = Vec::new();
        let find =
            |reqs: &[Requirement], item| reqs.iter().position(|r| r.item == item && r.count > 0);
        for &(combo, first, second) in &self.combos {
            let (Some(a), Some(b)) = (find(&reqs, first), find(&reqs, second)) else {
                continue;
            };
            if [first, second]
                .into_iter()
                .any(|r| ctx.unlimited(&self.high, r) || ctx.unlimited(&self.low, r))
            {
                continue;
            }
            let needed = reqs[a].count.min(reqs[b].count);
            let result = self.rune(ctx, combo, needed)?;
            if result.valid() {
                output.push(result);
                reqs[a].count = reqs[a].count.wrapping_sub(needed);
                reqs[b].count = reqs[b].count.wrapping_sub(needed);
            }
        }
        let mut search_combos = false;
        for req in &mut reqs {
            if req.count < 0 {
                return Err("Negative remaining rune count".into());
            }
            if req.count == 0 {
                continue;
            }
            if let Some(slot) = req.worn_slot {
                let result = self.worn(ctx, req.item, slot);
                if result.valid() {
                    req.count = 0;
                }
                output.push(result);
                continue;
            }
            let result = self.rune(ctx, req.item, req.count)?;
            if !result.valid() {
                search_combos = true;
                continue;
            }
            output.push(result);
            req.count = 0;
        }
        if !search_combos {
            return Ok(output);
        }
        for &(combo, first, second) in &self.combos {
            let (a, b) = (find(&reqs, first), find(&reqs, second));
            let needed = a
                .map_or(0, |i| reqs[i].count)
                .max(b.map_or(0, |i| reqs[i].count));
            if needed == 0 {
                continue;
            }
            let result = self.rune(ctx, combo, needed)?;
            if result.valid() {
                output.push(result);
                // The source subtracts the maximum from both requirements,
                // including the smaller one; negative leftovers are intentional.
                if let Some(i) = a {
                    reqs[i].count = reqs[i].count.wrapping_sub(needed);
                }
                if let Some(i) = b {
                    reqs[i].count = reqs[i].count.wrapping_sub(needed);
                }
            }
        }
        output.extend(
            reqs.iter()
                .filter(|r| r.count > 0 && r.worn_slot.is_none())
                .map(|r| Validation::NotEnough(r.item)),
        );
        Ok(output)
    }
}
