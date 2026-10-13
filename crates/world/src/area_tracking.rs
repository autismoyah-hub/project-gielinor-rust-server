//! PlayerAreaProcessor's ordered transitions. The caller supplies normalized
//! instance coordinates and queues the returned hooks behind protected access.
use crate::{areas::Areas, CoordGrid};
use std::collections::BTreeMap;

/// RegionRegistry's copied-zone lookup, shared by deterministic agents and area hooks.
#[derive(Clone, Debug, Default)]
pub struct Sources(BTreeMap<(i32, i32, i32), crate::tower_layout::Source>);
impl Sources {
    pub fn register(
        &mut self,
        origin: CoordGrid,
        template: &crate::tower_layout::Template,
    ) -> Result<(), &'static str> {
        if origin.level() != 0 || origin.x() % 8 != 0 || origin.z() % 8 != 0 {
            return Err("Invalid area source origin");
        }
        let mut prepared = BTreeMap::new();
        for (&(x, z, level), &source) in template {
            if source.rotation > 3
                || source.level > 3
                || source.x > 1023
                || source.z > 2047
                || level > 3
            {
                return Err("Invalid copied area source");
            }
            let destination = origin
                .translate(x as i32 * 8, z as i32 * 8, level as i32)
                .map_err(|_| "Area source outside world")?;
            let key = (
                destination.x() >> 3,
                destination.z() >> 3,
                destination.level(),
            );
            if self.0.contains_key(&key) {
                return Err("Area source already registered");
            }
            prepared.insert(key, source);
        }
        self.0.extend(prepared);
        Ok(())
    }
    pub fn normalize(&self, position: CoordGrid) -> CoordGrid {
        // RegionRegistry.START_COORD_X includes its 32-tile padding.
        if position.x() < 6432 {
            return position;
        }
        let Some(source) = self
            .0
            .get(&(position.x() >> 3, position.z() >> 3, position.level()))
        else {
            return CoordGrid::NULL;
        };
        // A zero packed RegionZoneCopy equals the map's default return value.
        if source.x == 0 && source.z == 0 && source.level == 0 && source.rotation == 0 {
            return CoordGrid::NULL;
        }
        let (x, z) = crate::region_rotation::translate(
            (4 - source.rotation) & 3,
            position.x() & 7,
            position.z() & 7,
            8,
            8,
            1,
            1,
        )
        .expect("validated copy rotation");
        CoordGrid::new(
            source.x as i32 * 8 + x,
            source.z as i32 * 8 + z,
            source.level as i32,
        )
        .expect("validated source zone")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Enter(u16),
    Exit(u16),
}
#[derive(Clone, Debug, Default)]
pub struct State {
    last_position: Option<CoordGrid>,
    active: Vec<u16>,
}
impl State {
    pub fn active(&self) -> &[u16] {
        &self.active
    }
    pub fn process(
        &mut self,
        position: CoordGrid,
        normalized: CoordGrid,
        force_exit: bool,
        index: &Areas,
    ) -> Vec<Event> {
        if !force_exit && self.last_position == Some(position) {
            return Vec::new();
        }
        self.last_position = Some(position);
        let pending = if force_exit {
            Vec::new()
        } else {
            index.at(normalized)
        };
        let mut events = Vec::new();
        for &area in &self.active {
            if !pending.contains(&area) {
                events.push(Event::Exit(area));
            }
        }
        for &area in &pending {
            if !self.active.contains(&area) {
                events.push(Event::Enter(area));
            }
        }
        self.active.clear();
        // Kotlin pendingAreas is a list, activeAreas is an insertion-ordered set.
        // Duplicate pending memberships can therefore emit duplicate first enters.
        for area in pending {
            if !self.active.contains(&area) {
                self.active.push(area);
            }
        }
        events
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_transitions_retain_duplicate_enters_but_unique_exits() {
        let a = CoordGrid::new(3200, 3200, 0).unwrap();
        let b = CoordGrid::new(3201, 3200, 0).unwrap();
        let mut index = Areas::default();
        index.register_coord(a, [7, 9]).unwrap();
        index.register_coord(b, [8]).unwrap();
        index.register_zone(a, [7]).unwrap();
        let mut state = State::default();
        assert_eq!(
            state.process(a, a, false, &index),
            [Event::Enter(7), Event::Enter(9), Event::Enter(7)]
        );
        assert_eq!(state.active(), [7, 9]);
        assert!(state.process(a, b, false, &index).is_empty());
        assert_eq!(
            state.process(b, b, false, &index),
            [Event::Exit(9), Event::Enter(8)]
        );
        assert_eq!(state.active(), [8, 7]);
        assert_eq!(
            state.process(b, b, true, &index),
            [Event::Exit(8), Event::Exit(7)]
        );
        assert!(state.process(b, b, true, &index).is_empty());
        // Logout retained the coordinate; no artificial re-entry without movement.
        assert!(state.process(b, b, false, &index).is_empty());
    }
    #[test]
    fn normalized_instances_use_source_area_membership() {
        let source = CoordGrid::new(3200, 3200, 0).unwrap();
        let instance = CoordGrid::new(10424, 120, 0).unwrap();
        let mut index = Areas::default();
        index.register_coord(source, [42]).unwrap();
        assert_eq!(
            State::default().process(instance, source, false, &index),
            [Event::Enter(42)]
        );
    }
    #[test]
    fn copies_normalize_inverse_rotations_and_reject_partial_registration() {
        use crate::tower_layout::{Source, Template};
        let origin = CoordGrid::new(10400, 64, 0).unwrap();
        let mut sources = Sources::default();
        let template: Template = (0..4)
            .map(|rotation| {
                (
                    (rotation, 0, 0),
                    Source {
                        x: 400,
                        z: 401,
                        level: 2,
                        rotation,
                    },
                )
            })
            .collect();
        sources.register(origin, &template).unwrap();
        for (rotation, (x, z)) in [(2, 5), (2, 2), (5, 2), (5, 5)].into_iter().enumerate() {
            let point = origin.translate(rotation as i32 * 8 + 2, 5, 0).unwrap();
            assert_eq!(
                sources.normalize(point),
                CoordGrid::new(3200 + x, 3208 + z, 2).unwrap()
            );
        }
        assert_eq!(
            sources.normalize(CoordGrid::new(3200, 3200, 0).unwrap()),
            CoordGrid::new(3200, 3200, 0).unwrap()
        );
        assert_eq!(
            sources.normalize(origin.translate(40, 0, 0).unwrap()),
            CoordGrid::NULL
        );
        let invalid = Template::from([
            (
                (4, 0, 0),
                Source {
                    x: 400,
                    z: 401,
                    level: 2,
                    rotation: 0,
                },
            ),
            (
                (5, 0, 0),
                Source {
                    x: 400,
                    z: 401,
                    level: 2,
                    rotation: 4,
                },
            ),
        ]);
        assert!(sources.register(origin, &invalid).is_err());
        assert_eq!(
            sources.normalize(origin.translate(32, 0, 0).unwrap()),
            CoordGrid::NULL
        );
    }
}
