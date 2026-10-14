use gielinor_world::{
    dialogue::{Effect, Expression, Page, Speaker},
    tutorial_lecture_tutor::{self as tutor, Config, Kind},
    tutorial_steps::Step,
};
fn hex(s: &str) -> String {
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn original_prayer_and_account_instructors_capture_branches_and_finish_after_continue() {
    let mut cases = 0;
    for row in include_str!("fixtures/tutorial-lecture-tutor.expected").lines() {
        let f: Vec<_> = row.split('|').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let config = Config {
            kind: if n(0) == 0 {
                Kind::Prayer
            } else {
                Kind::Account
            },
            step_varp: 5003,
        };
        let mut c = tutor::begin(config, Step::from_var(n(1)));
        let mut output = vec![];
        while let Some(page) = c.pages.pop_front() {
            match page {
                Page::Chat {
                    speaker,
                    expression,
                    text,
                }
                | Page::NamedChat {
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
                    if expression == Expression::Quiz {
                        "quiz"
                    } else {
                        "happy"
                    },
                    hex(text)
                )),
                Page::Choice(options) | Page::TitledChoice { options, .. } => {
                    output.push(format!(
                        "choice:{}:{}",
                        hex("Select an option"),
                        options.iter().map(|s| hex(s)).collect::<Vec<_>>().join(":")
                    ));
                    c = tutor::choice(config, n(2)).unwrap();
                }
            }
        }
        let Effect::LectureTutor { stage, .. } = c.after else {
            panic!("Wrong effect")
        };
        let (step, tab) = tutor::finish(config, stage);
        output.push(format!("step:{}:37", step as i32));
        if let Some(tab) = tab {
            output.push(format!("cs:915:{tab}"));
        }
        assert_eq!(step as i32, n(4), "{row}");
        assert_eq!(output.join(","), f[5], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 672);
}
