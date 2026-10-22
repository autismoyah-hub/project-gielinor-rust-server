//! Attested ordinary duck family callbacks, hosted by the existing NPC phase.
#[derive(Clone,Debug)]
pub struct Config {pub adults:std::collections::BTreeSet<u16>,pub male:u16,pub female:u16,pub ducklings:u16,pub sound:u16}
impl Config {pub fn contains(&self,id:u16)->bool{self.adults.contains(&id)||id==self.ducklings}}
