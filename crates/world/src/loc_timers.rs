//! Normal-world LocRepository duration queues. Reverts use exact entity identity,
//! and respawns precede despawns even when their deadlines are equal.
use crate::{
    locations::{Changed, Location},
    tick::MapClock,
};
#[derive(Default)]
pub struct Timers {
    additions: Vec<(Location, i32)>,
    deletions: Vec<(Location, i32)>,
    replacements: Vec<(Location, Location, u64, i32)>,
}
impl Timers {
    pub fn replace(
        &mut self,
        original: Location,
        replacement: Location,
        revision: u64,
        clock: MapClock,
        duration: i32,
    ) {
        self.replacements
            .push((original, replacement, revision, clock.deadline(duration)));
    }
    pub fn replacements_due(
        &mut self,
        clock: MapClock,
        locations: &crate::locations::Locations,
    ) -> Vec<Location> {
        let mut restore = vec![];
        self.replacements
            .retain(|&(original, replacement, revision, at)| {
                if clock.0 < at {
                    return true;
                }
                if locations.revision(replacement) == Some(revision) {
                    restore.push(original);
                }
                false
            });
        restore
    }

    pub fn record(
        &mut self,
        loc: Location,
        add: bool,
        changed: Changed,
        clock: MapClock,
        duration: i32,
    ) {
        if duration == i32::MAX
            || if add {
                changed != Changed::Spawned
            } else {
                changed != Changed::Map
            }
        {
            return;
        }
        // Kotlin removes only the first matching entity, not every object on its layer.
        for queue in [&mut self.additions, &mut self.deletions] {
            if let Some(index) = queue.iter().position(|(v, _)| {
                v.coords == loc.coords
                    && v.id == loc.id
                    && v.shape == loc.shape
                    && v.angle == loc.angle
            }) {
                queue.remove(index);
            }
        }
        let queue = if add {
            &mut self.additions
        } else {
            &mut self.deletions
        };
        queue.push((loc, clock.deadline(duration)));
    }
    pub fn due(&mut self, clock: MapClock) -> Vec<(Location, bool)> {
        let mut due = Vec::new();
        for (queue, add) in [(&mut self.deletions, true), (&mut self.additions, false)] {
            let mut index = 0;
            while index < queue.len() {
                if clock.0 >= queue[index].1 {
                    due.push((queue.remove(index).0, add));
                } else {
                    index += 1;
                }
            }
        }
        due
    }
}
