use gielinor_world::{
    bank_actions::Binding,
    combat_stance::{self, Config, Weapon},
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
#[test]
fn compiled_stance_selection_matches_validation_saved_vars_and_bulwark_delay() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/combat-stance.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let category: u8 = f[0].parse().unwrap();
        let present = f[1] == "1";
        let types: Vec<_> = f[7]
            .split(',')
            .enumerate()
            .map(|(i, m)| Varp {
                id: i as u16,
                transmit: match m {
                    "0" => Transmit::Never,
                    "1" => Transmit::Different,
                    _ => Transmit::Always,
                },
                permanent: true,
                bit_protect: false,
                client_code: -1,
            })
            .collect();
        let normalized = combat_stance::category(present.then_some(category));
        let config = Config {
            stance: types[0],
            melee: types[1],
            retaliate: types[2],
            saved: if f[5] == "1" {
                BTreeMap::from([(
                    normalized,
                    Binding {
                        base: types[3],
                        bit: Varbit {
                            base: 3,
                            lsb: 6,
                            msb: 7,
                        },
                    },
                )])
            } else {
                BTreeMap::new()
            },
            styles: BTreeMap::from([(normalized, f[3].parse().unwrap())]),
            weapons: BTreeMap::from([(
                1,
                Weapon {
                    category,
                    bulwark: f[2] == "1",
                },
            )]),
        };
        let mut vars = Variables::restore(
            f[8].split(',')
                .enumerate()
                .map(|(i, v)| (i as u16, v.parse().unwrap()))
                .collect(),
        );
        let result = config
            .set(
                &mut vars,
                present.then_some(1),
                f[4].parse().unwrap(),
                f[6].parse::<i32>().unwrap() > 0,
            )
            .unwrap();
        assert_eq!(
            (0..4)
                .map(|id| vars.get(id).to_string())
                .collect::<Vec<_>>()
                .join(","),
            f[9],
            "state {case}"
        );
        assert_eq!(result.bulwark_delay, f[10] == "1", "delay {case}");
        let text = result
            .variables
            .into_iter()
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
        assert_eq!(text, f[11], "packets {case}");
    }
}
