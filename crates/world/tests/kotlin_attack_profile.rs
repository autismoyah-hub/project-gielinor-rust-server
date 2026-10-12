use gielinor_world::attack_profile::{self, Attack};
#[test]
fn compiled_player_commons_and_weapon_speeds_match_attack_selection_and_range() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/attack-profile.expected"
    ))
    .expect("Run scripts/attack-profile-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let a: Vec<i32> = f[0].split(',').map(|v| v.parse().unwrap()).collect();
        let kind = (1..=7).contains(&a[1]).then_some(a[1] as u8);
        let style = (1..=9).contains(&a[2]).then_some(a[2] as u8);
        let selected =
            match attack_profile::resolve(a[0] == 1, kind, style, a[3] == 1, a[4] == 1, a[5]) {
                Err(_) => "ERR".into(),
                Ok(Attack::Spell { defensive }) => format!("S:{}", u8::from(defensive)),
                Ok(Attack::Melee {
                    kind,
                    style,
                    stance,
                }) => format!(
                    "M:{}:{}:{stance}",
                    kind.map(|n| n as i32 - 1).unwrap_or(-1),
                    style.map(|n| n as i32 - 1).unwrap_or(-1)
                ),
                Ok(Attack::Ranged { kind, style }) => format!(
                    "R:{}:{}",
                    kind as i32 - 5,
                    style.map(|n| n as i32 - 5).unwrap_or(-1)
                ),
                Ok(Attack::Staff { style }) => {
                    format!("T:{}", style.map(|n| n as i32 - 8).unwrap_or(-1))
                }
            };
        assert_eq!(selected, f[1], "attack {case}");
        let range = attack_profile::range((a[0] == 1).then_some(a[7]), style, a[8])
            .map(|n| n.to_string())
            .unwrap_or("ERR".into());
        assert_eq!(range, f[2], "range {case}");
        assert_eq!(
            attack_profile::speed((a[0] == 1).then_some(a[6]), style),
            f[3].parse::<i32>().unwrap(),
            "speed {case}"
        );
        let packed = (0..4)
            .map(|stance| {
                attack_profile::attack_type(a[9], stance)
                    .unwrap()
                    .unwrap_or(0)
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(packed, f[4], "packed {case}");
    }
}
