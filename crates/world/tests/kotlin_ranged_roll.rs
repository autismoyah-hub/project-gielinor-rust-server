use gielinor_world::{
    melee_roll::Error, npc::RandomSource, ranged_modifiers::Attributes, ranged_roll::Inputs,
};
struct Replay(std::collections::VecDeque<(u32, u32)>);
impl RandomSource for Replay {
    fn below(&mut self, bound: u32) -> u32 {
        let (expected, value) = self.0.pop_front().expect("Unexpected ranged draw");
        assert_eq!(bound, expected);
        assert!(value < bound);
        value
    }
}
#[test]
fn compiled_ranged_operations_match_order_and_random_draws() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ranged-roll.expected"
    ))
    .expect("Run scripts/ranged-roll-parity.ps1");
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let a: Vec<_> = f[0].split(',').collect();
        let n = |i: usize| a[i].parse::<i32>().unwrap();
        let d = |i: usize| a[i].parse::<f64>().unwrap();
        let input = Inputs {
            level: n(0),
            defence_level: n(1),
            attack_bonus: n(2),
            strength_bonus: n(3),
            defence_bonus: n(4),
            style: (n(5) != 0).then_some(n(5) as u8),
            accuracy_prayer: d(6),
            damage_prayer: d(7),
            accuracy_void: d(8),
            damage_void: d(9),
            invocation: n(10),
            speed: n(11),
            magic: n(12),
            distance: n(13),
            accuracy_multiplier: d(14),
            damage_multiplier: d(15),
            bolt_damage: n(16),
            attributes: Attributes {
                ranged: a[17].parse().unwrap(),
                npc: a[18].parse().unwrap(),
            },
        };
        let mut replay = Replay(
            f[4].split(',')
                .map(|draw| {
                    let (b, v) = draw.split_once(':').unwrap();
                    (b.parse().unwrap(), v.parse().unwrap())
                })
                .collect(),
        );
        assert_eq!(
            input.chance().unwrap(),
            f[1].parse::<i32>().unwrap(),
            "{row}"
        );
        let result = input.roll(&mut replay);
        if f[3] == "ERR" {
            assert_eq!(result, Err(Error::EmptyDamageRange), "{row}");
            assert_eq!(
                input.max_hit().unwrap(),
                f[2].parse::<i32>().unwrap(),
                "{row}"
            );
        } else {
            let result = result.unwrap();
            assert_eq!(
                result.max_hit.unwrap_or(-1),
                f[2].parse::<i32>().unwrap(),
                "{row}"
            );
            assert_eq!(result.damage, f[3].parse::<i32>().unwrap(), "{row}");
        }
        assert!(replay.0.is_empty(), "Unused draws: {row}");
        cases += 1;
    }
    assert_eq!(cases, 4096);
}
