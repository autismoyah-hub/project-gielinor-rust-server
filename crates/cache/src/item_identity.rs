//! Kotlin object identity hashes determine HashMap iteration in combo-rune loading.
use crate::definitions::{ItemType, Param};
pub fn string_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0i32, |hash, ch| {
        hash.wrapping_mul(31).wrapping_add(ch as i32)
    })
}
fn array_hash(values: impl IntoIterator<Item = i32>) -> i32 {
    values.into_iter().fold(1i32, |hash, value| {
        hash.wrapping_mul(31).wrapping_add(value)
    })
}
pub fn identity(id: i32, item: &ItemType) -> i64 {
    let boolean = |v| if v { 1231 } else { 1237 };
    let ops = |values: &[Option<String>]| {
        array_hash(values.iter().map(|v| v.as_deref().map_or(0, string_hash)))
    };
    let params = item.params.iter().fold(0i32, |hash, (&key, value)| {
        hash.wrapping_add(
            key as i32
                ^ match value {
                    Param::Integer(value) => *value,
                    Param::String(value) => string_hash(value),
                },
        )
    });
    let mut hash = id as i64;
    let mut add = |value: i32| {
        hash = hash.wrapping_mul(61).wrapping_add(value as i64);
    };
    for value in [
        string_hash(&item.name),
        boolean(item.stackable),
        item.cost,
        item.wear[0] as i32,
        item.wear[1] as i32,
        boolean(item.members),
        item.wear[2] as i32,
        ops(&item.ground_ops),
        ops(&item.inventory_ops),
        item.shift_click as i32,
    ] {
        add(value);
    }
    for parent in 0..5u8 {
        add(if item.sub_ops_present[parent as usize] {
            array_hash((0..20u8).map(|sub| {
                item.sub_ops
                    .get(&(parent, sub))
                    .map_or(0, |v| string_hash(v))
            }))
        } else {
            0
        });
    }
    for value in [
        item.bought.link,
        item.bought.template,
        item.placeholder.link,
        item.placeholder.template,
        params,
        array_hash(item.objvars.iter().map(|&v| v as i32)),
        boolean(item.tradeable),
        item.respawn_rate as i32,
        item.dummy as i32,
    ] {
        add(value);
    }
    hash & i64::MAX
}
pub fn hashed_type_hash(id: i32, item: &ItemType) -> i32 {
    let hash = identity(id, item) as u64;
    id.wrapping_mul(31)
        .wrapping_add((hash ^ (hash >> 32)) as i32)
}
