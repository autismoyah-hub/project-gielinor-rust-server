use gielinor_world::tower_hazards::step_off;
#[test]
fn cinder_escape_matches_compiled_floor_one_rules() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-hazard.expected"
    ))
    .expect("Run scripts/tower-hazard-parity.ps1");
    assert_eq!(fixture.lines().count(), 6400);
    let points = [[6, 5], [4, 5], [5, 6], [5, 4]];
    for line in fixture.lines() {
        let (input, expected) = line.split_once('|').unwrap();
        let v = input
            .split(',')
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        let actual = step_off(
            [5, 5],
            Some([5, 5]),
            [5 + v[2], 5 + v[3]],
            v[0],
            v[1],
            |t| v[4] & (1 << points.iter().position(|&p| p == t).unwrap()) != 0,
            |t| t == [6, 5],
        );
        let actual = actual.map_or("-".into(), |p| format!("{},{}", p[0], p[1]));
        assert_eq!(actual, expected, "case {input}");
    }
}
