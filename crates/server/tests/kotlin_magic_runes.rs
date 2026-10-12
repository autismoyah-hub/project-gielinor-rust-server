use gielinor_world::{inventory::Item, runes::*};
use std::collections::{BTreeMap, BTreeSet};
fn catalog() -> Catalog {
    let sets = |entries: &[(i32, &[i32])]| {
        entries
            .iter()
            .map(|(id, values)| (*id, values.iter().copied().collect::<BTreeSet<_>>()))
            .collect()
    };
    Catalog {
        compact: (1..=5).map(|i| (i, i)).collect(),
        high: sets(&[(1, &[100]), (3, &[102])]),
        low: sets(&[(2, &[101, 201]), (5, &[202])]),
        fake: BTreeMap::from([(1, 9), (2, 10), (3, 11), (4, 12)]),
        substitutes: BTreeMap::from([(1, vec![6, 7]), (2, vec![7]), (6, vec![8]), (5, vec![8])]),
        staff_substitutes: sets(&[(100, &[103])]),
        combos: vec![(3, 1, 2), (4, 2, 5)],
    }
}
fn ints(s: &str) -> Vec<i32> {
    s.split(',').map(|v| v.parse().unwrap()).collect()
}
fn format_result(v: Validation) -> String {
    match v {
        Validation::Unlimited => "U".into(),
        Validation::NotEnough(id) => format!("N:{id}"),
        Validation::NotWearing(id) => format!("W:{id}"),
        Validation::Enough(sources) => format!(
            "E:{}",
            sources
                .into_iter()
                .map(|s| match s {
                    Source::Inventory { item, slot, count } => format!("I,{item},{slot},{count}"),
                    Source::Varbit { id, count } => format!("V,{id},{count}"),
                })
                .collect::<Vec<_>>()
                .join(";")
        ),
    }
}
#[test]
fn compiled_magic_runes_matches_combo_priority_pouch_staff_and_fake_sources() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/magic-runes.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    let catalog = catalog();
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let inv: Vec<_> = ints(f[0])
            .into_iter()
            .enumerate()
            .map(|(i, count)| {
                (count >= 0).then_some(Item {
                    id: i as i32 + 1,
                    count,
                    vars: 0,
                })
            })
            .collect();
        let mut worn = vec![None; 14];
        for (slot, text, empty) in [(3, f[1], 99), (5, f[2], 199)] {
            let id = text.parse().unwrap();
            worn[slot] = (id != empty).then_some(Item {
                id,
                count: 1,
                vars: 0,
            });
        }
        let ids = ints(f[3]);
        let counts = ints(f[4]);
        let pouch: Vec<_> = if f[5] == "1" {
            (0..4)
                .map(|i| PouchSlot {
                    compact: ids[i],
                    count: counts[i],
                    varbit: (i != 3 || f[6] == "1").then_some(i as u32),
                })
                .collect()
        } else {
            vec![]
        };
        let reqs: Vec<_> = f[9]
            .split(';')
            .map(|r| {
                let n = ints(r);
                Requirement {
                    item: n[0],
                    count: n[1],
                    worn_slot: (n[2] >= 0).then_some(n[2] as usize),
                }
            })
            .collect();
        let ctx = Context {
            inventory: &inv,
            worn: &worn,
            pouch: &pouch,
            fake_runes: f[7] == "1",
            fountain: f[8] == "1",
        };
        let result = catalog
            .requirements(&ctx, &reqs)
            .unwrap()
            .into_iter()
            .map(format_result)
            .collect::<Vec<_>>()
            .join("/");
        assert_eq!(result, f[10], "case {case}: {line}");
    }
}
