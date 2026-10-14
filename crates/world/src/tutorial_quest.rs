//! QuestTutorScript uses the captured opening step and unconditionally sets the
//! destination step after the last page, including recaps from later progress.
use crate::{
    dialogue::{Conversation, Effect, Expression, Page, Speaker},
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    First,
    Finish,
    RecapChoice,
}
pub const OPTIONS: &[&str] = &["Yes!", "Nope, I'm ready to move on!"];
const TEXT: [&str;10]=[
    "Ah. Welcome, adventurer. I'm here to tell you all about quests. Let's start by opening your quest list.",
    "Now that you have the quest list open, you can see all the quests within it. Opening a quest's journal will provide you with more information on it.",
    "If you haven't started the quest, it will tell you where to begin and what requirements you need. If the quest is in progress, it will remind you what to do next.",
    "It's very easy to find quest start points. Just look out for the quest icon on your minimap. You should see one marking this house.",
    "The quests themselves can vary greatly from collecting beads to hunting down dragons. Completing quests will reward you with all sorts of things, such as new areas and better weapons!",
    "There's not a lot more I can tell you about questing. You have to experience the thrill of it yourself to fully understand. Let me know if you want a recap, otherwise you can move on.",
    "Would you like to hear about quests again?",
    "Within your quest list, you'll unsurprisingly find a list of quests. Opening a quest's journal will provide you with more information on it.",
    "If you haven't started the quest, it will tell you where to begin and what requirements you need. If the quest is in progress, it will remind you what to do next.",
    "Okay then.",
];
fn pages(range: std::ops::Range<usize>, stage: Stage) -> Conversation {
    Conversation {
        pages: range
            .map(|i| Page::Chat {
                speaker: Speaker::Npc,
                expression: Expression::Happy,
                text: TEXT[i],
            })
            .collect(),
        after: Effect::Quest(stage),
    }
}
pub fn begin(step: Step) -> Conversation {
    if step < Step::QuestList {
        pages(0..1, Stage::First)
    } else if step < Step::QuestLadder {
        pages(1..6, Stage::Finish)
    } else {
        let mut conversation = pages(6..7, Stage::RecapChoice);
        conversation.pages.push_back(Page::Choice(OPTIONS));
        conversation
    }
}
pub fn choice(choice: i32) -> Option<Conversation> {
    Some(pages(
        match choice {
            1 => 7..9,
            2 => 9..10,
            _ => return None,
        },
        Stage::Finish,
    ))
}
