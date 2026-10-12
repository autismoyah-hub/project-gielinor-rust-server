use gielinor_world::{
    collision::{CollisionMap, Strategy},
    loc_reach::Target,
    reach::Rectangle,
    routing::location_route,
    CoordGrid,
};
#[test]
fn compiled_all_location_reach_shapes_and_routes_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/location-reach.expected"
    ))
    .expect("Run scripts/location-reach-parity.ps1");
    assert_eq!(fixture.lines().count(), 8192);
    let mut map = CollisionMap::default();
    for x in (3128..=3272).step_by(8) {
        for z in (3128..=3272).step_by(8) {
            map.allocate(x, z, 0);
        }
    }
    for x in 3180..=3220 {
        for z in 3180..=3220 {
            map.set(
                x,
                z,
                0,
                (if x % 7 == 0 && z % 11 != 0 {
                    0x200000
                } else {
                    0
                }) | (if x % 3 == 0 { 0x88 } else { 0 })
                    | (if z % 3 == 0 { 0x22 } else { 0 })
                    | (if x % 5 == 0 { 0x100 } else { 0 }),
            );
        }
    }
    let source = CoordGrid::new(3200, 3200, 0).unwrap();
    for (case, line) in fixture.lines().enumerate() {
        let f = line.split_whitespace().collect::<Vec<_>>();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let target = Target {
            rectangle: Rectangle {
                coords: CoordGrid::new(n(1), n(2), 0).unwrap(),
                width: n(3),
                length: n(4),
                angle: n(5) as u8,
                blocked: n(6) as u8,
                exclusive: false,
            },
            shape: n(7) as i8,
        };
        assert_eq!(
            target.reached(&map, source, n(0)),
            f[8] == "true",
            "reach {case}: {line}"
        );
        if case >= 1024 {
            continue;
        }
        let route = location_route(&map, source, target, n(0), true, 25, Strategy::Normal);
        assert_eq!(
            (route.success, route.alternative),
            (f[9] == "true", f[10] == "true"),
            "route {case}: {line}"
        );
        let path = route
            .waypoints
            .iter()
            .map(|p| format!("{}:{}", p.x(), p.z()))
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(
            if path.is_empty() { "." } else { &path },
            f[11],
            "path {case}: {line}"
        );
    }
}
