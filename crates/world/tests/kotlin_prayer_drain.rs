use gielinor_world::{
    bank_actions::Binding,
    prayer_drain::{Config, Outcome},
    stats::Stats,
    variables::{Transmit, Varbit, Variables, Varp},
};
#[test]
fn original_private_drain_hook_matches_counter_and_visible_points() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/prayer-drain.expected"
    ))
    .expect("Run scripts/prayer-drain-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    let binding = |id| Binding {
        base: Varp {
            id,
            permanent: false,
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
    let c = Config {
        enabled: binding(0),
        counter: binding(1),
        resistance: binding(2),
        skill: 5,
        effects: vec![(0, 1), (1, 6), (2, 30)],
    };
    for line in fixture.lines() {
        let n = line
            .split('|')
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        let mut vars = Variables::default();
        vars.set(c.enabled.base, n[0], false);
        vars.set(c.counter.base, n[1], false);
        vars.set(c.resistance.base, n[2], false);
        let mut stats = Stats::default();
        stats.set_current(5, n[3] as i8);
        stats.set_base(5, 99);
        assert!(matches!(
            c.tick(&mut vars, &mut stats, true).unwrap(),
            Outcome::Tick { .. }
        ));
        assert_eq!(
            (c.counter.get(&vars).unwrap(), stats.current(5) as u8 as i32),
            (n[4], n[5]),
            "{line}"
        );
    }
}
