use gielinor_world::{
    bank_actions::Binding,
    gameframe::{Frame, Frames},
    ui::State,
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
#[test]
fn compiled_gameframe_queue_matches_delay_intermediate_and_immediate_output() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/gameframe-move.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let from: u16 = f[0].parse().unwrap();
        let into: u16 = f[1].parse().unwrap();
        let varp = |id, transmit| Varp {
            id,
            transmit,
            permanent: true,
            bit_protect: false,
            client_code: -1,
        };
        let mode = match f[5] {
            "0" => Transmit::Never,
            "1" => Transmit::Different,
            _ => Transmit::Always,
        };
        let frames = Frames {
            ordered: (0..3)
                .map(|i| Frame {
                    top: 100 + i,
                    overlays: vec![],
                    mappings: BTreeMap::new(),
                    client_mode: i as i32,
                    resizable: i != 0,
                    default: i == 0,
                    stones: i == 2,
                })
                .collect(),
            top: Binding {
                base: varp(0, mode),
                bit: Varbit {
                    base: 0,
                    lsb: 0,
                    msb: 15,
                },
            },
            stones: Binding {
                base: varp(1, Transmit::Never),
                bit: Varbit {
                    base: 1,
                    lsb: 0,
                    msb: 0,
                },
            },
        };
        let mut vars = Variables::restore(BTreeMap::from([
            (0, 100 + from as i32),
            (1, f[3].parse().unwrap()),
        ]));
        let mut ui = State {
            top_level: Some(100 + from),
            frame_resizable: f[2] == "1",
            ..State::default()
        };
        let result = frames
            .prepare_move(
                100 + into,
                &mut vars,
                &mut ui,
                f[4].parse::<i32>().unwrap() > 0,
            )
            .unwrap();
        assert_eq!(f[6], "0");
        assert_eq!(result.delay.to_string(), f[7]);
        assert_eq!(result.movement.from.top.to_string(), f[8]);
        assert_eq!(result.movement.into.top.to_string(), f[9]);
        assert_eq!(
            result
                .movement
                .intermediate
                .as_ref()
                .map(|v| v.top as i32)
                .unwrap_or(-1)
                .to_string(),
            f[10]
        );
        assert_eq!(vars.get(0).to_string(), f[11]);
        assert_eq!(vars.get(1).to_string(), f[12]);
        assert_eq!(ui.frame_resizable, f[13] == "1");
        assert_eq!(ui.top_level.unwrap().to_string(), f[14]);
        let mut packets = Vec::new();
        if let Some(mode) = result.client_mode {
            packets.push(
                gielinor_protocol::client_script::run(
                    3998,
                    &[gielinor_protocol::client_script::Argument::Int(mode)],
                )
                .unwrap(),
            );
        }
        packets.extend(
            result
                .variables
                .into_iter()
                .map(|v| gielinor_protocol::varp::update(v.id, v.value)),
        );
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
        assert_eq!(text, f[15], "case {case}");
    }
}
