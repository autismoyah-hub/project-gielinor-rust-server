//! TutorialSkillsScript.fish. The reference does not require a net here, and
//! permits a full inventory containing shrimp even if the strict add fails.
use crate::{
    inventory::{Catalog, Insert},
    simulation::PlayerData,
    skills,
    tutorial_progression::{Config as Progression, Effect as Progress},
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub shrimp: i32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub global_xp_rate: f64,
}
#[derive(Debug)]
pub enum Outcome {
    Denied(&'static str),
    Caught {
        advancement: skills::Advancement,
        progression: Vec<Progress>,
    },
}
impl Config {
    pub fn fish(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        logged: bool,
    ) -> Result<Outcome, crate::stats::InvalidXp> {
        if !self
            .progression
            .require_at_least(&player.variables, Step::SurvivalInv)
        {
            return Ok(Outcome::Denied(
                "You cannot fish here yet. You must progress further in the tutorial.",
            ));
        }
        let held = player
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| i.id == self.shrimp && i.count > 0);
        if player.inventory.free_space() == 0 && !held {
            return Ok(Outcome::Denied(
                "You don't have enough inventory space to hold that item.",
            ));
        }
        let _ = player
            .inventory
            .insert(catalog, Insert::new(self.shrimp, 1));
        let advancement = skills::advance(
            &mut player.stats,
            self.skill,
            10.0,
            player.xp_rate_hundreds as f64 / 100.0 * self.global_xp_rate,
            self.max_level,
        )?;
        // Unlike advance, setStep also runs for a completed player at an early
        // stored step. Later progress receives no focus refresh at all.
        let progression = if self.progression.current(&player.variables) <= Step::SurvivalFish {
            self.progression
                .set(&mut player.variables, Step::SurvivalSkills, None, logged)
        } else {
            vec![]
        };
        Ok(Outcome::Caught {
            advancement,
            progression,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Inventory, Item, StackType},
        stats::Stats,
        variables::{Transmit, Variables, Varp},
        CoordGrid,
    };
    #[test]
    fn progress_completion_and_full_inventory_follow_original_exceptions() {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let c = Config {
            progression: Progression {
                step: varp(1),
                completed: varp(2),
            },
            shrimp: 10,
            animation: 621,
            skill: 10,
            max_level: 99,
            global_xp_rate: 1.0,
        };
        for (step, completed) in [(5, 0), (5, 1), (5, 2), (6, 0), (7, 0), (8, 0), (20, 0)] {
            for held in [-1, 10, 20] {
                let mut p = PlayerData::new(
                    1,
                    "Fisher".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                );
                p.inventory = Inventory::new(1, StackType::Normal, false);
                if held >= 0 {
                    p.inventory
                        .restore_slot(
                            0,
                            Some(Item {
                                id: held,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                p.variables = Variables::restore([(1, step), (2, completed)].into());
                let result = c.fish(&mut p, &Catalog::default(), true).unwrap();
                let permitted = (completed == 1 || step >= 6) && held != 20;
                assert_eq!(
                    matches!(result, Outcome::Caught { .. }),
                    permitted,
                    "{step}/{completed}/{held}"
                );
                assert_eq!(p.stats.xp(10), if permitted { 10 } else { 0 });
                assert_eq!(
                    p.variables.get(1),
                    if permitted && step <= 7 { 8 } else { step }
                );
                if let Outcome::Caught { progression, .. } = result {
                    assert_eq!(p.inventory.get(0).unwrap().id, 10);
                    assert_eq!(progression.is_empty(), step > 7);
                }
            }
        }
    }
}
