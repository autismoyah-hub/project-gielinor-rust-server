use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    smelting::{resolve_ingredient, ItemType},
};
use std::collections::BTreeMap;

#[test]
fn original_smelting_resolver_inventory_order_and_case_semantics() {
    let text = |s: &str| {
        String::from_utf8(
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect(),
        )
        .unwrap()
    };
    let mut cases = 0;
    for line in include_str!("fixtures/smelting-ingredient.expected").lines() {
        let fields: Vec<_> = line.split('|').collect();
        let flags: u8 = fields[4].parse().unwrap();
        let types: BTreeMap<_, _> = (1..=3)
            .map(|id| {
                (
                    id,
                    ItemType {
                        name: text(fields[if id == 2 { 1 } else { 0 }]),
                        certificate: id == 2 && flags & 1 != 0,
                        placeholder: id == 2 && flags & 2 != 0,
                    },
                )
            })
            .collect();
        let stack = match fields[2] {
            "0" => StackType::Normal,
            "1" => StackType::Always,
            "2" => StackType::Never,
            _ => panic!("unknown stack mode"),
        };
        let mut inv = Inventory::new(4, stack, false);
        for (slot, value) in fields[5].split(',').enumerate() {
            if value == "." {
                continue;
            }
            let (id, count) = value.split_once(':').unwrap();
            inv.restore_slot(
                slot,
                Some(Item {
                    id: id.parse().unwrap(),
                    count: count.parse().unwrap(),
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let mut catalog = Catalog::default();
        if fields[3] == "1" {
            catalog.stackable.extend(1..=3);
        }
        assert_eq!(
            resolve_ingredient(&inv, &catalog, &types, 1).unwrap(),
            fields[6].parse::<i32>().unwrap(),
            "{line}"
        );
        cases += 1;
    }
    assert_eq!(cases, 3072);
}
