use gielinor_server::{
    appearance::Design,
    player_design::{Config, Control, Kits},
};
use gielinor_world::{
    bank_actions::Binding,
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;

#[test]
fn original_creator_controls_match_unsigned_colours_signed_kits_and_ui_writes() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/player-design.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let case: usize = f[0].parse().unwrap();
        let ints = |s: &str| {
            s.split(',')
                .map(|v| v.parse::<i32>().unwrap())
                .collect::<Vec<_>>()
        };
        let binding = |id| Binding {
            base: Varp {
                id,
                permanent: true,
                transmit: match f[9] {
                    "0" => Transmit::Never,
                    "1" => Transmit::Different,
                    _ => Transmit::Always,
                },
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: id,
                lsb: 2,
                msb: 3,
            },
        };
        let config = Config {
            kits: Kits {
                options: std::array::from_fn(|body| {
                    std::array::from_fn(|slot| {
                        if (case + slot + body) % 5 == 0 {
                            vec![]
                        } else {
                            vec![
                                (slot * 100 + body * 20) as u16,
                                (slot * 100 + body * 20 + 3) as u16,
                                (40000 + slot) as u16,
                            ]
                        }
                    })
                }),
            },
            body: binding(0),
            pronoun: binding(1),
        };
        let mut design = Design {
            body_type: f[4].parse().unwrap(),
            pronoun: f[5].parse().unwrap(),
            ..Default::default()
        };
        design.kits = ints(f[6])
            .into_iter()
            .map(|v| Some(v as u16))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        design.colours = ints(f[7])
            .into_iter()
            .map(|v| v as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let before = ints(f[8]);
        let mut vars = Variables::restore(BTreeMap::from([(0, before[0]), (1, before[1])]));
        let slot = f[2].parse().unwrap();
        let value = f[3].parse().unwrap();
        let control = match f[1] {
            "0" => Control::Kit { slot, delta: value },
            "1" => Control::Colour { slot, delta: value },
            "2" => Control::Body(value as u8),
            _ => Control::Pronoun(value),
        };
        let effects = config
            .apply(&mut design, &mut vars, control, f[10] == "true")
            .unwrap();
        assert_eq!(design.body_type.to_string(), f[11], "body {case}");
        assert_eq!(design.pronoun.to_string(), f[12], "pronoun {case}");
        assert_eq!(
            design.kits.map(|v| v.unwrap() as i32).to_vec(),
            ints(f[13]),
            "kits {case}"
        );
        assert_eq!(
            design.colours.map(i32::from).to_vec(),
            ints(f[14]),
            "colours {case}"
        );
        assert_eq!(effects.rebuild.to_string(), f[15], "rebuild {case}");
        assert_eq!(vec![vars.get(0), vars.get(1)], ints(f[16]), "vars {case}");
        let packets = effects
            .variables
            .iter()
            .map(|u| {
                let p = gielinor_protocol::varp::update(u.id, u.value);
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
        assert_eq!(packets, f[17], "packets {case}");
    }
}
