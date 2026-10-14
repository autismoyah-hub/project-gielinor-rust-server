use gielinor_protocol::zone::{Object, Projectile};
#[test]
fn original_enclosed_projectile_bytes_match_signed_indices_and_narrowing() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/projectile-wire.expected"
    ))
    .expect("Run scripts/projectile-wire-parity.ps1");
    let mut cases = 0;
    for row in fixture.lines() {
        let (input, expected) = row.split_once('|').unwrap();
        let a: Vec<i32> = input.split(',').map(|v| v.parse().unwrap()).collect();
        let value = Projectile {
            id: a[0] as u16,
            start_height: a[1] as u16,
            end_height: a[2] as u16,
            start_time: a[3] as u16,
            end_time: a[4] as u16,
            angle: a[5] as u8,
            progress: a[6] as u16,
            source_index: a[7],
            target_index: a[8],
            x: a[9] as u8,
            z: a[10] as u8,
            end: (a[13] << 28) | (a[11] << 14) | a[12],
        };
        let bytes = Object::Projectile(value).encode().unwrap();
        assert_eq!(bytes.len(), 25, "{row}");
        assert_eq!(
            bytes.iter().map(|v| format!("{v:02x}")).collect::<String>(),
            expected,
            "{row}"
        );
        cases += 1;
    }
    assert_eq!(cases, 4096);
}
