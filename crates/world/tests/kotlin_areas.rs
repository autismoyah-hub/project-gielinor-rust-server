use gielinor_world::{areas::Areas, CoordGrid};
#[test]
fn original_kotlin_area_order_and_overflow_packing() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/areas.expected"
    ))
    .expect("Run scripts/area-parity.ps1");
    assert_eq!(fixture.lines().count(), 3000);
    for (case, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let p = CoordGrid::from_packed(fields[0].parse().unwrap());
        let ids = |index: usize| {
            fields[index]
                .split(',')
                .map(|n| n.parse::<u16>().unwrap())
                .collect::<Vec<_>>()
        };
        let mut areas = Areas::default();
        areas.register_coord(p, ids(1)).unwrap();
        areas.register_zone(p, ids(2)).unwrap();
        areas
            .register_square(((p.x() >> 6) << 8 | (p.z() >> 6)) as u16, ids(3))
            .unwrap();
        let actual = areas
            .at(p)
            .into_iter()
            .map(|id| (id as i16).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, fields[4], "case {case}: {line}");
    }
}
