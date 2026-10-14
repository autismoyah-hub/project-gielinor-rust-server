use gielinor_world::{
    tutorial_progression::Config,
    tutorial_steps::Step,
    variables::{Transmit, Variables, Varp},
};
#[test]
fn original_tutorial_metadata_and_progression_queries_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-steps.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 108 + 12528);
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Never,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        step: varp(0),
        completed: varp(1),
    };
    let hex = |s: &str| {
        s.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        if f[0] == "S" {
            let step = Step::from_var(f[2].parse().unwrap());
            assert_eq!(step.name(), f[1]);
            assert_eq!(hex(step.title()), f[3]);
            assert_eq!(hex(step.body()), f[4]);
            assert_eq!(Step::MAX_PROGRESS.to_string(), f[5]);
        } else if f[0] == "F" {
            use gielinor_world::tutorial_focus;
            let step = Step::from_var(f[1].parse().unwrap());
            assert_eq!(
                tutorial_focus::hint_tile(step)
                    .map(|t| t.packed())
                    .unwrap_or(-1)
                    .to_string(),
                f[2],
                "hint tile {case}"
            );
            assert_eq!(
                tutorial_focus::hint_npc(step).unwrap_or("-"),
                f[3],
                "hint npc {case}"
            );
            assert_eq!(
                tutorial_focus::outline_npc(step).unwrap_or("-"),
                f[4],
                "outline npc {case}"
            );
            assert_eq!(
                tutorial_focus::outline_location(step).unwrap_or("-"),
                f[5],
                "outline location {case}"
            );
        } else {
            let mut vars = Variables::default();
            vars.set(config.step, f[1].parse().unwrap(), false);
            vars.set(config.completed, f[2].parse().unwrap(), false);
            let min = Step::from_var(f[3].parse().unwrap());
            assert_eq!(
                (config.current(&vars) as i32).to_string(),
                f[4],
                "current {case}"
            );
            assert_eq!(config.active(&vars).to_string(), f[5], "active {case}");
            assert_eq!(
                config.require_at_least(&vars, min).to_string(),
                f[6],
                "gate {case}"
            );
            assert_eq!(
                vars.get(0).to_string(),
                f[7],
                "query mutated raw step {case}"
            );
        }
    }
}
