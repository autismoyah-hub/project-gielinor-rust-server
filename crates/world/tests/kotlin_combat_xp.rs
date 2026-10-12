use gielinor_world::{
    attack_profile::Attack,
    combat_xp::{self, Skill},
    stats::Stats,
};
#[test]
fn compiled_combat_manager_matches_fine_xp_for_all_attack_families() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/combat-xp.expected"
    ))
    .expect("Run scripts/combat-xp-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    let skills = [
        Skill::Attack,
        Skill::Strength,
        Skill::Defence,
        Skill::Ranged,
        Skill::Magic,
        Skill::Hitpoints,
    ];
    for (case, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split('|').collect();
        let (input, expected) = (fields[0], fields[1]);
        let a: Vec<f64> = input.split(',').map(|v| v.parse().unwrap()).collect();
        let style = a[1] as u8;
        let attack = match a[0] as u8 {
            0 => Attack::Melee {
                kind: None,
                style: (1..=4).contains(&style).then_some(style),
                stance: 0,
            },
            1 => Attack::Ranged {
                kind: 5,
                style: (5..=7).contains(&style).then_some(style),
            },
            2 => Attack::Spell {
                defensive: a[2] == 1.0,
            },
            _ => Attack::Staff { style: None },
        };
        let mut stats = Stats::default();
        for skill in skills {
            stats.set_base(skill.id(), 99);
            stats.set_fine_xp(skill.id(), a[7] as i32).unwrap();
        }
        combat_xp::advance(&mut stats, attack, a[3] as i32, a[4] as i32, a[5], a[6]).unwrap();
        let actual = skills
            .iter()
            .map(|s| stats.fine_xp(s.id()).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, expected, "case {case}: {input}");
        let n: Vec<i32> = fields[2].split(',').map(|v| v.parse().unwrap()).collect();
        assert_eq!(
            combat_xp::npc_multiplier(
                (n[9] >= 0).then_some(n[9]),
                [n[0], n[1], n[2], n[3]],
                [n[6], n[7], n[8]],
                n[4],
                n[5]
            ),
            fields[3].parse::<i32>().unwrap(),
            "NPC multiplier {case}"
        );
    }
}
