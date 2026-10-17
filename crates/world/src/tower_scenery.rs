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
    },
    Landmark {
        x: 77,
        z: 58,
        dormant: 18512,
        restored: 18511,
        state: State::Water,
    },
    Landmark {
        x: 59,
        z: 49,
        dormant: 16810,
        restored: 16811,
        state: State::Altar,
    },
    Landmark {
        x: 55,
        z: 58,
        dormant: 16810,
        restored: 16811,
        state: State::Sanctuary,
    },
    // The foundry loses its consuming fire when the Warden is quenched.
    Landmark {
        x: 55,
        z: 78,
        dormant: 16811,
        restored: 16810,
        state: State::Foundry,
    },
    Landmark {
        x: 71,
        z: 76,
        dormant: 18526,
        restored: 18525,
        state: State::Imp,
    },
];
#[derive(Clone, Copy, Debug)]
pub struct Fixture {
    pub x: i32,
    pub z: i32,
    pub id: u32,
    pub shape: u8,
    pub angle: u8,
}
pub const FIXTURES: &[Fixture] = &[
    Fixture {
        x: 51,
        z: 53,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 60,
        z: 53,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 51,
        z: 59,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 60,
        z: 59,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 62,
        z: 59,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 62,
        z: 62,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 52,
        z: 35,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 59,
        z: 35,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 52,
        z: 41,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 59,
        z: 41,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 52,
        z: 47,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 59,
        z: 47,
        id: 7016,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 53,
        z: 49,
        id: 568,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 58,
        z: 49,
        id: 568,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 75,
        z: 57,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 75,
        z: 61,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 67,
        z: 70,
        id: 15215,
        shape: 22,
        angle: 0,
    },
    Fixture {
        x: 67,
        z: 74,
        id: 15215,
        shape: 22,
        angle: 0,
    },
    Fixture {
        x: 76,
        z: 70,
        id: 15215,
        shape: 22,
        angle: 0,
    },
    Fixture {
        x: 76,
        z: 74,
        id: 15215,
        shape: 22,
        angle: 0,
    },
    Fixture {
        x: 67,
        z: 68,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 76,
        z: 68,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 67,
        z: 76,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 76,
        z: 76,
        id: 7017,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 50,
        z: 69,
        id: 6143,
        shape: 0,
        angle: 0,
    },
    Fixture {
        x: 63,
        z: 75,
        id: 6143,
        shape: 0,
        angle: 0,
    },
    Fixture {
        x: 68,
        z: 70,
        id: 6143,
        shape: 0,
        angle: 0,
    },
    Fixture {
        x: 76,
        z: 74,
        id: 6143,
        shape: 0,
        angle: 0,
    },
    Fixture {
        x: 52,
        z: 67,
        id: 6143,
        shape: 0,
        angle: 3,
    },
    Fixture {
        x: 58,
        z: 78,
        id: 6915,
        shape: 10,
        angle: 0,
    },
    Fixture {
        x: 66,
        z: 78,
        id: 6915,
        shape: 10,
        angle: 0,
    },
];
pub const TRAINING_COOLANT: (i32, i32) = (69, 52);
pub const VENTS: [(i32, i32); 4] = [(67, 70), (67, 74), (76, 70), (76, 74)];

/// Earned guardians stay defeated for this restorer. Keep the dying actor long
/// enough to show its normal death sequence; hide its later shared respawn.
pub fn guardian_visible(run: Run, kind: u16, imp: u16, warden: u16, alive: bool) -> bool {
    !alive || !((kind == imp && run.imp) || (kind == warden && run.victory))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restoration_replays_services_but_keeps_first_clear_beacon() {
        let mut run = Run::default();
        run.begin();
        for state in [State::Forge, State::Water, State::Altar, State::Sanctuary] {
            assert!(!state.restored(run));
        }
        for service in [
            crate::tower_run::Service::Kitchen,
            crate::tower_run::Service::Altar,
            crate::tower_run::Service::Forge,
        ] {
            assert!(run.service(service));
        }
        assert!(!State::Sanctuary.restored(run));
        assert!(!State::Foundry.restored(run));
        run.defeat_imp();
        run.defeat_warden();
        assert!(State::Sanctuary.restored(run));
        assert!(State::Foundry.restored(run));
        run.paid(run.id);
        run.begin();
        assert!(State::Sanctuary.restored(run));
        for state in [State::Forge, State::Water, State::Altar, State::Foundry] {
            assert!(!state.restored(run));
        }
    }
    #[test]
    fn solid_scenery_does_not_intrude_into_hazard_fields() {
        let origin = crate::CoordGrid::new(10368, 64, 0).unwrap();
        for f in FIXTURES {
            if f.shape != 10 {
                continue;
            }
            for boss in [
                crate::cinderworks_encounter::Boss::Imp,
                crate::cinderworks_encounter::Boss::Warden,
            ] {
                assert!(
                    !boss.contains(origin, origin.translate(f.x, f.z, 0).unwrap()),
                    "Arena obstruction: {f:?}"
                );
            }
        }
    }
    #[test]
    fn completed_guardians_keep_their_death_sequence_then_return_on_replay() {
        let mut run = Run::default();
        run.imp = true;
        run.victory = true;
        assert!(guardian_visible(run, 200, 200, 201, false));
        assert!(!guardian_visible(run, 200, 200, 201, true));
        assert!(!guardian_visible(run, 201, 200, 201, true));
        assert!(guardian_visible(run, 202, 200, 201, true));
        run.imp = false;
        run.victory = false;
        assert!(guardian_visible(run, 200, 200, 201, true));
        assert!(guardian_visible(run, 201, 200, 201, true));
    }
    #[test]
    fn all_six_preparation_orders_preserve_the_guardian_gate() {
        use crate::tower_run::Service::{Altar, Forge, Kitchen};
        for order in [
            [Forge, Kitchen, Altar],
            [Forge, Altar, Kitchen],
            [Kitchen, Forge, Altar],
            [Kitchen, Altar, Forge],
            [Altar, Forge, Kitchen],
            [Altar, Kitchen, Forge],
        ] {
            let mut run = Run::default();
            run.begin();
            for (i, service) in order.into_iter().enumerate() {
                assert!(run.service(service));
                if i < 2 {
                    assert!(!run.defeat_imp());
                }
            }
            assert!(run.defeat_imp());
            assert!(!State::Sanctuary.restored(run));
            assert!(run.defeat_warden());
            assert!(State::Sanctuary.restored(run));
        }
    }
}
