//! MasterChefScript captures the opening step, but checks recap progress and
//! inventory only when its final suspension resumes.
use crate::{
    dialogue::{Conversation, Effect, Expression, Page, Speaker},
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub step_varp: u16,
    pub flour: i32,
    pub water: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Ingredients,
    RecapChoice,
    RecapFinish,
}
pub const OPTIONS: &[&str] = &[
    "Tell me about making dough again.",
    "Tell me about range cooking again.",
    "Nothing thanks.",
];
const TEXT: [&str; 7] = [
    "Ah! Welcome, newcomer. I am the Master Chef, Lev. It is here that I will teach you how to cook food truly fit for a king.",
    "I already know how to cook. Brynna taught me just now.",
    "Hahahahahaha! You call THAT cooking? Some shrimp on an open log fire? Oh, no, no, no. I am going to teach you the fine art of cooking bread.",
    "Time for you to learn the fine art of cooking bread.",
    "Do you need something?",
    "It's quite simple. Just use a pot of flour on a bucket of water, or vice versa, and you'll make dough. You can fill a bucket with water at any sink.",
    "The range is the only place you can cook a lot of the more complex foods in Gielinor. To cook on a range, all you need to do is click on it. You'll need to make sure you have the required items in your inventory though.",
];
fn pages(config: Config, range: std::ops::Range<usize>, stage: Stage) -> Conversation {
    Conversation {
        pages: range
            .map(|i| Page::Chat {
                speaker: if i == 1 {
                    Speaker::Player
                } else {
                    Speaker::Npc
                },
                expression: if i == 1 {
                    Expression::Neutral
                } else {
                    Expression::Happy
                },
                text: TEXT[i],
            })
            .collect(),
        after: Effect::Chef { config, stage },
    }
}
pub fn begin(config: Config, step: Step) -> Conversation {
    if step < Step::ChefDough {
        pages(config, 0..3, Stage::Ingredients)
    } else if step < Step::ChefBread {
        pages(config, 3..4, Stage::Ingredients)
    } else {
        let mut conversation = pages(config, 4..5, Stage::RecapChoice);
        conversation.pages.push_back(Page::Choice(OPTIONS));
        conversation
    }
}
pub fn choice(config: Config, choice: i32) -> Option<Conversation> {
    let range = match choice {
        1 => 5..6,
        2 => 6..7,
        3 => 0..0,
        _ => return None,
    };
    Some(pages(config, range, Stage::RecapFinish))
}
