use gielinor_world::{
    inventory::{Catalog, Item},
    location_content::Handler,
    simulation::PlayerData,
    stats::Stats,
    CoordGrid,
};
#[test]
fn crate_rewards_and_messages_match_compiled_floor_one_rules_for_every_capacity() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tower-vial.expected"
    ))
    .expect("Run scripts/tower-vial-parity.ps1");
    assert_eq!(fixture.lines().count(), 29);
    let origin = CoordGrid::new(10368, 64, 0).unwrap();
    for line in fixture.lines() {
        let fields = line.split('|').collect::<Vec<_>>();
        let free = fields[0].parse::<usize>().unwrap();
        let mut player = PlayerData::new(
            1,
            "Vials".into(),
            origin.translate(53, 57, 0).unwrap(),
            Stats::default(),
        );
        for slot in 0..28 - free {
            player
                .inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 227,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let before = player
            .inventory
            .slots()
            .iter()
            .flatten()
            .map(|i| i.count)
            .sum::<i32>();
        let message = Handler::VialCrate { origin, item: 227 }
            .apply(&mut player, &Catalog::default())
            .unwrap()
            .unwrap();
        let after = player
            .inventory
            .slots()
            .iter()
            .flatten()
            .map(|i| i.count)
            .sum::<i32>();
        assert_eq!(
            after - before,
            fields[1].parse::<i32>().unwrap(),
            "free slots {free}"
        );
        assert_eq!(message, fields[2], "free slots {free}");
    }
}
