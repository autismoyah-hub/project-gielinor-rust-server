//! TutorialSkillsScript.cookShrimp. Missing ingredients and early steps are silent.
use crate::{
    inventory::{Catalog, Delete, Insert},
    simulation::PlayerData,
    skills,
    tutorial_progression::{Config as Progression, Effect},
    tutorial_steps::Step,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub location: u32,
    pub raw: i32,
    pub cooked: i32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub global_xp_rate: f64,
}
pub struct Cooked {
    pub advancement: skills::Advancement,
    pub progression: Vec<Effect>,
}
impl Config {
    pub fn cook(
        &self,
        p: &mut PlayerData,
        catalog: &Catalog,
        logged: bool,
    ) -> Result<Option<Cooked>, crate::stats::InvalidXp> {
        if !self
            .progression
            .require_at_least(&p.variables, Step::SurvivalCook)
            || !p
                .inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == self.raw && i.count > 0)
        {
            return Ok(None);
        }
        // Kotlin ignores each strict transaction result independently.
        let _ = p.inventory.delete(catalog, Delete::new(self.raw, 1));
        let _ = p.inventory.insert(catalog, Insert::new(self.cooked, 1));
        let advancement = skills::advance(
            &mut p.stats,
            self.skill,
            30.0,
            p.xp_rate_hundreds as f64 / 100.0 * self.global_xp_rate,
            self.max_level,
        )?;
        let progression =
            self.progression
                .advance(&mut p.variables, Step::SurvivalGate, None, logged);
        Ok(Some(Cooked {
            advancement,
            progression,
        }))
    }
}
