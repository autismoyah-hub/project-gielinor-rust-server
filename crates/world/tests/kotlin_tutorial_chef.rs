use gielinor_world::{
    dialogue::{Effect, Expression, Page, Speaker},
    tutorial_chef::{self, Config, Stage},
    tutorial_steps::Step,
};
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn original_chef_dialogue_ingredient_queries_and_late_progress() {
    let config = Config {
        step_varp: 5003,
        flour: 1933,
        water: 1929,
    };
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-chef.expected"
    ))
    .unwrap();
    let mut count = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let initial: i32 = f[0].parse().unwrap();
        let choice = f[1].parse().unwrap();
        let mutate: i32 = f[2].parse().unwrap();
        let held: i32 = f[3].parse().unwrap();
        let mut raw = if mutate == -1 { initial } else { mutate };
        let mut conversation = tutorial_chef::begin(config, Step::from_var(initial));
        let mut output = vec![];
        loop {
            if let Some(page) = conversation.pages.pop_front() {
                match page {
                    Page::NamedChat { .. } => panic!("Unexpected named page in static reference"),
                    Page::Chat {
                        speaker,
                        expression,
                        text,
                    } => output.push(format!(
                        "chat:{}:{}:{}",
                        if speaker == Speaker::Npc {
                            "Npc"
                        } else {
                            "Player"
                        },
                        if expression == Expression::Happy {
                            "happy"
                        } else {
                            "neutral"
                        },
                        hex(text)
                    )),
                    Page::Choice(options) | Page::TitledChoice { options, .. } => {
                        output.push(format!(
                            "choice:{}",
                            options.iter().map(|s| hex(s)).collect::<Vec<_>>().join(":")
                        ));
                        conversation = tutorial_chef::choice(config, choice).unwrap();
                    }
                }
                continue;
            }
            let Effect::Chef { stage, .. } = conversation.after else {
                panic!("Unexpected effect")
            };
            let next = match stage {
                Stage::Ingredients => {
                    if held & 1 == 0 {
                        output.push("add:1933:1".into());
                    }
                    if held & 2 == 0 {
                        output.push("add:1929:1".into());
                    }
                    Some(16)
                }
                Stage::RecapFinish => (Step::from_var(raw) >= Step::ChefExit).then_some(18),
                Stage::RecapChoice => panic!("Unresolved choice"),
            };
            if let Some(step) = next {
                raw = step;
                output.push(format!("step:{step}:37"));
            }
            break;
        }
        assert_eq!(raw.to_string(), f[4], "{row}");
        assert_eq!(output.join(","), f[5], "{row}");
        count += 1;
    }
    assert_eq!(count, 2016);
}
