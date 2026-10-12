use gielinor_world::{
    collision::CollisionMap,
    raycast::{self, Bounds},
    CoordGrid,
};
#[test]
fn compiled_line_validator_matches_walk_sight_footprints_and_extra_flags() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/raycast.expected"
    ))
    .expect("Run scripts/raycast-parity.ps1");
    assert_eq!(fixture.lines().count(), 8192);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let a: Vec<i32> = f[0].split(',').map(|v| v.parse().unwrap()).collect();
        let mut map = CollisionMap::default();
        for x in 3192..=3215 {
            for z in 3192..=3215 {
                map.allocate(x, z, a[0]);
            }
        }
        for tile in f[2].split('/') {
            let t: Vec<i32> = tile.split(',').map(|v| v.parse().unwrap()).collect();
            map.set(t[0], t[1], a[0], t[2] as u32);
        }
        let source = Bounds {
            coords: CoordGrid::new(a[1], a[2], a[0]).unwrap(),
            width: a[5],
            length: a[6],
        };
        let target = Bounds {
            coords: CoordGrid::new(a[3], a[4], a[0]).unwrap(),
            width: a[7],
            length: a[8],
        };
        assert_eq!(
            format!(
                "{},{}",
                raycast::clear(&map, source, target, true, a[9] as u32),
                raycast::clear(&map, source, target, false, a[9] as u32)
            ),
            f[1],
            "case {case}"
        );
    }
}
