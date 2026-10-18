//! Ordinary ore profiles. Pickaxe controls cadence; rock controls success odds.
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Pickaxe {
    pub level: i32,
    pub animation: u16,
    pub interval: i32,
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
                id < 0 || !(1..=99).contains(&p.level) || !(2..=8).contains(&p.interval)
            })
        {
            return Err("Invalid mining tools");
        }
        for (&id, r) in &self.rocks {
            if id >= crate::locations::DELETED_ID
                || r.depleted >= crate::locations::DELETED_ID
                || id == r.depleted
                || r.product < 0
                || r.product_name.is_empty()
                || !(1..=99).contains(&r.level)
                || !r.xp.is_finite()
                || r.xp <= 0.0
                || r.respawn <= 0
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
        crate::gathering::best_tool(inv, worn, level, |id| self.picks.get(&id).map(|p| p.level))
            .map(|id| (id, self.picks[&id]))
    }
}
