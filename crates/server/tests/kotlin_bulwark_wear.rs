use gielinor_world::{
    bulwark_wear::Config,
    variables::{Transmit, Variables, Varp},
};
use std::collections::{BTreeMap, BTreeSet};
#[test]
fn original_bulwark_wear_matches_delay_overflow_slots_and_glow() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bulwark-wear.expected"
    ))
    .unwrap();
    assert_eq!(text.lines().count(), 576);
    for (case, line) in text.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let config = Config {
            items: if f[1] == "77" {
                BTreeSet::from([1])
            } else {
                BTreeSet::new()
            },
            glow: 1336,
            passive_delay: Varp {
                id: 0,
                permanent: true,
                transmit: match f[4] {
                    "0" => Transmit::Never,
                    "1" => Transmit::Different,
                    _ => Transmit::Always,
                },
                bit_protect: false,
                client_code: -1,
            },
        };
        let mut vars = Variables::restore(BTreeMap::from([(0, 42)]));
        let equipped: i32 = f[2].parse().unwrap();
        let change = config.changed(
            &mut vars,
            f[0].parse().unwrap(),
            1,
            (equipped != 0).then_some(equipped),
            f[3].parse().unwrap(),
            f[5] == "true",
        );
        assert_eq!(vars.get(0).to_string(), f[6], "state {case}");
        assert_eq!(
            change
                .as_ref()
                .map(|c| c.glow.to_string())
                .unwrap_or_default(),
            f[7],
            "packed default-slot spot {case}"
        );
        let packet = change
            .and_then(|c| c.variable)
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
            .unwrap_or_default();
        assert_eq!(packet, f[8], "packets {case}");
    }
}
