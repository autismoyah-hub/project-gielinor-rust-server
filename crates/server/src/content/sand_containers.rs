//! Manual sand disposal; gathering, furnace glass and quest deliveries are separate.
use super::{Definitions, Symbols};
pub fn sand_containers(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<(i32, gielinor_world::container_emptying::Rule), String> {
    for (symbol, id, name, member) in [
        ("bucket_sand", 1783, "Bucket of sand", true),
        ("bucket_empty", 1925, "Bucket", false),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased sand vessel {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing sand vessel definition")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || (id == 1783 && d.inventory_ops[3].as_deref() != Some("Empty"))
        {
            return Err(format!("Invalid sand vessel {id}"));
        }
    }
    // Both Kotlin inventory dispatchers map interface button6 to held-op4.
    Ok((
        1783,
        gielinor_world::container_emptying::Rule {
            output: 1925,
            members: true,
            button_operation: 6,
            message: None,
        },
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sand_containers_require_exact_empty_operation_and_ordinary_member_binding() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let objects = [("bucket_sand".into(), 1783), ("bucket_empty".into(), 1925)].into();
        defs.items.insert(
            1783,
            gielinor_cache::definitions::ItemType::decode(b"\x02Bucket of sand\0\x10\x26Empty\0\0")
                .unwrap(),
        );
        defs.items.insert(
            1925,
            gielinor_cache::definitions::ItemType::decode(b"\x02Bucket\0\0").unwrap(),
        );
        let (id, rule) = sand_containers(&objects, &defs).unwrap();
        assert_eq!((id, rule.output, rule.button_operation), (1783, 1925, 6));
        assert!(rule.members);
        defs.items.get_mut(&1783).unwrap().inventory_ops[3] = None;
        assert!(sand_containers(&objects, &defs).is_err());
        defs.items.get_mut(&1783).unwrap().inventory_ops[3] = Some("Empty".into());
        defs.items.get_mut(&1925).unwrap().placeholder.template = 14401;
        assert!(sand_containers(&objects, &defs).is_err());
    }
}
