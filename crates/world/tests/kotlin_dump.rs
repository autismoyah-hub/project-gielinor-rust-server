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
fn matches_original_kotlin_dump_partial_failures_filters_and_overflow() {
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
        "/tests/fixtures/dump.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 6000);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let b = |i: usize| f[i] == "true";
        let stack = [StackType::Normal, StackType::Always, StackType::Never];
        let mut source = Inventory::new(5, stack[n(0) as usize], b(2));
        let mut target = Inventory::new(5, stack[n(1) as usize], b(2));
        restore(&mut source, f[9]);
        restore(&mut target, f[10]);
        let mut insert = Insert::new(0, 0);
        insert.slot = n(3) as usize;
        insert.capacity = Some(n(4) as usize);
        insert.note = n(6) == 1;
        insert.unnote = n(6) == 2;
        insert.transform = n(7) == 1;
        insert.untransform = n(7) == 2;
        let keep = (0..5).filter(|i| n(5) & (1 << i) != 0).collect();
        let result = dump(&mut source, &mut target, &c, insert, &keep, b(8));
        let status = match result {
            Ok(r) => format!("ok:{}:{}", r.requested, r.completed),
            Err(e) => format!("{e:?}"),
        };
        assert_eq!(status, f[11], "result case {case}: {line}");
        assert_eq!(slots(&source), f[12], "source case {case}: {line}");
        assert_eq!(slots(&target), f[13], "target case {case}: {line}");
    }
}
