//! MagicTutorScript's captured dialogue branches and deferred rune replenishment.
//! The host executes actions only after the preceding page/choice is resumed.
use crate::{
    inventory::{Catalog, Insert, Inventory},
    smithing_make::total,
    tutorial_steps::Step,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub step_varp: u16,
    pub air: i32,
    pub mind: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    First,
    Runes,
    Waiting,
    Leave,
    Path,
    BrandNew,
    Ironman,
    Home,
    Confirm,
    Done,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Step(Step),
    Runes,
    RunesAndStep,
    Leave,
}
pub const FIRST: &[(&str, bool)] = &[
    ("Hello.", true),
    ("Good day, newcomer. My name is Terrova. I'm here to tell you about Magic. Let's start by opening your magic interface.", false),
];
pub const RUNES: &str = "Currently you can only cast one offensive spell called Wind Strike. Let's try it out on one of those chickens.";
pub const PLENTY: &str = "Sorry, I don't have any more runes just now. But I see you have plenty.";
pub const LEAVE: &str = "Well, you're all finished here now. I'll give you a reasonable number of runes when you leave.";
pub const BRAND_NEW: &str = "When you get to the mainland you will find yourself in the town of Lumbridge. If you want some ideas on where to go next, talk to my friend Adventurer Jon. You can't miss him; he's wearing light blue armour with a huge sword. There are also many tutors willing to teach you about the many skills you could learn.";
pub const IRONMAN: &[(&str, bool)] = &[
    ("If you're interested in that, you should go and ask the Ironman tutor before you leave this island. He's standing outside my house.", false),
    ("Thanks, I'd better go and find him.", true),
];
pub const HOME: &str = "Excellent, I think that covers everything. When you're ready to go, use your home teleport spell to teleport to Lumbridge!";
pub const GO_TITLE: &str = "Do you want to go to the mainland?";
pub const GO: &[&str] = &["Yes.", "No."];
pub const PATH_TITLE: &str = "Before you leave...";
pub const PATH: &[&str] = &[
    "I'm brand new to this.",
    "Tell me about Ironman first.",
    "I'm ready — home teleport me.",
];
pub const CONFIRM: &[&str] = &["Teleport me now.", "I'll cast it myself."];

pub fn begin(step: Step) -> Stage {
    if step < Step::MagicTab {
        Stage::First
    } else if step < Step::MagicRunes {
        Stage::Runes
    } else if step < Step::LeaveTalk {
        Stage::Waiting
    } else {
        Stage::Leave
    }
}
/// Evaluate both totals when entering waitingForCast, before any suspension.
pub fn plenty(config: Config, inv: &Inventory, catalog: &Catalog) -> bool {
    total(inv, catalog, config.air) >= 5 && total(inv, catalog, config.mind) >= 5
}
/// Independent invAdd transactions, including the reference's signed arithmetic.
pub fn replenish(config: Config, inv: &mut Inventory, catalog: &Catalog) {
    let air = total(inv, catalog, config.air);
    let mind = total(inv, catalog, config.mind);
    for (id, count) in [(config.air, air), (config.mind, mind)] {
        if count < 5 {
            let _ = inv.insert(catalog, Insert::new(id, 5i32.wrapping_sub(count)));
        }
    }
}
pub fn continued(stage: Stage) -> (Stage, Action) {
    match stage {
        Stage::First => (Stage::Done, Action::Step(Step::MagicTab)),
        Stage::Runes => (Stage::Done, Action::RunesAndStep),
        Stage::BrandNew => (Stage::Home, Action::None),
        Stage::Home => (Stage::Confirm, Action::None),
        _ => (Stage::Done, Action::None),
    }
}
pub fn choice(stage: Stage, option: i32) -> Option<(Stage, Action)> {
    Some(match (stage, option) {
        (Stage::Leave, 1) => (Stage::Path, Action::None),
        (Stage::Leave, 2) => (Stage::Done, Action::None),
        (Stage::Path, 1) => (Stage::BrandNew, Action::None),
        (Stage::Path, 2) => (Stage::Ironman, Action::None),
        (Stage::Path, 3) => (Stage::Home, Action::None),
        (Stage::Confirm, 1) => (Stage::Done, Action::Leave),
        (Stage::Confirm, 2) => (Stage::Done, Action::Step(Step::HomeTele)),
        _ => return None,
    })
}

pub fn conversation(config: Config, stage: Stage) -> crate::dialogue::Conversation {
    use crate::dialogue::{Conversation, Effect, Expression, Page, Speaker};
    let mut pages = std::collections::VecDeque::new();
    let mut chat = |text, player| {
        pages.push_back(Page::Chat {
            speaker: if player {
                Speaker::Player
            } else {
                Speaker::Npc
            },
            expression: Expression::Happy,
            text,
        })
    };
    match stage {
        Stage::First => {
            for &(text, player) in FIRST {
                chat(text, player);
            }
        }
        Stage::Runes => chat(RUNES, false),
        Stage::Waiting => chat(PLENTY, false),
        Stage::Leave => chat(LEAVE, false),
        Stage::BrandNew => chat(BRAND_NEW, false),
        Stage::Ironman => {
            for &(text, player) in IRONMAN {
                chat(text, player);
            }
        }
        Stage::Home => chat(HOME, false),
        _ => (),
    }
    match stage {
        Stage::Leave => pages.push_back(Page::TitledChoice {
            title: GO_TITLE,
            options: GO,
        }),
        Stage::Path => pages.push_back(Page::TitledChoice {
            title: PATH_TITLE,
            options: PATH,
        }),
        Stage::Confirm => pages.push_back(Page::Choice(CONFIRM)),
        _ => (),
    }
    Conversation {
        pages,
        after: Effect::MagicTutor { config, stage },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::StackType;
    #[test]
    fn branch_boundaries_and_leave_choices_preserve_original_paths() {
        for raw in -1..=54 {
            let step = Step::from_var(raw);
            let expected = if step < Step::MagicTab {
                Stage::First
            } else if step < Step::MagicRunes {
                Stage::Runes
            } else if step < Step::LeaveTalk {
                Stage::Waiting
            } else {
                Stage::Leave
            };
            assert_eq!(begin(step), expected);
        }
        assert_eq!(choice(Stage::Leave, 2), Some((Stage::Done, Action::None)));
        assert_eq!(choice(Stage::Path, 2), Some((Stage::Ironman, Action::None)));
        assert_eq!(continued(Stage::Ironman), (Stage::Done, Action::None));
        assert_eq!(continued(Stage::BrandNew), (Stage::Home, Action::None));
        assert_eq!(
            choice(Stage::Confirm, 1),
            Some((Stage::Done, Action::Leave))
        );
        assert_eq!(
            choice(Stage::Confirm, 2),
            Some((Stage::Done, Action::Step(Step::HomeTele)))
        );
        for option in [i32::MIN, -1, 0, 4, i32::MAX] {
            assert_eq!(choice(Stage::Path, option), None);
        }
    }
    #[test]
    fn independent_topups_preserve_existing_stacks_and_partial_full_inventory_success() {
        let config = Config {
            step_varp: 1,
            air: 556,
            mind: 558,
        };
        let mut catalog = Catalog::default();
        catalog.stackable.extend([556, 558]);
        for (air, mind) in [(0, 0), (1, 4), (5, 5), (25, 15)] {
            let mut inv = Inventory::new(28, StackType::Normal, false);
            if air > 0 {
                inv.insert(&catalog, Insert::new(556, air)).unwrap();
            }
            if mind > 0 {
                inv.insert(&catalog, Insert::new(558, mind)).unwrap();
            }
            replenish(config, &mut inv, &catalog);
            assert_eq!(
                (total(&inv, &catalog, 556), total(&inv, &catalog, 558)),
                (air.max(5), mind.max(5))
            );
            assert!(plenty(config, &inv, &catalog));
        }
        let mut full = Inventory::new(1, StackType::Normal, false);
        full.insert(&catalog, Insert::new(558, 1)).unwrap();
        replenish(config, &mut full, &catalog);
        assert_eq!(
            (total(&full, &catalog, 556), total(&full, &catalog, 558)),
            (0, 5)
        );
        assert!(!plenty(config, &full, &catalog));
    }
}
