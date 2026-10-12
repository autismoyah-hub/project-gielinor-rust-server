use gielinor_server::interface_controls::Control;
use gielinor_world::{
    bank_actions::Binding,
    variables::{Transmit, Varbit, Variables, Varp},
};
#[test]
fn kotlin_prayer_and_settings_changes_match_state_and_immediate_packets() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/interface-controls.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let kind: usize = f[0].parse().unwrap();
        let slot: i32 = f[1].parse().unwrap();
        let types: Vec<_> = f[2]
            .split(',')
            .enumerate()
            .map(|(i, mode)| Varp {
                id: i as u16,
                permanent: true,
                bit_protect: false,
                client_code: -1,
                transmit: match mode {
                    "0" => Transmit::Never,
                    "1" => Transmit::Different,
                    _ => Transmit::Always,
                },
            })
            .collect();
        let mut vars = Variables::restore(
            f[3].split(',')
                .enumerate()
                .map(|(i, v)| (i as u16, v.parse().unwrap()))
                .collect(),
        );
        let binding = |index: u8| Binding {
            base: types[0],
            bit: Varbit {
                base: 0,
                lsb: index * 2,
                msb: index * 2 + 1,
            },
        };
        let control = match kind {
            0..=4 => Control::ToggleSlot((0..5).map(binding).collect()),
            5..=8 => Control::Toggle(binding(kind as u8)),
            9 => Control::Priority {
                varp: types[1],
                last: 5,
            },
            10 => Control::Priority {
                varp: types[2],
                last: 4,
            },
            11 => Control::Resync(types[0]),
            _ => Control::Set(binding(9), (kind - 12) as i32),
        };
        let result = control.apply(
            &mut vars,
            if kind < 5 { kind as i32 } else { slot },
            f[4].parse::<i32>().unwrap() > 0,
        );
        assert_eq!(result.is_err(), f[5] == "1", "failure {case}");
        let state = (0..3)
            .map(|id| vars.get(id).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(state, f[6], "state {case}");
        let packet = result
            .ok()
            .flatten()
            .map(|u| gielinor_protocol::varp::update(u.id, u.value));
        let text = packet
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
            .unwrap_or_default();
        assert_eq!(text, f[7], "packets {case}");
    }
}
