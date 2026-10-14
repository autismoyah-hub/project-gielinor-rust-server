//! TutorialProgression's durable transitions. Focus effects require a runtime
//! adapter before this can drive the tutorial UI or gameplay scripts.
use crate::{
    simulation::NpcId,
    tutorial_steps::Step,
    variables::{Update, Variables, Varp},
};
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Variable(Update),
    Focus { step: Step, npc: Option<NpcId> },
    End,
}
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub step: Varp,
    pub completed: Varp,
}
impl Config {
    pub fn current(&self, vars: &Variables) -> Step {
        Step::from_var(vars.get(self.step.id))
    }
    pub fn active(&self, vars: &Variables) -> bool {
        vars.get(self.completed.id) != 1
    }
    pub fn require_at_least(&self, vars: &Variables, step: Step) -> bool {
        !self.active(vars) || self.current(vars) >= step
    }
    pub fn set(
        &self,
        vars: &mut Variables,
        step: Step,
        npc: Option<NpcId>,
        logged: bool,
    ) -> Vec<Effect> {
        let mut effects: Vec<_> = vars
            .set(self.step, step as i32, logged)
            .into_iter()
            .map(Effect::Variable)
            .collect();
        if step == Step::Completed {
            effects.extend(
                vars.set(self.completed, 1, logged)
                    .into_iter()
                    .map(Effect::Variable),
            );
            effects.push(Effect::End);
        } else {
            effects.push(Effect::Focus { step, npc });
        }
        effects
    }
    pub fn advance(
        &self,
        vars: &mut Variables,
        to: Step,
        npc: Option<NpcId>,
        logged: bool,
    ) -> Vec<Effect> {
        if !self.active(vars) {
            return vec![];
        }
        let current = self.current(vars);
        if current >= to {
            return vec![Effect::Focus { step: current, npc }];
        }
        self.set(vars, to, npc, logged)
    }
    pub fn resume(&self, vars: &Variables) -> Vec<Effect> {
        if self.active(vars) {
            vec![Effect::Focus {
                step: self.current(vars),
                npc: None,
            }]
        } else {
            vec![]
        }
    }
    pub fn complete_quest(&self, vars: &mut Variables, logged: bool) -> Vec<Effect> {
        if self.current(vars) < Step::LeaveTalk {
            self.set(vars, Step::LeaveTalk, None, logged)
        } else {
            vec![]
        }
    }
    pub fn finish_and_leave(&self, vars: &mut Variables, logged: bool) -> Vec<Effect> {
        self.set(vars, Step::Completed, None, logged)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::Transmit;
    #[test]
    fn refresh_does_not_rewind_and_quest_completion_differs_from_leaving() {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let config = Config {
            step: varp(1),
            completed: varp(2),
        };
        let mut vars = Variables::default();
        vars.set(config.step, 999, false);
        assert_eq!(config.current(&vars), Step::CharCreate);
        assert_eq!(vars.get(1), 999);
        config.advance(&mut vars, Step::MagicCast, None, true);
        assert_eq!(
            config.advance(&mut vars, Step::MineTalk, None, true),
            vec![Effect::Focus {
                step: Step::MagicCast,
                npc: None
            }]
        );
        config.complete_quest(&mut vars, true);
        assert_eq!(config.current(&vars), Step::LeaveTalk);
        assert!(config.active(&vars));
        assert!(config.complete_quest(&mut vars, true).is_empty());
        assert_eq!(
            config.finish_and_leave(&mut vars, true),
            vec![
                Effect::Variable(Update { id: 1, value: 53 }),
                Effect::Variable(Update { id: 2, value: 1 }),
                Effect::End
            ]
        );
        assert!(config.resume(&vars).is_empty());
        assert!(config
            .advance(&mut vars, Step::CharCreate, None, true)
            .is_empty());
        assert!(config.require_at_least(&vars, Step::Completed));
        // Kotlin boolVarp recognizes exactly1, not arbitrary nonzero values.
        vars.set(config.completed, 2, false);
        assert!(config.active(&vars));
    }
}
