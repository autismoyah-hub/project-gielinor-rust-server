//! Tutorial PlayerDesignScript's state transitions. Modal admission and cache
//! table loading belong to the caller; this module never trusts packet kit IDs.
use crate::appearance::Design;
use gielinor_world::{
    bank_actions::Binding,
    variables::{Error, Update, Variables},
};

#[derive(Clone, Copy, Debug)]
pub enum Control {
    Kit { slot: usize, delta: i32 },
    Colour { slot: usize, delta: i32 },
    Body(u8),
    Pronoun(i32),
}

#[derive(Default)]
pub struct Kits {
    /// Body A/B, then hair/jaw/torso/arms/hands/legs/feet, in reference DB order.
    pub options: [[Vec<u16>; 7]; 2],
}
impl Kits {
    pub fn resolve(
        rows: &std::collections::BTreeMap<u32, gielinor_cache::db::Row>,
        tables: &std::collections::BTreeMap<u32, gielinor_cache::db::Table>,
        table_names: &gielinor_cache::symbols::Symbols,
        columns: &gielinor_cache::symbols::Symbols,
    ) -> Result<Self, String> {
        use gielinor_cache::{db::Value, int_map_order};
        let order = int_map_order::order(rows.keys().map(|id| *id as i32));
        let mut result = Self::default();
        for (slot, name) in [
            "hair_styles",
            "facial_hair_styles",
            "torso_styles",
            "sleeve_styles",
            "hand_styles",
            "legging_styles",
            "shoe_styles",
        ]
        .into_iter()
        .enumerate()
        {
            let id = *table_names
                .get(name)
                .ok_or_else(|| format!("Missing design table {name}"))?;
            let table = tables
                .get(&(id as u32))
                .ok_or_else(|| format!("Missing design table data {name}"))?;
            for (body, suffix) in ["a", "b"].into_iter().enumerate() {
                let name = format!("{name}:player_kit_id_type_{suffix}");
                let packed = *columns
                    .get(&name)
                    .ok_or_else(|| format!("Missing design column {name}"))?
                    as u32;
                if packed >> 16 != id as u32 {
                    return Err(format!("Wrong design table for {name}"));
                }
                let column = u8::try_from(packed & 65535)
                    .map_err(|_| format!("Invalid design column {name}"))?;
                let options = &mut result.options[body][slot];
                for row_id in &order {
                    let row = &rows[&(*row_id as u32)];
                    if row.table != id {
                        continue;
                    }
                    let value = match row.column(table, column) {
                        None => continue,
                        Some([Value::Int(value), ..]) => *value,
                        _ => return Err(format!("Invalid design kit value in {name}")),
                    };
                    if value < 0 {
                        continue;
                    }
                    let kit = u16::try_from(value)
                        .map_err(|_| format!("Design kit out of range in {name}"))?;
                    // LinkedHashSet retains first occurrence in DbTableResolver row order.
                    if !options.contains(&kit) {
                        options.push(kit);
                    }
                }
            }
        }
        Ok(result)
    }
    fn options(&self, body: u8, slot: usize) -> &[u16] {
        &self.options[usize::from(body != 0)][slot]
    }
    fn defaults(&self, body: u8) -> [Option<u16>; 7] {
        let fallback =
            [[0, 10, 18, 26, 33, 36, 42], [45, 296, 56, 61, 67, 70, 79]][usize::from(body != 0)];
        std::array::from_fn(|slot| {
            Some(
                self.options(body, slot)
                    .first()
                    .copied()
                    .unwrap_or(fallback[slot]),
            )
        })
    }
}
pub struct Config {
    pub kits: Kits,
    pub body: Binding,
    pub pronoun: Binding,
}
pub struct Bindings {
    pub config: Config,
    pub buttons: std::collections::BTreeMap<u32, Control>,
    pub pronoun_component: u32,
}
impl Bindings {
    pub fn control(&self, component: u32, child: i32) -> Option<Control> {
        if component == self.pronoun_component {
            Some(Control::Pronoun(child))
        } else {
            self.buttons.get(&component).copied()
        }
    }
    pub fn resolve(
        kits: Kits,
        components: &gielinor_cache::symbols::Symbols,
        bits: &gielinor_cache::symbols::Symbols,
        defs: &gielinor_cache::game::Definitions,
    ) -> Result<Self, String> {
        let component = |name: &str| {
            components
                .get(&format!("player_design:{name}"))
                .copied()
                .map(|v| v as u32)
                .ok_or_else(|| format!("Missing creator component {name}"))
        };
        let binding = |name: &str| -> Result<Binding, String> {
            let id = *bits
                .get(name)
                .ok_or_else(|| format!("Missing creator variable {name}"))?;
            let bit = *defs
                .varbits
                .get(&(id as u32))
                .ok_or("Missing creator varbit")?;
            let base = *defs
                .varps
                .get(&(bit.base as u32))
                .ok_or("Missing creator base varp")?;
            Ok(Binding { base, bit })
        };
        let mut buttons = std::collections::BTreeMap::new();
        for (slot, name) in ["head", "jaw", "torso", "arms", "hands", "legs", "feet"]
            .into_iter()
            .enumerate()
        {
            for (suffix, delta) in [("left", -1), ("right", 1)] {
                buttons.insert(
                    component(&format!("{name}_{suffix}"))?,
                    Control::Kit { slot, delta },
                );
            }
        }
        for (slot, name) in ["hair", "torso_col", "legs_col", "feet_col", "skin"]
            .into_iter()
            .enumerate()
        {
            for (suffix, delta) in [("left", -1), ("right", 1)] {
                buttons.insert(
                    component(&format!("{name}_{suffix}"))?,
                    Control::Colour { slot, delta },
                );
            }
        }
        buttons.insert(component("gender_male")?, Control::Body(0));
        buttons.insert(component("gender_female")?, Control::Body(1));
        Ok(Self {
            config: Config {
                kits,
                body: binding("player_design_bodytype")?,
                pronoun: binding("settings_transmit_pronouns")?,
            },
            buttons,
            pronoun_component: component("pronouns_buttons")?,
        })
    }
}
#[derive(Default, Debug)]
pub struct Effects {
    pub variables: Vec<Update>,
    /// Kotlin flags an appearance rebuild even when a setter repeats a value.
    pub rebuild: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_cache::db::{Row, Table, Value};
    use std::collections::BTreeMap;
    #[test]
    fn cache_kits_preserve_reference_order_defaults_duplicates_and_negative_values() {
        let mut names = BTreeMap::new();
        let mut columns = BTreeMap::new();
        let mut tables = BTreeMap::new();
        for (index, name) in [
            "hair_styles",
            "facial_hair_styles",
            "torso_styles",
            "sleeve_styles",
            "hand_styles",
            "legging_styles",
            "shoe_styles",
        ]
        .into_iter()
        .enumerate()
        {
            let id = index as i32 + 1;
            names.insert(name.to_owned(), id);
            for (col, suffix) in ["a", "b"].into_iter().enumerate() {
                columns.insert(
                    format!("{name}:player_kit_id_type_{suffix}"),
                    (id << 16) | col as i32,
                );
            }
            tables.insert(
                id as u32,
                Table {
                    defaults: BTreeMap::from([(1, vec![Value::Int(42)])]),
                    ..Default::default()
                },
            );
        }
        let mut rows = BTreeMap::from([
            (
                0,
                Row {
                    table: 1,
                    data: BTreeMap::from([(0, vec![Value::Int(7)])]),
                    ..Default::default()
                },
            ),
            (
                1,
                Row {
                    table: 1,
                    data: BTreeMap::from([(0, vec![Value::Int(5)]), (1, vec![Value::Int(8)])]),
                    ..Default::default()
                },
            ),
            (
                2,
                Row {
                    table: 1,
                    data: BTreeMap::from([(0, vec![Value::Int(7)]), (1, vec![Value::Int(-1)])]),
                    ..Default::default()
                },
            ),
        ]);
        let kits = Kits::resolve(&rows, &tables, &names, &columns).unwrap();
        assert_eq!(kits.options[0][0], [7, 5]);
        assert_eq!(kits.options[1][0], [42, 8]);
        assert_eq!(kits.defaults(1), [42, 296, 56, 61, 67, 70, 79].map(Some));
        rows.get_mut(&2).unwrap().data.insert(1, vec![]);
        assert!(Kits::resolve(&rows, &tables, &names, &columns).is_err());
    }
}
impl Config {
    pub fn apply(
        &self,
        design: &mut Design,
        vars: &mut Variables,
        control: Control,
        logged: bool,
    ) -> Result<Effects, Error> {
        let mut effects = Effects::default();
        match control {
            Control::Kit { slot, delta } => {
                if slot >= 7 {
                    return Ok(effects);
                }
                let options = self.kits.options(design.body_type, slot);
                if options.is_empty() {
                    return Ok(effects);
                }
                // identKitSnapshot stores signed shorts; preserve that comparison.
                let current = design.kits[slot].map(|v| v as i16 as i32).unwrap_or(-1);
                let index = options
                    .iter()
                    .position(|v| i32::from(*v) == current)
                    .unwrap_or(0);
                let next = (index as i32)
                    .wrapping_add(delta)
                    .rem_euclid(options.len() as i32);
                design.kits[slot] = Some(options[next as usize]);
                effects.rebuild = true;
            }
            Control::Colour { slot, delta } => {
                let Some(&count) = [30, 28, 28, 6, 8].get(slot) else {
                    return Ok(effects);
                };
                design.colours[slot] = i32::from(design.colours[slot])
                    .wrapping_add(delta)
                    .rem_euclid(count) as u8;
                effects.rebuild = true;
            }
            Control::Body(body) => {
                // The UI binding is written before the same-body early return.
                effects.variables.extend(vars.set_bit(
                    self.body.base,
                    self.body.bit,
                    body.into(),
                    logged,
                )?);
                if design.body_type == body {
                    return Ok(effects);
                }
                design.body_type = body;
                design.pronoun = body;
                effects.variables.extend(vars.set_bit(
                    self.pronoun.base,
                    self.pronoun.bit,
                    body.into(),
                    logged,
                )?);
                design.kits = self.kits.defaults(body);
                effects.rebuild = true;
            }
            Control::Pronoun(child) => {
                if !(1..=3).contains(&child) {
                    return Ok(effects);
                }
                design.pronoun = (child - 1) as u8;
                effects.rebuild = true;
                effects.variables.extend(vars.set_bit(
                    self.pronoun.base,
                    self.pronoun.bit,
                    child - 1,
                    logged,
                )?);
            }
        }
        Ok(effects)
    }
}
