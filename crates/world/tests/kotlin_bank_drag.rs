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
fn compiled_bank_script_grid_swap_insert_and_extended_tab_drag() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bank-drag.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    let mut catalog = Catalog::default();
    catalog.stackable.extend([2, 4, 6, 8]);
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
        let _ = banking.drag(
            &mut bank,
            &catalog,
            f[3].parse().unwrap(),
            f[4].parse().unwrap(),
            f[2].parse().unwrap(),
        );
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
