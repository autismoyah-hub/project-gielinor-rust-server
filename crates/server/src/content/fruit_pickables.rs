//! Wild fruit stock cycles; Farming trees and fruit consumption are separate.
use super::{Definitions, Symbols};
use gielinor_world::{
    location_content::Handler,
    pickables::{Rule, Step},
};
use std::collections::BTreeMap;

const NODES: [(&str, u32, &str, &str); 13] = [
    ("bananatreefull", 2073, "Banana tree", "Pick"),
    ("bananatreefour", 2074, "Banana tree", "Pick"),
    ("bananatreethree", 2075, "Banana tree", "Pick"),
    ("bananatreetwo", 2076, "Banana tree", "Pick"),
    ("bananatreeone", 2077, "Banana tree", "Pick"),
    ("bananatreeempty", 2078, "Banana tree", "Search"),
    ("pineapple_plant", 1408, "Pineapple Plant", "Pick"),
    ("pineapple_plant_four", 1409, "Pineapple Plant", "Pick"),
    ("pineapple_plant_three", 1410, "Pineapple Plant", "Pick"),
    ("pineapple_plant_two", 1411, "Pineapple Plant", "Pick"),
    ("pineapple_plant_one", 1412, "Pineapple Plant", "Pick"),
    (
        "pineapple_plant_no_pineapples",
        1413,
        "Pineapple Plant",
        "Pick",
    ),
    ("mm_pineapple_plant_one", 4827, "Pineapple Plant", "Pick"),
];
const PRODUCTS: [(&str, i32, &str, bool, usize); 2] = [
    ("banana", 1963, "Banana", false, 0),
    ("pineapple", 2114, "Pineapple", true, 3),
];
pub fn fruit_pickables(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if sequences.get("human_pickupfloor") != Some(&827) || sounds.get("pick") != Some(&2581) {
        return Err("Unattested wild fruit pickup presentation".into());
    }
    for (symbol, id, name, members, op) in PRODUCTS {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || d.members != members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops[op].as_deref() != Some("Eat")
            })
        {
            return Err(format!("Unattested wild fruit product {symbol}"));
        }
    }
    for (symbol, id, name, operation) in NODES {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != name
                    || d.ops[0].as_deref() != Some(operation)
                    || d.ops[1..].iter().any(Option::is_some)
                    || d.collision.width != 2
                    || d.collision.length != 2
                    || !d.multi.is_empty()
                    || d.multi_default != -1
                    || d.multi_varbit != -1
                    || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested wild fruit stage {symbol}"));
        }
    }
    let mut handlers = BTreeMap::new();
    for (stages, empty, item, members, picked, full_message) in [
        (
            &[2073, 2074, 2075, 2076, 2077][..],
            2078,
            1963,
            false,
            "You pick some banana.",
            "You can't carry any more banana.",
        ),
        (
            &[1408, 1409, 1410, 1411, 1412][..],
            1413,
            2114,
            true,
            "You pick some pineapple.",
            "You can't carry any more pineapple.",
        ),
        (
            &[4827][..],
            1413,
            2114,
            true,
            "You pick some pineapple.",
            "You can't carry any more pineapple.",
        ),
    ] {
        let rule = Rule {
            item,
            members,
            animation: 827,
            sound: 2581,
            picked_message: picked,
            full_message,
        };
        for (index, &id) in stages.iter().enumerate() {
            let step = stages.get(index + 1).map_or(
                Step::Respawn {
                    empty,
                    full: stages[0],
                    delay: 500,
                },
                |&location| Step::Next { location },
            );
            for operation in [1, 2] {
                if handlers
                    .insert((id, operation), Handler::PickableStage { rule, step })
                    .is_some()
                {
                    return Err("Duplicate wild fruit productive stage".into());
                }
            }
        }
    }
    // Both pineapple regions share the empty type. Restoration identity belongs
    // to the existing per-position timer, never to this message-only handler.
    for (id, message) in [
        (2078, "There is nothing to pick here at the moment."),
        (1413, "There are no pineapples left on this plant."),
    ] {
        for operation in [1, 2] {
            handlers.insert((id, operation), Handler::EmptyPickable(message));
        }
    }
    Ok(handlers)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Symbols, Symbols, Symbols, Symbols, Definitions) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let (mut o, mut l) = (Symbols::new(), Symbols::new());
        for (symbol, id, name, members, op) in PRODUCTS {
            o.insert(symbol.into(), id);
            let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            t.name = name.into();
            t.members = members;
            t.inventory_ops[op] = Some("Eat".into());
            d.items.insert(id as u32, t);
        }
        for (symbol, id, name, op) in NODES {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = name.into();
            t.ops[0] = Some(op.into());
            t.collision.width = 2;
            t.collision.length = 2;
            d.locations.insert(id, t);
        }
        (
            o,
            l,
            [("human_pickupfloor".into(), 827)].into(),
            [("pick".into(), 2581)].into(),
            d,
        )
    }
    #[test]
    fn wild_fruit_bindings_are_exact_atomic_and_region_specific() {
        let (o, l, a, s, d) = fixture();
        let h = fruit_pickables(&o, &l, &a, &s, &d).unwrap();
        assert_eq!(h.len(), 26);
        for (last, empty, full, item, member) in [
            (2077, 2078, 2073, 1963, false),
            (1412, 1413, 1408, 2114, true),
            (4827, 1413, 4827, 2114, true),
        ] {
            for op in [1, 2] {
                let Handler::PickableStage { rule, step } = h[&(last, op)] else {
                    panic!("stage")
                };
                assert_eq!(
                    (rule.item, rule.members, step),
                    (
                        item,
                        member,
                        Step::Respawn {
                            empty,
                            full,
                            delay: 500
                        }
                    )
                );
            }
        }
        for mode in 0..8 {
            let (mut o, mut l, mut a, mut s, mut d) = fixture();
            match mode {
                0 => {
                    o.insert("banana".into(), 2114);
                }
                1 => d.items.get_mut(&2114).unwrap().members = false,
                2 => d.items.get_mut(&1963).unwrap().cert.template = 1,
                3 => {
                    l.insert("mm_pineapple_plant_one".into(), 1408);
                }
                4 => d.locations.get_mut(&4827).unwrap().multi_varp = 1,
                5 => d.locations.get_mut(&1413).unwrap().collision.width = 1,
                6 => a.clear(),
                _ => s.clear(),
            };
            assert!(fruit_pickables(&o, &l, &a, &s, &d).is_err());
        }
    }
    #[test]
    fn wild_fruit_actual_dispatch_depletes_five_five_one_and_restores_correct_root_after500() {
        use gielinor_world::{
            locations::Location,
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
        for (stages, empty, item, member) in [
            (&[2073, 2074, 2075, 2076, 2077][..], 2078, 1963, false),
            (&[1408, 1409, 1410, 1411, 1412][..], 1413, 2114, true),
            (&[4827][..], 1413, 2114, true),
        ] {
            let (o, l, a, s, d) = fixture();
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            for (&id, t) in &d.locations {
                w.location_types.insert(id, t.collision);
            }
            w.location_handlers = fruit_pickables(&o, &l, &a, &s, &d).unwrap();
            let full =
                Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), stages[0], 10, 0).unwrap();
            w.locations.load(full);
            gielinor_world::loc_collision::toggle(
                &mut w.collision,
                full.coords,
                10,
                0,
                d.locations[&full.id].collision,
                true,
            )
            .unwrap();
            w.queue_login(
                1,
                PlayerData::new(
                    1,
                    "Fruit".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                ),
            )
            .unwrap();
            w.tick().unwrap();
            let actor = w.actors().next().unwrap();
            w.set_membership(actor, member).unwrap();
            let stats = w.data(actor).unwrap().stats.clone();
            w.drain_events();
            for (index, &id) in stages.iter().enumerate() {
                w.submit(
                    actor,
                    Action::OperateLocation {
                        coords: full.coords,
                        id,
                        operation: if index % 2 == 0 { 1 } else { 2 },
                        control: false,
                    },
                )
                .unwrap();
                for _ in 0..6 {
                    w.tick().unwrap();
                    if w.locations.find_type(full.coords, id).is_none() {
                        break;
                    }
                }
                assert_eq!(
                    w.locations.find_shape(full.coords, 10).unwrap().id,
                    stages.get(index + 1).copied().unwrap_or(empty)
                );
            }
            assert_eq!(
                w.data(actor)
                    .unwrap()
                    .inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == item)
                    .count(),
                stages.len()
            );
            assert_eq!(w.data(actor).unwrap().stats, stats);
            let traces: Vec<_> = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::PickableAdvanced { trace, .. } = e {
                        Some(trace)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(traces.len(), stages.len());
            assert!(traces[..traces.len() - 1]
                .iter()
                .all(|t| t.restore_at.is_none()));
            let last = traces.last().unwrap();
            assert_eq!(last.restore, Some(full));
            assert_eq!(last.restore_at, Some(last.tick + 500));
            w.submit(
                actor,
                Action::OperateLocation {
                    coords: full.coords,
                    id: empty,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            w.tick().unwrap();
            assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::LocationMessage{message,..}if message.contains("nothing to pick")||message.contains("no pineapples"))));
            for _ in 0..498 {
                w.tick().unwrap();
                assert_eq!(w.locations.find_shape(full.coords, 10).unwrap().id, empty);
            }
            w.tick().unwrap();
            assert_eq!(w.locations.find_shape(full.coords, 10), Some(full));
            assert_eq!(
                w.data(actor)
                    .unwrap()
                    .inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == item)
                    .count(),
                stages.len()
            );
        }
    }
}
