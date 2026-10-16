use gielinor_world::{collision::CollisionMap, reach::Rectangle, CoordGrid};

#[test]
fn original_kotlin_full_byte_approach_flags_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/packed-reach.expected"
    ))
    .expect("Run scripts/packed-reach-parity.ps1");
    assert_eq!(fixture.lines().count(), 12288);
    let mut map = CollisionMap::default();
    for x in (3192..=3216).step_by(8) {
        for z in (3192..=3216).step_by(8) {
            map.allocate(x, z, 0);
        }
    }
    for line in fixture.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| fields[i].parse::<i32>().unwrap();
        let target = Rectangle {
            coords: CoordGrid::new(3202, 3202, 0).unwrap(),
            width: 1,
            length: 2,
            angle: n(1) as u8,
            blocked: n(0) as u8,
            exclusive: true,
        };
        assert_eq!(
            target.reached(&map, CoordGrid::new(n(3), n(4), 0).unwrap(), n(2)),
            fields[5] == "true",
            "{line}"
        );
    }
}
