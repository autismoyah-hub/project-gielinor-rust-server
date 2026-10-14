use gielinor_world::{
    dialogue::{Effect, Page, Speaker},
    tutorial_steps::Step,
    tutorial_survival::{self, Config, Stage},
};
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn original_survival_script_pages_choices_gift_calls_and_late_progress_check() {
    let config = Config {
        step_varp: 5003,
        net: 303,
        axe: 1351,
        tinderbox: 590,
    };
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-survival.expected"
    ))
    .unwrap();
    let mut cases = 0;
    for row in fixture.lines() {
        let fields: Vec<_> = row.split('|').collect();
        let initial: i32 = fields[0].parse().unwrap();
        let choice = fields[1].parse().unwrap();
        let mutation: i32 = fields[2].parse().unwrap();
        let mut raw = if mutation == -1 { initial } else { mutation };
        let mut conversation = tutorial_survival::begin(config, Step::from_var(initial));
        let mut output = vec![];
        loop {
            if let Some(page) = conversation.pages.pop_front() {
                match page {
                    Page::NamedChat { .. } => panic!("Unexpected named page in static reference"),
                    Page::Chat { speaker, text, .. } => output.push(format!(
                        "chat:{}:happy:{}",
                        if speaker == Speaker::Npc {
                            "Npc"
                        } else {
                            "Player"
                        },
                        hex(text)
                    )),
                    Page::Choice(options) | Page::TitledChoice { options, .. } => {
                        output.push(format!(
                            "choice:{}",
                            options.iter().map(|s| hex(s)).collect::<Vec<_>>().join(":")
                        ));
                        conversation = tutorial_survival::choice(config, choice).unwrap();
                    }
                }
                continue;
            }
            let next = match conversation.after {
                Effect::None => None,
                Effect::Survival { stage, .. } => match stage {
                    Stage::First => {
                        output.push("add:303:1".into());
                        Some((6, true))
                    }
                    Stage::Tools => {
                        output.extend(["add:1351:1".into(), "add:590:1".into()]);
                        Some((10, false))
                    }
                    Stage::RecapFinish => {
                        (Step::from_var(raw) >= Step::SurvivalGate).then_some((13, false))
                    }
                    Stage::RecapChoice => panic!("unresolved choice"),
                },
                _ => panic!("unexpected effect"),
            };
            if let Some((step, tab)) = next {
                raw = step;
                output.push(format!("step:{step}:37"));
                if tab {
                    output.push("cs:915:3".into());
                }
            }
            break;
        }
        assert_eq!(raw.to_string(), fields[3], "{row}");
        assert_eq!(output.join(","), fields[4], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 840);
}
