//! Existing PrayerTutorScript and AccountTutorScript dialogue branches.
use crate::{
    dialogue::{Conversation, Effect, Expression, Page, Speaker},
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Prayer,
    Account,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    First,
    Teaching,
    RecapChoice,
    Finish,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub kind: Kind,
    pub step_varp: u16,
}
// Original Prayer source SHA256: 59b36354232afb6bdb53ee14556be8d5d8fab73e8c2df80c0c517c4c0fd29bda
const PRAYER: &[Page] = &[
Page::Chat { speaker:Speaker::Player,expression:Expression::Happy,text:"Good day, brother." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Hello, I'm Brother Brace. I'm here to tell you all about Prayer." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"This is your Prayer list. Prayers can help a lot in combat. Click on the prayer you wish to use to activate it, and click it again to deactivate it." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Active prayers will drain your Prayer Points, which you can recharge by finding an altar or other holy spot and praying there." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"As you noticed, most enemies will drop bones when defeated. Burying bones, by clicking them in your inventory, will gain you Prayer experience." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"That's all I have to teach you for now. If you need a reminder about your prayers, just speak to me again." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Peace be with you, my child. Can I help you?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Ah, yes, your prayer list. Prayers can help a lot in combat. Click on the prayer you wish to use to activate it, and click it again to deactivate it." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Active prayers will drain your Prayer Points, which you can recharge by finding an altar or other holy spot and praying there." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"As you noticed, most enemies will drop bones when defeated. Burying bones, by clicking them in your inventory, will gain you Prayer experience." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Do you need anything else?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Okay then." },
];
// Original Account source SHA256: 8a0590015512f8a13a426960ee5c420fca8915bcecec45371f12f1643afbcfb6
const ACCOUNT: &[Page] = &[
Page::Chat { speaker:Speaker::Player,expression:Expression::Quiz,text:"Hello. Who are you?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"I'm the Account Guide. I'm here to tell you about your account." },
Page::Chat { speaker:Speaker::Player,expression:Expression::Quiz,text:"What's an account?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Your character IS your account! It's what you are using to play the game right now. Everyone here has one and some people even have multiple!" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"There's various things you can do with your account which is where the Account Management menu comes into play. Let's take a look at it now." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"As you can see, there's a few things you can do with your account." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"First, you have membership. All accounts are free to play by default but if you desire, you can buy membership to unlock members' worlds. These worlds give you access to extra skills, areas, quests, and more!" },
Page::Chat { speaker:Speaker::Player,expression:Expression::Quiz,text:"What's a world?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"A world is what you play the game on. Most worlds can hold up to 2000 players. They are split into two main types: free worlds and members' worlds. You can swap worlds with the World Switcher in the Logout menu." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"You are currently on a free world. If you want to switch to a members' world and enjoy all of the benefits, you'll need to purchase membership. You can do so using the store in the Account Management menu." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Bonds are another thing that you should know about. You can purchase Bonds from the store in the Account Management menu. Once you have some Bonds, you can redeem them for membership!" },
Page::Chat { speaker:Speaker::Player,expression:Expression::Quiz,text:"But why would I use a Bond instead of just buying membership normally?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Unlike membership, Bonds can be traded with other players. You could purchase a Bond and then sell it to another player for some gold. Alternatively, you could use your gold to buy a Bond from another player!" },
Page::Chat { speaker:Speaker::Player,expression:Expression::Quiz,text:"So if I used my gold to buy a Bond from another player, I could get membership without spending any real money?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Exactly!" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Next up, you have your inbox and name changer. The inbox is where you'll receive important information related to your account and the game. Be sure to keep an eye on it." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"The name changer allows you to change your display name if you don't like your current one. Be aware though that this is only available to accounts with membership." },
Page::Chat { speaker:Speaker::Player,expression:Expression::Quiz,text:"Awesome. Is there anything else I should know about accounts?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Your account is very valuable and it's important you look after it. Various tools can help with this, such as Bank PINs and two-factor authentication options." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"You'll find more information on account security on the Customer Support webpage. You can use the Useful Links section of the Account Management menu to get there." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Anyway, that's everything I have to tell you about accounts. Let me know if you need a recap, otherwise head on to your next instructor." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Hello. Would you like to hear about your account again?" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Excellent. Well there's a few things you can do with your account." },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"First, you have membership. All accounts are free to play by default but if you desire, you can buy membership to unlock members' worlds. These worlds give you access to extra skills, areas, quests, and more!" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Bonds are another thing that you should know about. You can purchase Bonds from the store in the Account Management menu. Once you have some Bonds, you can redeem them for membership!" },
Page::Chat { speaker:Speaker::Npc,expression:Expression::Happy,text:"Your account is very valuable and it's important you look after it. Various tools can help with this, such as Bank PINs and two-factor authentication options." },
];
pub fn begin(config: Config, step: Step) -> Conversation {
    let (tab, exit) = match config.kind {
        Kind::Prayer => (Step::PrayerTab, Step::PrayerExit),
        Kind::Account => (Step::AccountMgmt, Step::AccountExit),
    };
    let stage = if step < tab {
        Stage::First
    } else if step < exit {
        Stage::Teaching
    } else {
        Stage::RecapChoice
    };
    let range = match (config.kind, stage) {
        (Kind::Prayer, Stage::First) => 0..2,
        (Kind::Prayer, Stage::Teaching) => 2..6,
        (Kind::Prayer, _) => 6..7,
        (Kind::Account, Stage::First) => 0..5,
        (Kind::Account, Stage::Teaching) => 5..21,
        (Kind::Account, _) => 21..22,
    };
    let mut result = pages(config, range, stage);
    if stage == Stage::RecapChoice {
        result.pages.push_back(Page::Choice(match config.kind {
            Kind::Prayer => &[
                "Tell me about Prayers again.",
                "Nope, I'm ready to move on!",
            ],
            Kind::Account => &["Yes!", "No thanks."],
        }));
    }
    result
}
fn pages(config: Config, range: std::ops::Range<usize>, stage: Stage) -> Conversation {
    let source = match config.kind {
        Kind::Prayer => PRAYER,
        Kind::Account => ACCOUNT,
    };
    Conversation {
        pages: source[range].iter().cloned().collect(),
        after: Effect::LectureTutor { config, stage },
    }
}
pub fn choice(config: Config, choice: i32) -> Option<Conversation> {
    let range = match (config.kind, choice) {
        (Kind::Prayer, 1) => 7..11,
        (Kind::Prayer, 2) => 11..12,
        (Kind::Account, 1) => 22..26,
        (Kind::Account, 2) => 26..26,
        _ => return None,
    };
    Some(pages(config, range, Stage::Finish))
}
pub fn finish(config: Config, stage: Stage) -> (Step, Option<i32>) {
    match (config.kind, stage) {
        (Kind::Prayer, Stage::First) => (Step::PrayerTab, Some(5)),
        (Kind::Account, Stage::First) => (Step::AccountMgmt, Some(8)),
        (_, Stage::RecapChoice) => unreachable!("Choice remains suspended"),
        (Kind::Prayer, _) => (Step::PrayerExit, None),
        (Kind::Account, _) => (Step::AccountExit, None),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_page_counts_and_recap_completion_are_preserved() {
        assert_eq!((PRAYER.len(), ACCOUNT.len()), (12, 26));
        for kind in [Kind::Prayer, Kind::Account] {
            let config = Config {
                kind,
                step_varp: 5003,
            };
            let c = begin(config, Step::Completed);
            assert!(matches!(c.pages.back(), Some(Page::Choice(_))));
            assert!(choice(config, 0).is_none());
            assert!(choice(config, 3).is_none());
            let c = choice(config, 2).unwrap();
            assert_eq!(c.pages.len(), if kind == Kind::Prayer { 1 } else { 0 });
            let (step, tab) = finish(config, Stage::Finish);
            assert_eq!(
                step,
                if kind == Kind::Prayer {
                    Step::PrayerExit
                } else {
                    Step::AccountExit
                }
            );
            assert_eq!(tab, None);
        }
    }
}
