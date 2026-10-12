use gielinor_server::{gameframe_move::Config, interface_hooks::Hooks};
use gielinor_world::{
    bank_actions::Binding,
    gameframe::{Frame, Frames, Movement},
    ui::{Component, State},
    variables::{Transmit, Varbit, Variables, Varp},
};
#[test]
fn compiled_gameframe_transition_and_callbacks_match_full_packet_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/gameframe-transition.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let from: usize = f[0].parse().unwrap();
        let into: usize = f[1].parse().unwrap();
        let types: Vec<_> = f[3]
            .split(',')
            .enumerate()
            .map(|(i, m)| Varp {
                id: i as u16,
                permanent: true,
                bit_protect: false,
                client_code: -1,
                transmit: match m {
                    "0" => Transmit::Never,
                    "1" => Transmit::Different,
                    _ => Transmit::Always,
                },
            })
            .collect();
        let binding = |i, msb| Binding {
            base: types[i],
            bit: Varbit {
                base: i as u16,
                lsb: 0,
                msb,
            },
        };
        let targets: Vec<_> = (0..33).map(|i| Component::new(100, i)).collect();
        let frames = Frames {
            ordered: (0..3)
                .map(|i| Frame {
                    top: 200 + i,
                    overlays: vec![],
                    mappings: targets
                        .iter()
                        .enumerate()
                        .map(|(j, &c)| (c, Component::new(200 + i, 300 + j as u16)))
                        .collect(),
                    client_mode: i as i32,
                    resizable: i != 0,
                    default: i == 0,
                    stones: i == 2,
                })
                .collect(),
            top: binding(0, 15),
            stones: binding(1, 0),
        };
        let config = Config {
            button: 0,
            choices: [200, 201, 202],
            targets: targets.clone(),
            events: (0..14)
                .map(|i| (targets[6 + i], Component::new(100, 100 + i as u16)))
                .collect(),
            xp_drops: targets[24],
            ehc_listener: targets[32],
            orbs: (targets[23], 999),
            resync: [types[3], types[4]],
            minimap: binding(2, 3),
        };
        let mut vars = Variables::restore(
            f[4].split(',')
                .enumerate()
                .map(|(i, v)| (i as u16, v.parse().unwrap()))
                .collect(),
        );
        let intermediate: i32 = f[5].parse().unwrap();
        let movement = Movement {
            from: frames.ordered[from].clone(),
            into: frames.ordered[into].clone(),
            intermediate: if intermediate < 0 {
                None
            } else {
                Some(frames.ordered[intermediate as usize - 200].clone())
            },
        };
        let mut ui = State {
            top_level: Some(200 + from as u16),
            frame_resizable: into != 0,
            ..State::default()
        };
        let packets = config
            .execute(
                &frames,
                &movement,
                &mut ui,
                &mut vars,
                f[2].parse::<i32>().unwrap() > 0,
                &Hooks::default(),
            )
            .unwrap();
        assert_eq!(
            (0..5)
                .map(|id| vars.get(id).to_string())
                .collect::<Vec<_>>()
                .join(","),
            f[6],
            "state {case}"
        );
        assert_eq!(ui.top_level.unwrap().to_string(), f[7], "top {case}");
        let text = packets
            .iter()
            .map(|p| {
                format!(
                    "{}:{}",
                    p.opcode,
                    p.payload
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(text, f[8], "case {case}");
        if from != into {
            for i in 0..14 {
                assert_eq!(
                    ui.events.get(Component::new(100, 100 + i), -1),
                    Some(1u64 << 32)
                );
            }
        }
        assert_eq!(ui.overlays.get(targets[23]), Some(999));
    }
}
