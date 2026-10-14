use gielinor_protocol::hint_arrow::{Arrow, TilePosition};

#[test]
fn original_hint_arrow_encoder_matches() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hint-arrow.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1152);
    let positions = [
        TilePosition::Center,
        TilePosition::West,
        TilePosition::East,
        TilePosition::South,
        TilePosition::North,
    ];
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let kind: usize = f[0].parse().unwrap();
        let x = f[1].parse().unwrap();
        let z = f[2].parse().unwrap();
        let height: i32 = f[3].parse().unwrap();
        let arrow = match kind {
            0 => Arrow::Reset,
            1 => Arrow::Npc(x),
            2 => Arrow::Player(x),
            8 => Arrow::WorldEntity {
                index: x,
                height: height as u32,
            },
            _ => Arrow::Tile {
                x,
                z,
                height: height as u8,
                position: positions[kind - 3],
            },
        };
        let packet = arrow.encode();
        assert_eq!(packet.opcode, 50);
        assert_eq!(packet.payload.len(), 6);
        let hex = packet
            .payload
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(hex, f[4], "case {case}");
    }
}
