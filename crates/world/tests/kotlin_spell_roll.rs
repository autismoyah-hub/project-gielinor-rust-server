use gielinor_world::{
    melee_roll::Error, npc::RandomSource, spell_modifiers::Attributes, spell_roll::Inputs,
};
struct Replay(std::collections::VecDeque<(u32, u32)>);
impl RandomSource for Replay {
    fn below(&mut self, bound: u32) -> u32 {
        let (b, v) = self.0.pop_front().expect("Unexpected spell draw");
        assert_eq!(bound, b);
        assert!(v < bound);
        v
    }
}
#[test]
fn original_spell_formula_stages_and_draws() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/spell-roll.expected"
    ))
    .expect("Run scripts/spell-roll-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let a: Vec<_> = f[0].split(',').collect();
        let n = |i: usize| a[i].parse::<i32>().unwrap();
        let d = |i: usize| a[i].parse::<f64>().unwrap();
        let input = Inputs {
            magic: n(0),
            attack_bonus: n(1),
            defence_level: n(2),
            defence_bonus: n(3),
            accuracy_prayer: d(4),
            accuracy_void: d(5),
            damage_bonus: n(6),
            damage_prayer: n(7),
            invocation: n(8),
            weakness_percent: n(9),
            speed: n(10),
        };
        let attrs = Attributes {
            spell: a[12].parse().unwrap(),
            npc: a[13].parse().unwrap(),
        };
        let mut replay = Replay(
            f[4].split(',')
                .map(|v| {
                    let (b, v) = v.split_once(':').unwrap();
                    (b.parse().unwrap(), v.parse().unwrap())
                })
                .collect(),
        );
        assert_eq!(
            input.chance(attrs).unwrap(),
            f[1].parse::<i32>().unwrap(),
            "{row}"
        );
        let range = input.damage_range(attrs, n(11));
        assert_eq!(format!("{},{}", range.0, range.1), f[2], "{row}");
        let hit = input.accurate(attrs, &mut replay).unwrap();
        if f[3] == "MISS" {
            assert!(!hit, "{row}");
        } else {
            assert!(hit, "{row}");
            let damage = Inputs::damage(range, &mut replay);
            if f[3] == "ERR" {
                assert_eq!(damage, Err(Error::EmptyDamageRange), "{row}");
            } else {
                assert_eq!(damage.unwrap(), f[3].parse::<i32>().unwrap(), "{row}");
            }
        }
        assert!(replay.0.is_empty(), "{row}");
    }
}
#[test]
fn prayer_priorities_and_inclusive_zero_damage() {
    use gielinor_world::spell_roll::prayer_bonuses;
    assert_eq!(prayer_bonuses([1, 1, 1, 1, 1]), (1.05, 10));
    assert_eq!(prayer_bonuses([0, 0, 1, 1, 1]), (1.18, 30));
    assert_eq!(prayer_bonuses([0, 0, 1, 1, 2]), (1.15, 20));
    assert_eq!(prayer_bonuses([2, 2, 2, 1, 1]), (1.25, 40));
    let mut replay = Replay([(1, 0), (5, 4)].into());
    assert_eq!(Inputs::damage((0, 0), &mut replay), Ok(0));
    assert_eq!(Inputs::damage((2, 6), &mut replay), Ok(6));
    assert_eq!(
        Inputs::damage((1, 0), &mut replay),
        Err(Error::EmptyDamageRange)
    );
    assert_eq!(
        Inputs::damage((0, i32::MAX), &mut replay),
        Err(Error::EmptyDamageRange)
    );
    assert!(replay.0.is_empty());
}
