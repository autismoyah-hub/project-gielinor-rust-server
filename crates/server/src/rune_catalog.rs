//! Load rune source repositories with Kotlin's enum and HashMap iteration order.
use gielinor_cache::{
    enums::{EnumType, Value},
    game::Definitions,
    symbols::Symbols,
};
use gielinor_world::runes::Catalog;
use std::collections::BTreeMap;
pub fn resolve(
    symbols: &Symbols,
    tables: &BTreeMap<u32, EnumType>,
    defs: &Definitions,
) -> Result<Catalog, String> {
    let named = |name: &str| -> Result<i32, String> {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing rune enum {name}"))
    };
    let entries = |id: i32, object_keys: bool| -> Result<Vec<(i32, i32)>, String> {
        let table = tables
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing rune enum {id}"))?;
        let raw = gielinor_cache::jvm_hash::order(&table.insertion_order).map_err(str::to_owned)?;
        let mut values = Vec::new();
        for index in raw {
            let key = table.insertion_order[index];
            let Value::Int(value) = table.values[&key] else {
                return Err(format!("Noninteger rune enum {id}"));
            };
            // ComplexTypeDecoder uses a LinkedHashMap, preserving primitive order.
            if object_keys && !defs.items.contains_key(&(key as u32)) {
                continue;
            }
            let exists = |v: i32| match table.value_literal {
                33 => defs.items.contains_key(&(v as u32)),
                26 => tables.contains_key(&(v as u32)),
                _ => true,
            };
            let value = if exists(value) {
                value
            } else if let Some(default) = table.default_int.filter(|&v| exists(v)) {
                default
            } else {
                continue;
            };
            values.push((key, value));
        }
        Ok(values)
    };
    let mut catalog = Catalog::default();
    catalog
        .compact
        .extend(entries(named("rune_compact_ids")?, true)?);
    catalog.fake.extend(entries(named("fake_runes")?, true)?);
    for (rune, table) in entries(named("rune_staves")?, true)? {
        catalog.high.entry(rune).or_default().extend(
            entries(table, true)?
                .into_iter()
                .filter(|(_, enabled)| *enabled == 1)
                .map(|(id, _)| id),
        );
    }
    for (name, high) in [
        ("unlimited_runes_hiprio", true),
        ("unlimited_runes_loprio", false),
    ] {
        for (rune, table) in entries(named(name)?, true)? {
            let target = if high {
                &mut catalog.high
            } else {
                &mut catalog.low
            };
            target
                .entry(rune)
                .or_default()
                .extend(entries(table, false)?.into_iter().map(|(_, id)| id));
        }
    }
    for (rune, table) in entries(named("rune_substitutes")?, true)? {
        catalog.substitutes.insert(
            rune,
            entries(table, false)?
                .into_iter()
                .map(|(_, id)| id)
                .collect(),
        );
    }
    for (staff, table) in entries(named("staff_substitutes")?, true)? {
        catalog.staff_substitutes.insert(
            staff,
            entries(table, false)?
                .into_iter()
                .map(|(_, id)| id)
                .collect(),
        );
    }
    let combos = entries(named("combo_runes")?, true)?;
    let hashes: Vec<_> = combos
        .iter()
        .map(|&(id, _)| {
            gielinor_cache::item_identity::hashed_type_hash(id, &defs.items[&(id as u32)])
        })
        .collect();
    // ComboRuneRepository copies the linked enum entries into a HashMap of ObjType.
    for index in gielinor_cache::jvm_hash::order(&hashes).map_err(str::to_owned)? {
        let (rune, table) = combos[index];
        let values = entries(table, false)?;
        if values.len() != 2 {
            return Err(format!("Combo rune {rune} does not have two components"));
        }
        catalog.combos.push((rune, values[0].1, values[1].1));
    }
    Ok(catalog)
}
