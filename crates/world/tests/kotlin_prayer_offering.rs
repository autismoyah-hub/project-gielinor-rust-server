use gielinor_world::{
    inventory::{Catalog, Insert, Item},
    prayer::Config,
    simulation::PlayerData,
    stats::Stats,
    CoordGrid,
};
#[test]
fn repeated_chapel_awards_match_original_kotlin_stat_operations() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/prayer-offering.expected"
    ))
    .expect("Run scripts/prayer-offering-parity.ps1");
    assert_eq!(fixture.lines().count(), 2048);
    let catalog = Catalog::default();
    for line in fixture.lines() {
        let f = line.split('|').collect::<Vec<_>>();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let config = Config {
            skill: 5,
            animation: 827,
            bones: [(n(0), f[1].parse().unwrap())].into(),
            necklace: Some(22111),
            restore: [(n(0), 5)].into(),
            worn_xp_percent: [(900, n(6))].into(),
            chaos: None,
        };
        let mut p = PlayerData::new(
            1,
            "Offering oracle".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.xp_rate_hundreds = n(2);
        p.stats.set_base(5, n(3) as i8);
        p.stats.set_current(5, n(4) as i8);
        p.stats.set_fine_xp(5, n(5)).unwrap();
        for (slot, id) in [(0, 900), (2, 22111)] {
            p.equipment
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        p.inventory
            .insert(&catalog, Insert::new(n(0), n(7)))
            .unwrap();
        for (index, expected) in f[8].split(',').enumerate() {
            let result = config.finish_offering(&mut p, &catalog, n(0));
            let actual = format!(
                "{}:{}:{}:{}",
                p.stats.current(5) as u8,
                p.stats.base(5) as u8,
                p.stats.fine_xp(5),
                result.is_err()
            );
            assert_eq!(actual, expected, "case {line} offering {index}");
        }
        assert_eq!(p.inventory.free_space(), p.inventory.slots().len());
    }
}
