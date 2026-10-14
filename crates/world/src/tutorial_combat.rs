//! TutorialCombatScript death-queue effects; attacks themselves never grant progress.
use crate::{
    tutorial_progression::{self, Config as Progression},
    tutorial_steps::Step,
    variables::Variables,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub rat: u16,
    pub chicken: u16,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Message(&'static str),
    Progress(Vec<tutorial_progression::Effect>),
}
impl Config {
    /// Called exactly once when the NPC death queue starts, for the top hero if online.
    pub fn death(self, npc: u16, vars: &mut Variables, logged: bool) -> Vec<Effect> {
        let step = self.progression.current(vars);
        let mut out = vec![];
        if npc == self.rat {
            let next = match step {
                Step::CombatMelee => Some(Step::CombatRange),
                Step::CombatRange => Some(Step::CombatLadder),
                _ => None,
            };
            if let Some(next) = next {
                out.push(Effect::Message("You have defeated the giant rat!"));
                let effects = self.progression.advance(vars, next, None, logged);
                if !effects.is_empty() {
                    out.push(Effect::Progress(effects));
                }
            }
        } else if npc == self.chicken && (Step::MagicRunes..Step::LeaveTalk).contains(&step) {
            out.push(Effect::Message(
                "The chicken is defeated by your Wind Strike!",
            ));
            out.push(Effect::Message(
                "Congratulations, you've completed a quest: Learning the Ropes",
            ));
            // Kotlin calls completeQuest followed by setStep: both focus callbacks
            // occur, although only the first can transmit a changed step variable.
            let first = self.progression.complete_quest(vars, logged);
            if !first.is_empty() {
                out.push(Effect::Progress(first));
            }
            out.push(Effect::Progress(self.progression.set(
                vars,
                Step::LeaveTalk,
                None,
                logged,
            )));
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::{Transmit, Varp};
    fn config() -> Config {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        Config {
            progression: Progression {
                step: varp(1),
                completed: varp(2),
            },
            rat: 1,
            chicken: 2,
        }
    }
    #[test]
    fn rat_steps_are_exact_and_completed_flag_suppresses_advance_but_not_message() {
        let c = config();
        for raw in -1..=54 {
            for completed in [0, 1, 2] {
                let mut vars = Variables::restore([(1, raw), (2, completed)].into());
                let effects = c.death(1, &mut vars, true);
                let eligible = raw == Step::CombatMelee as i32 || raw == Step::CombatRange as i32;
                assert_eq!(
                    effects
                        .iter()
                        .filter(|e| matches!(e, Effect::Message(_)))
                        .count(),
                    usize::from(eligible)
                );
                assert_eq!(
                    vars.get(1),
                    if eligible && completed != 1 {
                        raw + 1
                    } else {
                        raw
                    }
                );
            }
        }
    }
    #[test]
    fn chicken_completes_quest_without_finishing_tutorial_and_keeps_two_focus_calls() {
        let c = config();
        for raw in -1..=54 {
            for completed in [0, 1, 2] {
                let mut vars = Variables::restore([(1, raw), (2, completed)].into());
                let effects = c.death(2, &mut vars, true);
                let eligible = (Step::MagicRunes as i32..Step::LeaveTalk as i32).contains(&raw);
                assert_eq!(
                    vars.get(1),
                    if eligible {
                        Step::LeaveTalk as i32
                    } else {
                        raw
                    }
                );
                assert_eq!(vars.get(2), completed);
                assert_eq!(
                    effects
                        .iter()
                        .filter(|e| matches!(e, Effect::Message(_)))
                        .count(),
                    if eligible { 2 } else { 0 }
                );
                let progress: Vec<_> = effects
                    .iter()
                    .filter_map(|e| {
                        if let Effect::Progress(p) = e {
                            Some(p)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect();
                assert_eq!(
                    progress
                        .iter()
                        .filter(|e| matches!(
                            e,
                            tutorial_progression::Effect::Focus {
                                step: Step::LeaveTalk,
                                npc: None
                            }
                        ))
                        .count(),
                    if eligible { 2 } else { 0 }
                );
                assert!(!progress
                    .iter()
                    .any(|e| matches!(e, tutorial_progression::Effect::End)));
            }
        }
    }
}
