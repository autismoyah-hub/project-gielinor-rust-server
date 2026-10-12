use gielinor_world::hero_points::{Hero, HeroPoints};
#[test]
fn bounded_contributions_and_tied_winners_match_kotlin() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hero-points.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split('|').collect();
        let mut heroes = HeroPoints::new(fields[0].parse().unwrap());
        for operation in fields[1].split(';') {
            if operation == "C" {
                heroes.clear();
            } else {
                let (uuid, points) = operation.split_once(',').unwrap();
                heroes.credit(uuid.parse().unwrap(), points.parse().unwrap());
            }
        }
        let entries: Vec<_> = fields[2]
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let (uuid, points) = s.split_once(',').unwrap();
                Hero {
                    uuid: uuid.parse().unwrap(),
                    points: points.parse().unwrap(),
                }
            })
            .collect();
        assert_eq!(heroes.entries(), entries, "case {case}");
        let winner = if fields[3] == "-1" {
            None
        } else {
            Some(fields[3].parse().unwrap())
        };
        assert_eq!(heroes.winner(), winner, "case {case}");
    }
}
