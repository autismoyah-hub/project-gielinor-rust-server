use gielinor_world::protected::State;
#[test]
fn compiled_kotlin_player_protection_predicates() {
    let fixture = include_str!("fixtures/protected.expected");
    let mut count = 0;
    for (line, row) in fixture.lines().enumerate() {
        let f: Vec<_> = row.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let b = |i: usize| f[i].parse::<bool>().unwrap();
        let mut state = State {
            current_clock: n(0),
            processed_clock: n(1),
            pending_shutdown: b(3),
            suspended: b(5),
            ..State::default()
        };
        state.delay(n(2));
        assert_eq!(
            (
                state.delay_until,
                state.delayed(),
                state.busy(b(4)),
                state.access_protected(b(4)),
                state.modal_button_protected()
            ),
            (n(6), b(7), b(8), b(9), b(10)),
            "line {}",
            line + 1
        );
        count += 1;
    }
    assert_eq!(count, 4096);
}
