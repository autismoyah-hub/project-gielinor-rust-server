#[test]
fn compiled_window_status_decoder_matches_unsigned_mode_and_dimensions() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/window-status.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for line in fixture.lines() {
        let f: Vec<_> = line.split_whitespace().collect();
        let payload: Vec<_> = (0..f[0].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&f[0][i..i + 2], 16).unwrap())
            .collect();
        let status = gielinor_protocol::game::window_status(&payload).unwrap();
        assert_eq!(status.mode.to_string(), f[1]);
        assert_eq!(status.width.to_string(), f[2]);
        assert_eq!(status.height.to_string(), f[3]);
        assert!(gielinor_protocol::game::window_status(&payload[..4]).is_err());
        let mut extra = payload;
        extra.push(0);
        assert!(gielinor_protocol::game::window_status(&extra).is_err());
    }
}
