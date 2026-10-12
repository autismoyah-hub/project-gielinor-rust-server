use gielinor_world::{
    bank_actions::Binding,
    inventory::{Inventory, Item, StackType},
    magic::{Config, Pack},
    runes::{Source, Validation},
    variables::{Transmit, Varbit, Varp},
};
#[test]
fn compiled_rune_packs_match_all_inventory_flag_combinations() {
    let binding = Binding {
        base: Varp {
            id: 0,
            transmit: Transmit::Never,
            permanent: false,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: 0,
            lsb: 0,
            msb: 7,
        },
    };
    let mut config = Config {
        catalog: Default::default(),
        fake: [binding; 2],
        fountain: binding,
        blighted: binding,
        regular_pouches: Default::default(),
        divine_pouches: Default::default(),
        pouch_types: [binding; 4],
        pouch_counts: [(0, binding); 4],
        lms_pouch: 100,
        packs: Default::default(),
        names: Default::default(),
    };
    for id in 1..=22 {
        let lms = matches!(id,1..=4|9|12|14|19..=22);
        let blighted = match id {
            3..=6 => Some(101),
            7..=9 => Some(102),
            10..=11 => Some(103),
            13..=14 => Some(104),
            15..=22 => Some(105),
            _ => None,
        };
        config.packs.insert(id, Pack { lms, blighted });
    }
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rune-packs.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 5888);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let id = f[0].parse().unwrap();
        let flags: u8 = f[1].parse().unwrap();
        let mask: u8 = f[2].parse().unwrap();
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..6 {
            if mask & (1 << slot) != 0 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: 100 + slot as i32,
                        count: 3,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
        }
        let actual = match config.validate_pack(&inv, id, flags & 1 != 0, flags & 2 != 0) {
            None => "-".into(),
            Some(Validation::Unlimited) => "U".into(),
            Some(Validation::Enough(sources)) => match &sources[..] {
                [Source::Inventory { item, slot, count }] => format!("{item},{slot},{count}"),
                _ => panic!("unexpected sources"),
            },
            _ => panic!("invalid pack"),
        };
        assert_eq!(actual, f[3], "case {case}");
    }
}
