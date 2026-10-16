//! Encounter patterns are deterministic and telegraphed before queued damage.
use crate::{
    simulation::{ActorId, NpcId},
    CoordGrid,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boss {
    Imp,
    Warden,
}
impl Boss {
    pub fn index(self) -> usize {
        if self == Self::Imp {
            0
        } else {
            1
        }
    }
    pub fn bounds(self) -> ([i32; 2], [i32; 2]) {
        match self {
            Self::Imp => ([66, 66], [77, 77]),
            Self::Warden => ([50, 67], [62, 77]),
        }
    }
    pub fn contains(self, origin: CoordGrid, tile: CoordGrid) -> bool {
        let (min, max) = self.bounds();
        tile.level() == origin.level()
            && (min[0]..=max[0]).contains(&(tile.x() - origin.x()))
            && (min[1]..=max[1]).contains(&(tile.z() - origin.z()))
    }
    pub fn phase(self, health: i32, maximum: i32) -> u8 {
        if self == Self::Imp {
            if health * 2 <= maximum {
                2
            } else {
                1
            }
        } else if health * 3 <= maximum {
            3
        } else if health * 3 <= maximum * 2 {
            2
        } else {
            1
        }
    }
    pub fn pattern(
        self,
        origin: CoordGrid,
        phase: u8,
        aim: CoordGrid,
        cycle: u32,
    ) -> Vec<CoordGrid> {
        let (min, max) = self.bounds();
        let mut tiles = Vec::new();
        for x in min[0]..=max[0] {
            for z in min[1]..=max[1] {
                let dx = origin.x() + x - aim.x();
                let dz = origin.z() + z - aim.z();
                let hot = match (self, phase) {
                    (Self::Imp, 1) => dx == 0 && dz == 0,
                    (Self::Imp, _) => dz == 0 && dx.abs() <= 1,
                    (Self::Warden, 1) => dx == 0,
                    (Self::Warden, 2) => dx == 0 || dz == 0,
                    (Self::Warden, _) => (x + z + cycle as i32).rem_euclid(3) == 0,
                };
                if hot {
                    tiles.push(origin.translate(x, z, 0).unwrap());
                }
            }
        }
        tiles
    }
}
#[derive(Clone, Debug)]
pub struct Encounter {
    pub owner: ActorId,
    pub run_id: u32,
    pub npc: NpcId,
    pub boss: Boss,
    pub phase: u8,
    pub next: i32,
    pub warning: Option<i32>,
    pub tiles: Vec<CoordGrid>,
    pub cycle: u32,
    pub exposed_until: i32,
    pub quench_ready: i32,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_phase_has_visible_danger_and_reachable_safe_neighbours() {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        for boss in [Boss::Imp, Boss::Warden] {
            for phase in 1..=3 {
                let aim = origin
                    .translate(if boss == Boss::Imp { 72 } else { 56 }, 72, 0)
                    .unwrap();
                for cycle in 0..3 {
                    let tiles = boss.pattern(origin, phase, aim, cycle);
                    assert!(!tiles.is_empty());
                    assert!(tiles.iter().all(|&tile| boss.contains(origin, tile)));
                    assert!([
                        (-1, -1),
                        (-1, 0),
                        (-1, 1),
                        (0, -1),
                        (0, 1),
                        (1, -1),
                        (1, 0),
                        (1, 1)
                    ]
                    .iter()
                    .any(|&(x, z)| !tiles.contains(&aim.translate(x, z, 0).unwrap())));
                }
            }
        }
    }
}
