//! Native empty-search content tags; quest loot and specialised boxes are separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::location_content::Handler;
use std::collections::{BTreeMap, BTreeSet};
const DEFAULT: &str = "You find nothing of interest.";
const NODES: [(&str, u32, &str, i32); 7] = [
    ("crate3", 355, "Crate", 13),
    ("crate2_old", 357, "Crate", 13),
    ("crate3_old", 358, "Crate", 13),
    ("boxes", 359, "Boxes", 15),
    ("sacks", 365, "Sacks", 16),
    ("crate", 366, "Crate", 13),
    ("qip_cook_crate_stacked", 12963, "Crate", 13),
];
pub fn location_content_search(
    locations: &Symbols,
    contents: &Symbols,
    params: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if contents.get("empty_crate") != Some(&13)
        || contents.get("empty_boxes") != Some(&15)
        || contents.get("empty_sacks") != Some(&16)
        || params.get("game_message") != Some(&65529)
    {
        return Err("Unattested empty search content/parameter symbols".into());
    }
    let actual: BTreeSet<_> = defs
        .locations
        .iter()
        .filter(|(_, d)| [13, 15, 16].contains(&d.content_group))
        .map(|(&id, _)| id)
        .collect();
    if actual != NODES.iter().map(|v| v.1).collect() {
        return Err("Unattested empty-search node set".into());
    }
    let mut result = BTreeMap::new();
    for (symbol, id, name, group) in NODES {
        let node = defs.locations.get(&id).ok_or("Missing empty search node")?;
        if locations.get(symbol) != Some(&(id as i32))
            || node.name != name
            || node.content_group != group
            || node.ops[0].as_deref() != Some("Search")
            || node.ops[1..].iter().any(Option::is_some)
            || node.collision.width != 1
            || node.collision.length != 1
            || !node.multi.is_empty()
            || node.multi_default != -1
            || node.multi_varbit != -1
            || node.multi_varp != -1
        {
            return Err(format!("Unattested empty-search node {symbol}"));
        }
        let message = match node.params.get(&65529) {
            None => DEFAULT,
            Some(Param::String(s)) => match s.as_str() {
                "You search the crate but find nothing." => {
                    "You search the crate but find nothing."
                }
                "There is nothing interesting in these boxes." => {
                    "There is nothing interesting in these boxes."
                }
                "There is nothing interesting in these sacks." => {
                    "There is nothing interesting in these sacks."
                }
                DEFAULT => DEFAULT,
                _ => return Err(format!("Unattested empty-search message for {id}")),
            },
            Some(_) => return Err(format!("Invalid empty-search message type for {id}")),
        };
        result.insert((id, 1), Handler::Message(message));
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Symbols, Symbols, Symbols, Definitions) {
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
        for (symbol, id, name, group) in NODES {
            l.insert(symbol.into(), id as i32);
            let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            t.name = name.into();
            t.ops[0] = Some("Search".into());
            t.content_group = group;
            t.params.insert(
                65529,
                Param::String(
                    match group {
                        13 => "You search the crate but find nothing.",
                        15 => "There is nothing interesting in these boxes.",
                        _ => "There is nothing interesting in these sacks.",
                    }
                    .into(),
                ),
            );
            d.locations.insert(id, t);
        }
        (
            l,
            [
                ("empty_crate".into(), 13),
                ("empty_boxes".into(), 15),
                ("empty_sacks".into(), 16),
            ]
            .into(),
            [("game_message".into(), 65529)].into(),
            d,
        )
    }
    #[test]
    fn location_content_search_exact_nodes_default_and_unknown_or_malformed_refusal() {
        let (l, c, p, mut d) = fixture();
        let h = location_content_search(&l, &c, &p, &d).unwrap();
        assert_eq!(h.len(), 7);
        for (_, id, _, _) in NODES {
            assert!(matches!(h[&(id, 1)], Handler::Message(_)));
            assert!(!h.contains_key(&(id, 2)));
        }
        d.locations.get_mut(&12963).unwrap().params.remove(&65529);
        assert!(matches!(
            location_content_search(&l, &c, &p, &d).unwrap()[&(12963, 1)],
            Handler::Message(DEFAULT)
        ));
        for mode in 0..6 {
            let (mut l, c, p, mut d) = fixture();
            match mode {
                0 => {
                    l.insert("qip_cook_crate_stacked".into(), 366);
                }
                1 => d.locations.get_mut(&12963).unwrap().content_group = 14,
                2 => {
                    d.locations
                        .get_mut(&12963)
                        .unwrap()
                        .params
                        .insert(65529, Param::Integer(7));
                }
                3 => {
                    d.locations
                        .get_mut(&12963)
                        .unwrap()
                        .params
                        .insert(65529, Param::String("Unknown quest reward".into()));
                }
                4 => d.locations.get_mut(&12963).unwrap().multi_varbit = 1,
                _ => {
                    d.locations.get_mut(&12963).unwrap().ops[0] = Some("Loot".into());
                }
            }
            assert!(location_content_search(&l, &c, &p, &d).is_err());
        }
    }
    #[test]
    fn location_content_search_actual_reached_dispatch_only_messages_without_stock_inventory_xp_or_rng(
    ) {
        use gielinor_world::{
            locations::Location,
            npc::RandomSource,
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
        struct Never;
        impl RandomSource for Never {
            fn below(&mut self, _: u32) -> u32 {
                panic!("empty search must not draw randomness")
            }
        }
        for (_, id, _, _) in NODES {
            let (l, c, p, d) = fixture();
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            for (&id, t) in &d.locations {
                w.location_types.insert(id, t.collision);
            }
            w.location_handlers = location_content_search(&l, &c, &p, &d).unwrap();
            assert!(w.has_location_handler(id, 1));
            assert!(!w.has_location_handler(id, 2));
            let node = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), id, 10, 0).unwrap();
            w.locations.load(node);
            w.queue_login(
                1,
                PlayerData::new(
                    1,
                    "Search".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                ),
            )
            .unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            let before = w.data(a).unwrap().inventory.clone();
            let stats = w.data(a).unwrap().stats.clone();
            let rev = w.locations.revision(node);
            w.drain_events();
            w.submit(
                a,
                Action::OperateLocation {
                    coords: node.coords,
                    id,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
            w.tick_with_random(&mut Never).unwrap();
            let e = w.drain_events();
            assert_eq!(
                e.iter()
                    .filter(|e| matches!(e,WorldEvent::LocationMessage{actor,..}if *actor==a))
                    .count(),
                1
            );
            assert!(!e.iter().any(|e| matches!(
                e,
                WorldEvent::PickableCollected { .. }
                    | WorldEvent::PlayerAnimation { .. }
                    | WorldEvent::PlayerSound { .. }
            )));
            assert_eq!(w.data(a).unwrap().inventory, before);
            assert_eq!(w.data(a).unwrap().stats, stats);
            assert_eq!(w.locations.revision(node), rev);
            w.tick_with_random(&mut Never).unwrap();
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::LocationMessage { .. })));
        }
    }
}
