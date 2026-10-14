//! TutorialGatesScript decisions. Effects must run in order: advancement/focus
//! uses the departure position, before teleport/telejump changes the player.
use crate::{locations::Location, tutorial_steps::Step, CoordError, CoordGrid};

#[derive(Clone)]
pub struct Config {
    pub progression: crate::tutorial_progression::Config,
    pub gates: std::collections::BTreeMap<u32, Gate>,
    pub hint_npcs: std::collections::BTreeMap<Step, u16>,
}

pub const BINDINGS: [(&str, Gate); 18] = [
    ("tut2_survival_entry", Gate::SurvivalEntry),
    ("tut2_survival_exit_l", Gate::SurvivalExit),
    ("tut2_survival_exit_r", Gate::SurvivalExit),
    ("tut2_nav_entry", Gate::ChefEntry),
    ("tut2_nav_exit", Gate::ChefExit),
    ("tut2_quest_entry", Gate::QuestEntry),
    ("tut2_quest_ladder", Gate::QuestLadder),
    ("tut2_mining_ladder", Gate::MiningLadderUp),
    ("tut2_mining_exit_l", Gate::MiningExit),
    ("tut2_mining_exit_r", Gate::MiningExit),
    ("tut2_combat_entry_l", Gate::CombatCage),
    ("tut2_combat_entry_r", Gate::CombatCage),
    ("tut2_combat_ladder", Gate::CombatLadder),
    ("tut2_bank_ladder", Gate::BankLadder),
    ("tut2_bank_exit", Gate::BankExit),
    ("tut2_prayer_door", Gate::PrayerDoor),
    ("tut2_prayer_exit_l", Gate::PrayerExit),
    ("tut2_prayer_exit_r", Gate::PrayerExit),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    SurvivalEntry,
    SurvivalExit,
    ChefEntry,
    ChefExit,
    QuestEntry,
    QuestLadder,
    MiningLadderUp,
    MiningExit,
    CombatCage,
    CombatLadder,
    BankLadder,
    BankExit,
    PrayerDoor,
    PrayerExit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Deny(&'static str),
    /// Call TutorialProgression.advance, preserving completed/no-rewind rules.
    Advance(Step),
    Teleport(CoordGrid),
    Telejump(CoordGrid),
}

impl Gate {
    pub fn decide(
        self,
        step: Step,
        completed: i32,
        source: CoordGrid,
        loc: Location,
    ) -> Result<Vec<Effect>, CoordError> {
        use Gate::*;

        let required = |minimum| completed == 1 || step >= minimum;
        let deny = |message| Ok(vec![Effect::Deny(message)]);
        let entering = source == loc.coords;
        let through = || -> Result<Effect, CoordError> {
            let target = if entering {
                let (dx, dz) = match loc.angle {
                    0 => (-1, 0),
                    1 => (0, 1),
                    2 => (1, 0),
                    _ => (0, -1),
                };
                loc.coords.translate(dx, dz, 0)?
            } else {
                loc.coords
            };
            Ok(Effect::Teleport(target))
        };
        // Directional gates intentionally differ from requireAtLeast: the rat
        // cage and outward bank checks inspect progress even on completed saves.
        if self == CombatCage {
            if entering && step < Step::CombatMelee {
                return deny("Oi! Get away from there. Only enter the rat cage when I say so.");
            }
            if entering && step >= Step::CombatRange && step < Step::CombatLadder {
                return deny("No, don't enter the pit. Range the rats from outside the cage.");
            }
            return Ok(vec![through()?]);
        }
        if self == BankExit {
            if entering {
                if !required(Step::BankOpen) {
                    return deny("You need to finish Brother Brace's tasks before you are allowed to proceed through this door.");
                }
                return Ok(vec![through()?]);
            }
            if step < Step::BankOpen {
                return deny("You need to open your bank first.");
            }
            if step < Step::PollView {
                return deny(
                    "You need to visit the poll booth before you can proceed through this door.",
                );
            }
            if step < Step::AccountExit {
                return deny("You need to talk to the Account Guide before you are allowed to proceed through this door.");
            }
            return Ok(vec![Effect::Advance(Step::Ironman), through()?]);
        }
        if self == PrayerExit && entering {
            if !required(Step::PrayerTalk) {
                return deny("You need to finish with the Combat Instructor before you are allowed to proceed through this door.");
            }
            return Ok(vec![through()?]);
        }
        let (minimum, next, message) = match self {
            SurvivalEntry => (Some(Step::GielinorDoor), Some(Step::SurvivalTalk), "You need to talk to the Gielinor Guide before you are allowed to proceed through this door."),
            SurvivalExit => (Some(Step::SurvivalGate), Some(Step::ChefDoor), "You need to talk to the Survival Guide and complete her tasks before you are allowed to proceed through this gate."),
            ChefEntry => (Some(Step::ChefDoor), Some(Step::ChefTalk), "You need to finish the Survival Expert's tasks first."),
            ChefExit => (Some(Step::ChefExit), Some(Step::QuestDoor), "You need to finish the Master Chef's tasks first."),
            QuestEntry => (Some(Step::QuestDoor), Some(Step::QuestTalk), "You need to finish the Master Chef's tasks first."),
            QuestLadder => (Some(Step::QuestLadder), Some(Step::MineTalk), "I don't think you're ready to go down there yet."),
            MiningLadderUp => (Some(Step::MineTalk), None, "You need to finish with Mining and Smithing first."),
            MiningExit => (Some(Step::MineGate), Some(Step::CombatTalk), "You need to finish with Mining and Smithing first."),
            CombatLadder => (Some(Step::CombatLadder), Some(Step::PrayerTalk), "You're not ready to continue yet. You need to know about combat before you go on."),
            BankLadder => (None, None, ""),
            PrayerDoor => (Some(Step::CombatLadder), Some(Step::PrayerTalk), "You need to finish with the Combat Instructor before you are allowed to proceed through this door."),
            PrayerExit => (Some(Step::PrayerExit), Some(Step::BankOpen), "You need to finish Brother Brace's tasks before you are allowed to proceed through this door."),
            CombatCage | BankExit => unreachable!(),
        };
        if minimum.is_some_and(|minimum| !required(minimum)) {
            return deny(message);
        }
        let mut effects = Vec::new();
        if let Some(next) = next {
            effects.push(Effect::Advance(next));
        }
        effects.push(match self {
            QuestLadder => Effect::Telejump(CoordGrid::new(1680, 12527, 0)?),
            MiningLadderUp => Effect::Telejump(CoordGrid::new(1680, 6127, 0)?),
            CombatLadder => Effect::Telejump(CoordGrid::new(1703, 6134, 0)?),
            BankLadder => Effect::Telejump(CoordGrid::new(1703, 12534, 0)?),
            _ => through()?,
        });
        Ok(effects)
    }
}
