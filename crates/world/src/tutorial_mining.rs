//! TutorialSkillsScript.mineTin/mineCopper. Requires a held bronze pickaxe;
//! equipped tools do not satisfy this script's inventory-only check.
use crate::{
    inventory::{Catalog, Insert},
    simulation::PlayerData,
    skills,
    tutorial_progression::{Config as Progression, Effect},
    tutorial_steps::Step,
};
pub const NOT_READY: &str =
    "You are not ready to mine yet. Please follow the tutorial and you'll be mining in no time.";
pub const FULL: &str = "You don't have enough room in your inventory to hold the ore. You'll need to drop something. To do so, right-click on an item you don't want and select drop. Once you've done that, try mining the rock again.";
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub tin_rock: u32,
    pub copper_rock: u32,
    pub tin: i32,
    pub copper: i32,
    pub pickaxe: i32,
    pub animation: u16,
    pub skill: u8,
    pub max_level: u8,
    pub global_xp_rate: f64,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        stats::Stats,
        variables::{Transmit, Varp},
        CoordGrid,
    };
    #[test]
    fn inventory_pickaxe_two_ores_and_fractional_xp_follow_script_order() {
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
            tin_rock: 1,
            copper_rock: 2,
            tin: 438,
            copper: 436,
            pickaxe: 1265,
            animation: 625,
            skill: 14,
            max_level: 99,
            global_xp_rate: 1.0,
        };
        let mut p = PlayerData::new(
            1,
            "Miner".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.variables.set(varp(1), 24, false);
        let pick = Item {
            id: 1265,
            count: 1,
            vars: 0,
        };
        p.equipment.restore_slot(3, Some(pick)).unwrap();
        assert!(matches!(
            c.mine(&mut p, &Catalog::default(), Ore::Tin, true).unwrap(),
            Outcome::Denied(NOT_READY)
        ));
        p.inventory.restore_slot(0, Some(pick)).unwrap();
        let Outcome::Mined { progression, .. } =
            c.mine(&mut p, &Catalog::default(), Ore::Tin, true).unwrap()
        else {
            panic!()
        };
        assert!(progression.is_empty());
        assert_eq!(p.variables.get(1), 24);
        assert_eq!(p.stats.fine_xp(14), 175);
        let Outcome::Mined { progression, .. } = c
            .mine(&mut p, &Catalog::default(), Ore::Copper, true)
            .unwrap()
        else {
            panic!()
        };
        assert!(!progression.is_empty());
        assert_eq!(p.variables.get(1), 25);
        assert_eq!(p.stats.xp(14), 35);
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Ore {
    Tin,
    Copper,
}
pub enum Outcome {
    Denied(&'static str),
    Mined {
        advancement: skills::Advancement,
        progression: Vec<Effect>,
        message: &'static str,
    },
}
impl Config {
    pub fn ore(&self, location: u32) -> Option<Ore> {
        if location == self.tin_rock {
            Some(Ore::Tin)
        } else if location == self.copper_rock {
            Some(Ore::Copper)
        } else {
            None
        }
    }
    pub fn mine(
        &self,
        p: &mut PlayerData,
        catalog: &Catalog,
        ore: Ore,
        logged: bool,
    ) -> Result<Outcome, crate::stats::InvalidXp> {
        let has = |p: &PlayerData, id| {
            p.inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == id && i.count > 0)
        };
        if !self
            .progression
            .require_at_least(&p.variables, Step::MineOres)
            || !has(p, self.pickaxe)
        {
            return Ok(Outcome::Denied(NOT_READY));
        }
        if p.inventory.free_space() == 0 {
            return Ok(Outcome::Denied(FULL));
        }
        let (item, message) = match ore {
            Ore::Tin => (self.tin, "You manage to mine some tin."),
            Ore::Copper => (self.copper, "You manage to mine some copper."),
        };
        let _ = p.inventory.insert(catalog, Insert::new(item, 1));
        let advancement = skills::advance(
            &mut p.stats,
            self.skill,
            17.5,
            p.xp_rate_hundreds as f64 / 100.0 * self.global_xp_rate,
            self.max_level,
        )?;
        let progression = if has(p, self.tin) && has(p, self.copper) {
            self.progression
                .advance(&mut p.variables, Step::MineSmelt, None, logged)
        } else {
            vec![]
        };
        Ok(Outcome::Mined {
            advancement,
            progression,
            message,
        })
    }
}
