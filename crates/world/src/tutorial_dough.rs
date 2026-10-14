//! TutorialSkillsScript.mixDough: sequential strict inventory transactions.
//! Individual failures do not roll back prior operations or prevent progression.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    tutorial_progression::{Config as Progression, Effect as Progress},
    tutorial_steps::Step,
    variables::Variables,
};

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub progression: Progression,
    pub flour: i32,
    pub water: i32,
    pub pot: i32,
    pub bucket: i32,
    pub dough: i32,
}
pub struct Outcome {
    pub message: &'static str,
    pub progression: Vec<Progress>,
}
impl Config {
    pub fn mix(
        &self,
        inventory: &mut Inventory,
        catalog: &Catalog,
        vars: &mut Variables,
        logged: bool,
    ) -> Option<Outcome> {
        if !self.progression.require_at_least(vars, Step::ChefDough) {
            return None;
        }
        let contains = |id| {
            inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == id && i.count > 0)
        };
        if !contains(self.flour) || !contains(self.water) {
            return None;
        }
        for id in [self.flour, self.water] {
            let _ = inventory.delete(catalog, Delete::new(id, 1));
        }
        for id in [self.pot, self.bucket, self.dough] {
            let _ = inventory.insert(catalog, Insert::new(id, 1));
        }
        Some(Outcome {
            message: "You make some dough.",
            progression: self
                .progression
                .advance(vars, Step::ChefBread, None, logged),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::StackType,
        variables::{Transmit, Varp},
    };
    fn fixture(size: usize, step: i32, completed: i32) -> (Config, Inventory, Catalog, Variables) {
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
            flour: 1933,
            water: 1929,
            pot: 1931,
            bucket: 1925,
            dough: 2307,
        };
        let mut vars = Variables::default();
        vars.set(config.progression.step, step, true);
        vars.set(config.progression.completed, completed, true);
        let mut inv = Inventory::new(size, StackType::Normal, false);
        let catalog = Catalog::default();
        inv.insert(&catalog, Insert::new(config.flour, 1)).unwrap();
        inv.insert(&catalog, Insert::new(config.water, 1)).unwrap();
        (config, inv, catalog, vars)
    }
    #[test]
    fn separate_commits_preserve_empty_containers_when_dough_cannot_fit() {
        for capacity in [2, 3] {
            let (config, mut inv, catalog, mut vars) = fixture(capacity, 16, 0);
            let out = config.mix(&mut inv, &catalog, &mut vars, true).unwrap();
            assert_eq!(out.message, "You make some dough.");
            assert_eq!(vars.get(1), 17);
            assert_eq!(inv.get(0).unwrap().id, config.pot);
            assert_eq!(inv.get(1).unwrap().id, config.bucket);
            assert_eq!(
                inv.get(2).map(|i| i.id),
                (capacity == 3).then_some(config.dough)
            );
            assert!(matches!(
                out.progression.last(),
                Some(Progress::Focus {
                    step: Step::ChefBread,
                    ..
                })
            ));
        }
    }
    #[test]
    fn progress_gate_completed_bypass_and_same_step_refresh() {
        for (step, completed, permitted, resulting) in [
            (15, 0, false, 15),
            (15, 2, false, 15),
            (15, 1, true, 15),
            (16, 0, true, 17),
            (30, 0, true, 30),
        ] {
            let (config, mut inv, catalog, mut vars) = fixture(3, step, completed);
            let before = inv.clone();
            let out = config.mix(&mut inv, &catalog, &mut vars, true);
            assert_eq!(out.is_some(), permitted);
            assert_eq!(vars.get(1), resulting);
            if !permitted {
                assert_eq!(inv, before);
            }
            if completed == 1 {
                assert!(out.unwrap().progression.is_empty());
            }
        }
        let (config, mut inv, catalog, mut vars) = fixture(3, 16, 0);
        inv.delete(&catalog, Delete::new(config.water, 1)).unwrap();
        let before = inv.clone();
        assert!(config.mix(&mut inv, &catalog, &mut vars, true).is_none());
        assert_eq!(inv, before);
        assert_eq!(vars.get(1), 16);
    }
}
