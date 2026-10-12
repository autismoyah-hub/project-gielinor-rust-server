use gielinor_world::{
    equipment::{self, WearCatalog, WearType},
    inventory::*,
    stats::Stats,
};
fn restore(text: &str) -> Inventory {
    let mut inv = Inventory::new(text.split(',').count(), StackType::Normal, false);
    for (slot, obj) in text.split(',').enumerate() {
        if obj == "." {
            continue;
        }
        let p: Vec<i32> = obj.split(':').map(|v| v.parse().unwrap()).collect();
        inv.restore_slot(
            slot,
            Some(Item {
                id: p[0],
                count: p[1],
                vars: p[2],
            }),
        )
        .unwrap();
    }
    inv
}
fn slots(inv: &Inventory) -> String {
    inv.slots()
        .iter()
        .map(|i| i.map_or(".".into(), |i| format!("{}:{}:{}", i.id, i.count, i.vars)))
        .collect::<Vec<String>>()
        .join(",")
}
#[test]
fn compiled_kotlin_equip_and_unequip_transactions() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/equipment.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    let mut c = Catalog::default();
    c.stackable.insert(4);
    let mut wear = WearCatalog::default();
    for id in 1..=5 {
        wear.types.insert(
            id,
            WearType {
                primary: Some(match id {
                    1 | 3 => 3,
                    2 => 5,
                    4 => 13,
                    _ => 0,
                }),
                secondary: [if id == 3 { Some(5) } else { None }, None],
                requirements: vec![],
            },
        );
    }
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let mut held = restore(f[2]);
        let mut worn = restore(f[3]);
        let slot = f[1].parse().unwrap();
        let result = if f[0] == "0" {
            equipment::equip(&mut held, &mut worn, slot, &c, &wear, &Stats::default())
        } else {
            equipment::unequip(&mut worn, &mut held, slot, &c)
        };
        let status = match result {
            Ok(_) => "Success".to_owned(),
            Err(equipment::Error::InvalidItem) => "InvalidObj".to_owned(),
            Err(equipment::Error::NotEnoughWornSpace) => "NotEnoughWornSpace".to_owned(),
            Err(equipment::Error::NotEnoughInventorySpace) => "NotEnoughInvSpace".to_owned(),
            Err(e) => format!("{e:?}"),
        };
        let expected = f[4];
        assert_eq!(status, expected, "case {case}: {line}");
        assert_eq!(slots(&held), f[5], "held case {case}: {line}");
        assert_eq!(slots(&worn), f[6], "worn case {case}: {line}");
    }
}
