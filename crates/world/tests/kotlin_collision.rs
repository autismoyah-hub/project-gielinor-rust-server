use gielinor_world::{
    collision::{CollisionMap, Strategy},
    CoordGrid,
};

#[test]
fn matches_original_kotlin_step_validator() {
    let strategies = [
        Strategy::Normal,
        Strategy::Blocked,
        Strategy::Indoors,
        Strategy::Outdoors,
        Strategy::LineOfSight,
    ];
    let mut map = CollisionMap::default();
    for x in (3192..=3223).step_by(8) {
        for z in (3192..=3223).step_by(8) {
            map.allocate(x, z, 0);
        }
    }
    let from = CoordGrid::new(3200, 3200, 0).unwrap();
    let fixture = include_str!("fixtures/collision.expected");
    assert!(fixture.lines().count() > 50_000);
    for line in fixture.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let n: Vec<i32> = fields[..7].iter().map(|s| s.parse().unwrap()).collect();
        map.set(3200 + n[4], 3200 + n[5], 0, n[6] as u32);
        let result = map.can_step(from, n[1], n[2], n[0], 0, strategies[n[3] as usize]);
        assert_eq!(result, fields[7] == "true", "{line}");
        map.set(3200 + n[4], 3200 + n[5], 0, 0);
    }
}
