//! Generic Dig metadata and independently registered content hook boundary.
use crate::simulation::{ActionError, ActorId, World};
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub item: i32,
    pub animation: u16,
    pub sound: u16,
    pub delay: i32,
}
impl Rule {
    pub fn valid(self) -> bool {
        self.item >= 0 && self.animation != u16::MAX && self.delay > 0
    }
}
/// Content owns claims, any pre-dig UI, and its completed gameplay transaction.
/// Core Dig never pretends that an unregistered quest or boss hook is present.
#[derive(Clone, Copy)]
pub struct Hook {
    pub name: &'static str,
    pub claims: fn(&World, ActorId) -> bool,
    pub before: fn(&mut World, ActorId) -> Result<bool, ActionError>,
    pub dig: fn(&mut World, ActorId) -> Result<(), ActionError>,
}
pub const NOTHING: &str = "You dig a hole in the ground... but find nothing.";
