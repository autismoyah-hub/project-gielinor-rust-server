#[test]
fn original_fastutil_db_row_iteration_matches_resize_and_duplicate_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/int-map-order.expected"
    ))
    .expect("Run scripts/int-map-order-oracle.ps1");
    assert_eq!(fixture.lines().count(), 11);
    let parse = |s: &str| {
        s.split(',')
            .filter(|v| !v.is_empty())
            .map(|v| v.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    };
    for (i, line) in fixture.lines().enumerate() {
        let (input, output) = line.split_once('|').unwrap();
        assert_eq!(
            gielinor_cache::int_map_order::order(parse(input)),
            parse(output),
            "case {i}"
        );
    }
}
#[test]
fn original_fastutil_map_copy_music_repository_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/int-map-copy.expected"
    ))
    .expect("Run scripts/int-map-order-oracle.ps1");
    assert_eq!(fixture.lines().count(), 11);
    let parse = |s: &str| {
        s.split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
    };
    for (i, line) in fixture.lines().enumerate() {
        let (input, output) = line.split_once('|').unwrap();
        assert_eq!(
            gielinor_cache::int_map_order::copied_map_order(&parse(input)),
            parse(output),
            "case {i}"
        );
    }
}
