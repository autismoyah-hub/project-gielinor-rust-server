use gielinor_world::{
    dialogue::{Conversation, Effect, Expression, Page, Speaker},
    tutorial_steps::Step,
};

#[test]
fn unchanged_kotlin_guide_script_matches_text_branch_and_completion_effects() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-guide.expected"
    ))
    .unwrap();
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let conversation = Conversation::tutorial_guide(Step::from_var(f[0].parse().unwrap()));
        let mut output = vec![];
        for page in conversation.pages {
            let Page::Chat {
                speaker: Speaker::Npc,
                expression: Expression::Happy,
                text,
            } = page
            else {
                panic!("unexpected guide page");
            };
            output.push(format!(
                "chat:happy:{}",
                text.as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            ));
        }
        let Effect::TutorialGuide {
            step,
            open_settings,
        } = conversation.after
        else {
            panic!("missing progression");
        };
        output.push(format!("step:{}:37", step as i32));
        if open_settings {
            output.push("cs:915:11".into());
        }
        assert_eq!(output.join(","), f[3], "{row}");
        assert_eq!((step as i32).to_string(), f[2], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 112);
}
