use gielinor_world::{
    dialogue::{Effect, Page},
    tutorial_quest::{self, Stage},
    tutorial_steps::Step,
};
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn original_quest_tutor_captured_progress_recap_rewind_and_tab_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-quest.expected"
    ))
    .unwrap();
    let mut count = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let initial: i32 = f[0].parse().unwrap();
        let choice = f[1].parse().unwrap();
        let mut conversation = tutorial_quest::begin(Step::from_var(initial));
        let mut output = vec![];
        let final_step = loop {
            if let Some(page) = conversation.pages.pop_front() {
                match page {
                    Page::NamedChat { .. } => panic!("Unexpected named page in static reference"),
                    Page::Chat { text, .. } => output.push(format!("chat:Npc:happy:{}", hex(text))),
                    Page::Choice(options) | Page::TitledChoice { options, .. } => {
                        output.push(format!(
                            "choice:{}",
                            options.iter().map(|s| hex(s)).collect::<Vec<_>>().join(":")
                        ));
                        conversation = tutorial_quest::choice(choice).unwrap();
                    }
                }
                continue;
            }
            let Effect::Quest(stage) = conversation.after else {
                panic!("Unexpected effect")
            };
            let step = match stage {
                Stage::First => 21,
                Stage::Finish => 22,
                Stage::RecapChoice => panic!("Unresolved choice"),
            };
            output.push(format!("step:{step}:37"));
            if stage == Stage::First {
                output.push("cs:915:2".into());
            }
            break step;
        };
        assert_eq!(final_step.to_string(), f[3], "{row}");
        assert_eq!(output.join(","), f[4], "{row}");
        count += 1;
    }
    assert_eq!(count, 336);
}
