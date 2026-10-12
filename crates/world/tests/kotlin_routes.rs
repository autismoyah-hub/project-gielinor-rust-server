use gielinor_world::{
    collision::{CollisionMap, Strategy},
    routing::coordinate_route,
    CoordGrid,
};
#[test]
fn original_kotlin_coordinate_routes_match() {
    let mut map = CollisionMap::default();
    for x in (3128..=3272).step_by(8) {
        for z in (3128..=3272).step_by(8) {
            map.allocate(x, z, 0);
        }
    }
    for x in 3180..=3220 {
        for z in 3180..=3220 {
            if (x % 7 == 0 && z % 11 != 0)
                || ((3208..=3212).contains(&x) && (3208..=3212).contains(&z))
            {
                map.set(x, z, 0, 0x200000);
            }
        }
    }
    let fixture = include_str!("fixtures/routes.expected");
    assert_eq!(fixture.lines().count(), 1452);
    for line in fixture.lines() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let r = coordinate_route(
            &map,
            CoordGrid::new(3200, 3200, 0).unwrap(),
            CoordGrid::new(n(1), n(2), 0).unwrap(),
            n(0),
            f[3] == "true",
            n(4) as usize,
            Strategy::Normal,
        );
        assert_eq!(
            (r.success, r.alternative),
            (f[5] == "true", f[6] == "true"),
            "{line}"
        );
        let path = if r.waypoints.is_empty() {
            ".".into()
        } else {
            r.waypoints
                .iter()
                .map(|c| format!("{}:{}", c.x(), c.z()))
                .collect::<Vec<_>>()
                .join(",")
        };
        assert_eq!(path, f[7], "{line}");
    }
}
