//! Ordinary sheep production profiles, resolved from the target cache.
use std::collections::BTreeMap;
#[derive(Clone,Debug)]
pub struct Config {
    pub targets:BTreeMap<u16,u16>,
    pub shears:i32,
    pub wool:i32,
    pub sequence:u16,
    pub sound:u16,
}
pub const REGROWTH:i32=50;
