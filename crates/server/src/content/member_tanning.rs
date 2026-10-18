//! Six ordinary member hide exchanges at the standard tanner price.
use super::{Definitions, Symbols};
use gielinor_world::tanning::Recipe;
pub(super) const ITEMS: [(&str, u32, &str); 11] = [
    ("templetrek_swamp_snake_hide", 7801, "Snake hide"),
    ("village_snake_hide", 6287, "Snake hide"),
    ("village_snake_skin", 6289, "Snakeskin"),
    ("dragonhide_green", 1753, "Green dragonhide"),
    ("dragon_leather", 1745, "Green dragon leather"),
    ("dragonhide_blue", 1751, "Blue dragonhide"),
    ("dragon_leather_blue", 2505, "Blue dragon leather"),
    ("dragonhide_red", 1749, "Red dragonhide"),
    ("dragon_leather_red", 2507, "Red dragon leather"),
    ("dragonhide_black", 1747, "Black dragonhide"),
    ("dragon_leather_black", 2509, "Black dragon leather"),
];
pub(super) fn member_tanning(objects: &Symbols, defs: &Definitions) -> Result<Vec<Recipe>, String> {
    for (symbol, id, name) in ITEMS {
        let item = defs
            .items
            .get(&id)
            .ok_or_else(|| format!("Missing member tanning item {id}"))?;
        if objects.get(symbol) != Some(&i32::try_from(id).map_err(|_| "Invalid member hide ID")?)
            || item.name != name
            || !item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid member tanning item {id}"));
        }
    }
    Ok([
        (7801, 6289, 20),
        (6287, 6289, 15),
        (1753, 1745, 20),
        (1751, 2505, 20),
        (1749, 2507, 20),
        (1747, 2509, 20),
    ]
    .into_iter()
    .map(|(input, output, cost)| Recipe {
        input,
        output,
        cost,
        members: true,
    })
    .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Definitions, Symbols) {
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
        for (symbol, id, name) in ITEMS {
            let mut raw = vec![2];
            raw.extend(name.bytes());
            raw.extend([0, 16, 0]);
            d.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
            o.insert(symbol.into(), id as i32);
        }
        (d, o)
    }
    #[test]
    fn member_tanning_exact_six_products_and_conflicting_definitions() {
        let (mut d, o) = fixture();
        let rows = member_tanning(&o, &d).unwrap();
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[0].cost, 20);
        assert_eq!(rows[1].cost, 15);
        assert!(rows.iter().all(|r| r.members));
        d.items.get_mut(&6289).unwrap().stackable = true;
        assert!(member_tanning(&o, &d).is_err());
    }
    #[test]
    fn member_tanning_six_actual_npc_choices_fee_clamping_and_live_member_loss() {
        use gielinor_world::{
            inventory::Item,
            npc::{BlockWalk, Npc, Restrict},
            npc_interaction::{Event, Handler},
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
        let (d, o) = fixture();
        let rows = member_tanning(&o, &d).unwrap();
        for (index, row) in rows.iter().enumerate() {
            for denial in 0..3 {
                let mut w = World::default();
                w.collision.allocate(3200, 3200, 0);
                w.object_types = d.inventory_catalog();
                w.object_types.stackable.insert(995);
                w.tanning.insert(3231, rows.clone());
                w.npc_handlers.insert((3231, 3), Handler::Tanning);
                let mut def = gielinor_world::npc_options::Definition::default();
                def.operations[2] = Some("Trade".into());
                w.npc_options.types.insert(3231, def);
                let npc = w
                    .spawn_npc(
                        Npc::new(
                            3231,
                            CoordGrid::new(3201, 3200, 0).unwrap(),
                            1,
                            Restrict::NoMove,
                            BlockWalk::None,
                        )
                        .unwrap(),
                    )
                    .unwrap();
                let mut data = PlayerData::new(
                    1,
                    "Member hides".into(),
                    CoordGrid::new(3200, 3200, 0).unwrap(),
                    Stats::default(),
                );
                for slot in 0..2 {
                    data.inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: row.input,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                data.inventory
                    .restore_slot(
                        2,
                        Some(Item {
                            id: 995,
                            count: row.cost + row.cost - 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                w.queue_login(1, data).unwrap();
                w.tick().unwrap();
                let a = w.actors().next().unwrap();
                w.set_membership(a, denial != 1).unwrap();
                w.drain_events();
                w.submit(
                    a,
                    Action::OperateNpc {
                        target: npc,
                        operation: 3,
                        control: false,
                    },
                )
                .unwrap();
                w.tick().unwrap();
                assert!(w.drain_events().iter().any(
                    |e| matches!(e,WorldEvent::NpcInteraction{event:Event::Tanning(n),..}if *n==npc)
                ));
                let req = w.tanning_request(a, npc).unwrap();
                let before = w.data(a).unwrap().clone();
                if denial == 1 {
                    assert_eq!(req.recipes().count(), 0);
                    continue;
                }
                let token = w.start_quantity_dialog(a).unwrap();
                assert!(w.resume_quantity_dialog(a, token).unwrap());
                if denial == 2 {
                    w.set_membership(a, false).unwrap();
                }
                w.complete_tanning(a, req, index, 28).unwrap();
                let after = w.data(a).unwrap();
                assert_eq!(after.stats, before.stats);
                if denial == 0 {
                    assert_eq!(
                        after
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == row.output)
                            .count(),
                        1
                    );
                    assert_eq!(after.inventory.get(2).unwrap().count, row.cost - 1);
                    assert_eq!(
                        after
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == row.input)
                            .count(),
                        1
                    );
                } else {
                    assert_eq!(after.inventory, before.inventory);
                }
            }
        }
    }
}
