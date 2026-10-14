use gielinor_world::ui::{Component, State};
#[test]
fn original_resume_pause_handler_close_queue_and_event_precedence() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/pause-close.expected"
    ))
    .expect("Run scripts/pause-close-parity.ps1");
    assert_eq!(fixture.lines().count(), 288);
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let mut ui = State::default();
        let button = Component::new(229, 4);
        if n(0) == 1 || n(0) == 3 {
            ui.modals.set(Component::new(548, 23), 229);
        }
        if n(0) == 2 || n(0) == 3 {
            ui.overlays.set(Component::new(548, 22), 229);
        }
        if n(2) != 0 {
            ui.events.add(button, -1, -1, (n(2) - 1) as u64).unwrap();
        }
        if n(4) != 0 {
            ui.close_queue.push(Component::new(548, 22));
        }
        ui.queue_pause_close(button, n(3), n(1));
        if n(5) != 0 {
            ui.queue_pause_close(button, n(3), n(1));
        }
        let actual = ui
            .close_queue
            .iter()
            .map(|p| p.0.to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, f[6], "{line}");
    }
}
