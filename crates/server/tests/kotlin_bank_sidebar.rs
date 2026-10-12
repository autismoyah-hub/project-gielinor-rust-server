use gielinor_cache::{
    definitions::{ItemType, Param},
    game::Definitions,
    parameters::ParameterType,
};
use gielinor_world::{
    inventory::{Inventory, Item, StackType},
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
#[test]
fn compiled_sidebar_masks_follow_special_custom_wear_and_consumable_priority() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bank-sidebar.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    let param_symbols = [
        "bankside_extraop_conditional_bit",
        "bankside_extraop_conditional_varbit",
        "bankside_extraop_conditional_flip",
        "bankside_extraop",
        "wear_op_index",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, s)| (s.into(), i as i32 + 1))
    .collect();
    let outputs = [
        ("if1".into(), 400),
        ("if2".into(), 401),
        ("if3".into(), 402),
    ]
    .into_iter()
    .collect();
    let groups = [("food".into(), 100), ("potion".into(), 101)]
        .into_iter()
        .collect();
    let types = BTreeMap::from([
        (3, ParameterType::decode(&[2, 0, 0, 0, 0, 0]).unwrap()),
        (4, ParameterType::decode(&[5, 0, 0]).unwrap()),
        (5, ParameterType::decode(&[2, 0, 0, 0, 3, 0]).unwrap()),
    ]);
    let texts = ["", " ", "\u{85}", "\u{1c}", "Custom", "\u{a0}"];
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let mut defs = Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::from([(400, varp(400)), (401, varp(401)), (402, varp(402))]),
            stats: BTreeMap::new(),
        };
        let mut vars = Variables::default();
        for (i, descriptor) in f[0].split(';').enumerate() {
            let d: Vec<_> = descriptor.split(':').collect();
            let mut item = ItemType::decode(&[0]).unwrap();
            item.wear[0] = d[4].parse().unwrap();
            item.content_group = d[7].parse().unwrap();
            item.inventory_ops[2] = Some(texts[d[6].parse::<usize>().unwrap()].into());
            item.params.insert(3, Param::Integer(d[2].parse().unwrap()));
            item.params.insert(
                4,
                Param::String(texts[d[3].parse::<usize>().unwrap()].into()),
            );
            item.params.insert(5, Param::Integer(d[5].parse().unwrap()));
            if d[0] != "-" {
                item.params.insert(1, Param::Integer(d[0].parse().unwrap()));
                item.params.insert(2, Param::Integer(500 + i as i32));
            }
            defs.items.insert(i as u32 + 1, item);
            defs.varbits.insert(
                500 + i as u32,
                Varbit {
                    base: 200 + i as u16,
                    lsb: 0,
                    msb: 30,
                },
            );
            vars.set(varp(200 + i as u16), d[1].parse().unwrap(), false);
        }
        let mut inv = Inventory::new(36, StackType::Normal, false);
        for (slot, item) in f[1].split(',').enumerate() {
            let id = item.parse::<i32>().unwrap();
            if id != 0 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
        }
        let sidebar = gielinor_server::content::bank_sidebar(
            &param_symbols,
            &outputs,
            &groups,
            &types,
            &defs,
        )
        .unwrap();
        let expected: Vec<i32> = f[2].split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            &sidebar.masks(&inv, &vars).unwrap()[..],
            &expected,
            "case {case}"
        );
        assert_eq!(
            sidebar
                .refresh(&inv, &mut vars)
                .unwrap()
                .iter()
                .map(|u| u.value)
                .collect::<Vec<_>>(),
            expected
        );
    }
}
