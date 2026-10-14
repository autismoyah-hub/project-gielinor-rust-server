use gielinor_protocol::spot_animation::{self, Animation};
#[test]
fn original_player_spots_match_slot_overwrite_order_and_wire_transforms() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/player-spots.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    for (case, line) in fixture.lines().enumerate() {
        let (inputs, expected) = line.split_once('|').unwrap();
        let input: Vec<_> = inputs
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let fields: Vec<u16> = s.split(':').map(|v| v.parse().unwrap()).collect();
                Animation {
                    slot: fields[0] as u8,
                    id: fields[1],
                    delay: fields[2],
                    height: fields[3],
                }
            })
            .collect();
        let bytes = spot_animation::encode(&input).unwrap();
        assert_eq!(
            bytes.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            expected,
            "case {case}"
        );
    }
}
