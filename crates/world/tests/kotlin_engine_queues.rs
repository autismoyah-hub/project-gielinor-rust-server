use gielinor_world::engine_queues::{Kind, Queue, Queues};
fn encode(queue: &Queue<i32>) -> String {
    format!("{}:{}:{}", queue.kind as i32, queue.label, queue.args)
}
#[test]
fn original_engine_queue_cursor_mutations_clear_and_protection() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/engine-queues.expected"
    ))
    .unwrap();
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let seed: i32 = f[0].parse().unwrap();
        let size: i32 = f[1].parse().unwrap();
        let mode: i32 = f[2].parse().unwrap();
        let mut queues = Queues::default();
        for i in 0..size {
            queues
                .push(Queue {
                    kind: Kind::ChangeStat,
                    label: i,
                    args: seed * 100 + i,
                })
                .unwrap();
        }
        let mut passes = vec![];
        for pass in 0..3 {
            let mut seen = vec![];
            let mut invoked = 0;
            queues
                .process(pass == 0 && f[3] == "1", |queues, queue| {
                    seen.push(encode(&queue));
                    match mode {
                        1 if invoked == 0 => queues
                            .push(Queue {
                                kind: Kind::AdvanceStat,
                                label: -1,
                                args: 900 + seed,
                            })
                            .unwrap(),
                        2 if invoked < 3 => {
                            for i in 0..2 {
                                queues
                                    .push(Queue {
                                        kind: Kind::Zone,
                                        label: seed,
                                        args: 910 + invoked * 2 + i,
                                    })
                                    .unwrap();
                            }
                        }
                        3 if invoked == 0 => {
                            queues.clear();
                            queues
                                .push(Queue {
                                    kind: Kind::Area,
                                    label: 8,
                                    args: 920 + seed,
                                })
                                .unwrap();
                        }
                        _ => (),
                    }
                    invoked += 1;
                    mode == 4
                })
                .unwrap();
            passes.push(format!(
                "{}/{}",
                seen.join(","),
                queues.iter().map(encode).collect::<Vec<_>>().join(",")
            ));
        }
        assert_eq!(passes.join(";"), f[4], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 320);
}
