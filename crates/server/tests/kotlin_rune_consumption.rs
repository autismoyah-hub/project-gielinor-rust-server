use gielinor_world::{
    bank_actions::Binding,
    inventory::{Catalog, Inventory, Item, StackType},
    rune_consumption::{consume, ResultKind},
    runes::{Source, Validation},
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
#[test]
fn compiled_rune_consumption_matches_strict_transactions_pouch_failures_and_sunfire() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rune-consumption.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    let mut catalog = Catalog::default();
    catalog.stackable.extend(1..=4);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let mut inv = Inventory::new(8, StackType::Normal, false);
        for (slot, v) in f[0].split(',').enumerate() {
            let count: i32 = v.parse().unwrap();
            if count >= 0 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: 1 + slot as i32 % 4,
                        count,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
        }
        let mut vars = Variables::restore(
            f[1].split(',')
                .enumerate()
                .map(|(i, v)| (i as u16, v.parse().unwrap()))
                .collect(),
        );
        let bits: BTreeMap<_, _> = f[2]
            .split(',')
            .enumerate()
            .map(|(i, m)| {
                (
                    i as u32,
                    Binding {
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
                            msb: 15,
                        },
                    },
                )
            })
            .collect();
        let validations = if f[4].is_empty() {
            vec![Validation::Unlimited]
        } else {
            vec![Validation::Enough(
                f[4].split(';')
                    .map(|s| {
                        let parts: Vec<_> = s.split(',').collect();
                        if parts[0] == "I" {
                            Source::Inventory {
                                item: parts[1].parse().unwrap(),
                                slot: parts[2].parse().unwrap(),
                                count: parts[3].parse().unwrap(),
                            }
                        } else {
                            Source::Varbit {
                                id: parts[1].parse().unwrap(),
                                count: parts[2].parse().unwrap(),
                            }
                        }
                    })
                    .collect(),
            )]
        };
        let result = consume(
            &mut inv,
            &mut vars,
            &catalog,
            &bits,
            &validations,
            4,
            f[3] == "1",
        )
        .unwrap();
        let kind = match result.kind {
            ResultKind::AllUnlimited => "U",
            ResultKind::Consumed { sunfire: true } => "S",
            ResultKind::Consumed { sunfire: false } => "C",
            ResultKind::NotEnoughVarbit => "V",
            ResultKind::NotEnoughInventory => "I",
            ResultKind::InvalidPouchResult => "E",
        };
        assert_eq!(kind, f[5], "result {case}");
        assert_eq!(
            inv.slots()
                .iter()
                .map(|i| i.map_or("-".into(), |i| format!("{}:{}", i.id, i.count)))
                .collect::<Vec<_>>()
                .join(","),
            f[6],
            "inventory {case}"
        );
        assert_eq!(
            (0..4)
                .map(|i| vars.get(i).to_string())
                .collect::<Vec<_>>()
                .join(","),
            f[7],
            "pouch {case}"
        );
        let packets = result
            .updates
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
        assert_eq!(packets, f[8], "packets {case}");
    }
}
