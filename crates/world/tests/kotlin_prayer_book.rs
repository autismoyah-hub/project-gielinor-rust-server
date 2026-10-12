use gielinor_world::{
    bank_actions::Binding,
    player_timers::Timers,
    prayer_book::Prayer,
    stats::Stats,
    variables::{Transmit, Varbit, Variables, Varp},
};
#[test]
fn compiled_kotlin_prayer_requirements_and_timer_packing() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/prayer-book.expected"
    ))
    .expect("Run scripts/prayer-book-parity.ps1");
    let bit = Binding {
        base: Varp {
            id: 3,
            permanent: false,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: 3,
            lsb: 0,
            msb: 3,
        },
    };
    let prayer = Prayer {
        id: 0,
        name: "Preserve".into(),
        level: 55,
        sound: 0,
        enabled: bit,
        overhead: None,
        unlock: Some(bit),
        unlock_state: 2,
        defence: Some(70),
        locked_message: Some("Unlock<br>this prayer.".into()),
        regen: None,
    };
    assert_eq!(fixture.lines().count(), 5120);
    for line in fixture.lines() {
        let f = line.split('|').collect::<Vec<_>>();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        match f[0] {
            "R" => {
                let mut vars = Variables::default();
                vars.set(bit.base, n(3), false);
                let mut stats = Stats::default();
                stats.set_base(5, n(1) as i8);
                stats.set_base(1, n(2) as i8);
                stats.set_current(5, n(4) as i8);
                assert_eq!(
                    prayer.requirements(&vars, &stats, 5, 1).unwrap(),
                    f[5] == "true",
                    "{line}"
                );
                assert_eq!(
                    stats.base(5) as u8 as i32 >= prayer.level,
                    f[6] == "true",
                    "{line}"
                );
            }
            "T" => {
                let mut timers = Timers::default();
                timers.schedule(7, n(1), n(2)).unwrap();
                let entry = timers.get(7).unwrap();
                assert_eq!((entry.expiry, entry.interval), (n(3), n(4)), "{line}");
            }
            _ => panic!("Unknown fixture {line}"),
        }
    }
}
