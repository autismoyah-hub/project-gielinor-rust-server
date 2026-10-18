//! Target-bound Crafting cape Boost; teleport and automatic equip effects are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::worn_stat_boost::Rule;
pub fn crafting_cape_boost(
    objects: &Symbols,
    stats: &Symbols,
    params: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<i32, Rule>, String> {
    for (name, id) in [
        ("wear_op1", 451),
        ("wear_op2", 452),
        ("statreq1_skill", 434),
        ("statreq1_level", 436),
    ] {
        if params.get(name) != Some(&id) {
            return Err("Missing cape Boost parameter aliases".into());
        }
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stats.get("crafting") != Some(&12)
        || stat.name != "Crafting"
        || stat.unreleased
        || stat.min_level != 1
        || stat.max_level != 99
    {
        return Err("Invalid cape Boost Crafting stat".into());
    }
    let mut result = std::collections::BTreeMap::new();
    for (symbol, id, name) in [
        ("skillcape_crafting", 9780, "Crafting cape"),
        ("skillcape_crafting_trimmed", 9781, "Crafting cape(t)"),
    ] {
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Crafting cape")?;
        if objects.get(symbol) != Some(&id)
            || d.name != name
            || !d.members
            || d.stackable
            || d.wear != [1, -1, -1]
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
            || d.inventory_ops[1].as_deref() != Some("Wear")
            || d.inventory_ops[2].as_deref() != Some("Teleport")
            || d.params.get(&451) != Some(&Param::String("Boost".into()))
            || d.params.get(&452) != Some(&Param::String("Teleport".into()))
            || d.params.get(&434) != Some(&Param::Integer(12))
            || d.params.get(&436) != Some(&Param::Integer(99))
        {
            return Err(format!("Invalid Crafting cape Boost {id}"));
        }
        result.insert(
            id,
            Rule {
                skill: 12,
                slot: 1,
                button_operation: 2,
                members: true,
            },
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worn_crafting_cape_boost_profile_pins_exact_labels_requirements_and_both_variants() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut objects = Symbols::new();
        for (sym, id, name) in [
            ("skillcape_crafting", 9780, "Crafting cape"),
            ("skillcape_crafting_trimmed", 9781, "Crafting cape(t)"),
        ] {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = true;
            t.wear = [1, -1, -1];
            t.inventory_ops[1] = Some("Wear".into());
            t.inventory_ops[2] = Some("Teleport".into());
            t.params.insert(451, Param::String("Boost".into()));
            t.params.insert(452, Param::String("Teleport".into()));
            t.params.insert(434, Param::Integer(12));
            t.params.insert(436, Param::Integer(99));
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        d.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        let stats = [("crafting".into(), 12)].into();
        let params = [
            ("wear_op1".into(), 451),
            ("wear_op2".into(), 452),
            ("statreq1_skill".into(), 434),
            ("statreq1_level".into(), 436),
        ]
        .into();
        let c = crafting_cape_boost(&objects, &stats, &params, &d).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[&9780].slot, 1);
        assert_eq!(c[&9781].button_operation, 2);
        d.items
            .get_mut(&9781)
            .unwrap()
            .params
            .insert(451, Param::String("Teleport".into()));
        assert!(crafting_cape_boost(&objects, &stats, &params, &d).is_err());
    }
}
