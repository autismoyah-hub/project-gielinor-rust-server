#![cfg(windows)]
use gielinor_world::{
    bank_actions::Binding,
    music::{Catalog, Track},
    music_player::{Config, Context},
    npc::RandomSource,
    variables::{Transmit, Varbit, Variables, Varp},
};
fn varp(id: u16) -> Varp {
    Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    }
}
fn config(all: Vec<u16>) -> Config {
    let bit = |id| Binding {
        base: varp(id),
        bit: Varbit {
            base: id,
            lsb: 0,
            msb: 15,
        },
    };
    let mut catalog = Catalog {
        all,
        ..Default::default()
    };
    for id in 1..=3 {
        catalog.tracks.insert(
            id,
            Track {
                id,
                row: id as u32,
                name: format!("Track{id}"),
                unlock_hint: "Hint".into(),
                duration: 2,
                midi: 100 + id as i32,
                unlock_varp: Some(varp(20)),
                unlock_bit: id as i32,
                hidden: false,
                secondary: None,
            },
        );
    }
    catalog.modern.insert(9, vec![1, 2]);
    catalog.classic.insert(9, 3);
    Config {
        catalog,
        mode: varp(30),
        playlist: varp(31),
        volume: varp(32),
        area_mode: bit(40),
        area: bit(41),
        current: bit(42),
        last: bit(43),
        clocks: bit(44),
        duration: bit(45),
        suppress_message: bit(46),
        transparency: bit(47),
        stop_midi: 147,
        timer: 8,
    }
}
struct Draws(std::collections::VecDeque<u32>);
impl RandomSource for Draws {
    fn below(&mut self, n: u32) -> u32 {
        let value = self.0.pop_front().expect("Extra random draw");
        assert!(value < n);
        value
    }
}
#[test]
fn original_kotlin_area_playback_timer_and_encoded_effects() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/music-player.expected"
    ))
    .expect("Run scripts/music-player-parity.ps1");
    assert_eq!(fixture.lines().count(), 144);
    let mut vars = Variables::default();
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let step: u8 = f[3].parse().unwrap();
        let config = config(f[4].split(',').map(|s| s.parse().unwrap()).collect());
        let context = Context {
            logged_in: true,
            resizable: f[2] == "true",
        };
        if step == 0 {
            vars = Variables::default();
            for (id, value) in [
                (30, f[0].parse().unwrap()),
                (40, f[1].parse().unwrap()),
                (47, 1),
            ] {
                vars.set(varp(id), value, false);
            }
        }
        let mut random = Draws(
            f[5].split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse().unwrap())
                .collect(),
        );
        let events = match step {
            0 => config.enter(&mut vars, context, 9, &mut random),
            1 | 3 | 4 | 5 => config.tick(&mut vars, context, &mut random),
            2 => {
                vars.set(varp(32), 255, false);
                config.tick(&mut vars, context, &mut random)
            }
            6 => config.resume(&mut vars, context),
            7 => config.play(&mut vars, context, 3, false),
            8 => {
                for (id, value) in [(30, 1), (31, 65281), (32, 255), (40, 0), (41, 10), (42, 0)] {
                    vars.set(varp(id), value, false);
                }
                config.tick(&mut vars, context, &mut random)
            }
            _ => unreachable!(),
        };
        let events = if step == 8 {
            assert_eq!(f[8], "IllegalArgumentException", "{line}");
            let failure = events.expect_err("playlist cycle256 must fail after playing");
            assert_eq!(failure.reason, "Music playlist: Cycle", "{line}");
            failure.events
        } else {
            assert_eq!(f[8], "", "{line}");
            events.unwrap_or_else(|e| panic!("{line}: {e}"))
        };
        assert!(random.0.is_empty(), "Unused reference draw: {line}");
        for state in f[6].split(',') {
            let (id, value) = state.split_once('=').unwrap();
            assert_eq!(
                vars.get(id.parse().unwrap()),
                value.parse::<i32>().unwrap(),
                "state {state}: {line}"
            );
        }
        let packets = gielinor_server::music::packets(&events, (239 << 16) | 9)
            .unwrap()
            .iter()
            .map(|p| {
                format!(
                    "{}:{}",
                    p.opcode,
                    p.payload
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(packets, f[7], "packets: {line}");
    }
}
