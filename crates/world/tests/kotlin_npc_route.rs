use gielinor_world::{npc_route, CoordGrid};
#[test]
fn npc_retaliation_retreat_and_entity_destinations_match_kotlin() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/npc-route.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 12288);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let v: Vec<i32> = f[1].split(',').map(|s| s.parse().unwrap()).collect();
        let c = |i: usize| CoordGrid::from_packed(v[i]);
        match f[0] {
            "R" => assert_eq!(
                npc_route::retreat_destination(c(0), c(1), c(2), v[3])
                    .unwrap()
                    .packed(),
                f[2].parse::<i32>().unwrap(),
                "case {case}"
            ),
            "C" => assert_eq!(
                npc_route::can_retaliate(v[0], v[1], v[2] != 0),
                f[2] == "1",
                "case {case}"
            ),
            "N" => {
                let dest = npc_route::entity_destination(
                    c(0),
                    (v[1], v[2]),
                    c(3),
                    (v[4], v[5]),
                    v[6] as u8,
                )
                .unwrap();
                assert_eq!(format!("{},{}", dest.x(), dest.z()), f[2], "case {case}");
            }
            _ => panic!("Unknown fixture"),
        }
    }
}
