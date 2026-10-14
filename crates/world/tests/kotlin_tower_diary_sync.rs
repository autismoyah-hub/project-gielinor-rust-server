use gielinor_world::tower_diary::Marks;
fn decode(text: &str) -> Marks {
    let v = text
        .split(',')
        .map(|n| n.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(v.len(), 16);
    Marks {
        counts: v[..6].try_into().unwrap(),
        levels: v[6..9].try_into().unwrap(),
        actions: std::array::from_fn(|i| v[9] & (1 << i) != 0),
        done: std::array::from_fn(|i| v[10] & (1 << i) != 0),
        pending: std::array::from_fn(|i| v[11] & (1 << i) != 0),
        bounty: v[12],
        schema11: v[13] != 0,
        imp_bones: v[14] != 0,
        staff_taken: v[15] != 0,
    }
}
#[test]
fn tanning_diary_sync_matches_original_rules_and_session_orchestration() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-diary-sync.expected"
    ))
    .expect("Run scripts/tower-diary-sync-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 5);
        let outcome = decode(f[0]).add(1, f[1].parse().unwrap()).session_sync();
        assert_eq!(outcome.marks, decode(f[2]), "marks {case}");
        assert_eq!(outcome.quiet_imp, f[3] == "true", "quiet {case}");
        assert_eq!(outcome.message.unwrap_or(""), f[4], "chat {case}");
    }
}
