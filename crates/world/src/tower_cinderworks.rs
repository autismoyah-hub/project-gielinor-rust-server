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
}
impl Config {
    pub fn forged_weapon(&self, id: i32) -> bool { id == self.dagger || id == self.scimitar }
    pub fn handles(&self, id: u32, operation: u8) -> bool {
        operation == 1 && [self.plaque, self.portal, self.chest].contains(&id)
    }
    pub fn credit(&self, data: &mut PlayerData, coords: CoordGrid, service: Service) -> Option<&'static str> {
        if !crate::tower_layout::contains(self.origin, coords) || !data.tower_run.service(service) { return None }
        Some(match service {
            Service::Forge => "The workshop is restored. Your forged bronze weapon can pierce the foundry guardians.",
            Service::Kitchen => "The kitchen is restored. Keep cooked food for the vent gallery and foundry.",
            Service::Altar => "The altar is restored. Read the plaque to check the sanctuary services.",
        })
    }
}
