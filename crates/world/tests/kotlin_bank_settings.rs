use gielinor_world::{
    bank_actions::{Binding, Config, Control, Request},
    inventory::Catalog,
    simulation::PlayerData,
    stats::Stats,
    variables::{Transmit, Varbit, Varp},
    CoordGrid,
};
#[test]
fn compiled_bank_script_quantity_and_tab_display() {
    let binding = |id| Binding {
        base: Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: id,
            lsb: 0,
            msb: 30,
        },
    };
    let config = Config {
        sidebar: None,
        tabs: std::array::from_fn(|i| binding(i as u16 + 3)),
        selected: binding(13),
        capacity: binding(14),
        notes: binding(15),
        placeholders: binding(16),
        quantity: binding(1),
        last_quantity: binding(0),
        restricted: Default::default(),
    };
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bank-settings.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    for line in fixture.lines() {
        let fields: Vec<i32> = line.split('|').map(|s| s.parse().unwrap()).collect();
        let mut data =
            PlayerData::new(1, "bank_settings".into(), CoordGrid::ZERO, Stats::default());
        config
            .last_quantity
            .set(&mut data.variables, fields[0])
            .unwrap();
        config
            .apply(
                &mut data,
                &Catalog::default(),
                Request::Control {
                    control: Control::QuantityX,
                    op: 1,
                    slot: -1,
                },
            )
            .unwrap();
        config
            .apply(
                &mut data,
                &Catalog::default(),
                Request::Control {
                    control: Control::TabDisplay(binding(2)),
                    op: 1,
                    slot: fields[1],
                },
            )
            .unwrap();
        assert_eq!(config.quantity.get(&data.variables), Ok(fields[2]));
        assert_eq!(binding(2).get(&data.variables), Ok(fields[3]));
    }
}
