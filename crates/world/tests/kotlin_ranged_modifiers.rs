use gielinor_world::ranged_modifiers::Attributes;
#[test]
fn compiled_ranged_modifiers_match_order_rounding_and_overflow() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ranged-modifiers.expected"
    ))
    .expect("Run scripts/ranged-modifiers-parity.ps1");
    assert_eq!(fixture.lines().count(), 8192);
    for (case, line) in fixture.lines().enumerate() {
        let (input, expected) = line.split_once('|').unwrap();
        let a: Vec<i64> = input.split(',').map(|v| v.parse().unwrap()).collect();
        let attrs = Attributes {
            ranged: a[0] as u64,
            npc: a[1] as u64,
        };
        let actual = [
            attrs.damage(a[3] as i32, a[4] as i32),
            attrs.accuracy(a[2] as i32, a[4] as i32, a[5] as i32),
            attrs.post_spec(a[3] as i32, a[6] as i32, a[7] as i32),
            attrs.player_damage(a[3] as i32),
            attrs.player_accuracy(a[2] as i32, a[5] as i32),
        ];
        assert_eq!(
            actual
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(","),
            expected,
            "case {case}"
        );
    }
}
