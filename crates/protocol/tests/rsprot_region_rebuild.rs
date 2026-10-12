use gielinor_protocol::region_rebuild::{encode, Zone};
#[test]
fn region_packets_match_original_encoder_including_empty_and_reused_chunks() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/region-rebuild.expected"
    ))
    .expect("Run scripts/region-rebuild-parity.ps1");
    assert_eq!(fixture.lines().count(), 128);
    for line in fixture.lines() {
        let (seed, expected) = line.split_once(' ').unwrap();
        let seed: i32 = seed.parse().unwrap();
        let cx = seed * 131 % 2048;
        let cz = seed * 73 % 2048;
        let packet = encode(cx as u16, cz as u16, seed % 2 == 0, |x, z, level| {
            let index = level as i32 * 169 + (x - cx + 6) * 13 + z - cz + 6;
            (seed % 8 != 0 && (index + seed) % 5 != 0).then_some(Zone {
                x: ((index * 17 + seed) % 1024) as u16,
                z: ((index * 31 + seed) % 2048) as u16,
                level: ((index + seed) % 4) as u8,
                rotation: ((index / 3 + seed) % 4) as u8,
            })
        })
        .unwrap();
        let actual: String = packet.payload.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(actual, expected, "case {seed}");
    }
}
