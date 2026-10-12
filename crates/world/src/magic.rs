//! Player-facing rune requirements, pouch state, and special rune packs.
use crate::{
    autocast::Spell,
    bank_actions::Binding,
    inventory::Inventory,
    runes::{self, Source, Validation},
    variables::Variables,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy)]
pub struct Pack {
    pub lms: bool,
    pub blighted: Option<i32>,
}
#[derive(Default)]
pub struct Names {
    pub missing: BTreeMap<i32, String>,
    pub not_wearing: BTreeMap<i32, String>,
}
pub struct Config {
    pub catalog: runes::Catalog,
    pub fake: [Binding; 2],
    pub fountain: Binding,
    pub blighted: Binding,
    pub regular_pouches: BTreeSet<i32>,
    pub divine_pouches: BTreeSet<i32>,
    pub pouch_types: [Binding; 4],
    pub pouch_counts: [(u32, Binding); 4],
    pub lms_pouch: i32,
    pub packs: BTreeMap<i32, Pack>,
    pub names: Names,
}
impl Config {
    pub fn validate_pack(
        &self,
        inventory: &Inventory,
        spell: i32,
        fake: bool,
        blighted: bool,
    ) -> Option<Validation> {
        let pack = self.packs.get(&spell)?;
        let first = |id| {
            inventory
                .slots()
                .iter()
                .position(|item| item.is_some_and(|i| i.id == id))
        };
        if pack.lms && fake && first(self.lms_pouch).is_some() {
            return Some(Validation::Unlimited);
        }
        if blighted {
            if let Some(item) = pack.blighted {
                if let Some(slot) = first(item) {
                    return Some(Validation::Enough(vec![Source::Inventory {
                        item,
                        slot,
                        count: 1,
                    }]));
                }
            }
        }
        None
    }
    pub fn validate(
        &self,
        vars: &Variables,
        inventory: &Inventory,
        worn: &Inventory,
        spell: &Spell,
    ) -> Result<Vec<Validation>, String> {
        let read = |binding: Binding| vars.bit(binding.bit).map_err(|e| format!("{e:?}"));
        let fake = read(self.fake[0])? == 1 || read(self.fake[1])? == 1;
        if let Some(result) =
            self.validate_pack(inventory, spell.item, fake, read(self.blighted)? == 1)
        {
            return Ok(vec![result]);
        }
        let divine = inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| self.divine_pouches.contains(&i.id));
        let has_pouch = divine
            || inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| self.regular_pouches.contains(&i.id));
        let mut pouch = Vec::new();
        if has_pouch {
            for i in 0..4 {
                let available = i != 3 || divine;
                pouch.push(runes::PouchSlot {
                    compact: if available {
                        read(self.pouch_types[i])?
                    } else {
                        0
                    },
                    varbit: available.then_some(self.pouch_counts[i].0),
                    count: if available {
                        read(self.pouch_counts[i].1)?
                    } else {
                        0
                    },
                });
            }
        }
        self.catalog.requirements(
            &runes::Context {
                inventory: inventory.slots(),
                worn: worn.slots(),
                pouch: &pouch,
                fake_runes: fake,
                fountain: read(self.fountain)? == 1,
            },
            &spell.requirements,
        )
    }
    pub fn missing_message(
        &self,
        vars: &Variables,
        inventory: &Inventory,
        worn: &Inventory,
        spell: &Spell,
    ) -> Result<Option<String>, String> {
        let invalid = self
            .validate(vars, inventory, worn, spell)?
            .into_iter()
            .find(|v| !v.valid());
        Ok(invalid.map(|v| {
            match v {
                Validation::NotEnough(id) => self.names.missing.get(&id).cloned(),
                Validation::NotWearing(id) => self.names.not_wearing.get(&id).cloned(),
                _ => None,
            }
            .unwrap_or_else(|| "You do not have enough runes to cast this spell.".into())
        }))
    }
}
