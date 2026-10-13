use gielinor_world::disconnect::Grace;

#[test]
fn original_kotlin_disconnect_hook_matches_rust_logout_tick() {
    let fixture = include_str!("fixtures/disconnect.expected");
    let mut cases = 0;
    for line in fixture.lines() {
        let (prevent, expected) = line.split_once('|').unwrap();
        let prevent = prevent.parse::<i32>().unwrap();
        let expected = expected.parse::<i32>().unwrap();
        let mut grace = Grace::default();
        grace.disconnect();
        let actual = (0..150).find(|tick| grace.tick(*tick, prevent)).unwrap();
        assert_eq!(actual, expected, "preventLogoutUntil={prevent}");
        cases += 1;
    }
    assert_eq!(cases, 9);
}
