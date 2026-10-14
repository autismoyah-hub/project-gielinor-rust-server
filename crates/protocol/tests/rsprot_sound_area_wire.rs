use gielinor_protocol::zone::{Object, SoundArea};
#[test]
fn original_sound_area_enclosure_matches_narrowing() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sound-area-wire.expected"
    ))
    .expect("Run scripts/sound-area-wire-parity.ps1");
    assert_eq!(text.lines().count(), 4096);
    for (case, line) in text.lines().enumerate() {
        let (input, expected) = line.split_once('|').unwrap();
        let a: Vec<i32> = input.split(',').map(|v| v.parse().unwrap()).collect();
        let bytes = Object::SoundArea(SoundArea {
            id: a[0] as u16,
            delay: a[1] as u8,
            loops: a[2] as u8,
            range: a[3] as u8,
            drop_off_range: a[4] as u8,
            x: a[5] as u8,
            z: a[6] as u8,
        })
        .encode()
        .unwrap();
        assert_eq!(
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            expected,
            "case {case}"
        );
    }
}
