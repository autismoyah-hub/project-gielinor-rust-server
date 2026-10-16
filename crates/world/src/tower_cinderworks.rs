//! Resolved Floor One content identities; all player operations use world actions.
use crate::{simulation::PlayerData, tower_run::Service, CoordGrid};
#[derive(Clone, Debug)]
pub struct Config {
    pub origin: CoordGrid,
    pub outside: CoordGrid,
    pub plaque: u32,
    pub portal: u32,
    pub chest: u32,
    pub tools: Vec<i32>,
    pub scimitar: i32,
    pub dagger: i32,
    pub coins: i32,
    pub beer: i32,
    pub imp: u16,
    pub warden: u16,
    pub quench: u32,
    pub quench_tiles: Vec<CoordGrid>,
    pub warning_graphic: u16,
    pub strike_graphic: u16,
    pub work: Option<WorkConfig>,
    pub death_animation: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct WorkConfig {
    pub fishing_spot: u16,
    pub net: i32,
    pub raw: i32,
    pub cooked: i32,
    pub cooking_loc: u32,
    pub fish_skill: u8,
    pub cook_skill: u8,
    pub fish_animation: u16,
    pub cook_animation: u16,
    pub queue: i32,
}
impl Config {
    pub fn forged_weapon(&self, id: i32) -> bool {
        id == self.dagger || id == self.scimitar
    }
    pub fn handles(&self, id: u32, operation: u8) -> bool {
        operation == 1 && [self.plaque, self.portal, self.chest, self.quench].contains(&id)
    }
    pub fn credit(
        &self,
        data: &mut PlayerData,
        coords: CoordGrid,
        service: Service,
    ) -> Option<&'static str> {
        if !crate::tower_layout::contains(self.origin, coords) || !data.tower_run.service(service) {
            return None;
        }
        Some(match service {
            Service::Forge => "The workshop is restored. Your forged bronze weapon can pierce the foundry guardians.",
            Service::Kitchen => "The kitchen is restored. Keep cooked food for the vent gallery and foundry.",
            Service::Altar => "The altar is restored. Read the plaque to check the sanctuary services.",
        })
    }
}
