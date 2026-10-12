use gielinor_world::melee_modifiers::Attributes;
#[test]
fn compiled_melee_and_npc_modifiers_match_order_rounding_and_overflow() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/melee-modifiers.expected"
    ))
    .expect("Run scripts/melee-modifiers-parity.ps1");
    assert_eq!(fixture.lines().count(), 8192);
    for (case, line) in fixture.lines().enumerate() {
        let (input, expected) = line.split_once('|').unwrap();
        let a: Vec<i64> = input.split(',').map(|v| v.parse().unwrap()).collect();
        let attrs = Attributes {
            melee: a[0] as u64,
            npc: a[1] as u64,
        };
        let actual = [
            attrs.damage(a[3] as i32),
            attrs.attack_roll(a[2] as i32),
            attrs.post_special(a[3] as i32, a[4] as i32, a[5] as i32, a[6] as i32),
            attrs.hit_chance(a[8] as i32, a[2] as i32, a[7] as i32),
            attrs.defence_roll(a[7] as i32, a[9] as i32),
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
