use gielinor_world::{
    bank::{Banking, Tabs},
    inventory::{Catalog, Inventory, Item, StackType, Template},
};
fn inventory(text: &str, stack: StackType) -> Inventory {
    let mut inv = Inventory::new(text.split(',').count(), stack, stack == StackType::Always);
    for (slot, item) in text.split(',').enumerate() {
        if item == "." {
            continue;
        }
        let f: Vec<i32> = item.split(':').map(|s| s.parse().unwrap()).collect();
        inv.restore_slot(
            slot,
            Some(Item {
                id: f[0],
                count: f[1],
                vars: f[2],
            }),
        )
        .unwrap();
    }
    inv
}
#[test]
fn compiled_bulk_deposits_match_main_and_numbered_tab_transactions() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bank-bulk.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    let mut catalog = Catalog::default();
    catalog.stackable.extend([1, 3]);
    catalog.notes.insert(
        2,
        Template {
            link: 3,
            template: 0,
        },
    );
    catalog.notes.insert(
        3,
        Template {
            link: 2,
            template: 799,
        },
    );
    catalog.placeholders.insert(
        2,
        Template {
            link: 4,
            template: 0,
        },
    );
    catalog.placeholders.insert(
        4,
        Template {
            link: 2,
            template: 1,
        },
    );
    catalog.transforms.insert(
        5,
        Template {
            link: 6,
            template: 0,
        },
    );
    catalog.transforms.insert(
        6,
        Template {
            link: 5,
            template: 1,
        },
    );
    let restricted = [8].into_iter().collect();
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let sizes: Vec<usize> = f[0].split(',').map(|s| s.parse().unwrap()).collect();
        let selected: usize = f[1].parse().unwrap();
        let mut banking = Banking {
            tabs: Tabs::restore(sizes.try_into().unwrap(), 32).unwrap(),
            selected: if selected == 0 { 9 } else { selected - 1 },
            capacity: f[2].parse().unwrap(),
            withdraw_notes: false,
            always_placehold: false,
        };
        let mut source = inventory(f[3], StackType::Normal);
        let mut bank = inventory(f[4], StackType::Always);
        let _ = banking.deposit_all(&mut source, &mut bank, &catalog, &restricted);
        assert_eq!(
            source.slots(),
            inventory(f[5], StackType::Normal).slots(),
            "source case {case}: {line}"
        );
        assert_eq!(
            bank.slots(),
            inventory(f[6], StackType::Always).slots(),
            "bank case {case}: {line}"
        );
        let sizes: Vec<usize> = f[7].split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(
            &banking.tabs.sizes()[..],
            &sizes,
            "tabs case {case}: {line}"
        );
        assert_eq!(
            if banking.selected == 9 {
                0
            } else {
                banking.selected + 1
            },
            f[8].parse::<usize>().unwrap()
        );
    }
}
