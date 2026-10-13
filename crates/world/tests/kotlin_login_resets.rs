use gielinor_world::{
    bank_actions::Binding,
    login_resets::Config,
    simulation::{PlayerData, World, WorldEvent},
    variables::{Transmit, Varbit, Variables, Varp},
};
fn binding(id: u16, lsb: u8, msb: u8) -> Binding {
    Binding {
        base: Varp {
            id,
            permanent: true,
            transmit: Transmit::Different,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit { base: id, lsb, msb },
    }
}
fn config() -> Config {
    Config {
        quick_prayer: binding(1, 0, 0),
        canoe_station: binding(2, 0, 8),
        canoe_type: binding(3, 4, 6),
        canoe_avoid_interface: binding(3, 12, 12),
    }
}
#[test]
fn admission_resets_restored_flags_before_initialized_event() {
    let mut world = World::default();
    world.login_resets = Some(config());
    let mut data = PlayerData::new(
        1,
        "reset".into(),
        gielinor_world::CoordGrid::new(3224, 3218, 0).unwrap(),
        Default::default(),
    );
    for id in 1..=3 {
        data.variables.set(binding(id, 0, 0).base, -1, false);
    }
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world
        .drain_events()
        .into_iter()
        .find_map(|e| match e {
            WorldEvent::Initialized { actor, .. } => Some(actor),
            _ => None,
        })
        .expect("admitted player");
    let vars = &world.data(actor).unwrap().variables;
    assert_eq!(vars.get(1), -2);
    assert_eq!(vars.get(2), -512);
    assert_eq!(vars.get(3), !((7 << 4) | (1 << 12)));
}
#[test]
fn original_login_hooks_reset_fields_without_erasing_neighbor_bits() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/login-resets.expected"
    ))
    .expect("Run scripts/login-resets-parity.ps1");
    assert_eq!(fixture.lines().count(), 8);
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let mut vars = Variables::default();
        if fields[0] != "absent" {
            for id in 1..=3 {
                vars.set(binding(id, 0, 0).base, fields[0].parse().unwrap(), false);
            }
        }
        for expected in &fields[1..] {
            config().apply(&mut vars).unwrap();
            let actual = (1..=3)
                .map(|id| {
                    assert!(vars.contains(id));
                    vars.get(id).to_string()
                })
                .collect::<Vec<_>>()
                .join(",");
            assert_eq!(&actual, expected, "{line}");
        }
    }
}
