//! Wiki-defined ordinary vessel filling; source identities are supplied by the host.
use crate::inventory::{Catalog, Delete, Insert, Inventory};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Sink,
    Well,
}
#[derive(Clone, Copy, Debug)]
pub struct Vessel {
    pub output: i32,
    pub well: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Config {
    pub sources: BTreeMap<u32, Source>,
    pub vessels: BTreeMap<i32, Vessel>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    StaleSlot,
    Inventory(crate::inventory::Error),
}
impl Config {
    pub fn output(&self, source: u32, input: i32) -> Option<i32> {
        let source = self.sources.get(&source)?;
        let vessel = self.vessels.get(&input)?;
        if *source == Source::Well && !vessel.well {
            return None;
        }
        (vessel.output != input).then_some(vessel.output)
    }

    /// One selected vessel, even when the inventory is full. No matching-slot fallback.
    /// World admission, timing and source-generation validation belong to the caller.
    pub fn fill(
        &self,
        source: u32,
        input: i32,
        slot: usize,
        inventory: &mut Inventory,
        catalog: &Catalog,
    ) -> Result<i32, Error> {
        let output = self.output(source, input).ok_or(Error::Unsupported)?;
        if inventory
            .get(slot)
            .is_none_or(|i| i.id != input || i.count != 1 || i.vars != 0)
        {
            return Err(Error::StaleSlot);
        }
        let mut next = inventory.clone();
        let mut debit = Delete::new(input, 1);
        debit.strict_slot = Some(slot);
        next.delete(catalog, debit).map_err(Error::Inventory)?;
        let mut credit = Insert::new(output, 1);
        credit.strict_slot = Some(slot);
        next.insert(catalog, credit).map_err(Error::Inventory)?;
        *inventory = next;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};

    fn fixture() -> (Config, Inventory, Catalog) {
        let config = Config {
            sources: [(11, Source::Sink), (12, Source::Well)].into(),
            vessels: (0..4)
                .map(|i| {
                    (
                        100 + i,
                        Vessel {
                            output: 200 + i,
                            well: i == 0,
                        },
                    )
                })
                .collect(),
        };
        let mut inv = Inventory::new(4, StackType::Normal, false);
        for slot in 0..4 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: 100 + slot as i32,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        (config, inv, Catalog::default())
    }

    #[test]
    fn sink_fills_each_selected_vessel_without_spare_capacity() {
        let (c, mut inv, catalog) = fixture();
        for slot in [3, 1, 0, 2] {
            let before = inv.clone();
            assert_eq!(
                c.fill(11, 100 + slot as i32, slot, &mut inv, &catalog),
                Ok(200 + slot as i32)
            );
            for other in 0..4 {
                if other != slot {
                    assert_eq!(inv.get(other), before.get(other));
                }
            }
            assert_eq!(inv.get(slot).unwrap().count, 1);
        }
    }

    #[test]
    fn well_policy_unknown_sources_and_stale_slots_never_debit() {
        let (c, mut inv, catalog) = fixture();
        let before = inv.clone();
        for slot in 1..4 {
            assert_eq!(
                c.fill(12, 100 + slot as i32, slot, &mut inv, &catalog),
                Err(Error::Unsupported)
            );
        }
        assert_eq!(
            c.fill(99, 100, 0, &mut inv, &catalog),
            Err(Error::Unsupported)
        );
        assert_eq!(
            c.fill(11, 100, 1, &mut inv, &catalog),
            Err(Error::StaleSlot)
        );
        assert_eq!(
            c.fill(11, 100, 99, &mut inv, &catalog),
            Err(Error::StaleSlot)
        );
        assert_eq!(inv, before);
        assert_eq!(c.fill(12, 100, 0, &mut inv, &catalog), Ok(200));
        let filled = inv.clone();
        assert_eq!(
            c.fill(12, 100, 0, &mut inv, &catalog),
            Err(Error::StaleSlot)
        );
        assert_eq!(inv, filled);
    }

    #[test]
    fn failed_credit_and_item_vars_preserve_every_slot() {
        let (c, mut inv, mut catalog) = fixture();
        catalog.dummy.insert(200);
        let before = inv.clone();
        assert!(matches!(
            c.fill(11, 100, 0, &mut inv, &catalog),
            Err(Error::Inventory(_))
        ));
        assert_eq!(inv, before);
        inv.restore_slot(
            0,
            Some(Item {
                id: 100,
                count: 1,
                vars: 7,
            }),
        )
        .unwrap();
        let before = inv.clone();
        assert_eq!(
            c.fill(11, 100, 0, &mut inv, &catalog),
            Err(Error::StaleSlot)
        );
        assert_eq!(inv, before);
    }
}

pub(crate) const QUEUE: i32 = -2391004;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Job {
    pub target: crate::locations::Location,
    pub revision: u64,
    pub origin: crate::CoordGrid,
    pub last_movement: i32,
    pub slot: usize,
    pub input: i32,
    pub queued_at: i32,
    pub repeat: bool,
}

/// Published only after the inventory transaction commits.
#[derive(Clone, Debug)]
pub struct Transition {
    pub target: crate::locations::Location,
    pub slot: usize,
    pub input: i32,
    pub output: i32,
    pub queued_at: i32,
    pub committed_at: i32,
    pub repeat: bool,
}
impl Transition {
    pub fn json(&self, actor: crate::simulation::ActorId) -> String {
        format!(
            r#"{{"actor_slot":{},"actor_generation":{},"location":{},"x":{},"z":{},"plane":{},"slot":{},"input":{},"output":{},"count":1,"queued_at":{},"committed_at":{},"repeat":{}}}"#,
            actor.slot,
            actor.generation,
            self.target.id,
            self.target.coords.x(),
            self.target.coords.z(),
            self.target.coords.level(),
            self.slot,
            self.input,
            self.output,
            self.queued_at,
            self.committed_at,
            self.repeat
        )
    }
}
