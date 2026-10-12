use gielinor_server::combat_tab::Presentation;
use gielinor_world::{
    bank_actions::Binding,
    variables::{Transmit, Varbit, Variables, Varp},
};
#[test]
fn compiled_combat_sidebar_preserves_duplicate_scripts_and_level_var_updates() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/combat-tab.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    let encode = |packets: Vec<gielinor_protocol::framing::Packet>| {
        packets
            .into_iter()
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
            .join(",")
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let types: Vec<_> = f[5]
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
        let config = Presentation {
            interface: 1,
            category: binding(0, 5),
            level: binding(1, 7),
            decimal: binding(2, 7),
        };
        let mut vars = Variables::restore(
            f[6].split(',')
                .enumerate()
                .map(|(i, v)| (i as u16, v.parse().unwrap()))
                .collect(),
        );
        let logged = f[4].parse::<i32>().unwrap() > 0;
        let combat = f[2].parse().unwrap();
        let packets = config
            .tab(
                &mut vars,
                if f[1] == "1" {
                    Some(f[0].parse().unwrap())
                } else {
                    None
                },
                combat,
                logged,
            )
            .unwrap();
        let state = |vars: &Variables| {
            (0..3)
                .map(|id| vars.get(id).to_string())
                .collect::<Vec<_>>()
                .join(",")
        };
        assert_eq!(state(&vars), f[7], "tab state {case}");
        assert_eq!(encode(packets), f[8], "tab packets {case}");
        let packets = config
            .level(&mut vars, combat, f[3].parse().unwrap(), logged)
            .unwrap();
        assert_eq!(state(&vars), f[9], "level state {case}");
        assert_eq!(encode(packets), f[10], "level packets {case}");
    }
}
