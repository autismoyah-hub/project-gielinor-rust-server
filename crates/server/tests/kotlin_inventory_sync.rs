use gielinor_server::inventory_sync::Transmissions;
use gielinor_world::inventory::{Inventory, Item, StackType};
use std::collections::BTreeMap;
fn ids(value: &str) -> Vec<u16> {
    if value == "-" {
        vec![]
    } else {
        value.split(',').map(|s| s.parse().unwrap()).collect()
    }
}
#[test]
fn original_processor_output_weight_and_shared_cleanup() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/inventory-sync.expected"
    ))
    .expect("Run scripts/inventory-sync-parity.ps1");
    let mut cases = 0;
    for line in fixture.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let mut inventories = BTreeMap::new();
        for encoded in fields[2].split(';') {
            let parts: Vec<_> = encoded.split('/').collect();
            let id: u16 = parts[0].parse().unwrap();
            let mut inventory = Inventory::new(8, StackType::Normal, false);
            for (slot, encoded) in parts[2].split(',').enumerate() {
                let item = if encoded == "." {
                    None
                } else {
                    let (id, count) = encoded.split_once(':').unwrap();
                    Some(Item {
                        id: id.parse().unwrap(),
                        count: count.parse().unwrap(),
                        vars: 0,
                    })
                };
                inventory.restore_slot(slot, item).unwrap();
            }
            inventory.clear_modified();
            for slot in ids(parts[3]) {
                inventory
                    .restore_slot(slot as usize, inventory.get(slot as usize))
                    .unwrap();
            }
            inventories.insert(id, (inventory, parts[1] == "true"));
        }
        let mut tx = Transmissions::default();
        for id in ids(fields[0]) {
            tx.start(id);
        }
        tx.cycle(|id| inventories.get(&id).map(|(inv, _)| inv))
            .unwrap();
        for id in ids(fields[1]) {
            tx.start(id);
        }
        let mut second = tx.clone();
        let first = tx
            .cycle(|id| inventories.get(&id).map(|(inv, _)| inv))
            .unwrap();
        let shared = second
            .cycle(|id| inventories.get(&id).map(|(inv, _)| inv))
            .unwrap();
        assert_eq!(first.packets, shared.packets);
        let encoded: Vec<_> = first
            .packets
            .iter()
            .map(|p| {
                let hex: String = p.payload.iter().map(|b| format!("{b:02x}")).collect();
                format!("{}:{hex}", p.opcode)
            })
            .collect();
        assert_eq!(
            if encoded.is_empty() {
                "-".into()
            } else {
                encoded.join(";")
            },
            fields[3],
            "{line}"
        );
        assert_eq!(
            tx.weight(
                |id| inventories.get(&id).map(|(inv, flag)| (inv, *flag)),
                |id| Some((id % 7 - 2) * 700)
            )
            .unwrap(),
            fields[4].parse::<i32>().unwrap()
        );
        assert_eq!(
            first.processed.iter().any(|id| inventories[id].1),
            fields[5] == "true"
        );
        for id in first.processed {
            inventories.get_mut(&id).unwrap().0.clear_modified();
        }
        let remaining: Vec<_> = inventories
            .values()
            .map(|(inv, _)| {
                if inv.modified().is_empty() {
                    "-".into()
                } else {
                    inv.modified()
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                }
            })
            .collect();
        assert_eq!(remaining.join(";"), fields[6]);
        cases += 1;
    }
    assert_eq!(cases, 1024);
}
