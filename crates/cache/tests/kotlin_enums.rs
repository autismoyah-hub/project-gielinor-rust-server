use gielinor_cache::enums::{EnumType, Value};
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn compiled_enum_decoder_matches_defaults_duplicate_keys_and_replaced_maps() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/enums.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let decoded = EnumType::decode(&bytes(f[0])).unwrap();
        if f[1] == "null" {
            assert!(decoded.is_none(), "case {case}");
            continue;
        }
        let value = decoded.unwrap();
        assert_eq!(value.key_literal, f[1].parse().unwrap());
        assert_eq!(value.value_literal, f[2].parse().unwrap());
        assert_eq!(
            value.default_int,
            if f[3] == "null" {
                None
            } else {
                Some(f[3].parse().unwrap())
            }
        );
        assert_eq!(
            value.default_string,
            if f[4] == "-" {
                None
            } else {
                Some(String::from_utf8(bytes(f[4])).unwrap())
            }
        );
        assert_eq!(value.transmit, f[5] == "true");
        assert_eq!(value.decoded_bytes, f[6].parse().unwrap());
        let expected = f[7]
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|entry| {
                let (key, value) = entry.split_once(':').unwrap();
                let value = if value.starts_with('s') {
                    Value::String(String::from_utf8(bytes(&value[1..])).unwrap())
                } else {
                    Value::Int(value[1..].parse().unwrap())
                };
                (key.parse().unwrap(), value)
            })
            .collect();
        assert_eq!(value.values, expected, "case {case}");
    }
    assert!(EnumType::decode(&[6, 0, 1, 0]).is_err());
    assert!(EnumType::decode(&[255]).is_err());
}
