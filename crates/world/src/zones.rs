//! Ground-object portion of PlayerZoneUpdateProcessor. Dynamic locations and
//! other zone effects can join the same ordered enclosure stream.
use crate::{
    ground::{self, Event, Object, Objects, Scope},
    CoordGrid,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Update {
    Graphic(Graphic),
    Sound(Sound),
    Projectile(crate::projectile::Flight),
    Add(Object),
    Delete(Object),
    Count { object: Object, old: i32 },
    Location(crate::locations::Event),
}
#[derive(Clone, Debug)]
pub enum Change {
    Graphic(Graphic),
    Sound(Sound),
    Projectile(crate::projectile::Flight),
    Ground(Event),
    Location(crate::locations::Event),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graphic {
    pub coords:CoordGrid,
    pub id:u16,
    pub height:u8,
    pub delay:u16,
    /// None is public. A character receiver uses the existing observer identity.
    pub receiver:Option<i64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sound {
    pub coords: CoordGrid,
    pub id: i32,
    pub delay: i32,
    pub loops: i32,
    pub radius: i32,
    pub size: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Emission {
    Reset(CoordGrid),
    Enclosed {
        base: CoordGrid,
        updates: Vec<Update>,
    },
}
#[derive(Clone, Default)]
pub struct Observer {
    previous: Option<CoordGrid>,
    visible: Vec<CoordGrid>,
}
impl Observer {
    pub fn visible(&self) -> &[CoordGrid] {
        &self.visible
    }
    pub fn cycle(
        &mut self,
        coords: CoordGrid,
        build_area: CoordGrid,
        receiver: Option<i64>,
        objects: &Objects,
        events: &[Event],
    ) -> Result<Vec<Emission>, &'static str> {
        self.cycle_all(
            coords,
            build_area,
            receiver,
            objects,
            &crate::locations::Locations::default(),
            &events
                .iter()
                .cloned()
                .map(Change::Ground)
                .collect::<Vec<_>>(),
        )
    }
    pub fn cycle_all(
        &mut self,
        coords: CoordGrid,
        build_area: CoordGrid,
        receiver: Option<i64>,
        objects: &Objects,
        locations: &crate::locations::Locations,
        events: &[Change],
    ) -> Result<Vec<Emission>, &'static str> {
        if coords == CoordGrid::NULL || build_area == CoordGrid::NULL {
            return Err("Invalid zone observation coordinates");
        }
        let zone = ground::zone_base(coords);
        let mut output = Vec::new();
        let old = if self.previous != Some(zone) {
            // ZoneKey.translate validates instead of wrapping either 11-bit axis.
            if !(3..=2044).contains(&(zone.x() >> 3)) || !(3..=2044).contains(&(zone.z() >> 3)) {
                return Err("Zone neighbourhood outside map bounds");
            }
            let mut current = Vec::with_capacity(49);
            for x in -3..=3 {
                for z in -3..=3 {
                    let base = CoordGrid::new(
                        ((zone.x() >> 3) + x) << 3,
                        ((zone.z() >> 3) + z) << 3,
                        zone.level(),
                    )
                    .unwrap();
                    if (0..104).contains(&(base.x() - build_area.x()))
                        && (0..104).contains(&(base.z() - build_area.z()))
                    {
                        current.push(base);
                    }
                }
            }
            for &base in current.iter().filter(|base| !self.visible.contains(base)) {
                output.push(Emission::Reset(base));
                let mut updates = locations
                    .spawned(base)
                    .map(|v| {
                        Update::Location(if v.id == crate::locations::DELETED_ID {
                            crate::locations::Event::Delete(v)
                        } else {
                            crate::locations::Event::Add(v)
                        })
                    })
                    .collect::<Vec<_>>();
                updates.extend(
                    objects
                        .in_zone(base)
                        .map(|h| h.snapshot())
                        .filter(|obj| obj.visible_to(receiver))
                        .map(Update::Add),
                );
                if !updates.is_empty() {
                    output.push(Emission::Enclosed { base, updates });
                }
            }
            let old = current
                .iter()
                .filter(|base| self.visible.contains(base))
                .copied()
                .collect::<Vec<_>>();
            self.visible = current;
            old
        } else {
            self.visible.clone()
        };
        for base in old {
            let mut private = Vec::new();
            let mut shared = Vec::new();
            for event in events {
                let event = match event {
                    Change::Graphic(graphic)=>{
                        if ground::zone_base(graphic.coords)==base && (graphic.receiver.is_none() || graphic.receiver==receiver){shared.push(Update::Graphic(*graphic));}
                        continue;
                    }
                    Change::Sound(sound) => {
                        if ground::zone_base(sound.coords) == base {
                            shared.push(Update::Sound(*sound));
                        }
                        continue;
                    }
                    Change::Projectile(flight) => {
                        if ground::zone_base(flight.start) == base {
                            shared.push(Update::Projectile(*flight));
                        }
                        continue;
                    }
                    Change::Ground(v) => v,
                    Change::Location(v) => {
                        if ground::zone_base(v.location().coords) == base {
                            shared.push(Update::Location(*v));
                        }
                        continue;
                    }
                };
                let object = match event {
                    Event::Added(_, obj)
                    | Event::Deleted(_, obj)
                    | Event::Revealed(_, obj)
                    | Event::Count(_, obj, _) => obj,
                };
                if ground::zone_base(object.coords) != base {
                    continue;
                }
                match event {
                    Event::Added(_, obj) | Event::Deleted(_, obj) => {
                        let update = if matches!(event, Event::Added(..)) {
                            Update::Add(obj.clone())
                        } else {
                            Update::Delete(obj.clone())
                        };
                        if obj.scope == Scope::Private {
                            if obj.visible_to(receiver) {
                                private.push(update);
                            }
                        } else {
                            shared.push(update);
                        }
                    }
                    Event::Count(_, obj, old) => {
                        if obj.visible_to(receiver) {
                            private.push(Update::Count {
                                object: obj.clone(),
                                old: *old,
                            });
                        }
                    }
                    Event::Revealed(_, obj) => {
                        // Kotlin compares to the raw non-null receiverId sentinel here.
                        if receiver != Some(obj.receiver.unwrap_or(i64::MIN)) {
                            private.push(Update::Add(obj.clone()));
                        }
                    }
                }
            }
            // Observer-specific buffers precede shared buffers even if individual
            // public mutations occurred first in the original update list.
            for updates in [private, shared] {
                if !updates.is_empty() {
                    output.push(Emission::Enclosed { base, updates });
                }
            }
        }
        self.previous = Some(zone);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Catalog, tick::MapClock};
    fn pos(x: i32, z: i32) -> CoordGrid {
        CoordGrid::new(x, z, 0).unwrap()
    }
    #[test]
    fn sounds_follow_existing_zone_observers_without_persistent_replay() {
        let coords = pos(3200, 3200);
        let build = pos(3152, 3152);
        let sound = Sound {
            coords,
            id: 50,
            delay: 40,
            loops: 1,
            radius: 10,
            size: 0,
        };
        let mut observer = Observer::default();
        let objects = Objects::default();
        let locations = crate::locations::Locations::default();
        let changes = [Change::Sound(sound)];
        let first = observer
            .cycle_all(coords, build, None, &objects, &locations, &changes)
            .unwrap();
        assert!(first.iter().all(|e| matches!(e, Emission::Reset(_))));
        assert_eq!(
            observer
                .cycle_all(coords, build, None, &objects, &locations, &changes)
                .unwrap(),
            vec![Emission::Enclosed {
                base: coords,
                updates: vec![Update::Sound(sound)]
            }]
        );
        assert!(observer
            .cycle_all(coords, build, None, &objects, &locations, &[])
            .unwrap()
            .is_empty());
        let other = Sound {
            coords: pos(3400, 3400),
            ..sound
        };
        assert!(observer
            .cycle_all(
                coords,
                build,
                None,
                &objects,
                &locations,
                &[Change::Sound(other)]
            )
            .unwrap()
            .is_empty());
    }
    #[test]
    fn projectile_is_transient_and_selected_by_source_zone_not_target() {
        use crate::projectile::{EntityIndex, Flight, Style};
        let style = Style {
            start_height: 172,
            end_height: 124,
            delay: 51,
            angle: 16,
            length_adjustment: -5,
            progress: 64,
            step_multiplier: 10,
        };
        let flight = Flight {
            spotanim: 91,
            style,
            duration: style.duration(2),
            source: EntityIndex::Player(1),
            target: EntityIndex::Npc(2),
            start: pos(3200, 3200),
            end: pos(3300, 3300),
        };
        let mut observer = Observer::default();
        let objects = Objects::default();
        let locations = crate::locations::Locations::default();
        let build = pos(3152, 3152);
        let changes = [Change::Projectile(flight)];
        let first = observer
            .cycle_all(flight.start, build, Some(7), &objects, &locations, &changes)
            .unwrap();
        assert_eq!(first.len(), 49);
        assert!(first.iter().all(|e| matches!(e, Emission::Reset(_))));
        let visible = observer
            .cycle_all(flight.start, build, Some(7), &objects, &locations, &changes)
            .unwrap();
        assert_eq!(
            visible,
            vec![Emission::Enclosed {
                base: flight.start,
                updates: vec![Update::Projectile(flight)]
            }]
        );
        assert!(observer
            .cycle_all(flight.start, build, Some(7), &objects, &locations, &[])
            .unwrap()
            .is_empty());
        let elsewhere = Flight {
            start: flight.end,
            end: flight.start,
            ..flight
        };
        assert!(observer
            .cycle_all(
                flight.start,
                build,
                Some(7),
                &objects,
                &locations,
                &[Change::Projectile(elsewhere)]
            )
            .unwrap()
            .is_empty());
    }
    fn object(x: i32, z: i32, scope: Scope, receiver: Option<i64>) -> Object {
        Object {
            coords: pos(x, z),
            item: 1,
            count: 1,
            scope,
            created: 0,
            receiver,
            owner: receiver,
        }
    }
    #[test]
    fn new_zones_send_ordered_persistent_objects_once_and_changes_are_per_observer() {
        let mut objects = Objects::default();
        let catalog = Catalog::default();
        let center = pos(3200, 3200);
        let build = pos(3152, 3152);
        let public = object(3207, 3207, Scope::Temporary, None);
        let private = object(3200, 3200, Scope::Private, Some(10));
        objects
            .add(public.clone(), &catalog, MapClock(0), i32::MAX, 100)
            .unwrap();
        objects
            .add(private.clone(), &catalog, MapClock(0), i32::MAX, 100)
            .unwrap();
        let events = objects.drain_events();
        let mut owner = Observer::default();
        let mut other = Observer::default();
        let initial = owner
            .cycle(center, build, Some(10), &objects, &events)
            .unwrap();
        assert_eq!(
            initial
                .iter()
                .filter(|e| matches!(e, Emission::Reset(_)))
                .count(),
            49
        );
        assert_eq!(
            initial
                .iter()
                .filter_map(|e| match e {
                    Emission::Enclosed { updates, .. } => Some(updates),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![&vec![
                Update::Add(public.clone()),
                Update::Add(private.clone())
            ]]
        );
        let initial_other = other
            .cycle(center, build, Some(20), &objects, &events)
            .unwrap();
        assert_eq!(
            initial_other
                .iter()
                .filter_map(|e| match e {
                    Emission::Enclosed { updates, .. } => Some(updates.len()),
                    _ => None,
                })
                .sum::<usize>(),
            1
        );
        let mut revealed = private.clone();
        revealed.scope = Scope::Temporary;
        let changes = [
            Event::Deleted(1, public.clone()),
            Event::Revealed(2, revealed.clone()),
            Event::Count(2, private.clone(), 2),
        ];
        let own = owner
            .cycle(center, build, Some(10), &objects, &changes)
            .unwrap();
        assert_eq!(
            own,
            vec![
                Emission::Enclosed {
                    base: center,
                    updates: vec![Update::Count {
                        object: private,
                        old: 2
                    }]
                },
                Emission::Enclosed {
                    base: center,
                    updates: vec![Update::Delete(public.clone())]
                }
            ]
        );
        assert_eq!(
            other
                .cycle(center, build, Some(20), &objects, &changes)
                .unwrap(),
            vec![
                Emission::Enclosed {
                    base: center,
                    updates: vec![Update::Add(revealed)]
                },
                Emission::Enclosed {
                    base: center,
                    updates: vec![Update::Delete(public)]
                }
            ]
        );
        let moving = owner
            .cycle(pos(3208, 3200), build, Some(10), &objects, &[])
            .unwrap();
        assert_eq!(
            moving
                .iter()
                .filter(|e| matches!(e, Emission::Reset(_)))
                .count(),
            7
        );
    }
    #[test]
    fn build_area_clipping_rejects_axis_overflow_and_respawn_moves_to_end_of_zone_order() {
        let mut objects = Objects::default();
        let catalog = Catalog::default();
        let first = objects
            .add(
                object(0, 0, Scope::Permanent, None),
                &catalog,
                MapClock(0),
                i32::MAX,
                100,
            )
            .unwrap()
            .remove(0);
        objects
            .add(
                object(7, 7, Scope::Permanent, None),
                &catalog,
                MapClock(0),
                i32::MAX,
                100,
            )
            .unwrap();
        assert!(objects.delete(&first, MapClock(0), 1));
        objects.process_durations(MapClock(1), &catalog).unwrap();
        assert_eq!(
            objects
                .in_zone(pos(0, 0))
                .map(|h| h.snapshot().coords)
                .collect::<Vec<_>>(),
            vec![pos(7, 7), pos(0, 0)]
        );
        let mut observer = Observer::default();
        let output = observer
            .cycle(pos(24, 24), pos(24, 24), None, &objects, &[])
            .unwrap();
        assert_eq!(
            output
                .iter()
                .filter(|e| matches!(e, Emission::Reset(_)))
                .count(),
            16
        );
        assert!(observer
            .visible()
            .iter()
            .all(|c| (24..=48).contains(&c.x()) && (24..=48).contains(&c.z())));
        assert!(observer
            .cycle(pos(0, 0), pos(0, 0), None, &objects, &[])
            .is_err());
        assert!(observer
            .cycle(CoordGrid::NULL, pos(0, 0), None, &objects, &[])
            .is_err());
    }
}


#[cfg(test)]
mod graphic_tests {
    use super::*;
    #[test]
    fn receiver_graphics_are_private_and_never_replay_after_idle_or_rebuild() {
        let at=CoordGrid::new(3200,3200,0).unwrap();let build=CoordGrid::new(3160,3160,0).unwrap();
        let objects=Objects::default();let locations=crate::locations::Locations::default();
        let mut one=Observer::default();let mut two=Observer::default();
        one.cycle_all(at,build,Some(1),&objects,&locations,&[]).unwrap();
        two.cycle_all(at,build,Some(2),&objects,&locations,&[]).unwrap();
        let changes=[Change::Graphic(Graphic{coords:at,id:86,height:0,delay:0,receiver:Some(1)}),Change::Graphic(Graphic{coords:at,id:78,height:0,delay:0,receiver:None})];
        let ids=|emissions:Vec<Emission>|emissions.into_iter().flat_map(|e|match e{Emission::Enclosed{updates,..}=>updates,_=>vec![]}).filter_map(|u|match u{Update::Graphic(g)=>Some(g.id),_=>None}).collect::<Vec<_>>();
        assert_eq!(ids(one.cycle_all(at,build,Some(1),&objects,&locations,&changes).unwrap()),vec![86,78]);
        assert_eq!(ids(two.cycle_all(at,build,Some(2),&objects,&locations,&changes).unwrap()),vec![78]);
        assert!(ids(one.cycle_all(at,build,Some(1),&objects,&locations,&[]).unwrap()).is_empty());
        assert!(ids(one.cycle_all(at.translate(8,0,0).unwrap(),build,Some(1),&objects,&locations,&[]).unwrap()).is_empty());
    }
}
