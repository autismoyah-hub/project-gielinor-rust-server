#[test]
fn count_dialog_matches_rsprot_signed_boundaries_and_random_inputs() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/count-dialog.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for line in fixture.lines() {
        let (hex, value) = line.split_once('|').unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        assert_eq!(
            gielinor_protocol::dialog::count(&bytes),
            Ok(value.parse().unwrap())
        );
        assert!(gielinor_protocol::dialog::count(&bytes[..3]).is_err());
        let mut extra = bytes;
        extra.push(0);
        assert!(gielinor_protocol::dialog::count(&extra).is_err());
    }
}
