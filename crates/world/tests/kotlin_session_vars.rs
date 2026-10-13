use gielinor_world::{
    bank_actions::Binding,
    session_vars::Config,
    variables::{Transmit, Varbit, Variables, Varp},
};

#[test]
fn original_kotlin_session_hooks_match_rust() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/session-vars.expected"
    ))
    .expect("Generate the original Kotlin fixture with scripts/session-vars-parity.ps1");
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        clock_timer: 1,
        special_timer: 6,
        milliseconds: Binding {
            base: varp(3077),
            bit: Varbit {
                base: 3077,
                lsb: 0,
                msb: 9,
            },
        },
        seconds: Binding {
            base: varp(3077),
            bit: Varbit {
                base: 3077,
                lsb: 10,
                msb: 15,
            },
        },
        map_clock: varp(3079),
        playtime: varp(65515),
        energy: varp(300),
        new_account: Binding {
            base: varp(100),
            bit: Varbit {
                base: 100,
                lsb: 0,
                msb: 0,
            },
        },
    };
    let mut counts = [0, 0, 0];
    for line in fixture.lines() {
        let parts: Vec<_> = line.split('|').collect();
        let n = |index: usize| parts[index].parse::<i32>().unwrap();
        let mut vars = Variables::default();
        match parts[0] {
            "L" => {
                vars.set_bit(config.new_account.base, config.new_account.bit, n(1), false)
                    .unwrap();
                let mut timers = gielinor_world::player_timers::Timers::default();
                assert_eq!(n(2), -1);
                config.login(&mut vars, &mut timers, n(2)).unwrap();
                assert_eq!(vars.get(300), n(3));
                assert_eq!(timers.get(1).unwrap().expiry, n(4));
                assert_eq!(timers.get(6).unwrap().expiry, n(5));
                counts[2] += 1;
            }
            "S" => {
                vars.set(config.energy, n(1), false);
                config.tick(6, &mut vars, 0, 0).unwrap();
                assert_eq!(vars.get(300), n(2), "{line}");
                counts[0] += 1;
            }
            "C" => {
                vars.set(config.playtime, n(2), false);
                vars.set(config.milliseconds.base, 1 << 20, false);
                config
                    .tick(1, &mut vars, n(1), parts[3].parse().unwrap())
                    .unwrap();
                assert_eq!(
                    [vars.get(3077), vars.get(3079), vars.get(65515)],
                    [n(4), n(5), n(6)],
                    "{line}"
                );
                counts[1] += 1;
            }
            _ => panic!("Unexpected oracle record: {line}"),
        }
    }
    assert_eq!(counts, [12, 4, 2]);
}
