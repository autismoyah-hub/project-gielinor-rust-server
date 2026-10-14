//! Existing SmithingActions cache configuration. Runtime registration is explicit.
use crate::startup::Names;
use gielinor_cache::{
    enums::{EnumType, Value},
    game::Definitions,
};
use gielinor_world::{
    anvil, anvil_products, bank_actions::Binding, ui::Component, variables::Varp,
};
use std::{
    collections::{BTreeMap, HashSet},
    io,
};

#[derive(Clone, Debug)]
pub struct Config {
    pub rules: anvil::Config,
    pub interface: u16,
    pub parent: Component,
    pub bar_type: Binding,
    pub bar_type_id: u32,
    pub quantity: Varp,
    /// None means the Make X count dialog, not a literal quantity.
    pub quantity_buttons: BTreeMap<u32, Option<i32>>,
    pub locations: HashSet<u32>,
    pub bars: HashSet<i32>,
}
fn object_entries(table: &EnumType) -> io::Result<BTreeMap<i32, i32>> {
    if table.key_literal != 33 || table.value_literal != 0 {
        return Err(io::Error::other(
            "Smithing enum must map object IDs to integers",
        ));
    }
    // SmithingActions.byObjId iterates entries. It does not materialize the
    // enum default for absent keys; each call site applies its own fallback.
    table
        .values
        .iter()
        .map(|(&id, value)| match value {
            Value::Int(value) => Ok((id, *value)),
            Value::String(_) => Err(io::Error::other("Invalid smithing enum value")),
        })
        .collect()
}
impl Config {
    pub fn resolve(
        names: &Names,
        definitions: &Definitions,
        tables: &BTreeMap<u32, EnumType>,
    ) -> io::Result<Self> {
        let entries = |name| {
            let id = names.id("enum", name)?;
            object_entries(
                tables
                    .get(&id)
                    .ok_or_else(|| io::Error::other(format!("Missing smithing enum {name}")))?,
            )
        };
        let mut products = BTreeMap::<i32, BTreeMap<u16, i32>>::new();
        for &(bar, slot, product) in anvil_products::PRODUCTS {
            products.entry(bar).or_default().insert(slot, product);
        }
        let bit_id = names.id("varbit", "smithing_bar_type")?;
        let bit = *definitions
            .varbits
            .get(&bit_id)
            .ok_or_else(|| io::Error::other("Missing smithing bar varbit"))?;
        bit.extract(0)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let base = *definitions
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| io::Error::other("Missing smithing bar base varp"))?;
        let quantity_id = names.id("varp", "makexcrafting")?;
        let quantity = *definitions
            .varps
            .get(&quantity_id)
            .ok_or_else(|| io::Error::other("Missing smithing quantity varp"))?;
        let mut quantity_buttons = BTreeMap::new();
        for (name, count) in [
            ("smithing:make_1", Some(1)),
            ("smithing:make_5", Some(5)),
            ("smithing:make_10", Some(10)),
            ("smithing:make_all", Some(28)),
            ("smithing:make_x", None),
        ] {
            quantity_buttons.insert(names.id("component", name)?, count);
        }
        let anvil_group = names.id("content", "smithing_anvil")? as i32;
        let bar_group = names.id("content", "smithing_bar")? as i32;
        let rules = anvil::Config {
            hammer: names.id("obj", "hammer")? as i32,
            skill: u8::try_from(names.id("stat", "smithing")?).map_err(io::Error::other)?,
            animation: names.short("seq", "human_smithing")?,
            bar_by_tier: anvil_products::TIERS.iter().copied().collect(),
            products,
            quantity: entries("smithing_product_to_quantity")?,
            bars: entries("smithing_product_to_bars_required")?,
            level: entries("smithing_product_to_requirement")?,
            xp_per_bar: anvil_products::XP_PER_BAR.iter().copied().collect(),
            names: definitions
                .items
                .iter()
                .map(|(&id, item)| (id as i32, item.name.clone()))
                .collect(),
        };
        if !rules.names.contains_key(&rules.hammer) {
            return Err(io::Error::other("Missing smithing hammer definition"));
        }
        Ok(Self {
            rules,
            interface: names.short("interface", "smithing")?,
            parent: Component(names.id("component", "toplevel_osrs_stretch:mainmodal")?),
            bar_type: Binding { base, bit },
            bar_type_id: bit_id,
            quantity,
            quantity_buttons,
            locations: definitions
                .locations
                .iter()
                .filter(|(_, loc)| loc.content_group == anvil_group)
                .map(|(&id, _)| id)
                .collect(),
            bars: definitions
                .items
                .iter()
                .filter(|(_, item)| item.content_group == bar_group)
                .map(|(&id, _)| id as i32)
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_enum_entries_do_not_inherit_default_costs_or_levels() {
        let table = EnumType {
            key_literal: 33,
            value_literal: 0,
            default_int: Some(2147483647),
            default_string: None,
            values: BTreeMap::from([(1205, Value::Int(0)), (1117, Value::Int(18))]),
            insertion_order: vec![1205, 1117],
            transmit: true,
            decoded_bytes: 0,
        };
        assert_eq!(
            object_entries(&table).unwrap(),
            BTreeMap::from([(1205, 0), (1117, 18)])
        );
        let mut invalid = table;
        invalid.values.insert(1205, Value::String("0".into()));
        assert!(object_entries(&invalid).is_err());
    }
}

#[derive(Clone, Copy)]
pub struct CountDialog {
    pub token: u64,
    pub parent: Component,
    pub interface: u16,
}
