use gielinor_world::CoordGrid;

#[test]
fn matches_pinned_kotlin_coordinate_fixtures() {
    let input: Vec<_> = include_str!("fixtures/coordinates.input").lines().collect();
    let expected: Vec<_> = include_str!("fixtures/coordinates.expected")
        .lines()
        .collect();
    assert_eq!(
        input.len(),
        4452,
        "do not silently truncate the oracle corpus"
    );
    assert_eq!(input.len(), expected.len());
    for (case, expected) in input.iter().zip(expected) {
        let parts: Vec<_> = case.split_whitespace().collect();
        let v: Vec<i32> = parts[1..].iter().map(|v| v.parse().unwrap()).collect();
        let c = match parts[0] {
            "new" => CoordGrid::new(v[0], v[1], v[2]),
            "raw" => Ok(CoordGrid::from_packed(v[0])),
            "move" => CoordGrid::from_packed(v[0]).translate(v[1], v[2], v[3]),
            "distance" => {
                assert_eq!(
                    CoordGrid::from_packed(v[0])
                        .chebyshev_distance(CoordGrid::from_packed(v[1]))
                        .to_string(),
                    expected,
                    "{case}"
                );
                continue;
            }
            op => panic!("unknown fixture operation {op}"),
        };
        let actual = match c {
            Ok(c) => format!(
                "{} {} {} {} {} {} {} {}",
                c.packed(),
                c.x(),
                c.z(),
                c.level(),
                c.mx(),
                c.mz(),
                c.lx(),
                c.lz()
            ),
            Err(_) => "invalid".to_string(),
        };
        assert_eq!(actual, expected, "{case}");
    }
}
