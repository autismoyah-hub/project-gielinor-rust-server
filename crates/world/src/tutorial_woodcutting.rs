//! TutorialSkillsScript.chop and oakDeny; these intentionally do not validate
//! an axe, and the upper progress bound also applies to completed characters.
use crate::{
    inventory::{Catalog, Insert},
    simulation::PlayerData,
    skills,
    tutorial_progression::{Config as Progression, Effect as Progress},
    tutorial_steps::Step,
};
pub const OAK:&str="You won't be able to chop oak trees until you have a Woodcutting level of 15. You'll advance to higher Woodcutting levels by chopping down normal trees.";
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub trees: [u32; 2],
    pub oak: u32,
    pub logs: i32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub global_xp_rate: f64,
}
#[derive(Debug)]
pub enum Outcome {
    Denied(&'static str),
    Chopped {
        advancement: skills::Advancement,
        progression: Vec<Progress>,
    },
}
impl Config {
    pub fn contains(&self, id: u32) -> bool {
        self.trees.contains(&id) || self.oak == id
    }
    pub fn chop(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        logged: bool,
    ) -> Result<Outcome, crate::stats::InvalidXp> {
        if !self
            .progression
            .require_at_least(&player.variables, Step::SurvivalWc)
        {
            return Ok(Outcome::Denied(
                "You cannot cut down this tree yet. You must progress further in the tutorial.",
            ));
        }
        if self.progression.current(&player.variables) > Step::SurvivalFm {
            return Ok(Outcome::Denied(
                "Perhaps you've done enough woodcutting now.",
            ));
        }
        if player.inventory.free_space() == 0 {
            return Ok(Outcome::Denied(
                "You don't have enough inventory space to hold that item.",
            ));
        }
        let _ = player.inventory.insert(catalog, Insert::new(self.logs, 1));
        let advancement = skills::advance(
            &mut player.stats,
            self.skill,
            25.0,
            player.xp_rate_hundreds as f64 / 100.0 * self.global_xp_rate,
            self.max_level,
        )?;
        let progression =
            self.progression
                .advance(&mut player.variables, Step::SurvivalFm, None, logged);
        Ok(Outcome::Chopped {
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
    fn lower_gate_upper_gate_and_capacity_have_distinct_precedence() {
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
            trees: [1, 2],
            oak: 3,
            logs: 10,
            animation: 879,
            skill: 8,
            max_level: 99,
            global_xp_rate: 1.0,
        };
        for step in [0, 9, 10, 11, 12, 53] {
            for completed in [0, 1, 2] {
                for full in [false, true] {
                    let mut p = PlayerData::new(
                        1,
                        "Cutter".into(),
                        CoordGrid::new(3200, 3200, 0).unwrap(),
                        Stats::default(),
                    );
                    p.variables = Variables::restore([(1, step), (2, completed)].into());
                    p.inventory = Inventory::new(1, StackType::Normal, false);
                    if full {
                        p.inventory
                            .restore_slot(
                                0,
                                Some(Item {
                                    id: 10,
                                    count: 1,
                                    vars: 0,
                                }),
                            )
                            .unwrap();
                    }
                    let result = c.chop(&mut p, &Catalog::default(), true).unwrap();
                    let allowed = (completed == 1 || step >= 10) && step <= 11 && !full;
                    assert_eq!(
                        matches!(result, Outcome::Chopped { .. }),
                        allowed,
                        "{step}/{completed}/{full}"
                    );
                    assert_eq!(p.stats.xp(8), if allowed { 25 } else { 0 });
                    assert_eq!(
                        p.variables.get(1),
                        if allowed && completed != 1 { 11 } else { step }
                    );
                }
            }
        }
        assert!(c.contains(1) && c.contains(2) && c.contains(3));
        assert!(!c.contains(4));
    }
}
