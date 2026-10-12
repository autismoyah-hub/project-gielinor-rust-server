use gielinor_protocol::buttons;
#[test]
fn original_button_and_drag_decoders() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/buttons.expected"
    ))
    .expect("Run scripts/button-parity.ps1");
    let mut count = 0;
    for line in source.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let bytes: Vec<_> = (0..fields[1].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&fields[1][i..i + 2], 16).unwrap())
            .collect();
        let number = |n: usize| fields[n].parse::<i64>().unwrap();
        if fields[0] == "button" {
            let value = buttons::button(&bytes).unwrap();
            assert_eq!(
                [
                    value.component as i64,
                    value.slot as i64,
                    value.object as i64,
                    value.op as i64
                ],
                [number(2), number(3), number(4), number(5)]
            );
            assert!(buttons::button(&bytes[..8]).is_err());
        } else {
            let value = buttons::drag(&bytes).unwrap();
            assert_eq!(
                [
                    value.source as i64,
                    value.source_slot as i64,
                    value.source_object as i64,
                    value.target as i64,
                    value.target_slot as i64,
                    value.target_object as i64
                ],
                [
                    number(2),
                    number(3),
                    number(4),
                    number(5),
                    number(6),
                    number(7)
                ]
            );
            assert!(buttons::drag(&bytes[..15]).is_err());
        }
        count += 1;
    }
    assert_eq!(count, 2048);
}
