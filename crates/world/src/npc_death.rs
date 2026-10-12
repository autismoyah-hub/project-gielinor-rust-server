use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Definition {
    pub animation: u16,
    pub animation_ticks: i32,
    pub sound: Option<u16>,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub definitions: BTreeMap<u16, Definition>,
    pub bones: u16,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum State {
    Arrival {
        at: i32,
        drop_coords: crate::CoordGrid,
    },
    Animation {
        until: i32,
        drop_coords: crate::CoordGrid,
    },
    Respawn {
        at: i32,
    },
}
