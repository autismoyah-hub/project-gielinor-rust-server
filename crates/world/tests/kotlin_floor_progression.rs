use gielinor_world::{floor_progression::Policy, simulation::PlayerData, stats::Stats, CoordGrid};

#[test]
fn compiled_floor_login_preserves_and_repairs_the_same_stats() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/floor-progression.expected"
    ))
    .expect("Run scripts/floor-progression-parity.ps1");
    assert_eq!(fixture.lines().count(), 1024);
    let policy = Policy {
        attack: 0,
        defence: 1,
        strength: 2,
        hitpoints: 3,
        runecrafting: 7,
        herblore: 8,
    };
    for (case, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split('|').collect();
        let input: Vec<i32> = fields[0].split(',').map(|s| s.parse().unwrap()).collect();
        let mut data = PlayerData::new(
            1,
            "Test".into(),
            CoordGrid::new(3222, 3218, 0).unwrap(),
            Stats::default(),
        );
        data.xp_rate_hundreds = input[0];
        for id in 0..9u8 {
            data.stats.set_base(id, input[3 + id as usize] as i8);
            data.stats.set_current(id, input[1] as i8);
            data.stats.set_fine_xp(id, input[2]).unwrap();
        }
        policy.login(&mut data);
        assert_eq!(
            data.xp_rate_hundreds,
            fields[1].parse::<i32>().unwrap(),
            "case {case}"
        );
        for (id, expected) in fields[2].split(';').enumerate() {
            let values: Vec<i32> = expected.split(',').map(|s| s.parse().unwrap()).collect();
            let id = id as u8;
            assert_eq!(
                [
                    data.stats.base(id) as i32,
                    data.stats.current(id) as i32,
                    data.stats.fine_xp(id)
                ],
                values.as_slice(),
                "case {case}, skill {id}"
            );
        }
    }
}
