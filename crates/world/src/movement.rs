//! Player movement mechanics from PlayerMovementProcessor and StepFactory.
use crate::{
    collision::{CollisionMap, Strategy},
    CoordGrid,
};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Stationary,
    Crawl,
    Walk,
    Run,
}
impl Speed {
    pub fn wire(self) -> u8 {
        match self {
            Self::Stationary => 127,
            Self::Crawl => 0,
            Self::Walk => 1,
            Self::Run => 2,
        }
    }
}

pub fn validated_step(
    map: &CollisionMap,
    from: CoordGrid,
    to: CoordGrid,
    size: i32,
    extra: u32,
    strategy: Strategy,
) -> Option<CoordGrid> {
    if from == to {
        return None;
    }
    let dx = (to.x() - from.x()).signum();
    let dz = (to.z() - from.z()).signum();
    for (x, z) in [(dx, dz), (dx, 0), (0, dz)] {
        if map.can_step(from, x, z, size, extra, strategy) {
            return CoordGrid::new(from.x() + x, from.z() + z, from.level()).ok();
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct Movement {
    pub coords: CoordGrid,
    pub last_processed: CoordGrid,
    pub last_movement: i32,
    pub speed: Speed,
    pub cached_speed: Speed,
    pub temporary_speed: Option<Speed>,
    pub pending_steps: u8,
    pub pending_teleport: bool,
    pub pending_telejump: bool,
    pub size: i32,
    pub hidden: bool,
    pub rooted: bool,
    pub collision_mask: Option<u32>,
    pub step_extra: u32,
    pub strategy: Strategy,
    pub player_rules: bool,
    pub modal_open: bool,
    pub pending_queue: bool,
    pub interacting: bool,
    waypoints: VecDeque<CoordGrid>,
}
impl Movement {
    pub fn new(coords: CoordGrid) -> Self {
        Self {
            coords,
            last_processed: CoordGrid::NULL,
            last_movement: -1,
            speed: Speed::Walk,
            cached_speed: Speed::Walk,
            temporary_speed: None,
            pending_steps: 0,
            pending_teleport: false,
            pending_telejump: false,
            size: 1,
            hidden: false,
            rooted: false,
            collision_mask: Some(0x80000),
            step_extra: 0x100000,
            strategy: Strategy::Normal,
            player_rules: true,
            modal_open: false,
            pending_queue: false,
            interacting: false,
            waypoints: VecDeque::new(),
        }
    }
    pub fn destination(&self) -> Option<CoordGrid> {
        self.waypoints.back().copied()
    }
    pub fn waypoint_count(&self) -> usize {
        self.waypoints.len()
    }
    /// Routes come from the server pathfinder; client coordinates are never treated as steps.
    pub fn route(&mut self, waypoints: impl IntoIterator<Item = CoordGrid>, active: Speed) {
        if self.rooted {
            self.abort();
            return;
        }
        self.waypoints = waypoints.into_iter().collect();
        self.cached_speed = self.temporary_speed.unwrap_or(active);
        self.speed = self.cached_speed;
    }
    pub fn abort(&mut self) {
        self.waypoints.clear();
        self.temporary_speed = None;
    }
    pub fn occupancy(&self, map: &mut CollisionMap, add: bool) {
        if self.hidden {
            return;
        }
        let Some(mask) = self.collision_mask else {
            return;
        };
        // Kotlin players block NPCs; their step validation checks BLOCK_PLAYERS.
        for x in 0..self.size {
            for z in 0..self.size {
                if add {
                    map.add(
                        self.coords.x() + x,
                        self.coords.z() + z,
                        self.coords.level(),
                        mask,
                    )
                } else {
                    map.remove(
                        self.coords.x() + x,
                        self.coords.z() + z,
                        self.coords.level(),
                        mask,
                    )
                }
            }
        }
    }
    pub fn process(&mut self, map: &mut CollisionMap, clock: i32) {
        if self.rooted {
            self.abort();
            return;
        }
        let moved_previous = self.last_movement == clock.wrapping_sub(1);
        let halted = self.player_rules
            && !(self.interacting && moved_previous)
            && self.modal_open
            && self.pending_queue;
        if self.waypoints.is_empty() || halted || self.speed == Speed::Stationary {
            return;
        }
        let steps = if self.speed == Speed::Crawl && !moved_previous {
            1
        } else {
            self.speed.wire()
        };
        let start = self.coords;
        let mut current = start;
        let mut target = *self.waypoints.front().unwrap();
        let mut completed = 0;
        self.occupancy(map, false);
        for _ in 0..steps {
            if current == target {
                let Some(next) = self.waypoints.pop_front() else {
                    break;
                };
                target = next;
                if current == target {
                    let Some(next) = self.waypoints.front() else {
                        break;
                    };
                    target = *next;
                }
            }
            let Some(step) = validated_step(
                map,
                current,
                target,
                self.size,
                self.step_extra,
                self.strategy,
            ) else {
                break;
            };
            self.last_processed = current;
            current = step;
            completed += 1;
        }
        if current == target {
            self.waypoints.pop_front();
        }
        self.coords = current;
        self.occupancy(map, true);
        if current != start {
            self.last_movement = clock;
        }
        self.pending_steps = completed;
        if self.player_rules && self.speed == Speed::Run && completed == 1 {
            self.speed = Speed::Walk;
        }
        if self.waypoints.is_empty() {
            self.temporary_speed = None;
        }
    }
    pub fn finish_tick(&mut self) {
        self.pending_steps = 0;
        self.pending_teleport = false;
        self.pending_telejump = false;
    }
    pub fn teleport(
        &mut self,
        map: &mut CollisionMap,
        dest: CoordGrid,
        clock: i32,
        jump: bool,
    ) -> Result<(), &'static str> {
        if dest == CoordGrid::NULL || !map.allocated(dest.x(), dest.z(), dest.level()) {
            return Err("Teleport destination has no allocated zone");
        }
        self.occupancy(map, false);
        self.coords = dest;
        self.last_processed = CoordGrid::NULL;
        self.abort();
        self.interacting = false;
        self.occupancy(map, true);
        if jump {
            self.pending_telejump = true;
            self.speed = Speed::Stationary;
        } else {
            self.pending_teleport = true;
            self.speed = Speed::Walk;
            self.last_movement = clock;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RunModifiers {
    pub weight_grams: i32,
    pub agility: i32,
    pub stamina: bool,
    pub improved_stamina: bool,
    pub graceful_rate: i32,
    pub graceful_pieces: u8,
}
pub fn update_energy(energy: i32, steps: u8, m: RunModifiers) -> i32 {
    if steps > 1 {
        let weight = (m.weight_grams / 1000).clamp(0, 64);
        let base = 60 + 67 * weight / 64;
        let mut loss = base * (300 - m.agility) / 300;
        if m.improved_stamina {
            loss = loss * 85 / 100;
        } else if m.stamina {
            loss = loss * 30 / 100;
        }
        (energy - loss).max(0)
    } else if energy >= 10000 {
        energy
    } else {
        let worn = if m.graceful_pieces == 6 {
            3000
        } else {
            m.graceful_rate
        };
        let recover = ((1.0 + worn as f64 / 10000.0) * (15 + m.agility / 10) as f64) as i32;
        (energy + recover).min(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Movement, CollisionMap) {
        let p = Movement::new(CoordGrid::new(3200, 3200, 0).unwrap());
        let mut map = CollisionMap::default();
        map.allocate(3200, 3200, 0);
        p.occupancy(&mut map, true);
        (p, map)
    }
    #[test]
    fn crawl_alternates_and_running_tracks_intermediate_tile() {
        let (mut p, mut map) = setup();
        p.route([CoordGrid::new(3205, 3200, 0).unwrap()], Speed::Crawl);
        p.process(&mut map, 1);
        assert_eq!(p.coords.x(), 3201);
        p.finish_tick();
        p.process(&mut map, 2);
        assert_eq!(p.coords.x(), 3201);
        p.process(&mut map, 3);
        assert_eq!(p.coords.x(), 3202);
        p.route([CoordGrid::new(3205, 3200, 0).unwrap()], Speed::Run);
        p.process(&mut map, 4);
        assert_eq!(p.coords.x(), 3204);
        assert_eq!(p.last_processed.x(), 3203);
        p.process(&mut map, 5);
        assert_eq!(p.speed, Speed::Walk);
        assert_eq!(p.destination(), None);
    }
    #[test]
    fn diagonal_falls_back_to_horizontal_and_blocked_route_is_retained() {
        let (mut p, mut map) = setup();
        map.add(3201, 3201, 0, 0x200000);
        p.route([CoordGrid::new(3201, 3201, 0).unwrap()], Speed::Walk);
        p.process(&mut map, 1);
        assert_eq!(p.coords, CoordGrid::new(3201, 3200, 0).unwrap());
        p.finish_tick();
        p.process(&mut map, 2);
        assert_eq!(p.pending_steps, 0);
        assert!(p.destination().is_some());
        p.rooted = true;
        p.process(&mut map, 3);
        assert!(p.destination().is_none());
    }
    #[test]
    fn run_energy_uses_integer_rounding_and_stamina_precedence() {
        let m = RunModifiers {
            agility: 99,
            weight_grams: 64000,
            stamina: true,
            improved_stamina: true,
            ..Default::default()
        };
        assert_eq!(update_energy(10000, 2, m), 9928);
        assert_eq!(
            update_energy(
                0,
                0,
                RunModifiers {
                    agility: 99,
                    graceful_pieces: 6,
                    ..Default::default()
                }
            ),
            31
        );
    }
}
