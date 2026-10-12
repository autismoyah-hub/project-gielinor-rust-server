use gielinor_world::{
    autocast::*,
    bank_actions::Binding,
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::{BTreeMap, BTreeSet};
#[test]
fn compiled_selection_matches_books_levels_staff_runes_cancel_and_packet_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/autocast-select.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let bindings: Vec<_> = f[8]
            .split(',')
            .enumerate()
            .map(|(i, m)| Binding {
                base: Varp {
                    id: i as u16,
                    transmit: match m {
                        "0" => Transmit::Never,
                        "1" => Transmit::Different,
                        _ => Transmit::Always,
                    },
                    permanent: true,
                    bit_protect: false,
                    client_code: -1,
                },
                bit: Varbit {
                    base: i as u16,
                    lsb: 0,
                    msb: 7,
                },
            })
            .collect();
        let config = Config {
            enabled: bindings[0],
            spell: bindings[1],
            defensive: bindings[2],
            staff: Saved {
                spell: bindings[3],
                defensive: bindings[4],
            },
            bladed: Saved {
                spell: bindings[5],
                defensive: bindings[6],
            },
            weapons: BTreeMap::from([(
                1,
                Weapon {
                    category: f[0].parse().unwrap(),
                    additional: if f[3] == "1" {
                        BTreeSet::from([1000])
                    } else {
                        BTreeSet::new()
                    },
                },
            )]),
            spells: if f[4] == "1" {
                BTreeMap::from([(
                    1,
                    Spell {
                        item: 1000,
                        requirements: vec![],
                    },
                )])
            } else {
                BTreeMap::new()
            },
            autocast_items: if f[5] == "1" {
                BTreeMap::from([(1, 1000)])
            } else {
                BTreeMap::new()
            },
            restricted: if f[2] == "1" {
                BTreeSet::from([1000])
            } else {
                BTreeSet::new()
            },
        };
        let mut vars = Variables::restore(
            f[9].split(',')
                .enumerate()
                .map(|(i, v)| (i as u16, v.parse().unwrap()))
                .collect(),
        );
        let book = Binding {
            base: Varp {
                id: 8,
                permanent: true,
                transmit: Transmit::Never,
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: 8,
                lsb: 0,
                msb: 1,
            },
        };
        book.set(&mut vars, f[13].parse().unwrap()).unwrap();
        let required = f[14].parse::<i32>().unwrap();
        let selection = gielinor_world::autocast_select::Config {
            book,
            magic_stat: 6,
            spells: [(
                1,
                gielinor_world::autocast_select::Access {
                    book: (required >= 0).then_some(required),
                    level: 50,
                },
            )]
            .into(),
        };
        let result = selection
            .select(
                &config,
                &mut vars,
                (f[1] == "1").then_some(1),
                f[12].parse().unwrap(),
                f[16] == "1",
                f[15].parse().unwrap(),
                f[7].parse::<i32>().unwrap() > 0,
                |_| {
                    Ok((f[6].parse::<i32>().unwrap() < 3)
                        .then_some("You do not have enough Air Runes to cast this spell.".into()))
                },
            )
            .unwrap();
        assert_eq!(result.restore_tab, f[17] == "1", "restore {case}");
        let events = result.events;
        assert_eq!(
            (0..7)
                .map(|i| vars.get(i).to_string())
                .collect::<Vec<_>>()
                .join(","),
            f[10],
            "state {case}"
        );
        let packets = events
            .into_iter()
            .map(|e| match e {
                Event::Variable(u) => gielinor_protocol::varp::update(u.id, u.value),
                Event::Message(s) => gielinor_protocol::chat::game(0, None, &s).unwrap(),
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
        assert_eq!(packets, f[11], "packets {case}");
    }
}
