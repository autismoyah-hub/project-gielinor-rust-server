//! Captured NPC identity and registered operation behavior for player interactions.
use crate::simulation::NpcId;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handler {
    Bank,
    Combat,
    Person,
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub target: NpcId,
    pub operation: u8,
    pub handler: Option<Handler>,
}
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Face(NpcId),
    Bank,
    Message(&'static str),
}
