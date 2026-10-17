//! Per-restorer scenery through ordinary zone updates. The durable Run is the
//! only source of truth. Shared collision and interactive fixtures never vary.
use gielinor_world::{
    ground::zone_base,
    locations::{Event, Location},
    tower_run::Run,
    tower_scenery::{LANDMARKS, VENTS},
    zones::{Emission, Graphic, Update},
    CoordGrid,
};

#[derive(Clone, Default)]
pub struct Observer {
    previous: Option<(CoordGrid, CoordGrid, Run)>,
    ambient: Option<i32>,
}
/// Encounter effects are global zone events, but earned guardian credit is
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
            let at = origin
                .translate(f.x, f.z, 0)
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
                let at = origin
                    .translate(x, z, 0)
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
            .map(|f| zone_base(origin.translate(f.x, f.z, 0).unwrap()))
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
        let left = ids(&a.cycle(o, hot, p, b, &z, &[], 1).unwrap());
        let right = ids(&c.cycle(o, cold, p, b, &z, &[], 1).unwrap());
        assert!(left.contains(&18526) && left.contains(&18511));
        assert!(right.contains(&18525) && right.contains(&18512));
        assert!(ids(&a.cycle(o, hot, p, b, &z, &[], 2).unwrap()).is_empty());
        assert!(ids(&c.cycle(o, cold, p, b, &z, &[], 2).unwrap()).is_empty());
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
        let first = ids(&observer.cycle(o, run, p, b, &z, &[], 1).unwrap());
        assert_eq!(
            first,
            ids(&Observer::default().cycle(o, run, p, b, &z, &[], 1).unwrap())
        );
        let resets = z.iter().copied().map(Emission::Reset).collect::<Vec<_>>();
        assert_eq!(
            first,
            ids(&observer.cycle(o, run, p, b, &z, &resets, 2).unwrap())
        );
        run.services = 0;
        run.victory = false;
        let replay = ids(&observer.cycle(o, run, p, b, &z, &[], 3).unwrap());
        assert_eq!(replay, [18525, 18512, 16810, 16811, 16811, 18526]);
    }
    #[test]
    fn boiler_mouth_keeps_its_real_rotation_through_progress_and_scene_resets() {
        let (o, b, z) = setup();
        let p = o.translate(71, 72, 0).unwrap();
        let boiler = o.translate(71, 76, 0).unwrap();
        let mut observer = Observer::default();
        for (cleared, expected) in [(false, 18526), (true, 18525), (false, 18526)] {
            let mut run = Run::default();
            run.imp = cleared;
            let resets = z.iter().copied().map(Emission::Reset).collect::<Vec<_>>();
            let updates = [
                observer.cycle(o, run, p, b, &z, &resets, 1).unwrap(),
                Observer::default().cycle(o, run, p, b, &z, &[], 1).unwrap(),
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
    #[test]
    fn hidden_zones_get_no_updates_and_imp_victory_stops_ambient_steam() {
        let (o, b, _) = setup();
        let p = o.translate(56, 56, 0).unwrap();
        let mut observer = Observer::default();
        assert!(observer
            .cycle(o, Run::default(), p, b, &[], &[], 1)
            .unwrap()
            .is_empty());
        let z = VENTS
            .iter()
            .map(|&(x, z)| zone_base(o.translate(x, z, 0).unwrap()))
            .collect::<Vec<_>>();
        let steaming = observer
            .cycle(o, Run::default(), p, b, &z, &[], 13)
            .unwrap();
        assert_eq!(steaming.len(), 4);
        let mut cleared = Run::default();
        cleared.imp = true;
        let cooled = observer.cycle(o, cleared, p, b, &z, &[], 25).unwrap();
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
                coords: o.translate(x, z, 0).unwrap(),
                id,
                height: 0,
                delay: 0,
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
}
