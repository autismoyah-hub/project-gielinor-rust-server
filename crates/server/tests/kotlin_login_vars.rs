use gielinor_world::{
    bank_actions::Binding,
    variables::{Transmit, Varbit, Variables, Varp},
};
#[test]
fn compiled_login_vars_preserve_muted_settings_and_transmission_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/login-vars.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    let names = ["Name", " ", "\u{85}", "\u{1c}", "\u{a0}", ""];
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let types: Vec<_> = f[0]
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
        let values = f[1]
            .split(',')
            .enumerate()
            .filter(|(_, v)| *v != "-")
            .map(|(i, v)| (i as u16, v.parse().unwrap()))
            .collect();
        let mut vars = Variables::restore(values);
        let config = gielinor_server::login_init::Config {
            chatbox: Binding {
                base: types[4],
                bit: Varbit {
                    base: 4,
                    lsb: 2,
                    msb: 2,
                },
            },
            roofs: Binding {
                base: types[4],
                bit: Varbit {
                    base: 4,
                    lsb: 3,
                    msb: 3,
                },
            },
            volumes: types[..4].try_into().unwrap(),
            music_mode: None, // Oracle covers LoginScript only, not MusicAreaScript.
            transmit: types,
        };
        let packets = config
            .variables(
                &mut vars,
                names[f[2].parse::<usize>().unwrap()],
                f[3].parse().unwrap(),
            )
            .unwrap();
        let state = (0..8)
            .map(|id| {
                if vars.contains(id) {
                    vars.get(id).to_string()
                } else {
                    "-".into()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(state, f[4], "vars {case}");
        let encoded = packets
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
        assert_eq!(encoded, f[5], "packets {case}");
    }
}
