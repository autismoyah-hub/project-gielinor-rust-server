//! Per-restorer scenery through ordinary zone updates. The durable Run is the
//! only source of truth. Shared collision and interactive fixtures never vary.
use gielinor_world::{
    ground::zone_base,
    locations::{Event, Location},
    tower_run::Run,
    cinderworks_scenery::{LANDMARKS, VENTS},
    zones::{Emission, Graphic, Update},
    CoordGrid,
};

#[derive(Clone, Default)]
pub struct Observer {
    previous: Option<(CoordGrid, CoordGrid, Run)>,
    ambient: Option<i32>,
}
/// Earned guardian credit is
/// personal. Do not show another restorer's attacks in an already quiet court.
/// Active restorers receive the original effects unchanged.
pub fn phase_effects(origin: CoordGrid, run: Run, position: CoordGrid, emissions: &mut [Emission]) {
    if !gielinor_world::tower_layout::contains(origin, position) {
        return;
    }
    for emission in emissions {
        if let Emission::Enclosed { updates, .. } = emission {
            updates.retain(|update| {
                let Update::Graphic(g) = update else {
                    return true;
                };
                if ![78, 86].contains(&g.id) {
                    return true;
                }
                !((run.imp
                    && gielinor_world::cinderworks_encounter::Boss::Imp.contains(origin, g.coords))
                    || (run.victory
                        && gielinor_world::cinderworks_encounter::Boss::Warden
                            .contains(origin, g.coords)))
            });
        }
    }
}
impl Observer {
    pub fn cycle(
        &mut self,
        origin: CoordGrid,
        spatial: &gielinor_world::cinderworks_floor::Spatial,
        run: Run,
        position: CoordGrid,
        build: CoordGrid,
        visible: &[CoordGrid],
        baseline: &[Emission],
        tick: i32,
    ) -> Result<Vec<Emission>, &'static str> {
        if !gielinor_world::tower_layout::contains(origin, position) {
            self.previous = None;
            self.ambient = None;
            return Ok(vec![]);
        }
        let changed = self.previous != Some((origin, build, run));
        let reset: Vec<_> = baseline
            .iter()
            .filter_map(|e| {
                if let Emission::Reset(p) = e {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();
        let mut output = vec![];
        for f in LANDMARKS {
            let at = spatial.at(origin,f.x,f.z)
                .map_err(|_| "Scenery outside world")?;
            let zone = zone_base(at);
            if visible.contains(&zone) && (changed || reset.contains(&zone)) {
                output.push(Emission::Enclosed {
                    base: zone,
                    updates: vec![Update::Location(Event::Add(Location::new(
                        at,
                        f.id(run),
                        10,
                        f.angle,
                    )?))],
                });
            }
        }
        // Quiet perimeter grates teach the source of the arena's heat. This
        // ambient steam has no damage; real attacks retain their own scheduler.
        if !run.imp
            && !run.victory
            && self
                .ambient
                .is_none_or(|last| tick.wrapping_sub(last) >= 12)
        {
            for (x, z) in VENTS {
                let at = spatial.at(origin,x,z)
                    .map_err(|_| "Vent outside world")?;
                let zone = zone_base(at);
                if visible.contains(&zone) {
                    output.push(Emission::Enclosed {
                        base: zone,
                        updates: vec![Update::Graphic(Graphic {
                            coords: at,
                            id: 86,
                            height: 0,
                            delay: 0,
                            receiver: None,
                        })],
                    });
                }
            }
            self.ambient = Some(tick);
        }
        self.previous = Some((origin, build, run));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (CoordGrid, CoordGrid, Vec<CoordGrid>) {
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let build = origin.translate(8, 8, 0).unwrap();
        let zones = LANDMARKS
            .iter()
            .map(|f| zone_base(gielinor_world::cinderworks_floor::Spatial::default().at(origin,f.x,f.z).unwrap()))
            .collect();
        (origin, build, zones)
    }
    fn ids(events: &[Emission]) -> Vec<u32> {
        events
            .iter()
            .flat_map(|e| match e {
                Emission::Enclosed { updates, .. } => updates.as_slice(),
                _ => &[],
            })
            .filter_map(|u| {
                if let Update::Location(Event::Add(l)) = u {
                    Some(l.id)
                } else {
                    None
                }
            })
            .collect()
    }
    #[test]
    fn independent_observers_derive_restoration_from_their_own_durable_run() {
        let (o, b, z) = setup();
        let p = o.translate(56, 56, 0).unwrap();
        let mut a = Observer::default();
        let mut c = Observer::default();
        let cold = Run::default();
        let mut hot = cold;
        hot.services = 7;
        let left = ids(&a.cycle(o, &Default::default(), hot, p, b, &z, &[], 1).unwrap());
        let right = ids(&c.cycle(o, &Default::default(), cold, p, b, &z, &[], 1).unwrap());
        assert!(left.contains(&18526) && left.contains(&18511));
        assert!(right.contains(&18525) && right.contains(&18512));
        assert!(ids(&a.cycle(o, &Default::default(), hot, p, b, &z, &[], 2).unwrap()).is_empty());
        assert!(ids(&c.cycle(o, &Default::default(), cold, p, b, &z, &[], 2).unwrap()).is_empty());
    }
    #[test]
    fn region_reset_relogin_and_replay_reconstruct_the_correct_landmarks() {
        let (o, b, z) = setup();
        let p = o.translate(56, 56, 0).unwrap();
        let mut observer = Observer::default();
        let mut run = Run::default();
        run.id = 2;
        run.services = 7;
        run.victory = true;
        run.first_reward_paid = true;
        let first = ids(&observer.cycle(o, &Default::default(), run, p, b, &z, &[], 1).unwrap());
        assert_eq!(
            first,
            ids(&Observer::default().cycle(o, &Default::default(), run, p, b, &z, &[], 1).unwrap())
        );
        let resets = z.iter().copied().map(Emission::Reset).collect::<Vec<_>>();
        assert_eq!(
            first,
            ids(&observer.cycle(o, &Default::default(), run, p, b, &z, &resets, 2).unwrap())
        );
        run.services = 0;
        run.victory = false;
        let replay = ids(&observer.cycle(o, &Default::default(), run, p, b, &z, &[], 3).unwrap());
        assert_eq!(
            replay,
            [18525, 18512, 16810, 16811, 18526, 18526, 18526, 18526]
        );
    }
    #[test]
    fn machinery_mouths_keep_real_rotations_and_independent_progress_on_scene_resets() {
        let (o, b, z) = setup();
        let p = o.translate(71, 72, 0).unwrap();
        for (x, local_z, is_imp) in [(71, 76, true), (51, 78, false), (55, 78, false), (60, 78, false)] {
            let boiler = gielinor_world::cinderworks_floor::Spatial::default().at(o,x,local_z).unwrap();
            let mut observer = Observer::default();
            for (cleared, expected) in [(false, 18526), (true, 18525), (false, 18526)] {
                let mut run = Run::default();
                run.imp = cleared && is_imp;
                run.victory = cleared && !is_imp;
                let resets = z.iter().copied().map(Emission::Reset).collect::<Vec<_>>();
                let updates = [
                    observer.cycle(o, &Default::default(), run, p, b, &z, &resets, 1).unwrap(),
                    Observer::default().cycle(o, &Default::default(), run, p, b, &z, &[], 1).unwrap(),
                ];
                for events in updates {
                    let directions = events
                        .iter()
                        .flat_map(|e| match e {
                            Emission::Enclosed { updates, .. } => updates.as_slice(),
                            _ => &[],
                        })
                        .filter_map(|u| match u {
                            Update::Location(Event::Add(loc)) if loc.coords == boiler => {
                                Some((loc.id, loc.shape, loc.angle))
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(directions, [(expected, 10, 1)]);
                }
            }
        }
    }
    #[test]
    fn hidden_zones_get_no_updates_and_imp_victory_stops_ambient_steam() {
        let (o, b, _) = setup();
        let p = o.translate(56, 56, 0).unwrap();
        let mut observer = Observer::default();
        assert!(observer
            .cycle(o, &Default::default(), Run::default(), p, b, &[], &[], 1)
            .unwrap()
            .is_empty());
        let z = VENTS
            .iter()
            .map(|&(x, z)| zone_base(gielinor_world::cinderworks_floor::Spatial::default().at(o,x,z).unwrap()))
            .collect::<Vec<_>>();
        let steaming = observer
            .cycle(o, &Default::default(), Run::default(), p, b, &z, &[], 13)
            .unwrap();
        assert_eq!(steaming.len(), 4);
        let mut cleared = Run::default();
        cleared.imp = true;
        // Authored relocation places the boiler in a separate zone from vents.
        // Admit that actual zone before expecting its private restored landmark.
        let mut visible=z.clone();
        visible.push(zone_base(gielinor_world::cinderworks_floor::Spatial::default().at(o,71,76).unwrap()));
        let cooled = observer.cycle(o, &Default::default(), cleared, p, b, &visible, &[], 25).unwrap();
        assert!(ids(&cooled).contains(&18525));
        assert!(cooled.iter().all(|e| match e {
            Emission::Enclosed { updates, .. } =>
                updates.iter().all(|u| !matches!(u, Update::Graphic(_))),
            _ => true,
        }));
    }
    #[test]
    fn quiet_courts_hide_peer_telegraphs_without_hiding_the_active_owners_effects() {
        let (o, _, _) = setup();
        let p = o.translate(56, 56, 0).unwrap();
        let graphic = |x, z, id| {
            Update::Graphic(Graphic {
                coords: gielinor_world::cinderworks_floor::Spatial::default().at(o,x,z).unwrap(),
                id,
                height: 0,
                delay: 0,
                receiver: None,
            })
        };
        let baseline = vec![Emission::Enclosed {
            base: zone_base(p),
            updates: vec![
                graphic(72, 72, 86),
                graphic(56, 72, 78),
                graphic(69, 52, 86),
                graphic(72, 72, 1),
            ],
        }];
        let mut active = baseline.clone();
        phase_effects(o, Run::default(), p, &mut active);
        assert_eq!(active, baseline);
        let mut quiet = Run::default();
        quiet.imp = true;
        quiet.victory = true;
        let mut projected = baseline.clone();
        phase_effects(o, quiet, p, &mut projected);
        let Emission::Enclosed { updates, .. } = &projected[0] else {
            panic!()
        };
        assert_eq!(updates.len(), 2);
        assert!(updates.contains(&graphic(69, 52, 86)));
        assert!(updates.contains(&graphic(72, 72, 1)));
        let outside = CoordGrid::new(3224, 3218, 0).unwrap();
        phase_effects(o, quiet, outside, &mut active);
        assert_eq!(active, baseline);
    }
    #[test]
    fn explicit_authored_overrides_place_all_eight_landmarks_on_the_pinned_floor() {
        let spatial=gielinor_world::cinderworks_floor::Spatial::parse("CINDERWORKS_SPATIAL 1\nM 43 50 38 49\nM 72 58 63 63\nM 55 78 88 64\nM 51 78 87 71\nM 60 78 87 78\nM 71 76 71 78\nM 67 53 64 43\n").unwrap();
        let origin=gielinor_world::cinderworks_floor::origin();
        let visible=LANDMARKS.iter().map(|f|zone_base(spatial.at(origin,f.x,f.z).unwrap())).collect::<Vec<_>>();
        let position=spatial.at(origin,56,56).unwrap();
        let events=Observer::default().cycle(origin,&spatial,Run::default(),position,origin,&visible,&[],0).unwrap();
        let coords=events.iter().flat_map(|e|match e{Emission::Enclosed{updates,..}=>updates.as_slice(),_=>&[]}).filter_map(|u|match u{Update::Location(Event::Add(l))=>Some((l.coords.x()-origin.x(),l.coords.z()-origin.z(),l.shape,l.angle)),_=>None}).collect::<Vec<_>>();
        assert_eq!(coords,[(38,49,10,0),(63,63,10,0),(55,39,10,0),(51,48,10,0),(88,64,10,1),(71,78,10,1),(87,71,10,1),(87,78,10,1)]);
    }
}
