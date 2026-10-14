use gielinor_protocol::loc_input;
#[test]
fn original_location_target_decoder_nullable_fields_and_mixed_orders() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/location-target.expected"
    ))
    .unwrap();
    let mut count = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split_whitespace().collect();
        let bytes: Vec<_> = (0..f[0].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&f[0][i..i + 2], 16).unwrap())
            .collect();
        let b = loc_input::target(&bytes).unwrap();
        let expected: Vec<i64> = f[1..].iter().map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            [
                b.id as i64,
                b.x as i64,
                b.z as i64,
                b.control as i64,
                b.source as i64,
                b.source_slot as i64,
                b.source_object as i64
            ]
            .as_slice(),
            expected,
            "{row}"
        );
        for len in 0..15 {
            assert!(loc_input::target(&bytes[..len]).is_err());
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(loc_input::target(&trailing).is_err());
        count += 1;
    }
    assert_eq!(count, 2048);
}
