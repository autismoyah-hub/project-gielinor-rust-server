use gielinor_cache::maps::Tiles;
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
#[test]
fn original_kotlin_terrain_rules_and_sparse_eof() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/terrain.expected"
    ))
    .expect("Generate with scripts/terrain-parity.ps1");
    assert_eq!(fixture.lines().count(), 64);
    for (case, line) in fixture.lines().enumerate() {
        let (input, expected) = line.split_once(' ').unwrap();
        let tiles = Tiles::decode(&hex(input)).unwrap();
        let expected = hex(expected);
        assert_eq!(expected.len(), 16384);
        for level in 0..4 {
            for x in 0..64 {
                for z in 0..64 {
                    assert_eq!(
                        tiles.get(x, z, level),
                        expected[level << 12 | x << 6 | z],
                        "case {case}, tile {level}/{x}/{z}"
                    );
                }
            }
        }
    }
}
