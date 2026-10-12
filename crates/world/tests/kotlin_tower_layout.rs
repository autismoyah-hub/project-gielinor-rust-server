use gielinor_world::tower_layout;
#[test]
fn copied_zones_and_openings_match_compiled_tower_content() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-layout.expected"
    ))
    .expect("Run scripts/tower-layout-parity.ps1");
    assert_eq!(fixture.lines().count(), 1048);
    let zones = tower_layout::template();
    let mut doors = std::collections::BTreeSet::new();
    for row in fixture.lines() {
        let (key, value) = row.split_once('|').unwrap();
        if key == "door" {
            let (x, z) = value.split_once(',').unwrap();
            doors.insert((x.parse().unwrap(), z.parse().unwrap()));
        } else {
            let key: Vec<u8> = key.split(',').map(|s| s.parse().unwrap()).collect();
            let source = zones.get(&(key[0], key[1], key[2]));
            let actual = source.map_or("-".into(), |s| {
                format!("{},{},{},{}", s.x, s.z, s.level, s.rotation)
            });
            assert_eq!(actual, value, "zone {key:?}");
        }
    }
    assert_eq!(tower_layout::openings(), doors);
}
