use gielinor_world::tower_diary::{Marks, RowState};
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
fn diary_transitions_match_compiled_floor_one_rules() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-diary.expected"
    ))
    .expect("Run scripts/tower-diary-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    for (i, line) in fixture.lines().enumerate() {
        let fields = line.split('|').collect::<Vec<_>>();
        assert_eq!(fields.len(), 11);
        let m = decode(fields[0]);
        assert_eq!(m.sync(), decode(fields[1]), "sync case {i}");
        assert_eq!(m.pay(), decode(fields[2]), "pay case {i}");
        let (row, amount) = fields[3].split_once(',').unwrap();
        assert_eq!(
            m.add(row.parse().unwrap(), amount.parse().unwrap()),
            decode(fields[4]),
            "add case {i}"
        );
        assert_eq!(
            m.note(fields[5].parse().unwrap()),
            decode(fields[6]),
            "note case {i}"
        );
        let (row, armed) = fields[7].split_once(',').unwrap();
        let credit = m.credit_boss(row.parse().unwrap(), armed == "1").unwrap();
        assert_eq!(credit.marks, decode(fields[8]), "boss case {i}");
        let flags = [
            credit.credited,
            credit.banked,
            credit.quiet_imp,
            credit.drop_imp_bones,
            credit.warden_first_loot,
        ]
        .into_iter()
        .enumerate()
        .fold(0, |m, (i, b)| m | (i32::from(b) << i));
        assert_eq!(
            flags,
            fields[9].parse::<i32>().unwrap(),
            "boss flags case {i}"
        );
        let states = (1..=11)
            .map(|row| match m.state(row) {
                RowState::Done => "Done",
                RowState::Owed => "Owed",
                RowState::Now => "Now",
                RowState::Ahead => "Ahead",
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(states, fields[10], "row states case {i}");
    }
}
