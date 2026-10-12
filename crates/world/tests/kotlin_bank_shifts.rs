use gielinor_world::inventory::*;
#[test]
fn original_kotlin_bank_shift_transactions() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bank-shifts.expected"
    ))
    .expect("Run scripts/bank-shift-parity.ps1 to generate the Kotlin corpus");
    assert_eq!(fixture.lines().count(), 6000);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<usize>().unwrap();
        let mut inv = Inventory::new(12, StackType::Always, true);
        for (slot, value) in f[6].split(',').enumerate() {
            if value == "." {
                continue;
            }
            let fields: Vec<i32> = value.split(':').map(|s| s.parse().unwrap()).collect();
            inv.restore_slot(
                slot,
                Some(Item {
                    id: fields[0],
                    count: fields[1],
                    vars: fields[2],
                }),
            )
            .unwrap();
        }
        let result = match n(0) {
            0 => inv.shift_insert(n(1), n(2)),
            1 => inv.shift_left(n(1), n(1).min(n(2)), n(3), f[5] == "true"),
            2 => inv.shift_right(n(1), n(4)),
            3 => inv.shift_bulk(n(1)..n(3), n(2)),
            4 => inv.compact_slots(n(1)..n(3)),
            _ => unreachable!(),
        };
        let status = match result {
            Ok(s) => format!("ok:{}:{}", s.requested, s.completed),
            Err(e) => format!("{e:?}"),
        };
        assert_eq!(status, f[7], "status case {case}: {line}");
        let slots = inv
            .slots()
            .iter()
            .map(|v| match v {
                None => ".".into(),
                Some(v) => format!("{}:{}:{}", v.id, v.count, v.vars),
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(slots, f[8], "slots case {case}: {line}");
    }
}
