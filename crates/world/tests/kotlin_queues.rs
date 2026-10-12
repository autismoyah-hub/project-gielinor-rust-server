use gielinor_world::queues::{Access, Category, Event, Scheduler};
fn category(n: i32) -> Category {
    match n {
        0 => Category::Soft,
        1 => Category::Normal,
        2 => Category::Weak,
        3 => Category::Strong,
        4 => Category::LongAccelerate,
        5 => Category::LongDiscard,
        _ => panic!(),
    }
}
fn text(values: Vec<String>, separator: &str) -> String {
    if values.is_empty() {
        "-".into()
    } else {
        values.join(separator)
    }
}
#[test]
fn original_player_queue_processor_mutation_and_protection() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/queues.expected"
    ))
    .expect("Run scripts/queue-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let clock = f[0].parse().unwrap();
        let mut access = Access {
            protected: f[1] == "true",
            pending_logout: f[2] == "true",
        };
        let unlock = f[3] == "true";
        let mut scheduler = Scheduler::default();
        if f[4] != "-" {
            for entry in f[4].split(';') {
                let n: Vec<i32> = entry.split(',').map(|s| s.parse().unwrap()).collect();
                scheduler.add(n[0], category(n[1]), n[2], n[3]).unwrap();
            }
        }
        let mut events = vec![];
        scheduler
            .process(clock, &mut access, 10000, |s, a, event| match event {
                Event::CloseModals => {
                    events.push("C".into());
                    if unlock {
                        a.protected = false;
                    }
                }
                Event::Launch(q) => {
                    events.push(format!(
                        "{}{}",
                        if q.category == Category::Soft {
                            "S"
                        } else {
                            "P"
                        },
                        q.id
                    ));
                    match q.args {
                        1 => s.add(99, Category::Normal, 1, 0).unwrap(),
                        2 => a.protected = true,
                        3 => s.clear_weak(),
                        4 => s.clear(),
                        _ => (),
                    }
                }
            })
            .unwrap_or_else(|e| panic!("case {case}: {line}: {e:?}"));
        assert_eq!(text(events, ","), f[5], "events {case}: {line}");
        assert_eq!(access.protected.to_string(), f[6], "protected {case}");
        for (weak, index) in [(false, 7), (true, 8)] {
            let state = scheduler
                .snapshot()
                .into_iter()
                .filter(|q| (q.category == Category::Weak) == weak)
                .map(|q| {
                    format!(
                        "{},{},{},{},{}",
                        q.id, q.category as u8, q.remaining, q.processed_cycle, q.args
                    )
                })
                .collect();
            assert_eq!(text(state, ";"), f[index], "state {case}: {line}");
        }
    }
}
