//! SurvivalTutorScript dialogue stages; gifts are two independent transactions.
use crate::{
    dialogue::{Conversation, Effect, Page},
    tutorial_steps::Step,
    tutorial_survival_text::PAGES,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub step_varp: u16,
    pub net: i32,
    pub axe: i32,
    pub tinderbox: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    First,
    Tools,
    RecapChoice,
    RecapFinish,
}
pub const OPTIONS: &[&str] = &[
    "Tell me about my skills again.",
    "Tell me about Woodcutting again.",
    "Tell me about Firemaking again.",
    "Tell me about Fishing again.",
    "Tell me about Cooking again.",
];
fn pages(config: Config, range: std::ops::Range<usize>, stage: Option<Stage>) -> Conversation {
    Conversation {
        pages: PAGES[range].iter().cloned().collect(),
        after: stage.map_or(Effect::None, |stage| Effect::Survival { config, stage }),
    }
}
pub fn begin(config: Config, step: Step) -> Conversation {
    use Step::*;
    match step {
        s if s < SurvivalInv => pages(config, 6..8, Some(Stage::First)),
        SurvivalInv => pages(config, 0..1, None),
        SurvivalFish => pages(config, 1..2, None),
        SurvivalSkills => pages(config, 2..3, None),
        SurvivalTools => pages(config, 8..10, Some(Stage::Tools)),
        SurvivalWc => pages(config, 3..4, None),
        SurvivalFm => pages(config, 4..5, None),
        SurvivalCook => pages(config, 5..6, None),
        _ => {
            let mut conversation = pages(config, 10..11, Some(Stage::RecapChoice));
            conversation.pages.push_back(Page::Choice(OPTIONS));
            conversation
        }
    }
}
pub fn choice(config: Config, choice: i32) -> Option<Conversation> {
    let range = match choice {
        1 => 11..14,
        2 => 14..17,
        3 => 17..20,
        4 => 20..24,
        5 => 24..26,
        _ => return None,
    };
    Some(pages(config, range, Some(Stage::RecapFinish)))
}
