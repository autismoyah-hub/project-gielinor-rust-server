use gielinor_world::{
    area_tracking::{Event, State},
    areas::Areas,
    queues::{Category, Scheduler},
    CoordGrid,
};
#[test]
fn original_pending_logout_queue_barriers_and_idempotent_area_exits() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/logout-queues.expected"
    ))
    .expect("Run scripts/logout-queues-parity.ps1");
    assert_eq!(fixture.lines().count(), 64);
    let position = CoordGrid::new(3200, 3200, 0).unwrap();
    let mut index = Areas::default();
    index.register_coord(position, [7, 9, 7]).unwrap();
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let mask: u8 = fields[0].parse().unwrap();
        let mut queues = Scheduler::default();
        for category in [
            Category::Soft,
            Category::Normal,
            Category::Weak,
            Category::Strong,
            Category::LongAccelerate,
            Category::LongDiscard,
        ] {
            if mask & (1 << (category as u8)) != 0 {
                queues.add(category as i32, category, 100, ()).unwrap();
            }
        }
        assert_eq!(queues.has_non_discardable(), fields[1] == "true", "{line}");
        let mut state = State::default();
        state.process(position, position, false, &index);
        let mut events = vec![Event::Enter(99)];
        events.extend(state.process(position, position, true, &index));
        events.extend(state.process(position, position, true, &index));
        assert_eq!(state.active().len().to_string(), fields[2]);
        let actual = events
            .iter()
            .map(|event| match event {
                Event::Enter(id) => format!("7:{id}"),
                Event::Exit(id) => format!("8:{id}"),
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, fields[3], "{line}");
    }
}
