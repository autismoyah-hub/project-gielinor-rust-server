//! Wild two-berry cycles, separate from Farming bushes and the tailored quest bush.
use super::{Definitions, Symbols};
use gielinor_world::{
    location_content::Handler,
    pickables::{Rule, Step},
};
use std::collections::BTreeMap;
pub fn cyclic_berry_pickables(
    objects: &Symbols,
    locations: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if sequences.get("human_pickupfloor") != Some(&827) || sounds.get("pick") != Some(&2581) {
        return Err("Unattested berry pickup presentation".into());
    }
    for (symbol, id, name) in [
        ("cadavaberries", 753, "Cadava berries"),
        ("redberries", 1951, "Redberries"),
    ] {
        if objects.get(symbol) != Some(&id)
            || defs.items.get(&(id as u32)).is_none_or(|d| {
                d.name != name
                    || d.members
                    || d.stackable
                    || d.dummy == 1
                    || d.cert.template != 0
                    || d.placeholder.template != 0
                    || d.transform.template != 0
                    || d.inventory_ops.iter().any(Option::is_some)
            })
        {
            return Err(format!("Unattested wild berry {symbol}"));
        }
    }
    for (prefix, ids, name) in [
        (
            "fai_varrock_cadavabush",
            [23625, 23626, 23627],
            "Cadava bush",
        ),
        (
            "fai_varrock_redberrybush",
            [23628, 23629, 23630],
            "Redberry bush",
        ),
    ] {
        for (stage, id) in [2, 1, 0].into_iter().zip(ids) {
            let symbol = format!("{prefix}_{stage}");
            if locations.get(&symbol) != Some(&id)
                || defs.locations.get(&(id as u32)).is_none_or(|d| {
                    d.name != name
                        || d.ops[0].as_deref() != Some("Pick-from")
                        || d.ops[1..].iter().any(Option::is_some)
                        || d.collision.width != 1
                        || d.collision.length != 1
                        || !d.multi.is_empty()
                        || d.multi_default != -1
                        || d.multi_varbit != -1
                        || d.multi_varp != -1
                })
            {
                return Err(format!("Unattested wild berry stage {symbol}"));
            }
        }
    }
    let mut handlers = BTreeMap::new();
    for (full, one, empty, item, picked, bagfull) in [
        (
            23625,
            23626,
            23627,
            753,
            "You pick some cadava berries.",
            "You can't carry any more cadava berries.",
        ),
        (
            23628,
            23629,
            23630,
            1951,
            "You pick some redberries.",
            "You can't carry any more redberries.",
        ),
    ] {
        let rule = Rule {
            item,
            members: false,
            animation: 827,
            sound: 2581,
            picked_message: picked,
            full_message: bagfull,
        };
        for op in [1, 2] {
            handlers.insert(
                (full, op),
                Handler::PickableStage {
                    rule,
                    step: Step::Next { location: one },
                },
            );
            handlers.insert(
                (one, op),
                Handler::PickableStage {
                    rule,
                    step: Step::Respawn {
                        empty,
                        full,
                        delay: 200,
                    },
                },
            );
            handlers.insert(
                (empty, op),
                Handler::EmptyPickable("There are no berries on this bush at the moment."),
            );
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
        let mut o = Symbols::new();
        let mut l = Symbols::new();
        for (s, id, name) in [
            ("cadavaberries", 753, "Cadava berries"),
            ("redberries", 1951, "Redberries"),
        ] {
            o.insert(s.into(), id);
            let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            t.name = name.into();
            d.items.insert(id as u32, t);
        }
        for (prefix, ids, name) in [
            (
                "fai_varrock_cadavabush",
                [23625, 23626, 23627],
                "Cadava bush",
            ),
            (
                "fai_varrock_redberrybush",
                [23628, 23629, 23630],
                "Redberry bush",
            ),
        ] {
            for (stage, id) in [2, 1, 0].into_iter().zip(ids) {
                l.insert(format!("{prefix}_{stage}"), id);
                let mut t = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
                t.name = name.into();
                t.ops[0] = Some("Pick-from".into());
                d.locations.insert(id as u32, t);
            }
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
    fn cyclic_berry_bindings_validate_six_wild_stages_two_f2p_products_and_full_root200() {
        let (o, l, a, s, d) = fixture();
        let h = cyclic_berry_pickables(&o, &l, &a, &s, &d).unwrap();
        assert_eq!(h.len(), 12);
        for (full, one, empty, item) in [(23625, 23626, 23627, 753), (23628, 23629, 23630, 1951)] {
            for op in [1, 2] {
                let Handler::PickableStage { rule, step } = h[&(full, op)] else {
                    panic!("missing full")
                };
                assert_eq!(
                    (rule.item, rule.members, step),
                    (item, false, Step::Next { location: one })
                );
                let Handler::PickableStage { step, .. } = h[&(one, op)] else {
                    panic!("missing last")
                };
                assert_eq!(
                    step,
                    Step::Respawn {
                        empty,
                        full,
                        delay: 200
                    }
                );
                assert!(matches!(h[&(empty, op)], Handler::EmptyPickable(_)));
            }
        }
        assert!(!h.keys().any(|(id, _)| *id == 33183));
    }
    #[test]
    fn cyclic_berry_bindings_refuse_farming_alias_member_templates_dynamic_or_wrong_stage() {
        for mode in 0..8 {
            let (mut o, mut l, mut a, mut s, mut d) = fixture();
            match mode {
                0 => {
                    o.insert("redberries".into(), 753);
                }
                1 => d.items.get_mut(&1951).unwrap().members = true,
                2 => d.items.get_mut(&1951).unwrap().stackable = true,
                3 => {
                    l.insert("fai_varrock_redberrybush_0".into(), 7701);
                }
                4 => d.locations.get_mut(&23630).unwrap().multi_varbit = 1,
                5 => d.locations.get_mut(&23630).unwrap().ops[0] = None,
                6 => {
                    a.clear();
                }
                _ => {
                    s.clear();
                }
            }
            assert!(cyclic_berry_pickables(&o, &l, &a, &s, &d).is_err());
        }
    }
    #[test]
    fn cyclic_berry_native_config_actual_two_picks_and200_tick_shared_restoration() {
        use gielinor_world::{
            locations::Location,
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
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
        w.location_handlers = cyclic_berry_pickables(&o, &l, &a, &s, &d).unwrap();
        let full = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 23628, 10, 0).unwrap();
        w.locations.load(full);
        gielinor_world::loc_collision::toggle(
            &mut w.collision,
            full.coords,
            full.shape,
            full.angle,
            d.locations[&23628].collision,
            true,
        )
        .unwrap();
        w.queue_login(
            1,
            PlayerData::new(
                1,
                "Nativeberry".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            ),
        )
        .unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        for id in [23628, 23629] {
            w.submit(
                actor,
                Action::OperateLocation {
                    coords: full.coords,
                    id,
                    operation: 1,
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
        }
        let count = w
            .data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 1951)
            .count();
        assert_eq!(count, 2);
        assert_eq!(
            w.locations.find_shape(full.coords, full.shape).unwrap().id,
            23630
        );
        let trace = w
            .drain_events()
            .into_iter()
            .find_map(|e| {
                if let WorldEvent::PickableAdvanced { trace, .. } = e {
                    trace.restore_at.map(|_| trace)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(trace.restore_at, Some(trace.tick + 200));
        assert_eq!(trace.restore.unwrap().id, 23628);
        for _ in 0..199 {
            w.tick().unwrap();
            assert_eq!(
                w.locations.find_shape(full.coords, full.shape).unwrap().id,
                23630
            );
        }
        w.tick().unwrap();
        assert_eq!(w.locations.find_shape(full.coords, full.shape), Some(full));
        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::LocationRestoration(t)if t.result=="restored"&&Some(t.tick)==trace.restore_at)));
    }
}
