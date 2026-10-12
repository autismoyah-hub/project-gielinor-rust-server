use gielinor_protocol::{
    game::{inventory_full, inventory_partial, InventoryItem},
    player_sync::{rebuild_normal, update_stat},
};
#[test]
fn original_rsprot_inventory_rebuild_and_stats() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/wire.expected"
    ))
    .expect("Run scripts/wire-parity.ps1 to generate original RSProt bytes");
    assert_eq!(text.lines().count(), 1024);
    for (case, line) in text.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let expected = f.last().unwrap();
        let bytes = match f[0] {
            "rebuild" => rebuild_normal(n(1) as u16, n(2) as u16, n(3) as u16).to_vec(),
            "stat" => update_stat(n(1) as u8, n(2) as u8, n(3) as u8, n(4)).to_vec(),
            "full" | "partial" => {
                let values: Vec<_> = f[3]
                    .split(',')
                    .map(|s| {
                        if s == "." {
                            None
                        } else {
                            let (id, count) = s.split_once(':').unwrap();
                            Some(InventoryItem {
                                id: id.parse().unwrap(),
                                count: count.parse().unwrap(),
                            })
                        }
                    })
                    .collect();
                if f[0] == "full" {
                    inventory_full(n(1), n(2) as u16, &values).unwrap()
                } else {
                    inventory_partial(
                        n(1),
                        n(2) as u16,
                        &[0, 127, 128, 255, 32767]
                            .into_iter()
                            .zip(values)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap()
                }
            }
            _ => panic!("Unknown fixture"),
        };
        let actual: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(&actual, expected, "case {case}: {line}");
    }
}
