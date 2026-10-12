use gielinor_world::{
    bank_actions::Binding,
    gameframe::{Frame, Frames},
    ui::{Component, State},
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
#[test]
fn compiled_gameframe_login_selects_saved_matching_fallback_and_default_layouts() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/gameframe.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    let binding = |id, msb| Binding {
        base: Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: id,
            lsb: 0,
            msb,
        },
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let frames = Frames {
            top: binding(0, 15),
            stones: binding(1, 0),
            ordered: f[0]
                .split(',')
                .enumerate()
                .map(|(i, flag)| {
                    let flag: u8 = flag.parse().unwrap();
                    Frame {
                        top: 100 + i as u16,
                        overlays: vec![],
                        mappings: BTreeMap::from([(Component(1), Component(i as u32 + 10))]),
                        client_mode: i as i32,
                        resizable: flag & 1 != 0,
                        stones: flag & 2 != 0,
                        default: i == f[1].parse::<usize>().unwrap(),
                    }
                })
                .collect(),
        };
        let mut vars = Variables::restore(BTreeMap::from([
            (0, f[2].parse().unwrap()),
            (1, f[4].parse().unwrap()),
        ]));
        let mut ui = State {
            frame_resizable: f[3] == "1",
            ..Default::default()
        };
        let selected = frames.login(&mut vars, &mut ui, false).unwrap();
        assert_eq!(ui.top_level, Some(f[5].parse().unwrap()), "case {case}");
        assert_eq!(ui.frame_resizable, f[6] == "1");
        assert_eq!(vars.get(0), f[7].parse().unwrap());
        assert_eq!(vars.get(1), f[8].parse().unwrap());
        assert_eq!(
            ui.translate(Component(1)),
            Component(selected.frame.top as u32 - 90)
        );
        assert_eq!(ui.translate(Component(2)), Component(2));
    }
}
