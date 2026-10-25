//! Explicit registered melee specials. Energy regeneration and persistence use
//! the existing session varps/timer; toggling never charges or checks energy.
use crate::variables::{Update, Variables, Varp};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct Melee {
    pub cost: i32,
    pub accuracy: f64,
    pub max_hit: f64,
    pub block_type: u8,
    pub animation: u16,
    pub spot: u16,
    pub spot_slot: u8,
    pub spot_height: u16,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub energy: Varp,
    pub armed: Varp,
    pub melee: BTreeMap<i32, Melee>,
}

impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.energy.id == self.armed.id || self.melee.is_empty()
            || self.melee.iter().any(|(&item, s)| item < 0 || !(10..=1000).contains(&s.cost)
                || !(1..=3).contains(&s.block_type)
                || !s.accuracy.is_finite() || s.accuracy <= 0.0
                || !s.max_hit.is_finite() || s.max_hit <= 0.0)
        {
            return Err("Invalid registered melee special");
        }
        Ok(())
    }

    /// CombatTabScript: Weapon disarms, Shield stays armed, None only arms a
    /// registered combat special. Empty hands do nothing; no energy admission.
    pub fn toggle(&self, vars: &mut Variables, weapon: Option<i32>, logged: bool)
        -> (Option<Update>, Option<&'static str>)
    {
        match vars.get(self.armed.id) {
            1 => (vars.set(self.armed, 0, logged), None),
            2 => (None, None),
            _ => match weapon {
                None => (None, None),
                Some(item) if self.melee.contains_key(&item) => (vars.set(self.armed, 1, logged), None),
                Some(_) => (vars.set(self.armed, 0, logged), Some("This weapon does not have a special attack.")),
            },
        }
    }
}
