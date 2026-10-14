//! TutorialSkillsScript.lightFire has one suspension after log deletion and
//! animation. Coordinates are captured before the four-cycle delay.
use crate::{
    inventory::{Catalog, Delete},
    simulation::PlayerData,
    tutorial_progression::Config as Progression,
    tutorial_steps::Step,
    CoordGrid,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub logs: i32,
    pub location: u32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub global_xp_rate: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub config: Config,
    pub coords: CoordGrid,
    pub main_modal: Option<u16>,
}
impl Config {
    pub fn begin(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        coords: CoordGrid,
        main_modal: Option<u16>,
    ) -> Option<Pending> {
        if !self
            .progression
            .require_at_least(&player.variables, Step::SurvivalFm)
            || !player
                .inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == self.logs && i.count > 0)
        {
            return None;
        }
        let _ = player.inventory.delete(catalog, Delete::new(self.logs, 1));
        Some(Pending {
            config: *self,
            coords,
            main_modal,
        })
    }
}
