#[test]
fn rsprot_chat_and_player_sequence() {
    let mut count = 0;
    for line in include_str!("fixtures/food-output.expected").lines() {
        let f: Vec<_> = line.split('\t').collect();
        let (actual, expected) = if f[0] == "chat" {
            (
                gielinor_protocol::chat::game(
                    f[1].parse().unwrap(),
                    (f[2] != "-").then_some(f[2]),
                    f[3],
                )
                .unwrap()
                .payload,
                f[4],
            )
        } else {
            let bytes = gielinor_protocol::appearance::Updates {
                sequence: Some((f[1].parse().unwrap(), f[2].parse().unwrap())),
                ..Default::default()
            }
            .encode()
            .unwrap();
            assert_eq!(bytes[0], 64);
            (bytes[1..].to_vec(), f[3])
        };
        let hex: String = actual.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, expected, "{line}");
        count += 1;
    }
    assert_eq!(count, 2048);
}
