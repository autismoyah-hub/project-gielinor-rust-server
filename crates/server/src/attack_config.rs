//! Cache-resolved attack type/style tables and weapon timing/reach parameters.
use gielinor_cache::{
    definitions::Param,
    enums::{EnumType, Value},
    game::Definitions,
    parameters::ParameterType,
    symbols::Symbols,
};
use gielinor_world::attack_profile::{Catalog, Weapon};
use std::collections::BTreeMap;
pub fn resolve(
    params: &Symbols,
    enums: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    tables: &BTreeMap<u32, EnumType>,
    defs: &Definitions,
) -> Result<Catalog, String> {
    let symbol = |map: &Symbols, name: &str| {
        map.get(name)
            .copied()
            .map(|v| v as u32)
            .ok_or_else(|| format!("Missing attack symbol {name}"))
    };
    let speed = symbol(params, "attackrate")?;
    let range = symbol(params, "attack_range")?;
    let table = |name: &str| -> Result<BTreeMap<i32, i32>, String> {
        let id = symbol(enums, name)?;
        tables
            .get(&id)
            .ok_or_else(|| format!("Missing attack table {name}"))?
            .values
            .iter()
            .map(|(&key, value)| match value {
                Value::Int(value) => Ok((key, *value)),
                _ => Err(format!("Noninteger attack table {name}")),
            })
            .collect()
    };
    let mut weapons = BTreeMap::new();
    for (&id, item) in &defs.items {
        let int = |key| -> Result<i32, String> {
            match item.params.get(&key) {
                Some(Param::Integer(v)) => Ok(*v),
                Some(_) => Err(format!("Invalid attack param {key} on {id}")),
                None => parameter_types
                    .get(&key)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing attack default {key}")),
            }
        };
        weapons.insert(
            id as i32,
            Weapon {
                category: item.weapon_category,
                range: int(range)?,
                speed: int(speed)?,
            },
        );
    }
    Ok(Catalog {
        weapons,
        types: table("weapon_attack_types")?,
        styles: table("weapon_attack_styles")?,
    })
}
