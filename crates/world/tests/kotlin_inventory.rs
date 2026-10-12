use gielinor_world::inventory::*;
fn slots(inv: &Inventory) -> String {
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
fn matches_original_kotlin_transactions() {
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
    c.placeholders.insert(
        1038,
        Template {
            link: 14387,
            template: 0,
        },
    );
    c.placeholders.insert(
        14387,
        Template {
            link: 1038,
            template: 14401,
        },
    );
    c.dummy.insert(799);
    let fixture = include_str!("fixtures/inventory.expected");
    assert_eq!(fixture.lines().count(), 6000);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let b = |i: usize| f[i] == "true";
        let mut inv = Inventory::new(
            5,
            [StackType::Normal, StackType::Always, StackType::Never][n(0) as usize],
            b(1),
        );
        for (slot, item) in f[10].split(',').enumerate() {
            if item != "." {
                let v: Vec<i32> = item.split(':').map(|s| s.parse().unwrap()).collect();
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
        }
        let result = if n(2) == 0 {
            let mut q = Insert::new(n(3), n(4));
            q.strict = b(5);
            q.slot = n(6) as usize;
            q.vars = n(7);
            q.note = n(8) == 1;
            q.unnote = n(8) == 2;
            inv.insert(&c, q)
        } else {
            let mut q = Delete::new(n(3), n(4));
            q.strict = b(5);
            q.slot = n(6) as usize;
            q.placehold = b(9);
            inv.delete(&c, q)
        };
        let status = match result {
            Ok(r) => format!("ok:{}:{}", r.requested, r.completed),
            Err(e) => format!("{e:?}"),
        };
        assert_eq!(status, f[11], "case {case}: {line}");
        assert_eq!(slots(&inv), f[12], "case {case}: {line}");
    }
}
