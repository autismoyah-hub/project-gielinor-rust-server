//! Granite coating only; cannonball smelting, cannon combat and dust acquisition are separate.
use super::{Definitions, Symbols};
pub fn granite_cannonballs(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::counted_assembly::Recipe, String> {
    if stats.get("smithing") != Some(&13) {
        return Err("Missing Smithing stat".into());
    }
    let stat = defs.stats.get(&13).ok_or("Missing Smithing definition")?;
    if stat.name != "Smithing" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Smithing stat".into());
    }
    for (symbol, id, name) in [
        ("mcannonball", 2, "Steel cannonball"),
        ("granite_dust", 21726, "Granite dust"),
        ("granite_cannonball", 21728, "Granite cannonball"),
    ] {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased granite material {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing granite material")?;
        if d.name != name
            || !d.members
            || !d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid granite material {id}"));
        }
    }
    Ok(gielinor_world::counted_assembly::Recipe {
        first: 2,
        second: 21726,
        output: 21728,
        skill: 13,
        level: 50,
        members: true,
        message: "You apply a thick coating of granite dust to your steel cannonballs.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        simulation::{PlayerData, World, WorldEvent},
        stats::Stats,
        CoordGrid,
    };
    fn fixture() -> (Definitions, Symbols, Symbols) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        d.stats.insert(
            13,
            gielinor_cache::definitions::StatType::decode(b"\x01Smithing\0\0").unwrap(),
        );
        let mut o = Symbols::new();
        for (symbol, id, name) in [
            ("mcannonball", 2, "Steel cannonball"),
            ("granite_dust", 21726, "Granite dust"),
            ("granite_cannonball", 21728, "Granite cannonball"),
        ] {
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(b"\0\x0b\x10\0");
            d.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
            o.insert(symbol.into(), id);
        }
        (d, o, [("smithing".into(), 13)].into())
    }
    #[test]
    fn granite_cannonballs_actual_counted_held_flow_is_instant_zero_xp_and_one_use() {
        let (defs, objs, stats) = fixture();
        let recipe = granite_cannonballs(&objs, &stats, &defs).unwrap();
        for reverse in [false, true] {
            for scenario in 0..12 {
                let mut w = World::default();
                w.collision.allocate(3200, 3200, 0);
                w.object_types = defs.inventory_catalog();
                w.held_use_handlers
                    .register(Scope::Types, 2, 21726, Handler::CountedAssembly(recipe))
                    .unwrap();
                for id in [2, 21726] {
                    w.held_use_types.insert(id, Type { id, content: -1 });
                }
                let mut data = PlayerData::new(
                    1,
                    "Coater".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                );
                data.stats
                    .set_current(13, if scenario == 1 { 49 } else { 50 });
                data.xp_rate_hundreds = 0;
                for (slot, id, count) in [(4, 2, 100), (19, 21726, 80)] {
                    data.inventory
                        .restore_slot(slot, Some(Item { id, count, vars: 0 }))
                        .unwrap();
                }
                if scenario == 7 {
                    for slot in 0..28 {
                        if slot != 4 && slot != 19 {
                            data.inventory
                                .restore_slot(
                                    slot,
                                    Some(Item {
                                        id: 1511,
                                        count: 1,
                                        vars: 0,
                                    }),
                                )
                                .unwrap();
                        }
                    }
                }
                if scenario == 8 {
                    data.inventory
                        .restore_slot(
                            8,
                            Some(Item {
                                id: 21728,
                                count: i32::MAX,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                w.queue_login(1, data).unwrap();
                w.tick().unwrap();
                let a = w.actors().next().unwrap();
                w.set_membership(a, scenario != 2).unwrap();
                w.drain_events();
                let held = if reverse {
                    Request {
                        selected_slot: 19,
                        selected_id: 21726,
                        target_slot: 4,
                        target_id: 2,
                    }
                } else {
                    Request {
                        selected_slot: 4,
                        selected_id: 2,
                        target_slot: 19,
                        target_id: 21726,
                    }
                };
                w.prepare_held_use(a, held).unwrap();
                w.finish_held_use(a, held).unwrap();
                let Some(request) = w.take_counted_assembly_request(a).unwrap() else {
                    assert!(scenario == 1 || scenario == 2);
                    continue;
                };
                let token = w.start_number_dialog(a).unwrap();
                if scenario == 3 {
                    w.set_membership(a, false).unwrap();
                }
                if scenario == 4 {
                    w.data_mut(a).unwrap().stats.set_current(13, 49);
                }
                if scenario == 5 {
                    w.close_dialogs(a).unwrap();
                }
                if scenario == 6 {
                    w.object_types.dummy.insert(21728);
                }
                if scenario == 10 {
                    w.submit(
                        a,
                        gielinor_world::simulation::Action::MoveClick {
                            destination: CoordGrid::new(3202, 3200, 0).unwrap(),
                            control: false,
                        },
                    )
                    .unwrap();
                    w.tick().unwrap();
                    w.tick().unwrap();
                }
                if scenario == 11 {
                    let item = w.data(a).unwrap().inventory.get(4).unwrap();
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(4, None)
                        .unwrap();
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(5, Some(item))
                        .unwrap();
                }
                let before = w.data(a).unwrap().clone();
                let clock = w.clock.0;
                let count = if scenario == 9 { -1 } else { 7 };
                w.complete_counted_assembly(a, request, token, count)
                    .unwrap();
                assert_eq!(w.clock.0, clock);
                let after = w.data(a).unwrap();
                assert_eq!(after.stats, before.stats);
                if scenario == 0 {
                    assert_eq!(after.inventory.get(4).unwrap().count, 93);
                    assert_eq!(after.inventory.get(19).unwrap().count, 73);
                    assert_eq!(
                        after
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|v| v.id == 21728)
                            .map(|v| v.count)
                            .sum::<i32>(),
                        7
                    );
                    assert!(w
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, WorldEvent::CountedAssemblyMade { count: 7, .. })));
                } else {
                    assert_eq!(after.inventory, before.inventory);
                }
                let committed = w.data(a).unwrap().clone();
                w.complete_counted_assembly(a, request, token, 7).unwrap();
                assert_eq!(w.data(a).unwrap().inventory, committed.inventory);
            }
        }
    }
    #[test]
    fn granite_cannonballs_strict_types_fullbag_clamping_and_origin_vars() {
        let (mut d, o, n) = fixture();
        let recipe = granite_cannonballs(&o, &n, &d).unwrap();
        use gielinor_world::{
            counted_assembly,
            inventory::{Inventory, StackType},
        };
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..28 {
            inv.restore_slot(
                slot,
                Some(Item {
                    id: if slot == 4 {
                        2
                    } else if slot == 19 {
                        21726
                    } else {
                        1511
                    },
                    count: if slot == 4 {
                        100
                    } else if slot == 19 {
                        80
                    } else {
                        1
                    },
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let h = Request {
            selected_slot: 4,
            selected_id: 2,
            target_slot: 19,
            target_id: 21726,
        };
        let origin = CoordGrid::new(3200, 3200, 0).unwrap();
        let request = counted_assembly::Request::bind(recipe, h, &inv, origin, 0).unwrap();
        let mut s = Stats::default();
        s.set_current(13, 50);
        let cat = d.inventory_catalog();
        assert_eq!(request.complete(&mut inv, &cat, &s, true, i32::MAX), Ok(80));
        assert_eq!(inv.get(4).unwrap().count, 20);
        assert_eq!(inv.get(19).unwrap().id, 21728);
        inv.restore_slot(
            19,
            Some(Item {
                id: 21726,
                count: 1,
                vars: 37,
            }),
        )
        .unwrap();
        assert!(counted_assembly::Request::bind(recipe, h, &inv, origin, 0).is_none());
        d.items.get_mut(&21728).unwrap().placeholder.template = 1;
        assert!(granite_cannonballs(&o, &n, &d).is_err());
    }
}
