#[test]
fn sound_and_map_flag_match_actual_rsprot_encoders() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/door-effects.expected"
    ))
    .expect("Run scripts/door-effects-parity.ps1");
    assert_eq!(rows.lines().count(), 513);
    for line in rows.lines() {
        let (input, expected) = line.split_once('|').unwrap();
        let packet = if input == "reset" {
            gielinor_protocol::game::clear_map_flag()
        } else {
            let n: Vec<u16> = input.split(',').map(|s| s.parse().unwrap()).collect();
            gielinor_protocol::game::synth_sound(n[0], n[1] as u8, n[2])
        };
        let actual: String = packet.payload.iter().map(|v| format!("{v:02x}")).collect();
        assert_eq!(actual, expected);
    }
}
