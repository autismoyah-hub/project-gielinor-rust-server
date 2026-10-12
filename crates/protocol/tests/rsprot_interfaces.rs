use gielinor_protocol::interfaces;
#[test]
fn original_interface_lifecycle_and_event_encoders() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/interfaces.expected"
    ))
    .expect("Run scripts/interface-parity.ps1");
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split_whitespace().collect();
        let packet = match f[0] {
            "top" => interfaces::open_top(f[1].parse().unwrap()),
            "open" => {
                interfaces::open_sub(f[1].parse().unwrap(), f[2].parse().unwrap(), f[3] == "1")
            }
            "close" => interfaces::close_sub(f[1].parse().unwrap()),
            "npchead" => interfaces::npc_head(f[1].parse().unwrap(), f[2].parse().unwrap()),
            "playerhead" => interfaces::player_head(f[1].parse().unwrap()),
            "animation" => interfaces::animation(
                f[1].parse().unwrap(),
                match f[2].parse::<i32>().unwrap() {
                    -1 => None,
                    value => Some(value as u16),
                },
            ),
            "move" => interfaces::move_sub(f[1].parse().unwrap(), f[2].parse().unwrap()),
            "text" => interfaces::text(f[1].parse().unwrap(), &format!("Bonus_{}€", f[2])).unwrap(),
            "events" => interfaces::events(
                f[1].parse().unwrap(),
                f[2].parse().unwrap(),
                f[3].parse().unwrap(),
                f[4].parse().unwrap(),
            )
            .unwrap(),
            _ => panic!("Unknown fixture row"),
        };
        let hex: String = packet.payload.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, *f.last().unwrap(), "{line}");
        count += 1;
    }
    assert_eq!(count, 4608);
}
