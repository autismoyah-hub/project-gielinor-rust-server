use gielinor_world::tower_diary::Marks;
fn decode(text: &str) -> Marks {
    let v = text
        .split(',')
        .map(|n| n.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(v.len(), 16);
    Marks {
        counts: v[..6].try_into().unwrap(),
        levels: v[6..9].try_into().unwrap(),
        actions: std::array::from_fn(|i| v[9] & (1 << i) != 0),
        done: std::array::from_fn(|i| v[10] & (1 << i) != 0),
        pending: std::array::from_fn(|i| v[11] & (1 << i) != 0),
        bounty: v[12],
        schema11: v[13] != 0,
        imp_bones: v[14] != 0,
        staff_taken: v[15] != 0,
    }
}

use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType, Template},
    tower_chest::Config,
};
fn inventory(text: &str) -> Inventory {
    let mut inv = Inventory::new(5, StackType::Normal, false);
    for (slot, value) in text.split(',').enumerate() {
        if value == "." {
            continue;
        }
        let v: Vec<i32> = value.split(':').map(|n| n.parse().unwrap()).collect();
        inv.restore_slot(
            slot,
            Some(Item {
                id: v[0],
                count: v[1],
                vars: v[2],
            }),
        )
        .unwrap();
    }
    inv
}
#[test]
fn chest_grants_and_partial_failure_match_original_composition() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-chest.expected"
    ))
    .expect("Run scripts/tower-chest-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    let config = Config {
        ids: std::array::from_fn(|i| i as i32),
        merge_aliases: std::array::from_fn(|i| {
            let target = ["Thread", "Bronze arrow", "Coins"][i];
            [
                "Bronze axe",
                "Needle",
                "Thread",
                "Bronze pickaxe",
                "Tinderbox",
                "Small fishing net",
                "Knife",
                "Hammer",
                "Attack potion(3)",
                "Bronze arrow",
                "Coins",
                "Beer",
                "Coins",
                "COİNS",
                "Coınſ",
                "Coınß",
            ]
            .iter()
            .enumerate()
            .filter_map(|(id, name)| {
                (id != 12 && gielinor_world::tower_chest::name_matches(name, target))
                    .then_some(id as i32)
            })
            .collect()
        }),
    };
    let mut catalog = Catalog::default();
    catalog.stackable.extend([2, 9, 10, 13, 14, 15]);
    catalog.notes.insert(
        12,
        Template {
            link: 10,
            template: 799,
        },
    );
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 7);
        let mut inv = inventory(f[1]);
        let expected = inventory(f[4]);
        let outcome = config.claim(
            &mut inv,
            &inventory(f[2]),
            &inventory(f[3]),
            &catalog,
            decode(f[0]),
        );
        assert_eq!(inv.slots(), expected.slots(), "inventory {case}");
        assert_eq!(outcome.messages.join("~"), f[5], "messages {case}");
        let expected_marks = (f[6] != "-").then(|| decode(f[6]));
        assert_eq!(
            outcome.sync.map(|sync| sync.marks),
            expected_marks,
            "marks {case}"
        );
    }
}
