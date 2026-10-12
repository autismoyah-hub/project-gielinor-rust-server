use gielinor_world::bonuses::{attack_speed, attack_style};
#[test]
fn compiled_packed_styles_and_weapon_speed_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/weapon-speed.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<i32> = line.split('|').map(|v| v.parse().unwrap()).collect();
        let style = attack_style(f[2], f[3]).unwrap();
        assert_eq!(style.unwrap_or(0) as i32, f[4], "style {case}");
        let base = if f[1] == 0 { 4 } else { f[0] };
        assert_eq!(base, f[5]);
        assert_eq!(attack_speed(base, style == Some(6)), f[6], "speed {case}");
    }
    assert!(attack_style(0, -1).is_err());
    assert!(attack_style(0, 4).is_err());
}
