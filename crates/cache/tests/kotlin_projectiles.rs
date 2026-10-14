use gielinor_cache::projectiles::ProjectileType;
#[test]
fn original_decoder_required_fields_signed_values_and_flight_timing() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/projectiles.expected"
    ))
    .expect("Run scripts/projectiles-parity.ps1");
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let bytes: Vec<_> = (0..f[0].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&f[0][i..i + 2], 16).unwrap())
            .collect();
        let result = ProjectileType::decode(&bytes);
        if f[2] == "error" {
            assert!(result.is_err(), "{row}");
        } else {
            let value = result.unwrap();
            let s = value.style;
            assert_eq!(value.decoded_bytes, f[3].parse::<usize>().unwrap(), "{row}");
            let fields = [
                s.start_height,
                s.end_height,
                s.delay,
                s.angle,
                s.length_adjustment,
                s.progress,
                s.step_multiplier,
            ];
            assert_eq!(
                fields.to_vec(),
                f[4].split(',')
                    .map(|v| v.parse::<i32>().unwrap())
                    .collect::<Vec<_>>(),
                "{row}"
            );
            let duration = s.duration(f[1].parse().unwrap());
            assert_eq!(
                (duration.client_delay, duration.server_delay),
                (f[5].parse().unwrap(), f[6].parse().unwrap()),
                "{row}"
            );
        }
        cases += 1;
    }
    assert_eq!(cases, 4096);
}
