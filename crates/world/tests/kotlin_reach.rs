use gielinor_world::{
    collision::{CollisionMap, Strategy},
    reach::Rectangle,
    routing::rectangle_route,
    CoordGrid,
};
#[test]
fn original_rectangular_reach_and_routes() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/reach.expected"
    ))
    .expect("Run scripts/reach-parity.ps1");
    assert_eq!(fixture.lines().count(), 6000);
    let mut map = CollisionMap::default();
    for x in (3128..=3272).step_by(8) {
        for z in (3128..=3272).step_by(8) {
            map.allocate(x, z, 0);
        }
    }
    for x in 3180..=3220 {
        for z in 3180..=3220 {
            let value = (if x % 7 == 0 && z % 11 != 0 {
                0x200000
            } else {
                0
            }) | (if x % 3 == 0 { 0x88 } else { 0 })
                | (if z % 3 == 0 { 0x22 } else { 0 });
            map.set(x, z, 0, value);
        }
    }
    let source = CoordGrid::new(3200, 3200, 0).unwrap();
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let target = Rectangle {
            coords: CoordGrid::new(n(1), n(2), 0).unwrap(),
            width: n(3),
            length: n(4),
            angle: n(5) as u8,
            blocked: n(6) as u8,
            exclusive: f[7] == "true",
        };
        assert_eq!(
            target.reached(&map, source, n(0)),
            f[8] == "true",
            "reach {case}: {line}"
        );
        if case >= 600 {
            continue;
        }
        let route = rectangle_route(&map, source, target, n(0), true, 25, Strategy::Normal);
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
