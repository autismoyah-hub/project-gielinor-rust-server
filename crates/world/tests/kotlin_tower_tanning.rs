use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType, Template},
    tower_tanning,
};
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
fn descending_tanning_composition_matches_original_classifier_and_transactions() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-tanning.expected"
    ))
    .expect("Run scripts/tower-tanning-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    let mut catalog = Catalog::default();
    catalog.stackable.insert(900);
    catalog.notes.insert(
        903,
        Template {
            link: 1739,
            template: 799,
        },
    );
    let config = tower_tanning::Config {
        hides: [
            (1739, "renamed"),
            (900, "Heavy Cowhide"),
            (901, "Cow hide"),
            (902, "Leather body"),
            (903, "Cowhide"),
            (1741, "Leather"),
        ]
        .into_iter()
        .filter(|(id, name)| tower_tanning::is_cowhide(*id, name, *id == 903))
        .map(|(id, _)| id)
        .collect(),
        leather: 1741,
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let mut inv = Inventory::new(
            5,
            [StackType::Normal, StackType::Always, StackType::Never]
                [f[0].parse::<usize>().unwrap()],
            false,
        );
        for (slot, item) in f[1].split(',').enumerate() {
            if item != "." {
                let n: Vec<i32> = item.split(':').map(|s| s.parse().unwrap()).collect();
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
        }
        let result = config.tan(&mut inv, &catalog);
        assert_eq!(result.found_hides, f[2] == "true", "found {case}");
        assert_eq!(
            result.converted,
            f[3].parse::<i32>().unwrap(),
            "converted {case}"
        );
        assert_eq!(result.no_space, f[4] == "true", "space {case}");
        assert_eq!(encode(&inv), f[5], "inventory {case}");
    }
}
