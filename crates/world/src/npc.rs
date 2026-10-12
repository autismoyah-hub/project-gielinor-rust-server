//! NPC movement, wandering and regeneration; random decisions are supplied by the simulation.
use crate::{
    collision::{CollisionMap, Strategy},
    movement::{Movement, Speed},
    CoordGrid,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restrict {
    Normal,
    Blocked,
    BlockedNormal,
    Indoors,
    Outdoors,
    NoMove,
    PassThru,
}
#[derive(Debug, Clone, Copy)]
pub enum BlockWalk {
    None,
    Npc,
    All,
}
pub trait RandomSource {
    fn below(&mut self, exclusive: u32) -> u32;
}
/// Reproducible simulation randomness. Differential tests can supply recorded Kotlin draws.
/// This generator is deliberately not claimed to reproduce Kotlin's default RNG seed stream.
#[derive(Clone, Copy)]
pub struct SeededRandom(pub u64);
impl RandomSource for SeededRandom {
    fn below(&mut self, exclusive: u32) -> u32 {
        assert!(exclusive > 0);
        let threshold = exclusive.wrapping_neg() % exclusive;
        loop {
            self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            let value = (z ^ (z >> 31)) as u32;
            if value >= threshold {
                return value % exclusive;
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    None,
    Wander,
    Patrol,
}
#[derive(Debug, Clone)]
pub struct Npc {
    pub type_id: u16,
    /// Initial low-resolution facing; RSProt retains this across movement steps.
    pub respawn_direction: u8,
    pub movement: Movement,
    pub spawn: CoordGrid,
    pub restrict: Restrict,
    pub mode: Mode,
    pub wander_radius: i32,
    pub idle_cycles: i32,
    pub visible: bool,
    pub delayed: bool,
    pub viewed: bool,
    pub headless: bool,
    pub stats: [i32; 6],
    pub base_stats: [i32; 6],
    pub combat_xp_multiplier: i32,
    pub last_combat: i32,
    pub aggressive_player: Option<crate::simulation::ActorId>,
    pub face_player: Option<crate::simulation::ActorId>,
    /// PlayerFaceClose owns movement independently of the NPC's default mode.
    pub dialogue_face: Option<crate::simulation::ActorId>,
    pub face_changed: bool,
    pub attacking_player: Option<crate::simulation::ActorId>,
    pub escape_player: Option<crate::simulation::ActorId>,
    pub escape_idle: i32,
    pub retaliate_due: std::collections::VecDeque<i32>,
    pub combat_action_delay: i32,
    pub last_attack: i32,
    pub flat_armour: i32,
    pub hero_points: crate::hero_points::HeroPoints,
    pub respawns: bool,
    pub respawn_rate: i32,
    pub regen_rate: i32,
    pub regen_clock: i32,
    pub patrol: Vec<(CoordGrid, u8)>,
    pub patrol_index: usize,
    pub patrol_idle: i32,
    pub patrol_pause: i32,
}
impl Npc {
    pub fn new(
        type_id: u16,
        spawn: CoordGrid,
        size: i32,
        restrict: Restrict,
        block: BlockWalk,
    ) -> Result<Self, &'static str> {
        if !(1..=64).contains(&size) {
            return Err("NPC size outside supported bounds");
        }
        let mut movement = Movement::new(spawn);
        movement.size = size;
        movement.player_rules = false;
        movement.collision_mask = match block {
            BlockWalk::None => None,
            BlockWalk::Npc => Some(0x80000),
            BlockWalk::All => Some(0x180000),
        };
        movement.strategy = match restrict {
            Restrict::Blocked => Strategy::Blocked,
            Restrict::BlockedNormal => Strategy::LineOfSight,
            Restrict::Indoors => Strategy::Indoors,
            Restrict::Outdoors => Strategy::Outdoors,
            _ => Strategy::Normal,
        };
        movement.step_extra = if matches!(restrict, Restrict::Blocked | Restrict::PassThru) {
            0
        } else {
            0x80000
        };
        Ok(Self {
            type_id,
            respawn_direction: 6,
            movement,
            spawn,
            restrict,
            mode: Mode::Wander,
            wander_radius: 0,
            idle_cycles: 0,
            visible: true,
            delayed: false,
            viewed: false,
            headless: false,
            stats: [1; 6],
            base_stats: [1; 6],
            combat_xp_multiplier: 0,
            last_combat: -1000,
            aggressive_player: None,
            face_player: None,
            dialogue_face: None,
            face_changed: false,
            attacking_player: None,
            escape_player: None,
            escape_idle: 0,
            retaliate_due: Default::default(),
            combat_action_delay: 0,
            last_attack: -1000,
            flat_armour: 0,
            hero_points: Default::default(),
            respawns: false,
            respawn_rate: 100,
            regen_rate: 0,
            regen_clock: 0,
            patrol: vec![],
            patrol_index: 0,
            patrol_idle: 0,
            patrol_pause: 0,
        })
    }
    pub fn move_to(&mut self, dest: CoordGrid, speed: Speed) -> Result<(), &'static str> {
        if dest.level() != self.movement.coords.level() {
            return Err("NPC route changes plane");
        }
        self.movement.route([dest], speed);
        Ok(())
    }
    pub fn process_movement(&mut self, map: &mut CollisionMap, clock: i32) {
        if self.restrict == Restrict::NoMove {
            self.movement.abort();
            return;
        }
        self.movement.process(map, clock);
    }
    pub fn face_player(&mut self, target: Option<crate::simulation::ActorId>) {
        self.dialogue_face = None;
        if self.face_player != target {
            self.face_player = target;
            self.face_changed = true;
        }
    }
    pub fn regenerate(&mut self) {
        if self.regen_rate == 0 {
            return;
        }
        if self.regen_clock == 0 {
            self.regen_clock = self.regen_rate;
            for (current, base) in self.stats.iter_mut().zip(self.base_stats) {
                *current = current.wrapping_add(base.wrapping_sub(*current).signum());
            }
        } else {
            self.regen_clock = self.regen_clock.wrapping_sub(1);
        }
    }
    pub fn wander(
        &mut self,
        map: &mut CollisionMap,
        clock: i32,
        random: &mut impl RandomSource,
    ) -> Result<(), &'static str> {
        self.idle_cycles = if self.movement.last_movement == clock.wrapping_sub(1) {
            0
        } else {
            self.idle_cycles.wrapping_add(1)
        };
        if self.wander_radius <= 0 && self.movement.coords == self.spawn {
            return Ok(());
        }
        if self.idle_cycles >= 500 {
            return self.movement.teleport(map, self.spawn, clock, false);
        }
        if self.wander_radius <= 0 {
            return self.move_to(self.spawn, Speed::Walk);
        }
        if !self.visible
            || self.delayed
            || self.movement.destination().is_some()
            || (!self.viewed && !self.headless)
        {
            return Ok(());
        }
        if random.below(4) == 0 {
            let radius = self.wander_radius;
            if radius > 16383 {
                return Err("NPC wander radius outside world bounds");
            }
            let bound = (radius * 2 + 1) as u32;
            let dx = random.below(bound) as i32 - radius;
            let dz = random.below(bound) as i32 - radius;
            let dest = CoordGrid::new(self.spawn.x() + dx, self.spawn.z() + dz, self.spawn.level())
                .map_err(|_| "NPC wander target outside world")?;
            self.move_to(dest, Speed::Walk)?;
        }
        Ok(())
    }
    pub fn patrol(&mut self, map: &mut CollisionMap, clock: i32) -> Result<(), &'static str> {
        if self.patrol.is_empty() {
            return Ok(());
        }
        self.patrol_idle = if self.movement.last_movement == clock.wrapping_sub(1) {
            0
        } else {
            self.patrol_idle.wrapping_add(1)
        };
        let (destination, pause) = self.patrol[self.patrol_index % self.patrol.len()];
        if self.patrol_idle > 30 {
            self.movement.teleport(map, destination, clock, false)?;
        }
        if self.movement.coords == destination {
            if pause as i32 > self.patrol_pause {
                self.patrol_pause += 1;
                self.patrol_idle = 0;
            } else {
                self.patrol_pause = 0;
                self.patrol_index = self.patrol_index.wrapping_add(1);
                let next = self.patrol[self.patrol_index % self.patrol.len()].0;
                if next.level() != self.movement.coords.level() {
                    self.movement.teleport(map, next, clock, false)?;
                } else {
                    self.move_to(next, Speed::Walk)?;
                }
            }
        } else {
            self.move_to(destination, Speed::Walk)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Fixed;
    #[test]
    fn patrol_teleport_advances_and_moves_in_the_same_cycle() {
        let waypoint = CoordGrid::new(3203, 3203, 0).unwrap();
        let next = CoordGrid::new(3206, 3201, 0).unwrap();
        let mut map = CollisionMap::default();
        map.allocate(3200, 3200, 0);
        let mut npc = Npc::new(
            1,
            CoordGrid::new(3201, 3203, 0).unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::Npc,
        )
        .unwrap();
        npc.patrol = vec![(waypoint, 0), (next, 0)];
        npc.patrol_idle = 30;
        npc.movement.occupancy(&mut map, true);
        npc.patrol(&mut map, 100).unwrap();
        assert_eq!(npc.movement.coords, waypoint);
        assert!(npc.movement.pending_teleport);
        assert_eq!(npc.movement.destination(), Some(next));
        npc.process_movement(&mut map, 100);
        assert_eq!(npc.movement.coords, CoordGrid::new(3204, 3202, 0).unwrap());
        assert_eq!(map.get(3201, 3203, 0) & 0x80000, 0);
    }
    #[test]
    fn patrol_pause_and_plane_change_and_invalid_teleport_are_atomic() {
        let start = CoordGrid::new(3203, 3203, 0).unwrap();
        let next = CoordGrid::new(3203, 3203, 1).unwrap();
        let mut map = CollisionMap::default();
        map.allocate(3200, 3200, 0);
        map.allocate(3200, 3200, 1);
        let mut npc = Npc::new(1, start, 1, Restrict::Normal, BlockWalk::Npc).unwrap();
        npc.patrol = vec![(start, 2), (next, 0)];
        npc.movement.occupancy(&mut map, true);
        for clock in [100, 101] {
            npc.patrol(&mut map, clock).unwrap();
            assert_eq!(npc.movement.coords, start);
        }
        npc.patrol(&mut map, 102).unwrap();
        assert_eq!(npc.movement.coords, next);
        assert_eq!(npc.patrol_pause, 0);
        let invalid = CoordGrid::new(4000, 4000, 0).unwrap();
        assert!(npc
            .movement
            .teleport(&mut map, invalid, 103, false)
            .is_err());
        assert_eq!(npc.movement.coords, next);
        assert_ne!(map.get(next.x(), next.z(), next.level()) & 0x80000, 0);
    }
    impl RandomSource for Fixed {
        fn below(&mut self, _: u32) -> u32 {
            0
        }
    }
    #[test]
    fn stationary_wander_retreat_and_regen_interval() {
        let spawn = CoordGrid::new(3203, 3203, 0).unwrap();
        let mut n = Npc::new(1, spawn, 1, Restrict::Normal, BlockWalk::Npc).unwrap();
        let mut map = CollisionMap::default();
        map.allocate(3200, 3200, 0);
        n.movement.coords = CoordGrid::new(3205, 3203, 0).unwrap();
        n.movement.occupancy(&mut map, true);
        n.wander(&mut map, 1, &mut Fixed).unwrap();
        n.process_movement(&mut map, 1);
        assert_eq!(n.movement.coords.x(), 3204);
        n.stats[3] = 1;
        n.base_stats[3] = 10;
        n.regen_rate = 2;
        n.regenerate();
        assert_eq!(n.stats[3], 2);
        n.regenerate();
        n.regenerate();
        assert_eq!(n.stats[3], 2);
        n.regenerate();
        assert_eq!(n.stats[3], 3);
    }
    #[test]
    fn npcs_respect_player_occupancy_unless_pass_through() {
        let from = CoordGrid::new(3201, 3201, 0).unwrap();
        let to = CoordGrid::new(3202, 3201, 0).unwrap();
        for restrict in [Restrict::Normal, Restrict::PassThru] {
            let mut map = CollisionMap::default();
            map.allocate(3200, 3200, 0);
            map.add(to.x(), to.z(), 0, 0x80000);
            let mut n = Npc::new(1, from, 1, restrict, BlockWalk::Npc).unwrap();
            n.movement.occupancy(&mut map, true);
            n.move_to(to, Speed::Walk).unwrap();
            n.process_movement(&mut map, 1);
            assert_eq!(n.movement.coords == to, restrict == Restrict::PassThru);
        }
    }
}
