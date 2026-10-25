//! Ordinary firemaking: OpenRune BurnLogEvents over a weak queue. The log is
//! dropped (or already lies) on the fire tile, every attempt rolls the Wiki
//! success curve, and a lit fire is a timed location that leaves ashes.
use crate::{collision::CollisionMap, locations::Locations, CoordGrid};
use std::collections::BTreeMap;
pub(crate) const QUEUE: i32 = -2391018;
/// BurnLogEvents re-queues `firemaking_light` four cycles after each attempt.
pub const ATTEMPT_TICKS: i32 = 4;
pub const ATTEMPT: &str = "You attempt to light the logs.";
pub const CATCHES: &str = "The fire catches and the logs begin to burn.";
pub const NO_SOURCE: &str = "You do not have any fire source to light this.";
pub const BLOCKED: &str = "You can't light a fire here.";
#[derive(Clone, Debug)]
pub struct Log {
    pub level: u8,
    /// Wiki base XP in tenths (mahogany 157.5, yew 202.5); the player rate applies x5.
    pub xp_tenths: u32,
    pub members: bool,
    /// Lower-case item name for the level refusal, e.g. "oak logs".
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct Config {
    pub tinderbox: i32,
    pub logs: BTreeMap<i32, Log>,
    pub fire: u32,
    pub ashes: i32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    /// Skilling success low/high: 65/256 at level 1, 513/256 at 99.
    pub success: (i32, i32),
    /// Inclusive random fire lifetime in ticks.
    pub fire_ticks: [i32; 2],
    /// Lifetime of the dropped log and of the ashes on the ground.
    pub object_ticks: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    NoTinderbox,
    Level(u8),
    Tile,
}
impl Config {
    pub fn refusal_text(&self, log: i32, refusal: Refusal) -> String {
        match refusal {
            Refusal::NoTinderbox => NO_SOURCE.into(),
            Refusal::Tile => BLOCKED.into(),
            Refusal::Level(level) => format!(
                "You need a Firemaking level of {level} to burn {}.",
                self.logs.get(&log).map_or("these logs", |l| l.name.as_str())
            ),
        }
    }
    /// BurnLogEvents.canBurn order: fire source, level, then the fire tile.
    pub fn check(
        &self,
        log: &Log,
        has_tinderbox: bool,
        level: u8,
        tile_open: bool,
    ) -> Result<(), Refusal> {
        if !has_tinderbox {
            return Err(Refusal::NoTinderbox);
        }
        if level < log.level {
            return Err(Refusal::Level(log.level));
        }
        if !tile_open {
            return Err(Refusal::Tile);
        }
        Ok(())
    }
    pub fn success_threshold(&self, level: u8) -> f64 {
        crate::skills::success_rate(
            self.success.0,
            self.success.1,
            i32::from(level).clamp(1, 99),
            99,
        )
        .clamp(0.0, 1.0)
            * 256.0
    }
}
/// A fire needs empty ground: no wall, centrepiece or ground decoration on the
/// tile (Wiki Fire: "any part of scenery"), and no blocking footprint or floor.
/// Wall decorations hang on an edge and do not occupy the ground.
pub fn tile_open(locations: &Locations, collision: &CollisionMap, coords: CoordGrid) -> bool {
    collision.get(coords.x(), coords.z(), coords.level()) & (0x100 | 0x40000 | 0x200000) == 0
        && !locations
            .all(coords)
            .iter()
            .any(|l| l.coords == coords && l.layer != 1)
}
/// Firemaking.walkDirections: west, east, south, north from the fire tile.
pub fn step_away(collision: &CollisionMap, fire: CoordGrid) -> Option<CoordGrid> {
    [(-1, 0), (1, 0), (0, -1), (0, 1)]
        .into_iter()
        .find(|&(dx, dz)| {
            collision.can_step(fire, dx, dz, 1, 0, crate::collision::Strategy::Normal)
        })
        .and_then(|(dx, dz)| fire.translate(dx, dz, 0).ok())
}
/// Fires lit by players; a timed despawn of one of these leaves ashes.
#[derive(Default)]
pub struct Fires(Vec<crate::locations::Location>);
impl Fires {
    pub fn lit(&mut self, fire: crate::locations::Location) {
        self.0.push(fire);
    }
    pub fn expired(&mut self, loc: crate::locations::Location) -> bool {
        let Some(index) = self.0.iter().position(|&f| f == loc) else {
            return false;
        };
        self.0.remove(index);
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn config() -> Config {
        let log = |level, xp_tenths, members, name: &str| Log {
            level,
            xp_tenths,
            members,
            name: name.into(),
        };
        Config {
            tinderbox: 590,
            logs: [
                (1511, log(1, 400, false, "logs")),
                (1521, log(15, 600, false, "oak logs")),
                (6333, log(35, 1050, true, "teak logs")),
                (1515, log(60, 2025, false, "yew logs")),
            ]
            .into(),
            fire: 26185,
            ashes: 592,
            animation: 733,
            skill: 11,
            max_level: 99,
            success: (64, 512),
            fire_ticks: [100, 198],
            object_ticks: 200,
        }
    }
    #[test]
    fn wiki_success_curve_and_refusal_order() {
        let c = config();
        // Firemaking rev 15367339: 65/256 at 1, 256/256 reached at 43, 513/256 at 99.
        assert_eq!(c.success_threshold(1), 65.0);
        assert!(c.success_threshold(42) < 256.0);
        assert_eq!(c.success_threshold(43), 256.0);
        assert_eq!(c.success_threshold(99), 256.0);
        let yew = &c.logs[&1515];
        assert_eq!(c.check(yew, false, 1, false), Err(Refusal::NoTinderbox));
        assert_eq!(c.check(yew, true, 59, false), Err(Refusal::Level(60)));
        assert_eq!(c.check(yew, true, 60, false), Err(Refusal::Tile));
        assert_eq!(c.check(yew, true, 60, true), Ok(()));
        assert_eq!(
            c.refusal_text(1521, Refusal::Level(15)),
            "You need a Firemaking level of 15 to burn oak logs."
        );
    }
    #[test]
    fn steps_west_then_east_south_north() {
        let mut m = CollisionMap::default();
        m.allocate(3200, 3200, 0);
        let fire = CoordGrid::new(3203, 3203, 0).unwrap();
        assert_eq!(step_away(&m, fire), fire.translate(-1, 0, 0).ok());
        for (blocked, next) in [((-1, 0), (1, 0)), ((1, 0), (0, -1)), ((0, -1), (0, 1))] {
            let tile = fire.translate(blocked.0, blocked.1, 0).unwrap();
            m.add(tile.x(), tile.z(), 0, 0x100);
            assert_eq!(step_away(&m, fire), fire.translate(next.0, next.1, 0).ok());
        }
        let north = fire.translate(0, 1, 0).unwrap();
        m.add(north.x(), north.z(), 0, 0x100);
        assert_eq!(step_away(&m, fire), None);
    }
    #[test]
    fn tile_needs_empty_ground() {
        use crate::locations::Location;
        let mut m = CollisionMap::default();
        m.allocate(3200, 3200, 0);
        let mut locs = Locations::default();
        let tile = CoordGrid::new(3201, 3201, 0).unwrap();
        assert!(tile_open(&locs, &m, tile));
        // Wall decoration on the tile's edge does not occupy the ground.
        locs.load(Location::new(tile, 7, 4, 0).unwrap());
        assert!(tile_open(&locs, &m, tile));
        for shape in [0, 10, 22] {
            let mut l = Locations::default();
            l.load(Location::new(tile, 7, shape, 0).unwrap());
            assert!(!tile_open(&l, &m, tile), "shape {shape}");
        }
        for flag in [0x100, 0x40000, 0x200000] {
            let mut blocked = m.clone();
            blocked.add(tile.x(), tile.z(), 0, flag);
            assert!(!tile_open(&locs, &blocked, tile), "flag {flag:x}");
        }
        // Unloaded zones are never open ground.
        assert!(!tile_open(&locs, &m, CoordGrid::new(3300, 3300, 0).unwrap()));
    }
}
