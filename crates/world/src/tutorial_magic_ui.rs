//! TutorialMagicUiScript decisions. Home Teleport must enter protected access first;
//! selecting Wind Strike is an unprotected overlay callback in the reference.
use crate::tutorial_steps::Step;
pub const WIND_MESSAGE: &str = "Click a chicken to cast Wind Strike.";
pub const EARLY_HOME: &str =
    "You can't cast Lumbridge Home Teleport until you've finished talking to the Magic Instructor.";
pub const SPEAK_FIRST: &str = "Speak to the Magic Instructor before leaving the island.";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Home {
    None,
    Deny(&'static str),
    Leave,
}
pub fn home(step: Step, completed: i32) -> Home {
    if completed == 1 {
        Home::None
    } else if step < Step::LeaveTalk {
        Home::Deny(EARLY_HOME)
    } else if matches!(step, Step::LeaveTalk | Step::HomeTele) {
        Home::Leave
    } else {
        Home::Deny(SPEAK_FIRST)
    }
}
pub fn wind(step: Step, completed: i32) -> bool {
    completed != 1 && matches!(step, Step::MagicRunes | Step::MagicCast)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completed_boolean_is_exact_and_spells_keep_distinct_step_gates() {
        for raw in -1..=54 {
            let step = Step::from_var(raw);
            for done in [0, 1, 2] {
                assert_eq!(wind(step, done), done != 1 && (raw == 49 || raw == 50));
                let expected = if done == 1 {
                    Home::None
                } else if matches!(raw, 51 | 52) {
                    Home::Leave
                } else if raw == 53 {
                    Home::Deny(SPEAK_FIRST)
                } else {
                    Home::Deny(EARLY_HOME)
                };
                assert_eq!(home(step, done), expected, "raw={raw} done={done}");
            }
        }
    }
}
