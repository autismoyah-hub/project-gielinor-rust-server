use gielinor_cache::parameters::ParameterType;
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn compiled_parameter_decoder_defaults_types_and_ignored_trailing_data() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/parameters.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let expected = ParameterType {
            literal: (f[1] != "-").then(|| f[1].parse().unwrap()),
            default_int: (f[2] != "-").then(|| f[2].parse().unwrap()),
            default_string: (f[3] != "-").then(|| String::from_utf8(hex(f[3])).unwrap()),
            auto_disable: f[4].parse().unwrap(),
            transmit: f[5].parse().unwrap(),
            decoded_bytes: f[6].parse().unwrap(),
        };
        assert_eq!(
            ParameterType::decode(&hex(f[0])),
            Ok(expected),
            "case {case}"
        );
    }
    for bytes in [&[1][..], &[2, 0, 0], &[5, 65], &[7, 0, 0], &[8], &[255]] {
        assert!(ParameterType::decode(bytes).is_err());
    }
}
