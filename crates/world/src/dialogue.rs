//! Suspended content dialogue, independent of network UI and available to test actors.
//! Text and branch order follow GenericPerson.kt. Random draws occur after greeting.
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression {
    Happy,
    Neutral,
    Angry,
    Confused,
    Quiz,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    Player,
    Npc,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    Chat {
        speaker: Speaker,
        expression: Expression,
        text: &'static str,
    },
    /// Resolve the player name when this page is presented, after prior suspensions.
    NamedChat {
        speaker: Speaker,
        expression: Expression,
        text: &'static str,
    },
    Choice(&'static [&'static str]),
    TitledChoice {
        title: &'static str,
        options: &'static [&'static str],
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    SkillcapeVendor {ctx:crate::skillcape_vendor::Context,stage:crate::skillcape_vendor::Stage},
    LectureTutor {
        config: crate::tutorial_lecture_tutor::Config,
        stage: crate::tutorial_lecture_tutor::Stage,
    },
    MagicTutor {
        config: crate::tutorial_magic_tutor::Config,
        stage: crate::tutorial_magic_tutor::Stage,
    },
    CombatTutor {
        config: crate::tutorial_combat_tutor::Config,
        stage: crate::tutorial_combat_tutor::Stage,
    },
    MiningTutor {
        config: crate::tutorial_mining_tutor::Config,
        stage: crate::tutorial_mining_tutor::Stage,
    },
    Quest(crate::tutorial_quest::Stage),
    Chef {
        config: crate::tutorial_chef::Config,
        stage: crate::tutorial_chef::Stage,
    },
    Survival {
        config: crate::tutorial_survival::Config,
        stage: crate::tutorial_survival::Stage,
    },
    None,
    RollPersonResponse,
    GiveFlyer,
    Retaliate,
    TutorialGuide {
        step: crate::tutorial_steps::Step,
        open_settings: bool,
    },
}
#[derive(Clone, Debug)]
pub struct Conversation {
    pub pages: VecDeque<Page>,
    pub after: Effect,
}
#[derive(Clone, Debug)]
pub struct Pending {
    pub token: u64,
    pub npc: crate::simulation::NpcId,
    pub conversation: Conversation,
}
#[derive(Clone, Debug)]
pub enum Event {
    TutorialStep {
        npc: crate::simulation::NpcId,
        step: crate::tutorial_steps::Step,
        open_tab: Option<i32>,
    },
    Survival {
        npc: crate::simulation::NpcId,
        step: crate::tutorial_steps::Step,
        inventory_tab: bool,
    },
    TutorialGuide {
        npc: crate::simulation::NpcId,
        step: crate::tutorial_steps::Step,
        open_settings: bool,
    },
    Page {
        token: u64,
        npc: crate::simulation::NpcId,
        page: Page,
    },
    Closed,
}
const TRADE: &str = "No, I have nothing I wish to get rid of. If you want to do some trading, there are plenty of shops and market stalls around though.";
const ENEMIES: &str = "I've heard there are many fearsome creatures that dwell under the ground...";
pub const OPTIONS: &[&str] = &[
    "Do you wish to trade?",
    "I'm in search of a quest.",
    "I'm in search of enemies to kill.",
];
impl Conversation {
    /// BasicsTutorScript captures progress when the conversation starts.
    pub fn tutorial_guide(step: crate::tutorial_steps::Step) -> Self {
        use crate::tutorial_steps::Step;
        let (pages, next, open_settings): (&[&'static str], _, _) = if step < Step::GielinorSettings
        {
            (&[
                "Greetings! I see you are a new arrival to the world of Gielinor. My job is to welcome all new visitors. So welcome!",
                "You have already learned the first thing needed to succeed in this world: talking to other people!",
                "You will find many inhabitants of this world have useful things to say to you. By clicking on them you can talk to them.",
                "Now then, let's start by looking at your settings menu.",
            ], Step::GielinorSettings, true)
        } else if step < Step::GielinorDoor {
            (&[
                "Looks like you're making good progress! The menu you've just opened is one of many. You'll learn about the rest as you progress through the tutorial.",
                "Anyway, I'd say it's time for you to go and meet your first instructor!",
            ], Step::GielinorDoor, false)
        } else {
            (&[
                "Welcome back. You have already learnt the first thing needed to succeed in this world: talking to other people!",
                "You will find many inhabitants of this world have useful things to say to you. By clicking on them, you can talk to them.",
                "To continue the tutorial go through that door over there and speak to your first instructor!",
            ], Step::GielinorDoor, false)
        };
        Self {
            pages: pages
                .iter()
                .map(|&text| Page::Chat {
                    speaker: Speaker::Npc,
                    expression: Expression::Happy,
                    text,
                })
                .collect(),
            after: Effect::TutorialGuide {
                step: next,
                open_settings,
            },
        }
    }
    pub fn person() -> Self {
        let mut result = Self {
            pages: VecDeque::new(),
            after: Effect::RollPersonResponse,
        };
        result.player(Expression::Happy, "Hello, how's it going?");
        result
    }
    fn player(&mut self, expression: Expression, text: &'static str) {
        self.pages.push_back(Page::Chat {
            speaker: Speaker::Player,
            expression,
            text,
        });
    }
    fn npc(&mut self, expression: Expression, text: &'static str) {
        self.pages.push_back(Page::Chat {
            speaker: Speaker::Npc,
            expression,
            text,
        });
    }
    /// Caller preserves randomBoolean(128), inventory-space and duplicate checks.
    pub fn response(flyer: bool, roll: u32) -> Option<Self> {
        use Expression::*;
        let mut result = Self {
            pages: VecDeque::new(),
            after: Effect::None,
        };
        if flyer {
            result.npc(Happy, "Have this flyer...");
            result.after = Effect::GiveFlyer;
            return Some(result);
        }
        match roll {
            0 => { result.npc(Happy, "I'm fine, how are you?"); result.player(Happy, "Very well thank you."); }
            1 => result.npc(Neutral, "I think we need a new king. The one we've got isn't very good."),
            2 => { result.npc(Neutral, "How can I help you?"); result.pages.push_back(Page::Choice(OPTIONS)); }
            3 => result.npc(Angry, "None of your business."),
            4 => result.npc(Happy, "Not too bad thanks."),
            5 => { result.npc(Neutral, "Not too bad, but I'm a little worried about the increase of goblins these days."); result.player(Happy, "Don't worry, I'll kill them."); }
            6 => { result.npc(Confused, "Who are you?"); result.player(Happy, "I'm a bold adventurer."); result.npc(Happy, "Ah, a very noble profession."); }
            7 => result.npc(Happy, "I'm very well thank you."),
            8 => result.npc(Happy, "Hello there! Nice weather we've been having."),
            9..=11 => {
                result.npc(Happy, "Hello, how's it going?");
                let choice = match roll { 9 => 3, 10 => 2, _ => 1 };
                result.pages.extend(Self::choice(choice)?.pages);
            }
            12 => result.npc(Happy, "Hello."),
            13 => result.npc(Confused, "Do I know you? I'm in a hurry!"),
            14 => result.npc(Neutral, "I'm a little worried - I've heard there's lots of people going about, killing citizens at random."),
            15 => result.npc(Angry, "Get out of my way, I'm in a hurry!"),
            16 => result.npc(Neutral, "I'm busy right now."),
            17 => { result.npc(Angry, "Are you asking for a fight?"); result.after = Effect::Retaliate; }
            18 => result.npc(Happy, "Yo, wassup!"),
            19 => result.npc(Neutral, "That is classified information."),
            20 => result.npc(Neutral, "No I don't have any spare change."),
            21 => result.npc(Angry, "No, I don't want to buy anything!"),
            _ => return None,
        }
        Some(result)
    }
    pub fn choice(choice: i32) -> Option<Self> {
        use Expression::*;
        let mut result = Self {
            pages: VecDeque::new(),
            after: Effect::None,
        };
        match choice {
            1 => {
                result.player(Neutral, OPTIONS[0]);
                result.npc(Neutral, TRADE);
            }
            2 => {
                result.player(Happy, OPTIONS[1]);
                result.npc(Neutral, "I'm sorry I can't help you there.");
            }
            3 => {
                result.player(Neutral, OPTIONS[2]);
                result.npc(Neutral, ENEMIES);
            }
            _ => return None,
        }
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn person_effects_wait_until_their_pages_are_acknowledged() {
        let greeting = Conversation::person();
        assert_eq!(greeting.after, Effect::RollPersonResponse);
        assert!(matches!(
            greeting.pages.front(),
            Some(Page::Chat {
                speaker: Speaker::Player,
                ..
            })
        ));
        for roll in 0..22 {
            let response = Conversation::response(false, roll).unwrap();
            assert!(!response.pages.is_empty());
            assert_eq!(
                response.after,
                if roll == 17 {
                    Effect::Retaliate
                } else {
                    Effect::None
                }
            );
        }
        assert!(Conversation::response(false, 22).is_none());
        assert_eq!(
            Conversation::response(true, 22).unwrap().after,
            Effect::GiveFlyer
        );
        for choice in [-1, 0, 4, i32::MAX] {
            assert!(Conversation::choice(choice).is_none());
        }
        for choice in 1..=3 {
            assert_eq!(Conversation::choice(choice).unwrap().pages.len(), 2);
        }
    }
}
