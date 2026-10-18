//! Two independently attested perpetual single-item bushes; wider Pickables remain separate.
use super::{Definitions, Symbols};
use gielinor_world::{location_content::Handler, pickables::Rule};
use std::collections::BTreeMap;
pub fn perpetual_pickables(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if sequences.get("human_pickupfloor") != Some(&827) || sounds.get("pick") != Some(&2581) {
        return Err("Unattested pickable presentation symbols".into());
    }
    for (symbol, id, name) in [
        ("caerula_berries", 30937, "Caerula berries"),
        ("damiana_leaves", 30977, "Damiana leaves"),
    ] {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || !d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
            })
        {
            return Err(format!("Unattested pickable product {symbol}"));
        }
    }
    for (symbol, id, name, op) in [
        ("caerula_bush", 56449, "Caerula bush", "Pick-from"),
        ("damiana_shrub", 56935, "Damiana", "Pick"),
    ] {
        if locations.get(symbol) != Some(&id)
            || defs.locations.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || d.ops[0].as_deref() != Some(op)
                    || d.ops[1..].iter().any(Option::is_some)
                    || d.collision.width != 1
                    || d.collision.length != 1
                    || !d.multi.is_empty()
                    || d.multi_default != -1
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested perpetual pickable {symbol}"));
        }
    }
    let mut handlers = BTreeMap::new();
    for (id, item, picked, full) in [
        (
            56449,
            30937,
            "You pick some caerula berries.",
            "You can't carry any more caerula berries.",
        ),
        (
            56935,
            30977,
            "You pick some damiana leaves.",
            "You can't carry any more damiana leaves.",
        ),
    ] {
        let rule = Rule {
            item,
            members: true,
            animation: 827,
            sound: 2581,
            picked_message: picked,
            full_message: full,
        };
        for op in [1, 2] {
            handlers.insert((id, op), Handler::Pickable(rule));
        }
    }
    Ok(handlers)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Symbols, Symbols, Symbols, Symbols, Definitions) {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut objects = Symbols::new();
        let mut locations = Symbols::new();
        for (s, id, name) in [
            ("caerula_berries", 30937, "Caerula berries"),
            ("damiana_leaves", 30977, "Damiana leaves"),
        ] {
            objects.insert(s.into(), id);
            let mut d = gielinor_cache::definitions::ItemType::decode(&[16, 0]).unwrap();
            d.name = name.into();
            defs.items.insert(id as u32, d);
        }
        for (s, id, name, op) in [
            ("caerula_bush", 56449, "Caerula bush", "Pick-from"),
            ("damiana_shrub", 56935, "Damiana", "Pick"),
        ] {
            locations.insert(s.into(), id);
            let mut d = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            d.name = name.into();
            d.ops[0] = Some(op.into());
            defs.locations.insert(id as u32, d);
        }
        (
            objects,
            locations,
            [("human_pickupfloor".into(), 827)].into(),
            [("pick".into(), 2581)].into(),
            defs,
        )
    }
    #[test]
    fn perpetual_pickables_bind_both_operations_and_exact_ordinary_products_without_food_or_timer_registration(
    ) {
        let (o, l, a, s, d) = fixture();
        let h = perpetual_pickables(&o, &l, &a, &s, &d).unwrap();
        assert_eq!(h.len(), 4);
        for (loc, item) in [(56449, 30937), (56935, 30977)] {
            for op in [1, 2] {
                let Handler::Pickable(r) = h[&(loc, op)] else {
                    panic!("wrong handler")
                };
                assert_eq!(
                    (r.item, r.members, r.animation, r.sound),
                    (item, true, 827, 2581)
                );
                assert!(h[&(loc, op)].arrival_delay());
            }
        }
        assert!(d.items[&30977].inventory_ops.iter().all(Option::is_none));
    }
    #[test]
    fn perpetual_pickables_reject_late_identity_member_geometry_transform_operation_and_presentation_mismatch(
    ) {
        for mode in 0..9 {
            let (mut o, mut l, mut a, mut s, mut d) = fixture();
            match mode {
                0 => {
                    o.insert("damiana_leaves".into(), 30937);
                }
                1 => d.items.get_mut(&30977).unwrap().members = false,
                2 => d.items.get_mut(&30977).unwrap().transform.template = 65534,
                3 => {
                    l.insert("damiana_shrub".into(), 56449);
                }
                4 => d.locations.get_mut(&56935).unwrap().ops[0] = Some("Chop down".into()),
                5 => d.locations.get_mut(&56935).unwrap().collision.width = 2,
                6 => d.locations.get_mut(&56935).unwrap().multi_varp = 1,
                7 => {
                    a.insert("human_pickupfloor".into(), 828);
                }
                _ => {
                    s.clear();
                }
            }
            assert!(perpetual_pickables(&o, &l, &a, &s, &d).is_err());
        }
    }
}
