use gielinor_world::{
    bank::{Banking, Tabs},
    inventory::{Catalog, Inventory, Item, StackType},
};
fn inventory(text: &str) -> Inventory {
    let mut inv = Inventory::new(32, StackType::Always, false);
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
fn compiled_bank_script_tab_collapse_reorder_and_placeholder_release() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bank-tabs.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    let mut catalog = Catalog::default();
    catalog.stackable.extend([2, 4, 6, 8]);
    catalog.placeholders.insert(
        8,
        gielinor_world::inventory::Template {
            link: 7,
            template: 1,
        },
    );
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let sizes: Vec<usize> = f[0].split(',').map(|s| s.parse().unwrap()).collect();
        let selected: usize = f[1].parse().unwrap();
        let mut banking = Banking {
            tabs: Tabs::restore(sizes.try_into().unwrap(), 32).unwrap(),
            selected: if selected == 0 { 9 } else { selected - 1 },
            capacity: 32,
            withdraw_notes: false,
            always_placehold: false,
        };
        let mut bank = inventory(f[5]);
        let from = f[3].parse().unwrap();
        let into = f[4].parse().unwrap();
        match f[2] {
            "0" => {
                let _ = banking.collapse_tab(&mut bank, &catalog, from);
            }
            "1" => {
                let _ = banking.reorder_tabs(&mut bank, from, into);
            }
            "2" => {
                let _ = banking.release_tab_placeholders(&mut bank, &catalog, from);
            }
            "3" => {
                let _ = banking.compact(&mut bank);
            }
            _ => panic!("Unexpected operation"),
        }
        assert_eq!(
            bank.slots(),
            inventory(f[6]).slots(),
            "inventory case {case}: {line}"
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
            f[8].parse::<usize>().unwrap(),
            "selected case {case}"
        );
    }
}
