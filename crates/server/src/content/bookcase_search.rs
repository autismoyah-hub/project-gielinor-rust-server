//! Only native generic bookcase content6; quest-specific books remain separate.
use super::{Definitions, Symbols};
use gielinor_world::location_content::Handler;
use std::collections::{BTreeMap, BTreeSet};
const NODES: [(&str, u32, u8); 2] = [
    ("bookcase", 380, 2),
    ("fai_varrock_posh_bookcase_short_east_offset", 7180, 1),
];
pub fn bookcase_search(
    locations: &Symbols,
    contents: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if contents.get("bookcase") != Some(&6) {
        return Err("Unattested bookcase content symbol".into());
    }
    let actual: BTreeSet<_> = defs
        .locations
        .iter()
        .filter(|(_, d)| d.content_group == 6)
        .map(|(&id, _)| id)
        .collect();
    if actual != NODES.iter().map(|v| v.1).collect() {
        return Err("Unattested native bookcase set".into());
    }
    for (symbol, id, length) in NODES {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Bookcase"
                    || d.ops[0].as_deref() != Some("Search")
                    || d.ops[1..].iter().any(Option::is_some)
                    || d.collision.width != 1
                    || d.collision.length != length
                    || !d.multi.is_empty()
                    || d.multi_default != -1
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested native bookcase {symbol}"));
        }
    }
    Ok(NODES
        .into_iter()
        .map(|(_, id, _)| ((id, 1), Handler::Bookcase))
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bookcase_native_two_static_shapes_content_and_operation_bindings_refuse_alias_or_quest_variant(
    ) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut l = Symbols::new();
        for (symbol, id, length) in NODES {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Bookcase".into();
            t.content_group = 6;
            t.ops[0] = Some("Search".into());
            t.collision.length = length;
            d.locations.insert(id, t);
        }
        let c = [("bookcase".into(), 6)].into();
        let h = bookcase_search(&l, &c, &d).unwrap();
        assert_eq!(h.len(), 2);
        assert!(matches!(h[&(380, 1)], Handler::Bookcase));
        assert!(!h.contains_key(&(381, 1)));
        d.locations.get_mut(&7180).unwrap().multi_varp = 1;
        assert!(bookcase_search(&l, &c, &d).is_err());
    }
}
