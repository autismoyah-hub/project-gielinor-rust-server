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
fn diary_journal_text_matches_original_for_legacy_paid_owed_and_active_rows() {
    use gielinor_world::tower_diary_ui as ui;
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-diary-ui.expected"
    ))
    .expect("Run scripts/tower-diary-ui-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 5);
        let marks = decode(f[0]);
        assert_eq!(ui::title(marks), f[1], "title {case}");
        assert_eq!(ui::body(marks), f[2], "body {case}");
        assert_eq!(ui::lines(marks), f[3], "lines {case}");
        assert_eq!(ui::status(marks).join("~"), f[4], "status {case}");
    }
}
