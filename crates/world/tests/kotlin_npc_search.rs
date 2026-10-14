use gielinor_world::{
    npc::{BlockWalk, Npc, Restrict},
    simulation::World,
    CoordGrid,
};

#[test]
fn original_hunt_and_npc_search_match_zones_visibility_current_distance_and_ties() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/npc-search.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let mut world = World::default();
        let mut identities = vec![];
        for record in f[3].split(',') {
            let r: Vec<_> = record.split(':').collect();
            let npc = Npc::new(
                r[0].parse().unwrap(),
                CoordGrid::from_packed(r[1].parse().unwrap()),
                1,
                Restrict::NoMove,
                BlockWalk::None,
            )
            .unwrap();
            let id = world.spawn_npc(npc).unwrap();
            // The oracle labels candidates by insertion ordinal, independent
            // of either registry's reserved wire slots.
            identities.push(id);
            let npc = world.npc_mut(id).unwrap();
            npc.visible = r[3] == "1";
            npc.movement.coords = CoordGrid::from_packed(r[2].parse().unwrap());
        }
        let found = world.find_npc(
            CoordGrid::from_packed(f[0].parse().unwrap()),
            f[1].parse().unwrap(),
            f[2].parse().unwrap(),
        );
        let actual = match found {
            Err(_) => "ERR".to_owned(),
            Ok(id) => id
                .map_or(-1, |id| {
                    identities
                        .iter()
                        .position(|candidate| *candidate == id)
                        .unwrap() as i32
                })
                .to_string(),
        };
        assert_eq!(actual, f[4], "case {case}");
    }
}
