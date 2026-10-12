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
fn restore(inv: &mut Inventory, text: &str) {
    for (slot, item) in text.split(',').enumerate() {
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
}
#[test]
fn matches_original_kotlin_transfers_and_swaps() {
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
    c.transforms.insert(
        11907,
        Template {
            link: 11908,
            template: 0,
        },
    );
    c.transforms.insert(
        11908,
        Template {
            link: 11907,
            template: 1,
        },
    );
    c.dummy.insert(799);
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/exchange.expected"
    ))
    .expect("Generate the original Kotlin fixture with scripts/exchange-parity.ps1");
    assert_eq!(fixture.lines().count(), 8000);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let b = |i: usize| f[i] == "true";
        let stack = [StackType::Normal, StackType::Always, StackType::Never];
        let mut a = Inventory::new(5, stack[n(0) as usize], b(2));
        let mut target = Inventory::new(5, stack[n(1) as usize], b(2));
        restore(&mut a, f[12]);
        restore(&mut target, f[13]);
        let result = if n(3) == 0 {
            let mut q = Insert::new(0, n(6));
            q.slot = n(5) as usize;
            q.strict = b(7);
            q.note = n(8) == 1;
            q.unnote = n(8) == 2;
            q.transform = n(9) == 1;
            q.untransform = n(9) == 2;
            transfer(&mut a, &mut target, &c, n(4) as usize, n(6), q, b(10))
        } else {
            swap(
                &mut a,
                &mut target,
                &c,
                n(4) as usize,
                n(5) as usize,
                Swap {
                    merge: b(11),
                    relaxed_source_slot: !b(7),
                    note: n(8) == 1,
                    unnote: n(8) == 2,
                    transform: n(9) == 1,
                    untransform: n(9) == 2,
                },
            )
        };
        let status = match result {
            Ok(r) => format!("ok:{}:{}", r.requested, r.completed),
            Err(e) => format!("{e:?}"),
        };
        assert_eq!(status, f[14], "case {case}: {line}");
        assert_eq!(slots(&a), f[15], "source case {case}: {line}");
        assert_eq!(slots(&target), f[16], "target case {case}: {line}");
    }
}
