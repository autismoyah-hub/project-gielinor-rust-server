//! Original Cinderworks restoration landmarks from audited f9180b23.
//! Authored Fallen Cinderworks palette. Restoration changes presentation only;
//! each pair must have identical real cache clipping and no interactive ops.
use crate::tower_run::Run;

// Cache-authored dwarven interior masonry and low stone rail; no menu ops.
pub const MASONRY: [u32; 2] = [6145, 6143];

#[derive(Clone, Copy, Debug)]
pub enum State {
    Forge,
    Water,
    Altar,
    Sanctuary,
    Foundry,
    Imp,
}
impl State {
    pub fn restored(self, run: Run) -> bool {
        match self {
            Self::Forge => run.services & 1 != 0,
            Self::Water => run.services & 2 != 0,
            Self::Altar => run.services & 4 != 0,
            Self::Sanctuary => run.victory || run.first_reward_paid,
            Self::Foundry => run.victory,
            Self::Imp => run.imp,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Landmark {
    pub x: i32,
    pub z: i32,
    pub dormant: u32,
    pub restored: u32,
    pub state: State,
    pub angle: u8,
}
impl Landmark {
    pub fn id(self, run: Run) -> u32 {
        if self.state.restored(run) {
            self.restored
        } else {
            self.dormant
        }
    }
}
pub const LANDMARKS: &[Landmark] = &[
    // Entries retain their established order for the existing five-landmark
    // validation interface; the Imp boiler is appended below.
    // Auxiliary blast furnace, river wheel, altar fire, central sanctuary hearth.
    Landmark {
        x: 43,
        z: 50,
        dormant: 18525,
        restored: 18526,
        state: State::Forge,
        angle: 0,
    },
    Landmark {
        x: 72,
        z: 58,
        dormant: 18512,
        restored: 18511,
        state: State::Water,
        angle: 0,
    },
    Landmark {
        x: 59,
        z: 49,
        dormant: 16810,
        restored: 16811,
        state: State::Altar,
        angle: 0,
    },
    Landmark {
        x: 55,
        z: 58,
        dormant: 16810,
        restored: 16811,
        state: State::Sanctuary,
        angle: 0,
    },
    // The foundry loses its consuming fire when the Warden is quenched.
    Landmark {
        x: 55,
        z: 78,
        dormant: 18526,
        restored: 18525,
        state: State::Foundry,
        angle: 1,
    },
    Landmark {
        x: 71,
        z: 76,
        dormant: 18526,
        restored: 18525,
        state: State::Imp,
        angle: 1,
    },
    // Flanking forge banks belong to the foundry's actual working rear wall.
    // The Warden's defeat extinguishes both mouths for this restorer.
    Landmark {
        x: 51,
        z: 78,
        dormant: 18526,
        restored: 18525,
        state: State::Foundry,
        angle: 1,
    },
    Landmark {
        x: 60,
        z: 78,
        dormant: 18526,
        restored: 18525,
        state: State::Foundry,
        angle: 1,
    },
];

pub const TRAINING_COOLANT:(i32,i32)=(69,52);
pub const VENTS:[(i32,i32);4]=[(67,70),(67,74),(76,70),(76,74)];
