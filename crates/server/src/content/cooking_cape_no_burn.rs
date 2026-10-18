//! Target-bound Cooking cape perk for the existing manual cooking engine.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::cooking::NoBurnCape;
pub fn cooking_cape_no_burn(objects: &Symbols, defs: &Definitions) -> Result<NoBurnCape, String> {
    for (symbol, id, name) in [
        ("skillcape_cooking", 9801, "Cooking cape"),
        ("skillcape_cooking_trimmed", 9802, "Cooking cape(t)"),
        ("raw_oomlie", 2337, "Raw oomlie"),
    ] {
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Cooking cape perk item")?;
        if objects.get(symbol) != Some(&id)
            || item.name != name
            || !item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid Cooking cape perk item {id}"));
        }
        if id != 2337
            && (item.wear != [1, -1, -1]
                || item.inventory_ops[1].as_deref() != Some("Wear")
                || item.params.get(&434) != Some(&Param::Integer(7))
                || item.params.get(&436) != Some(&Param::Integer(99)))
        {
            return Err(format!("Invalid Cooking cape perk wear requirement {id}"));
        }
    }
    Ok(NoBurnCape {
        capes: [9801, 9802],
        excluded_raw: 2337,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cooking_cape_perk_profiles_require_both_ordinary_capes_and_raw_oomlie_exception() {
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
            ("skillcape_cooking", 9801, "Cooking cape"),
            ("skillcape_cooking_trimmed", 9802, "Cooking cape(t)"),
            ("raw_oomlie", 2337, "Raw oomlie"),
        ] {
            let mut t = gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();
            t.name = name.into();
            t.members = true;
            if id != 2337 {
                t.wear = [1, -1, -1];
                t.inventory_ops[1] = Some("Wear".into());
                t.params.insert(434, Param::Integer(7));
                t.params.insert(436, Param::Integer(99));
            }
            d.items.insert(id as u32, t);
            objects.insert(sym.into(), id);
        }
        let perk = cooking_cape_no_burn(&objects, &d).unwrap();
        assert_eq!(perk.capes, [9801, 9802]);
        assert_eq!(perk.excluded_raw, 2337);
        d.items.get_mut(&9802).unwrap().transform.template = 1;
        assert!(cooking_cape_no_burn(&objects, &d).is_err());
        d.items.get_mut(&9802).unwrap().transform.template = 0;
        objects.insert("raw_oomlie".into(), 2341);
        assert!(cooking_cape_no_burn(&objects, &d).is_err());
    }
}
