use gielinor_server::{interface_hooks::Hooks, journal::Journal};
use gielinor_world::{
    bank_actions::Binding,
    ui::{Component, State},
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
#[test]
fn compiled_journal_open_selects_tabs_and_initializes_summary_in_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/journal.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    let config = Journal {
        interface: 100,
        tabs: [200, 201, 202],
        parent: Component::new(100, 5),
        contents: Component::new(200, 6),
        click_layer: Component::new(200, 7),
        selected: Binding {
            base: Varp {
                id: 0,
                permanent: true,
                transmit: Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: 0,
                lsb: 0,
                msb: 3,
            },
        },
        playtime: 1,
        switching: None,
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let tab: i32 = f[0].parse().unwrap();
        let vars = Variables::restore(BTreeMap::from([(0, tab), (1, f[1].parse().unwrap())]));
        let mut ui = State::default();
        let packets = config
            .opened(
                100,
                &mut ui,
                &vars,
                f[2].parse().unwrap(),
                &Hooks::default(),
            )
            .unwrap();
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
        assert_eq!(text, f[3], "case {case}");
        assert_eq!(
            ui.overlays.get(config.parent),
            Some(if tab < 3 { 200 + tab as u16 } else { 200 })
        );
        assert_eq!(
            vars.get(0),
            tab,
            "unknown values fall back without rewriting saves"
        );
    }
}
#[test]
fn compiled_journal_switch_closes_prepares_and_opens_in_source_order() {
    use gielinor_server::journal::Switching;
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/journal-switch.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let previous: i32 = f[0].parse().unwrap();
        let combat: i32 = f[2].parse().unwrap();
        let types: Vec<_> = f[6]
            .split(',')
            .enumerate()
            .map(|(i, mode)| Varp {
                id: if i == 0 { 0 } else { i as u16 + 1 },
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
        let mut values: BTreeMap<u16, i32> = f[7]
            .split(',')
            .enumerate()
            .map(|(i, v)| (i as u16 + 2, v.parse().unwrap()))
            .collect();
        values.insert(0, previous);
        values.insert(1, f[1].parse().unwrap());
        let mut vars = Variables::restore(values);
        let config = Journal {
            interface: 100,
            tabs: [200, 201, 202],
            parent: Component::new(100, 5),
            contents: Component::new(200, 6),
            click_layer: Component::new(200, 7),
            selected: Binding {
                base: types[0],
                bit: Varbit {
                    base: 0,
                    lsb: 0,
                    msb: 3,
                },
            },
            playtime: 1,
            switching: Some(Switching {
                buttons: BTreeMap::new(),
                collection: types[1..].try_into().unwrap(),
            }),
        };
        let mut ui = State::default();
        config
            .opened(100, &mut ui, &vars, combat, &Hooks::default())
            .unwrap();
        let packets = config
            .switch(
                f[3].parse().unwrap(),
                &mut ui,
                &mut vars,
                combat,
                f[4] == "1",
                f[5].parse::<i32>().unwrap() > 0,
                &Hooks::default(),
            )
            .unwrap();
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
        assert_eq!(vars.get(0).to_string(), f[8], "state {case}");
        assert_eq!(text, f[9], "case {case}");
        assert_eq!(
            ui.overlays.get(config.parent),
            Some(200 + f[3].parse::<u16>().unwrap())
        );
    }
}
