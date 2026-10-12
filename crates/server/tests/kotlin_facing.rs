use gielinor_world::{facing::square_angle, CoordGrid};
#[test]
fn kotlin_player_face_square_angles_and_movement_deferral() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/facing.expected"
    ))
    .expect("Run scripts/facing-parity.ps1");
    assert_eq!(rows.lines().count(), 8192);
    for line in rows.lines() {
        let n: Vec<i32> = line.split('|').map(|v| v.parse().unwrap()).collect();
        let source = CoordGrid::from_packed(n[0]);
        let target = CoordGrid::from_packed(n[1]);
        let angle = square_angle(source, 1, target, n[2], n[3]).map_or(-1, i32::from);
        assert_eq!(angle, n[4], "{line}");
        assert_eq!(if n[5] < 100 && n[6] != 100 { angle } else { -1 }, n[7]);
        assert_eq!(
            if n[5] < 100 { CoordGrid::NULL } else { target },
            CoordGrid::from_packed(n[8])
        );
    }
}
