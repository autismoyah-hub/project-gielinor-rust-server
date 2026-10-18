//! Optional location-bank eligibility; bank contents and transfers stay shared.
use crate::{
    locations::Location,
    stats::Stats,
    variables::{Varbit, Variables},
    CoordGrid,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Full,
    DepositBox,
}
#[derive(Clone, Debug)]
pub struct Gate {
    pub kind: Kind,
    pub members: bool,
    pub skill: u8,
    pub base_level: u8,
    pub any_bits: Vec<Varbit>,
}
impl Gate {
    pub fn allows(&self, member: bool, stats: &Stats, vars: &Variables) -> bool {
        if self.skill >= 24 || !(1..=99).contains(&self.base_level) || self.members && !member {
            return false;
        }
        stats.base(self.skill) as u8 >= self.base_level
            || self
                .any_bits
                .iter()
                .any(|&bit| vars.bit(bit).is_ok_and(|v| v != 0))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub(crate) kind: Kind,
    pub(crate) location: Location,
    pub(crate) origin: CoordGrid,
    pub(crate) last_movement: i32,
}

impl Request {
    pub fn kind(self) -> Kind {
        self.kind
    }
}
