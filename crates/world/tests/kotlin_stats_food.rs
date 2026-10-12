use gielinor_world::{food, stat_ops::Operation, stats::Stats};
#[test]
fn original_stat_operations_overflow_caps_notifications_and_errors() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/stat-ops.expected"
    ))
    .expect("Run scripts/stat-food-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let hp = f[5] == "true";
        let id = if hp { 3 } else { 0 };
        let mut stats = Stats::default();
        stats.set_base(id, n(0) as i8);
        stats.set_current(id, n(1) as i8);
        let op = [
            Operation::Add,
            Operation::Subtract,
            Operation::Boost,
            Operation::Drain,
            Operation::Heal,
            Operation::Restore,
        ][n(2) as usize];
        let result = stats.apply(id, op, n(3), n(4), hp);
        assert_eq!(result.is_err(), f[10] == "true", "error {case}: {line}");
        assert_eq!(stats.current(id) as u8 as i32, n(6), "level {case}: {line}");
        let actual = result
            .map(|u| (u.notify, u.changed, u.clear_hero_points))
            .unwrap_or((false, false, false));
        assert_eq!(
            actual,
            (f[7] == "true", f[8] == "true", f[9] == "true"),
            "effects {case}: {line}"
        );
    }
}
#[test]
fn original_food_catalog_heals_delays_replacements_and_messages() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/food.expected"
    ))
    .expect("Run scripts/stat-food-parity.ps1");
    let foods = food::catalog();
    assert_eq!(fixture.lines().count(), foods.len());
    for (food, line) in foods.iter().zip(fixture.lines()) {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(food.symbol, f[0]);
        assert_eq!(food.heal.to_string(), f[1]);
        assert_eq!(food.delay.to_string(), f[2]);
        assert_eq!(food.replacement.unwrap_or("-"), f[3]);
        assert_eq!(food.message.as_deref().unwrap_or("-"), f[4]);
        assert_eq!(
            food.message.as_deref().unwrap_or("You eat the test food."),
            f[5]
        );
    }
}
