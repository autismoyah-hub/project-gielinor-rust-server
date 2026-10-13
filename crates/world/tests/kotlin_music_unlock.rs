use gielinor_world::{
    music::Track,
    variables::{Transmit, Variables, Varp},
};
#[test]
fn original_kotlin_music_unlock_preserves_bit_flags() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/music-unlock.expected"
    ))
    .expect("Run scripts/music-unlock-parity.ps1");
    assert_eq!(fixture.lines().count(), 54);
    let varp = Varp {
        id: 20,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let track = Track {
            id: 1,
            row: 1,
            name: "Track".into(),
            unlock_hint: String::new(),
            duration: 100,
            midi: 1,
            unlock_varp: Some(varp),
            unlock_bit: fields[1].parse().unwrap(),
            hidden: false,
            secondary: None,
        };
        let mut vars = Variables::default();
        vars.set(varp, fields[0].parse().unwrap(), false);
        assert!(track
            .unlock(&mut vars, true, false, false)
            .unwrap()
            .is_empty());
        assert_eq!(vars.get(20), fields[2].parse::<i32>().unwrap(), "{line}");
        assert_eq!(track.unlocked(&vars), fields[3] == "true", "{line}");
    }
}
