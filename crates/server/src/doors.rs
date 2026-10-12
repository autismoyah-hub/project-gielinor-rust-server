//! Resolve the generic single/double door script registrations from packed types.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::doors::{Catalog, Door, Kind};
use std::collections::BTreeMap;
pub fn resolve(
    content: &Symbols,
    params: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<Catalog, String> {
    let symbol = |names: &Symbols, name: &str| {
        names
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing door symbol {name}"))
    };
    let mut groups = BTreeMap::new();
    for (name, kind) in [
        ("closed_single_door", Kind::ClosedSingle),
        ("opened_single_door", Kind::OpenedSingle),
        ("closed_left_door", Kind::ClosedLeft),
        ("closed_right_door", Kind::ClosedRight),
        ("opened_left_door", Kind::OpenedLeft),
        ("opened_right_door", Kind::OpenedRight),
    ] {
        groups.insert(symbol(content, name)?, kind);
    }
    let next = symbol(params, "next_loc_stage")? as u32;
    let open = symbol(params, "opensound")? as u32;
    let close = symbol(params, "closesound")? as u32;
    let mut result = Catalog::new();
    for (&id, ty) in &defs.locations {
        let Some(&kind) = groups.get(&ty.content_group) else {
            continue;
        };
        // OpLocHandler checks the type's operation before launching a script.
        if ty.ops[0].is_none() {
            continue;
        }
        let int = |key: u32| -> Result<i32, String> {
            match ty.params.get(&key) {
                Some(Param::Integer(v)) => Ok(*v),
                Some(_) => Err(format!("Noninteger door parameter {key} on {id}")),
                None => parameter_types
                    .get(&key)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing door parameter {key} on {id}")),
            }
        };
        let next =
            u32::try_from(int(next)?).map_err(|_| format!("Invalid next door stage on {id}"))?;
        if !defs.locations.contains_key(&next) {
            return Err(format!("Unknown door stage {next} on {id}"));
        }
        // DoubleDoorScript uses opensound even for closing both halves.
        let sound = u16::try_from(int(if kind == Kind::OpenedSingle {
            close
        } else {
            open
        })?)
        .map_err(|_| format!("Invalid door sound on {id}"))?;
        result.insert(id, Door { kind, next, sound });
    }
    Ok(result)
}
