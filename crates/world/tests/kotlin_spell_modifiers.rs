use gielinor_world::spell_modifiers::Attributes;
#[test]
fn compiled_spell_modifiers_match_order_rounding_and_overflow() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/spell-modifiers.expected"
    ))
    .expect("Run scripts/spell-modifiers-parity.ps1");
    assert_eq!(fixture.lines().count(), 8192);
    for (case, line) in fixture.lines().enumerate() {
        let (input, expected) = line.split_once('|').unwrap();
        let a: Vec<i64> = input.split(',').map(|v| v.parse().unwrap()).collect();
        let attrs = Attributes {
            spell: a[0] as u64,
            npc: a[1] as u64,
        };
        let damage = attrs.damage(a[3] as i32, a[4] as i32, a[5] as i32, a[6] as i32);
        let range = attrs.damage_range(damage, a[3] as i32, a[8] as i32, a[7] as i32);
        let pdamage = attrs.player_damage(a[3] as i32, a[4] as i32, a[5] as i32, a[6] as i32);
        let prange = attrs.player_damage_range(pdamage);
        let actual = [
            attrs.accuracy(a[2] as i32, a[7] as i32),
            damage,
            range.0,
            range.1,
            attrs.player_accuracy(a[2] as i32),
            pdamage,
            prange.0,
            prange.1,
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
