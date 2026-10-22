//! Bounded content callbacks on the existing NPC AI countdown.
use std::collections::BTreeMap;
#[derive(Clone,Copy,Debug)]
pub struct Rule {pub text:&'static str,pub sound:u16,pub radius:i32,pub animation:Option<u16>}
#[derive(Clone,Debug,Default)]
pub struct Config {pub types:BTreeMap<u16,Rule>}
