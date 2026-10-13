use gielinor_world::{
    area_tracking::{Event, State},
    areas::Areas,
    CoordGrid,
};
#[test]
fn original_kotlin_area_processor_matches_entry_exit_and_logout_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/area-transition.expected"
    ))
    .expect("Run scripts/area-transition-parity.ps1");
    let a = CoordGrid::new(3200, 3200, 0).unwrap();
    let b = CoordGrid::new(3201, 3200, 0).unwrap();
    let mut areas = Areas::default();
    areas.register_coord(a, [7, 9]).unwrap();
    areas.register_coord(b, [8]).unwrap();
    areas.register_zone(a, [7]).unwrap();
    let mut state = State::default();
    assert_eq!(fixture.lines().count(), 8);
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let packed: i32 = fields[0].parse().unwrap();
        let position = CoordGrid::new(
            (packed >> 14) & 16383,
            packed & 16383,
            (packed >> 28) & 3,
        )
        .unwrap();
        let events = state.process(position, position, fields[1] == "true", &areas);
        let actual = events
            .iter()
            .map(|e| match e {
                Event::Enter(id) => format!("7:{}", *id as i16),
                Event::Exit(id) => format!("8:{}", *id as i16),
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, fields[2], "{line}");
        assert_eq!(
            state
                .active()
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(","),
            fields[3]
        );
    }
}
