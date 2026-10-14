//! TutorialSkillsScript.cookBread, including its asymmetric early-progress text.
use crate::{
    inventory::{Catalog, Delete, Insert},
    simulation::PlayerData,
    skills,
    tutorial_progression::{Config as Progression, Effect as Progress},
    tutorial_steps::Step,
};

pub const MISSING: &str =
    "You haven't got anything suitable for cooking! The master chef can help you out with that.";
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub location: u32,
    pub dough: i32,
    pub bread: i32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub global_xp_rate: f64,
}
#[derive(Debug)]
pub enum Outcome {
    Silent,
    Missing,
    Cooked {
        advancement: skills::Advancement,
        progression: Vec<Progress>,
    },
}
impl Config {
    pub fn cook(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        logged: bool,
    ) -> Result<Outcome, crate::stats::InvalidXp> {
        let vars = &player.variables;
        if !self.progression.require_at_least(vars, Step::ChefBread) {
            return Ok(if self.progression.current(vars) < Step::ChefDough {
                Outcome::Missing
            } else {
                Outcome::Silent
            });
        }
        if !player
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| i.id == self.dough && i.count > 0)
        {
            return Ok(Outcome::Missing);
        }
        let _ = player.inventory.delete(catalog, Delete::new(self.dough, 1));
        let _ = player.inventory.insert(catalog, Insert::new(self.bread, 1));
        let advancement = skills::advance(
            &mut player.stats,
            self.skill,
            40.0,
            player.xp_rate_hundreds as f64 / 100.0 * self.global_xp_rate,
            self.max_level,
        )?;
        let progression =
            self.progression
                .advance(&mut player.variables, Step::ChefExit, None, logged);
        Ok(Outcome::Cooked {
            advancement,
            progression,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Inventory,
        stats::Stats,
        variables::{Transmit, Variables, Varp},
        CoordGrid,
    };
    fn fixture(step: i32, completed: i32, held: bool) -> (Config, PlayerData, Catalog) {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let config = Config {
            progression: Progression {
                step: varp(1),
                completed: varp(2),
            },
            location: 1,
            dough: 10,
            bread: 20,
            animation: 897,
            skill: 7,
            max_level: 99,
            global_xp_rate: 1.0,
        };
        let mut p = PlayerData::new(
            1,
            "Cook".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.variables = Variables::restore([(1, step), (2, completed)].into());
        p.inventory = Inventory::new(1, crate::inventory::StackType::Normal, false);
        let catalog = Catalog::default();
        if held {
            p.inventory.insert(&catalog, Insert::new(10, 1)).unwrap();
        }
        (config, p, catalog)
    }
    #[test]
    fn dough_step_is_silent_even_if_dough_already_exists() {
        for held in [false, true] {
            for step in [0, 15, 16, 17] {
                let (c, mut p, cat) = fixture(step, 0, held);
                let outcome = c.cook(&mut p, &cat, true).unwrap();
                match (step, held, outcome) {
                    (16, _, Outcome::Silent)
                    | (0 | 15, _, Outcome::Missing)
                    | (17, false, Outcome::Missing) => {}
                    (17, true, Outcome::Cooked { .. }) => {}
                    other => panic!("Unexpected {other:?}"),
                }
            }
        }
    }
    #[test]
    fn completion_bypass_xp_rate_and_refresh_preserve_reference_behavior() {
        for (step, completed, expected) in [(0, 1, 0), (17, 0, 18), (30, 0, 30)] {
            let (mut c, mut p, cat) = fixture(step, completed, true);
            c.global_xp_rate = 2.0;
            p.xp_rate_hundreds = 150;
            let Outcome::Cooked {
                advancement,
                progression,
            } = c.cook(&mut p, &cat, true).unwrap()
            else {
                panic!()
            };
            assert_eq!(p.inventory.get(0).unwrap().id, 20);
            assert_eq!(p.stats.xp(7), 120);
            assert_eq!((advancement.old_level, advancement.new_level), (1, 2));
            assert_eq!(p.variables.get(1), expected);
            assert_eq!(progression.is_empty(), completed == 1);
        }
    }
}
