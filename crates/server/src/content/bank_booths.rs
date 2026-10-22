//! Target-backed static booth opening; transfers remain in shared banking.
use super::{Definitions, Symbols};
use std::collections::BTreeSet;

pub fn ordinary_bank_booths(
    locations: &Symbols,
    contents: &Symbols,
    defs: &Definitions,
) -> Result<BTreeSet<u32>, String> {
    if contents.get("bank_booth") != Some(&18) || locations.get("aide_bankbooth") != Some(&18491) {
        return Err("Unattested ordinary bank booth symbols".into());
    }
    let ty = defs
        .locations
        .get(&18491)
        .ok_or("Missing ordinary bank booth")?;
    if ty.name != "Bank booth"
        || ty.content_group != 18
        || ty.ops[0].is_some()
        || ty.ops[1].as_deref() != Some("Bank")
        || ty.ops[2..].iter().any(Option::is_some)
        || ty.collision.width != 1
        || ty.collision.length != 1
        || !ty.multi.is_empty()
        || ty.multi_default != -1
        || ty.multi_varbit != -1
        || ty.multi_varp != -1
    {
        return Err("Unattested static ordinary bank booth profile".into());
    }
    // Raw27291 switches between18491 and Deadman27292 using4153. Registering
    // it as an ordinary static object would bypass effective-definition policy.
    Ok([18491].into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        CoordGrid, loc_collision,
        locations::Location,
        simulation::{Action, ActorId, PlayerData, World, WorldEvent},
        stats::Stats,
    };

    fn definitions() -> (Symbols, Symbols, Definitions) {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut ty = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
        ty.name = "Bank booth".into();
        ty.content_group = 18;
        ty.ops[1] = Some("Bank".into());
        defs.locations.insert(18491, ty);
        (
            [("aide_bankbooth".into(), 18491)].into(),
            [("bank_booth".into(), 18)].into(),
            defs,
        )
    }
    fn setup(angle: u8, member: bool, distant: bool) -> (World, ActorId, Location) {
        let (l, c, d) = definitions();
        let mut w = World::default();
        w.ordinary_bank_booths = ordinary_bank_booths(&l, &c, &d).unwrap();
        for x in (3192..3216).step_by(8) {
            for z in (3192..3216).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 18491, 10, angle).unwrap();
        let ty = d.locations[&18491].collision;
        w.location_types.insert(18491, ty);
        w.locations.load(loc);
        loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Booth fixture".into(),
                CoordGrid::new(if distant { 3199 } else { 3200 }, 3200, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, member).unwrap();
        w.drain_events();
        (w, a, loc)
    }
    fn open(loc: Location, operation: u8) -> Action {
        Action::OperateLocation {
            coords: loc.coords,
            id: loc.id,
            operation,
            control: false,
        }
    }
    fn request(w: &mut World) -> Option<gielinor_world::bank_access::Request> {
        w.drain_events().into_iter().find_map(|e| {
            if let WorldEvent::LocationBankRequested { request, .. } = e {
                Some(request)
            } else {
                None
            }
        })
    }

    #[test]
    fn ordinary_bank_booth_binding_is_static_second_operation_and_rejects_drift() {
        let (l, c, d) = definitions();
        assert_eq!(ordinary_bank_booths(&l, &c, &d).unwrap(), [18491].into());
        for mode in 0..9 {
            let (l, mut c, mut d) = definitions();
            let ty = d.locations.get_mut(&18491).unwrap();
            match mode {
                0 => ty.ops[0] = Some("Bank".into()),
                1 => ty.ops[1] = None,
                2 => ty.ops[2] = Some("Collect".into()),
                3 => ty.collision.width = 2,
                4 => ty.content_group = 19,
                5 => ty.multi_varbit = 4153,
                6 => ty.multi = vec![18491, 27292],
                7 => ty.name = "null".into(),
                _ => {
                    c.insert("bank_booth".into(), 19);
                }
            }
            assert!(ordinary_bank_booths(&l, &c, &d).is_err(), "mode{mode}");
        }
        assert!(ordinary_bank_booths(&Symbols::new(), &c, &d).is_err());
    }

    #[test]
    fn ordinary_bank_booth_world_member_angles_and_walk_use_shared_full_bank_request() {
        let mut cases = 0;
        for angle in 0..4 {
            for member in [false, true] {
                for distant in [false, true] {
                    let (mut w, a, loc) = setup(angle, member, distant);
                    let before = w.data(a).unwrap().clone();
                    assert!(w.has_location_handler(18491, 2));
                    assert!(!w.has_location_handler(18491, 1));
                    assert!(!w.has_location_handler(27291, 2));
                    w.submit(a, open(loc, 2)).unwrap();
                    let mut found = None;
                    for _ in 0..8 {
                        w.tick().unwrap();
                        found = request(&mut w);
                        if found.is_some() {
                            break;
                        }
                    }
                    let r = found.expect("reached ordinary booth full-bank request");
                    assert_eq!(r.kind(), gielinor_world::bank_access::Kind::Full);
                    assert!(w.location_bank_request_valid(a, r));
                    let after = w.data(a).unwrap();
                    assert_eq!(after.inventory, before.inventory);
                    assert_eq!(after.bank, before.bank);
                    assert_eq!(after.equipment, before.equipment);
                    assert_eq!(after.stats, before.stats);
                    w.set_modal_open(a, true).unwrap();
                    assert!(!w.location_bank_request_valid(a, r));
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 16);
    }

    #[test]
    fn ordinary_bank_booth_wrong_operation_life_and_stale_target_do_not_open() {
        for mode in 0..4 {
            let (mut w, a, loc) = setup(0, false, false);
            if mode == 1 {
                w.data_mut(a).unwrap().stats.set_current(3, 0);
            }
            w.submit(a, open(loc, if mode == 0 { 1 } else { 2 }))
                .unwrap();
            if mode == 2 {
                w.locations = Default::default();
            }
            if mode == 3 {
                w.submit(
                    a,
                    Action::Walk {
                        destination: CoordGrid::new(3199, 3199, 0).unwrap(),
                        run: false,
                    },
                )
                .unwrap();
            }
            for _ in 0..5 {
                w.tick().unwrap();
                assert!(request(&mut w).is_none(), "mode{mode}");
            }
        }
        let (mut w, a, loc) = setup(0, false, false);
        w.submit(a, open(loc, 2)).unwrap();
        w.tick().unwrap();
        let r = request(&mut w).unwrap();
        w.locations = Default::default();
        assert!(!w.location_bank_request_valid(a, r));
    }

    #[test]
    fn ordinary_bank_booth_life_admission_and_pending_ui_request_recheck_actual_death() {
        use gielinor_world::hits::{Hit, Style, Target};
        let mut cases = 0;
        for late in [false, true] {
            for phase in 0..3 {
                let (mut w, a, loc) = setup(0, false, false);
                let pending = if late {
                    w.submit(a, open(loc, 2)).unwrap();
                    w.tick().unwrap();
                    Some(request(&mut w).unwrap())
                } else {
                    None
                };
                if phase == 0 {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                } else {
                    w.queue_hit(
                        Target::Player(a),
                        Hit {
                            display: None,
                            source: None,
                            style: Style::Typeless,
                            damage: 99,
                        },
                        0,
                    )
                    .unwrap();
                    w.tick().unwrap();
                    assert!(w.drain_events().iter().any(
                        |e| matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)
                    ));
                    if phase == 2 {
                        w.data_mut(a).unwrap().stats.set_current(3, 10);
                    }
                }
                let before = w.data(a).unwrap().clone();
                if let Some(r) = pending {
                    assert!(!w.location_bank_request_valid(a, r));
                } else {
                    w.submit(a, open(loc, 2)).unwrap();
                }
                for _ in 0..6 {
                    w.tick().unwrap();
                    assert!(request(&mut w).is_none());
                }
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert_eq!(w.data(a).unwrap().bank, before.bank);
                assert_eq!(w.data(a).unwrap().stats, before.stats);
                cases += 1;
            }
        }
        assert_eq!(cases, 6);
    }
}
