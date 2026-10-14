use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType, Template},
    tower_offering::{self, Bones},
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
fn fallback_matches_original_transfer_and_named_selection() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-bones.expected"
    ))
    .expect("Run scripts/tower-bones-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    let mut catalog = Catalog::default();
    catalog.notes.insert(
        527,
        Template {
            link: 526,
            template: 799,
        },
    );
    let bones = Bones {
        named: [
            (526, "Bones"),
            (527, "Bones"),
            (532, "Big bones"),
            (999, "Burnt bones"),
            (1000, "Bone fragments"),
        ]
        .into_iter()
        .filter_map(|(id, name)| tower_offering::is_bones(name).then_some(id))
        .collect(),
    };
    for line in fixture.lines() {
        let f = line.split('|').collect::<Vec<_>>();
        let mut inv = decode(f[0], StackType::Normal, false);
        let mut bank = decode(f[1], StackType::Always, true);
        assert_eq!(
            bones.slot(&inv).map_or(-1, |slot| slot as i32),
            f[2].parse::<i32>().unwrap(),
            "{line}"
        );
        assert_eq!(bones.count(&inv), f[3].parse::<i32>().unwrap(), "{line}");
        assert_eq!(
            bones.take_from_bank(&mut inv, &mut bank, &catalog),
            f[4] == "true",
            "{line}"
        );
        assert_eq!(encode(&inv), f[5], "{line}");
        assert_eq!(encode(&bank), f[6], "{line}");
    }
}
