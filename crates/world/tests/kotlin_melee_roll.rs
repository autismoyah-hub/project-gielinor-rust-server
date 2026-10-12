use gielinor_world::{
    melee_modifiers::Attributes,
    melee_roll::{Error, Inputs},
    npc::RandomSource,
};
struct Replay {
    draws: std::collections::VecDeque<(u32, u32)>,
}
impl RandomSource for Replay {
    fn below(&mut self, bound: u32) -> u32 {
        let (expected, value) = self.draws.pop_front().expect("Unexpected draw");
        assert_eq!(bound, expected);
        assert!(value < bound);
        value
    }
}
#[test]
fn compiled_melee_operations_match_pipeline_and_recorded_draw_sequence() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/melee-roll.expected"
    ))
    .expect("Run scripts/melee-roll-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let a: Vec<f64> = f[0].split(',').map(|s| s.parse().unwrap()).collect();
        let input = Inputs {
            attack_level: a[0] as i32,
            strength_level: a[1] as i32,
            defence_level: a[2] as i32,
            attack_bonus: a[3] as i32,
            strength_bonus: a[4] as i32,
            defence_bonus: a[5] as i32,
            style: (a[6] != 0.0).then_some(a[6] as u8),
            attack_prayer: a[7],
            strength_prayer: a[8],
            void: a[9],
            soul: a[10],
            invocation: a[11] as i32,
            speed: a[12] as i32,
            health: a[13] as i32,
            max_health: a[14] as i32,
            attributes: Attributes {
                melee: a[15] as u64,
                npc: a[16] as u64,
            },
        };
        let mut replay = Replay {
            draws: f[4]
                .split(',')
                .map(|s| {
                    let (b, v) = s.split_once(':').unwrap();
                    (b.parse().unwrap(), v.parse().unwrap())
                })
                .collect(),
        };
        let result = input.roll(&mut replay);
        if f[3] == "ERR" {
            assert_eq!(result, Err(Error::EmptyDamageRange), "case {case}");
        } else {
            let result = result.unwrap();
            assert_eq!(result.chance, f[1].parse::<i32>().unwrap(), "chance {case}");
            assert_eq!(
                result.max_hit.unwrap_or(-1),
                f[2].parse::<i32>().unwrap(),
                "max {case}"
            );
            assert_eq!(result.damage, f[3].parse::<i32>().unwrap(), "damage {case}");
        }
        assert!(replay.draws.is_empty(), "Unconsumed draws {case}");
    }
}
