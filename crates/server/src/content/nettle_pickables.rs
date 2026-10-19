//! Nettles: protective gloves, one incidental sting, and shared cache respawn.
use super::{Definitions, Symbols};
use gielinor_world::{hit_display::Mark, location_content::Handler, pickables::Rule};
use std::collections::BTreeMap;
const NODES: [(&str, u32); 7] = [
    ("nettles", 1181),
    ("nettles1", 5253),
    ("nettles2", 5254),
    ("nettles3", 5255),
    ("nettles4", 5256),
    ("nettles5", 5257),
    ("nettles6", 5258),
];
pub fn nettle_pickables(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    hitmarks: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if objects.get("nettles_picked") != Some(&4241)
        || objects.get("beekeeper_gloves") != Some(&25135)
        || sequences.get("human_pickupfloor") != Some(&827)
        || sounds.get("pick") != Some(&2581)
        || hitmarks.get("poison_damage_lit") != Some(&65)
        || hitmarks.get("poison_damage_tint") != Some(&66)
    {
        return Err("Unattested nettle symbols/presentation".into());
    }
    let item = defs.items.get(&4241).ok_or("Missing picked nettles")?;
    let gloves = defs
        .items
        .get(&25135)
        .ok_or("Missing excluded beekeeper gloves")?;
    if item.name != "Nettles"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
        || item.inventory_ops.iter().any(Option::is_some)
        || gloves.name != "Beekeeper's gloves"
        || gloves.wear[0] != 9
        || gloves.stackable
        || gloves.dummy == 1
        || gloves.cert.template != 0
        || gloves.placeholder.template != 0
        || gloves.transform.template != 0
    {
        return Err("Unattested nettle product/glove exception".into());
    }
    for (symbol, id) in NODES {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Nettles"
                    || d.ops[0].as_deref() != Some("Pick")
                    || d.ops[1..].iter().any(Option::is_some)
                    || d.collision.width != 1
                    || d.collision.length != 1
                    || !d.multi.is_empty()
                    || d.multi_default != -1
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested nettle node {symbol}"));
        }
    }
    let handler = Handler::Nettles {
        rule: Rule {
            item: 4241,
            members: true,
            animation: 827,
            sound: 2581,
            picked_message: "You pick some nettles.",
            full_message: "You can't carry any more nettles.",
        },
        excluded_glove: 25135,
        mark: Mark {
            target: 65,
            source: 65,
            public: Some(66),
            delay: 0,
        },
    };
    Ok(NODES
        .into_iter()
        .flat_map(|(_, id)| [1, 2].map(|op| ((id, op), handler)))
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nettle_native_configuration_checks_all_nodes_glove_exception_and_green_hitmarks() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut item = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
        item.name = "Nettles".into();
        d.items.insert(4241, item);
        let mut glove = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        glove.name = "Beekeeper's gloves".into();
        glove.wear[0] = 9;
        d.items.insert(25135, glove);
        let mut l = Symbols::new();
        for (symbol, id) in NODES {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = "Nettles".into();
            t.ops[0] = Some("Pick".into());
            d.locations.insert(id, t);
        }
        let o = [
            ("nettles_picked".into(), 4241),
            ("beekeeper_gloves".into(), 25135),
        ]
        .into();
        let a = [("human_pickupfloor".into(), 827)].into();
        let s = [("pick".into(), 2581)].into();
        let h = [
            ("poison_damage_lit".into(), 65),
            ("poison_damage_tint".into(), 66),
        ]
        .into();
        let bound = nettle_pickables(&o, &l, &a, &s, &h, &d).unwrap();
        assert_eq!(bound.len(), 14);
        for (_, id) in NODES {
            for op in [1, 2] {
                let Handler::Nettles {
                    rule,
                    excluded_glove,
                    mark,
                } = bound[&(id, op)]
                else {
                    panic!("nettles")
                };
                assert_eq!(
                    (
                        rule.item,
                        rule.members,
                        excluded_glove,
                        mark.target,
                        mark.public
                    ),
                    (4241, true, 25135, 65, Some(66))
                );
            }
        }
        d.locations.get_mut(&5258).unwrap().ops[1] = Some("Search".into());
        assert!(nettle_pickables(&o, &l, &a, &s, &h, &d).is_err());
    }
}
