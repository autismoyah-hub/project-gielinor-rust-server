use gielinor_world::inventory::*;
#[test]
fn same_inventory_transaction_matches_kotlin() {
    let mut c = Catalog::default();
    c.stackable.insert(995);
    c.notes.insert(
        1038,
        Template {
            link: 1039,
            template: 0,
        },
    );
    c.notes.insert(
        1039,
        Template {
            link: 1038,
            template: 799,
        },
    );
    c.dummy.insert(799);
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/move-slot.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 8000);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<usize>().unwrap();
        let mut inv = Inventory::new(
            5,
            [StackType::Normal, StackType::Always, StackType::Never][n(0)],
            false,
        );
        for (slot, item) in f[3].split(',').enumerate() {
            if item == "." {
                continue;
            }
            let v: Vec<i32> = item.split(':').map(|v| v.parse().unwrap()).collect();
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
        let status = match inv.move_slot(&c, n(1), n(2)) {
            Ok(r) => format!("ok:{}:{}", r.requested, r.completed),
            Err(e) => format!("{e:?}"),
        };
        assert_eq!(status, f[4], "case {case}: {line}");
        let slots = inv
            .slots()
            .iter()
            .map(|i| i.map_or(".".into(), |i| format!("{}:{}:{}", i.id, i.count, i.vars)))
            .collect::<Vec<String>>()
            .join(",");
        assert_eq!(slots, f[5], "case {case}: {line}");
    }
}
