use gielinor_protocol::npc_sync::{Movement, Npc, Position, State};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn npc() -> Npc<'static> {
    Npc {
        index: 1,
        generation: 1,
        type_id: 42,
        position: Position {
            x: 8192,
            z: 8192,
            level: 0,
        },
        visible: true,
        teleport: false,
        jump: true,
        direction: 6,
        spawn_cycle: None,
        movement: Movement::None,
        extended: &[],
        initial_extended: &[],
    }
}
#[test]
fn original_rsprot_npc_additions_and_movement() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/npcs.expected"
    ))
    .expect("Run scripts/npc-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    let origin = npc().position;
    let directions = [
        (-1, 1),
        (0, 1),
        (1, 1),
        (-1, 0),
        (1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ];
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let mut state = State::default();
        let mut actor = npc();
        let (large, expected) = if f[0] == "spawn" {
            actor.index = n(3) as u16;
            actor.type_id = n(4) as u16;
            actor.position.x = (8192 + n(5)) as u16;
            actor.position.z = (8192 + n(6)) as u16;
            actor.direction = n(7) as u8;
            actor.spawn_cycle = Some(f[8].parse().unwrap());
            actor.jump = f[9] == "true";
            if f[2] == "true" {
                actor.initial_extended = &[1, 2];
            }
            (f[1] == "true", f[10])
        } else {
            state.encode(origin, false, &[npc()], 255).unwrap();
            let a = n(2) as u8;
            let b = n(3) as u8;
            actor.movement = match n(1) {
                0 => Movement::None,
                1 => Movement::Walk(a),
                2 => Movement::Crawl(a),
                3 => Movement::Run(a, b),
                _ => panic!(),
            };
            let mut step = |d: u8| {
                let (dx, dz) = directions[d as usize];
                actor.position.x = (actor.position.x as i32 + dx) as u16;
                actor.position.z = (actor.position.z as i32 + dz) as u16;
            };
            if n(1) > 0 {
                step(a);
            }
            if n(1) == 3 {
                step(b);
            }
            if f[4] == "true" {
                actor.extended = &[1, 2];
            }
            (false, f[5])
        };
        assert_eq!(
            hex(&state.encode(origin, large, &[actor], 255).unwrap()),
            expected,
            "case {case}: {line}"
        );
    }
}
