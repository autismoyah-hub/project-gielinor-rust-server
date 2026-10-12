use gielinor_world::bonuses::Bonuses;
#[test]
fn compiled_bank_bonus_display_matches_text_and_packet_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../world/tests/fixtures/bonuses.expected"
    ))
    .unwrap();
    let display = gielinor_server::bonuses::BankDisplay {
        components: std::array::from_fn(|i| (12 << 16) | i as u32),
        tooltip: (12 << 16) | 18,
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let v: Vec<i32> = f[3].split(',').map(|v| v.parse().unwrap()).collect();
        let b = Bonuses {
            values: v[..16].try_into().unwrap(),
            magic_additive: v[16],
            magic_multiplier: v[17],
            undead_melee_only: v[18] != 0,
            slayer_melee_only: v[19] != 0,
        };
        let actual = gielinor_server::bonuses::bank_text(&b, 4, if f[2] == "2" { 3 } else { 4 });
        let expected: Vec<_> = f[4]
            .split(';')
            .map(|hex| {
                String::from_utf8(
                    (0..hex.len())
                        .step_by(2)
                        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
                        .collect(),
                )
                .unwrap()
            })
            .collect();
        assert_eq!(actual.as_slice(), expected, "case {case}");
        let packets = display.packets(&b, 4, 4).unwrap();
        assert_eq!(packets.len(), 19);
        assert_eq!(packets[17].opcode, 114);
        assert!(packets
            .iter()
            .enumerate()
            .all(|(i, p)| i == 17 || p.opcode == 80));
    }
}
