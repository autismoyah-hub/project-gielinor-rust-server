use gielinor_cache::sequences::SequenceType;
#[test]
fn sequence_decoder_and_animation_ticks_match_kotlin() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sequences.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let bytes: Vec<_> = f[0]
            .as_bytes()
            .chunks_exact(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect();
        let seq = SequenceType::decode(&bytes).unwrap();
        assert_eq!(
            seq.tick_duration(),
            f[1].parse::<i32>().unwrap(),
            "case {case}"
        );
        assert_eq!(seq.priority, f[2].parse::<u8>().unwrap(), "case {case}");
        assert_eq!(
            seq.duplicate_behaviour,
            f[3].parse::<u8>().unwrap(),
            "case {case}"
        );
        assert_eq!(
            seq.decoded_bytes,
            f[4].parse::<usize>().unwrap(),
            "case {case}"
        );
    }
}
