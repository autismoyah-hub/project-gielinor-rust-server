//! Wild flax only. Miscellania Rake/hidden variants require separate kingdom behavior.
use super::{Definitions, Symbols};
use gielinor_world::{
    location_content::Handler,
    pickables::{Depletion, Rule},
};
use std::collections::BTreeMap;
pub fn flax_pickables(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if objects.get("flax") != Some(&1779)
        || locations.get("flax") != Some(&14896)
        || sequences.get("human_pickupfloor") != Some(&827)
        || sounds.get("pick") != Some(&2581)
    {
        return Err("Unattested wild flax symbols".into());
    }
    let item = defs.items.get(&1779).ok_or("Missing flax product")?;
    let node = defs.locations.get(&14896).ok_or("Missing wild flax node")?;
    if item.name != "Flax"
        || !item.members
        || item.stackable
        || item.dummy == 1
        || item.cert.template != 0
        || item.placeholder.template != 0
        || item.transform.template != 0
        || item.inventory_ops.iter().any(Option::is_some)
        || node.name != "Flax"
        || node.ops[0].is_some()
        || node.ops[1].as_deref() != Some("Pick")
        || node.ops[2..].iter().any(Option::is_some)
        || node.collision.width != 1
        || node.collision.length != 1
        || !node.multi.is_empty()
        || node.multi_default != -1
        || node.multi_varbit != -1
        || node.multi_varp != -1
    {
        return Err("Unattested wild flax definitions".into());
    }
    let handler = Handler::PickableChance {
        rule: Rule {
            item: 1779,
            members: true,
            animation: 827,
            sound: 2581,
            picked_message: "You pick some flax.",
            full_message: "You can't carry any more flax.",
        },
        depletion: Depletion {
            numerator: 3,
            denominator: 16,
            delay: 10,
        },
    };
    Ok([((14896, 1), handler), ((14896, 2), handler)].into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wild_flax_binds_real_second_pick_and_excludes_kingdom_rake_nodes() {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut t = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
        t.name = "Flax".into();
        d.items.insert(1779, t);
        let mut l = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
        l.name = "Flax".into();
        l.ops[1] = Some("Pick".into());
        d.locations.insert(14896, l);
        let objects = [("flax".into(), 1779)].into();
        let locations = [("flax".into(), 14896)].into();
        let a = [("human_pickupfloor".into(), 827)].into();
        let s = [("pick".into(), 2581)].into();
        let h = flax_pickables(&objects, &locations, &a, &s, &d).unwrap();
        assert_eq!(h.len(), 2);
        for op in [1, 2] {
            let Handler::PickableChance { rule, depletion } = h[&(14896, op)] else {
                panic!("chance")
            };
            assert_eq!(
                (
                    rule.item,
                    rule.members,
                    depletion.numerator,
                    depletion.denominator,
                    depletion.delay
                ),
                (1779, true, 3, 16, 10)
            );
        }
        for id in [15075, 15076, 15077, 15078, 14909] {
            assert!(!h.keys().any(|&(node, _)| node == id));
        }
        d.locations.get_mut(&14896).unwrap().ops[0] = Some("Rake".into());
        assert!(flax_pickables(&objects, &locations, &a, &s, &d).is_err());
    }
}
