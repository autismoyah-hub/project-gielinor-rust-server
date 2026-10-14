use gielinor_world::{
    bank_actions::Binding,
    simulation::PlayerData,
    variables::{Transmit, Varbit, Variables, Varp},
    wearpos::{Config, Event, Item},
};
use std::collections::BTreeMap;
#[test]
fn original_wearpos_special_and_combat_updates_match_packet_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/wearpos.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let types: Vec<_> = f[7]
            .split(',')
            .enumerate()
            .map(|(id, m)| Varp {
                id: id as u16,
                permanent: true,
                transmit: match m {
                    "0" => Transmit::Never,
                    "1" => Transmit::Different,
                    _ => Transmit::Always,
                },
                bit_protect: false,
                client_code: -1,
            })
            .collect();
        let binding = |id: usize| Binding {
            base: types[id],
            bit: Varbit {
                base: id as u16,
                lsb: 0,
                msb: if id == 2 { 5 } else { 7 },
            },
        };
        let config = Config {
            special: types[0],
            stance: types[1],
            category: binding(2),
            level: binding(3),
            decimal: binding(4),
            combat_stats: [0, 1, 2, 3, 4, 6, 5],
            items: BTreeMap::from([(
                1,
                Item {
                    category: f[0].parse().unwrap(),
                    two_handed: f[2] == "1",
                    sound: match f[11] {
                        "0" => None,
                        "1" => Some(2238),
                        _ => Some(20),
                    },
                    player_op: (f[11] == "2").then(|| "Pelt".into()),
                },
            )]),
        };
        let mut data = PlayerData::new(
            1,
            "oracle".into(),
            gielinor_world::CoordGrid::new(3224, 3218, 0).unwrap(),
            Default::default(),
        );
        if f[1] == "1" {
            data.equipment
                .restore_slot(
                    3,
                    Some(gielinor_world::inventory::Item {
                        id: 1,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        for (id, level) in config.combat_stats.into_iter().zip(f[5].split(',')) {
            data.stats.set_base(id, level.parse().unwrap());
        }
        data.variables = Variables::restore(
            f[8].split(',')
                .enumerate()
                .map(|(id, v)| (id as u16, v.parse().unwrap()))
                .collect(),
        );
        let events = config
            .changed(
                &mut data,
                f[3].parse().unwrap(),
                1,
                f[4].parse().unwrap(),
                f[6].parse::<i32>().unwrap() > 0,
            )
            .unwrap();
        assert_eq!(
            (0..5)
                .map(|id| data.variables.get(id).to_string())
                .collect::<Vec<_>>()
                .join(","),
            f[9],
            "state {case}"
        );
        let packets = events
            .into_iter()
            .filter_map(|e| match e {
                Event::Variable(u) => Some(gielinor_protocol::varp::update(u.id, u.value)),
                Event::CombatRange(level) => Some(
                    gielinor_protocol::client_script::run(
                        5224,
                        &[gielinor_protocol::client_script::Argument::Int(level)],
                    )
                    .unwrap(),
                ),
                Event::Sound(sound) => Some(gielinor_protocol::game::synth_sound(sound, 1, 0)),
                Event::PlayerOp(op) => Some(
                    gielinor_protocol::login_init::player_operation(5, op.is_some(), op.as_deref())
                        .unwrap(),
                ),
            })
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
        assert_eq!(packets, f[10], "packets {case}");
    }
}
