use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    tower_diary::Progress,
    tower_kit::Config,
};
fn decode(text: &str, stack: StackType, placeholders: bool) -> Inventory {
    let mut inv = Inventory::new(text.split(',').count(), stack, placeholders);
    for (slot, item) in text.split(',').enumerate() {
        if item == "." {
            continue;
        }
        let n = item
            .split(':')
            .map(|v| v.parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        inv.restore_slot(
            slot,
            Some(Item {
                id: n[0],
                count: n[1],
                vars: n[2],
            }),
        )
        .unwrap();
    }
    inv
}
fn encode(inv: &Inventory) -> String {
    inv.slots()
        .iter()
        .map(|i| match i {
            None => ".".into(),
            Some(i) => format!("{}:{}:{}", i.id, i.count, i.vars),
        })
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn kit_banking_matches_original_transfer_composition() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-kit.expected"
    ))
    .expect("Run scripts/tower-kit-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    let mut config = Config::default();
    for (i, ids) in config.named.iter_mut().enumerate() {
        ids.insert(10 + i as i32);
    }
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let mask = f[0].parse::<u32>().unwrap();
        let mut diary = Progress::default();
        for i in 0..7 {
            diary.done[i] = mask & (1 << i) != 0;
        }
        let mut inv = decode(f[1], StackType::Normal, false);
        let worn = decode(f[2], StackType::Normal, false);
        let mut bank = decode(f[3], StackType::Always, false);
        let result = config.bank_extras(&mut inv, &worn, &mut bank, &Catalog::default(), diary);
        assert_eq!(result.moved, f[4].parse::<i32>().unwrap(), "{line}");
        assert_eq!(encode(&inv), f[5], "{line}");
        assert_eq!(encode(&bank), f[6], "{line}");
    }
}
