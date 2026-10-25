//! Floor One hub banker (`threshold_banker`), from TowerBankerNpc.kt and the generic Banker.kt
//! Talk-to it inherits through `content.banker`. The approved Tower menu keeps the bank and
//! "What is this place?"; PIN settings, the collection box and bank-slot purchases are
//! separate services, not part of this menu.
use crate::dialogue::{Conversation, Effect, Expression, Page, Speaker};
use std::collections::VecDeque;

/// TowerBankerNpc.kt `desc`, sent for OPNPC6 like OpNpc6Handler.kt.
pub const EXAMINE: &str = "He can look after your items.";
/// OpNpc6Handler.kt `ChatType.NpcExamine`.
pub const EXAMINE_CHAT_TYPE: u16 = 28;
pub const GREETING: &str = "Good day, how may I help you?";
pub const OPTIONS: &[&str] = &[
    "I'd like to access my bank account, please.",
    "What is this place?",
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Choice,
    OpenBank,
}
fn chat(speaker: Speaker, expression: Expression, text: &'static str) -> Page {
    Page::Chat {
        speaker,
        expression,
        text,
    }
}
pub fn talk() -> Conversation {
    Conversation {
        pages: VecDeque::from([
            chat(Speaker::Npc, Expression::Quiz, GREETING),
            Page::Choice(OPTIONS),
        ]),
        after: Effect::TowerBanker(Stage::Choice),
    }
}
/// Option numbers are 1-based, as resumed by the shared choice dialogue.
pub fn choice(choice: i32) -> Option<Conversation> {
    use Expression::*;
    use Speaker::*;
    match choice {
        // Banker.kt opens the bank directly, without echoing the option.
        1 => Some(Conversation {
            pages: VecDeque::new(),
            after: Effect::TowerBanker(Stage::OpenBank),
        }),
        2 => Some(Conversation {
            pages: VecDeque::from([
                chat(Player, Quiz, "What is this place?"),
                chat(Npc, Happy, "This is a branch of the Bank of Gielinor. We have branches in many towns."),
                chat(Player, Quiz, "And what do you do?"),
                chat(Npc, Happy, "We will look after your items and money for you. Leave your valuables with us if you want to keep them safe."),
            ]),
            after: Effect::None,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn talk_greets_then_offers_approved_tower_options() {
        let talk = talk();
        assert_eq!(talk.after, Effect::TowerBanker(Stage::Choice));
        assert_eq!(
            talk.pages,
            VecDeque::from([
                chat(Speaker::Npc, Expression::Quiz, GREETING),
                Page::Choice(OPTIONS),
            ])
        );
        assert_eq!(OPTIONS.len(), 2);
    }
    #[test]
    fn bank_option_opens_bank_and_place_option_explains_without_effect() {
        let bank = choice(1).unwrap();
        assert!(bank.pages.is_empty());
        assert_eq!(bank.after, Effect::TowerBanker(Stage::OpenBank));
        let place = choice(2).unwrap();
        assert_eq!(place.pages.len(), 4);
        assert_eq!(place.after, Effect::None);
        assert!(matches!(
            place.pages.front(),
            Some(Page::Chat { speaker: Speaker::Player, text: "What is this place?", .. })
        ));
        for invalid in [-1, 0, 3, i32::MAX] {
            assert!(choice(invalid).is_none());
        }
        assert_eq!(EXAMINE, "He can look after your items.");
    }
}
