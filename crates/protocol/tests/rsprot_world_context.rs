use gielinor_protocol::world_context;
#[test]
fn original_root_context_and_empty_world_entity_stream() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/world-context.expected"
    ))
    .expect("Run scripts/world-context-parity.ps1");
    let mut count = 0;
    for line in fixture.lines() {
        let words: Vec<_> = line.split_whitespace().collect();
        let packet = match words[0] {
            "active" => world_context::active_root(words[1].parse().unwrap()).unwrap(),
            "origin" => {
                world_context::npc_origin(words[1].parse().unwrap(), words[2].parse().unwrap())
            }
            "empty" => world_context::empty_entities(),
            "energy" => gielinor_protocol::framing::Packet {
                opcode: 64,
                payload: gielinor_protocol::game::run_energy(words[1].parse().unwrap())
                    .unwrap()
                    .to_vec(),
            },
            "weight" => gielinor_protocol::framing::Packet {
                opcode: 31,
                payload: gielinor_protocol::game::run_weight(words[1].parse().unwrap()).to_vec(),
            },
            "stop" => gielinor_protocol::framing::Packet {
                opcode: 45,
                payload: gielinor_protocol::game::inventory_stop(words[1].parse().unwrap())
                    .to_vec(),
            },
            _ => panic!("Unknown fixture row"),
        };
        let actual: String = packet
            .payload
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(actual, *words.last().unwrap(), "{line}");
        count += 1;
    }
    assert_eq!(count, 1032);
}
