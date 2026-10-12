#[test]
fn reference_rotations_match_footprints_including_shifted_anchors() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/region-rotation.expected"
    ))
    .expect("Run scripts/region-rotation-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for line in fixture.lines() {
        let (input, expected) = line.split_once('|').unwrap();
        let v: Vec<i32> = input.split(',').map(|s| s.parse().unwrap()).collect();
        let (x, z) =
            gielinor_world::region_rotation::translate(v[0] as u8, v[1], v[2], 8, 8, v[3], v[4])
                .unwrap();
        assert_eq!(format!("{x},{z}"), expected, "{input}");
    }
}
