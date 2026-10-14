use gielinor_world::{
    tutorial_combat::{Config, Effect},
    tutorial_progression::{self, Effect as Progress},
    variables::{Transmit, Variables, Varp},
};
#[test]
fn original_death_hooks_progression_and_journal_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-combat-death.expected"
    ))
    .expect("Run scripts/tutorial-combat-death-parity.ps1");
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        progression: tutorial_progression::Config {
            step: varp(1),
            completed: varp(2),
        },
        rat: 1,
        chicken: 2,
    };
    let mut count = 0;
    for row in fixture.lines() {
        let fields: Vec<_> = row.split('|').collect();
        let raw = fields[0].parse::<i32>().unwrap();
        let completed = fields[1].parse::<i32>().unwrap();
        let hero = fields[2] == "true";
        let npc = fields[3].parse::<u16>().unwrap();
        let mut vars = Variables::restore([(1, raw), (2, completed)].into());
        let mut output = Vec::new();
        if hero {
            for effect in config.death(npc, &mut vars, true) {
                match effect {
                    Effect::Message(text) => output.push(format!(
                        "message:{}",
                        text.bytes().map(|b| format!("{b:02x}")).collect::<String>()
                    )),
                    Effect::Progress(effects) => {
                        for effect in effects {
                            output.push(match effect {
                                Progress::Variable(update) => {
                                    format!("var:{}:{}", update.id, update.value)
                                }
                                Progress::Focus { step, npc } => {
                                    assert!(npc.is_none());
                                    format!("focus:{}", step as i32)
                                }
                                Progress::End => "end".into(),
                            });
                        }
                    }
                }
            }
        }
        // Oracle deathWithDrops is a call-recording double. Real death scheduling,
        // exactly-once progression, drops and respawn are separate world tests.
        output.push("death".into());
        assert_eq!(vars.get(1), fields[4].parse::<i32>().unwrap(), "{row}");
        assert_eq!(vars.get(2), fields[5].parse::<i32>().unwrap(), "{row}");
        assert_eq!(output.join(","), fields[6], "{row}");
        count += 1;
    }
    assert_eq!(count, 672);
}
