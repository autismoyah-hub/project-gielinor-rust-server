//! Ordinary death recovery. Inventory loss is deliberately absent, matching the
//! reference server's unimplemented death-inventory step. Cinderworks owns rescue.
use crate::{variables::Varp, CoordGrid};
#[derive(Clone, Debug)]
pub struct Config {
    pub centre: CoordGrid,
    pub animation: u16,
    pub skills: Vec<u8>,
    pub special_attack: Varp,
    pub stop_music: u16,
    pub jingle: u16,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    pub destination: CoordGrid,
    pub deadline: i32,
    pub animated: bool,
}
