//! DinhsBulwarkScript's equipment transition, independent of the combat actor.
use crate::variables::{Update, Variables, Varp};
use std::collections::BTreeSet;
pub struct Config {
    pub items: BTreeSet<i32>,
    pub passive_delay: Varp,
    pub glow: u16,
}
pub struct Change {
    pub glow: u16,
    pub variable: Option<Update>,
}
impl Config {
    pub fn changed(
        &self,
        vars: &mut Variables,
        slot: usize,
        changed_item: i32,
        worn: Option<i32>,
        clock: i32,
        logged: bool,
    ) -> Option<Change> {
        if slot != 3 || !self.items.contains(&changed_item) || worn != Some(changed_item) {
            return None;
        }
        Some(Change {
            glow: self.glow,
            variable: vars.set(self.passive_delay, clock.wrapping_add(8), logged),
        })
    }
}
