use gielinor_world::{
    tutorial_magic_ui::{self, Home},
    tutorial_progression::{Config, Effect},
    variables::{Transmit, Variables, Varp},
};
#[test]
fn original_magic_overlay_callbacks_match_all_step_and_completed_values() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-magic-ui.expected"
    ))
    .expect("Run scripts/tutorial-magic-ui-parity.ps1");
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let c = Config {
        step: varp(1),
        completed: varp(2),
    };
    let message = |text: &str| {
        format!(
            "message:{}",
            text.bytes().map(|b| format!("{b:02x}")).collect::<String>()
        )
    };
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let raw = f[0].parse::<i32>().unwrap();
        let done = f[1].parse::<i32>().unwrap();
        let mut vars = Variables::restore([(1, raw), (2, done)].into());
        let mut output = Vec::new();
        let step = c.current(&vars);
        if f[2] == "true" {
            if tutorial_magic_ui::wind(step, done) {
                for effect in c.set(
                    &mut vars,
                    gielinor_world::tutorial_steps::Step::MagicCast,
                    None,
                    true,
                ) {
                    output.push(match effect {
                        Effect::Variable(v) => format!("var:{}:{}", v.id, v.value),
                        Effect::Focus { step, npc } => {
                            assert!(npc.is_none());
                            format!("focus:{}", step as i32)
                        }
                        Effect::End => panic!("Unexpected completion"),
                    });
                }
                output.push(message(tutorial_magic_ui::WIND_MESSAGE));
            }
        } else {
            match tutorial_magic_ui::home(step, done) {
                Home::None => (),
                Home::Deny(text) => output.push(message(text)),
                Home::Leave => output.push("leave".into()),
            }
        }
        assert_eq!(vars.get(1), f[3].parse::<i32>().unwrap(), "{row}");
        assert_eq!(vars.get(2), f[4].parse::<i32>().unwrap(), "{row}");
        assert_eq!(output.join(","), f[5], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 336);
}
