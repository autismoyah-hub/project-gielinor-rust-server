use gielinor_cache::components::ComponentType;
#[test]
fn original_component_encoder_and_decoder() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/components.expected"
    ))
    .expect("Run scripts/component-parity.ps1");
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split_whitespace().collect();
        let number = |i: usize| f[i].parse::<i64>().unwrap();
        let bytes: Vec<_> = (0..f[8].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&f[8][i..i + 2], 16).unwrap())
            .collect();
        let value = ComponentType::decode(number(0) as u32, &bytes).unwrap();
        assert_eq!(value.v3, f[1] == "true");
        assert_eq!(
            [
                value.kind as i64,
                value.button_type as i64,
                value.layer.map_or(-1, |v| v as i64),
                value.events as i64,
                value.operations.len() as i64,
                value.decoded_bytes as i64
            ],
            [
                number(2),
                number(3),
                number(4),
                number(5),
                number(6),
                number(7)
            ],
            "{line}"
        );
        count += 1;
    }
    assert_eq!(count, 1024);
}
