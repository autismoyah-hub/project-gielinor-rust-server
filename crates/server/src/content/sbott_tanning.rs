//! Member Sbott price profile; Canifis access and dialogue remain separate.
use super::{Definitions, Symbols};
use gielinor_world::tanning::Recipe;
pub fn sbott_tanning(
    npcs: &Symbols,
    defs: &Definitions,
    standard: &[Recipe],
) -> Result<(u16, Vec<Recipe>), String> {
    let npc = defs.npcs.get(&6526).ok_or("Missing Sbott definition")?;
    if npcs.get("werewolftanner") != Some(&6526)
        || npc.name != "Sbott"
        || npc.size != 1
        || npc.ops[2].as_deref() != Some("Trade")
        || !npc.multi.is_empty()
        || npc.multi_default.is_some()
        || npc.multi_varbit.is_some()
        || npc.multi_varp.is_some()
    {
        return Err("Invalid static Sbott Trade definition".into());
    }
    let expected = [
        (1739, 1741, 1, 2),
        (1739, 1743, 3, 5),
        (7801, 6289, 20, 45),
        (6287, 6289, 15, 25),
        (1753, 1745, 20, 45),
        (1751, 2505, 20, 45),
        (1749, 2507, 20, 45),
        (1747, 2509, 20, 45),
    ];
    if standard.len() != expected.len() {
        return Err("Incomplete standard tanning dependency".into());
    }
    let mut recipes = Vec::new();
    for (row, (input, output, base, cost)) in standard.iter().zip(expected) {
        if row.input != input
            || row.output != output
            || row.cost != base
            || row.members != (input != 1739)
        {
            return Err("Conflicting standard tanning dependency".into());
        }
        recipes.push(Recipe {
            input,
            output,
            cost,
            members: true,
        });
    }
    Ok((6526, recipes))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Definitions, Symbols, Vec<Recipe>) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        d.npcs.insert(
            6526,
            gielinor_cache::npcs::NpcType::decode(b"\x02Sbott\0\x20Trade\0\0").unwrap(),
        );
        let rows = [
            (1739, 1741, 1),
            (1739, 1743, 3),
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
            members: input != 1739,
        })
        .collect();
        (d, [("werewolftanner".into(), 6526)].into(), rows)
    }
    #[test]
    fn sbott_tanning_attested_prices_static_npc_and_dependency_shape() {
        let (mut d, n, mut standard) = fixture();
        let (id, rows) = sbott_tanning(&n, &d, &standard).unwrap();
        assert_eq!(id, 6526);
        assert_eq!(
            rows.iter().map(|r| r.cost).collect::<Vec<_>>(),
            vec![2, 5, 45, 25, 45, 45, 45, 45]
        );
        assert!(rows.iter().all(|r| r.members));
        standard[0].cost = 3;
        assert!(sbott_tanning(&n, &d, &standard).is_err());
        standard[0].cost = 1;
        d.npcs.get_mut(&6526).unwrap().ops[2] = None;
        assert!(sbott_tanning(&n, &d, &standard).is_err());
    }
    #[test]
    fn sbott_tanning_reached_trade_all_eight_member_fees_and_live_refusals() {
        use gielinor_world::{
            inventory::Item,
            npc::{BlockWalk, Npc, Restrict},
            npc_interaction::{Event, Handler},
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
        let (d, n, standard) = fixture();
        let (kind, rows) = sbott_tanning(&n, &d, &standard).unwrap();
        for (index, row) in rows.iter().enumerate() {
            for denial in 0..3 {
                let mut w = World::default();
                w.collision.allocate(3200, 3200, 0);
                w.object_types.stackable.insert(995);
                w.tanning.insert(kind, rows.clone());
                w.npc_handlers.insert((kind, 3), Handler::Tanning);
                let mut def = gielinor_world::npc_options::Definition::default();
                def.operations[2] = Some("Trade".into());
                w.npc_options.types.insert(kind, def);
                let npc = w
                    .spawn_npc(
                        Npc::new(
                            kind,
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
                    "Sbott".into(),
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
                            count: row.cost * 2 - 1,
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
                let request = w.tanning_request(a, npc).unwrap();
                let before = w.data(a).unwrap().clone();
                if denial == 1 {
                    assert_eq!(request.recipes().count(), 0);
                    continue;
                }
                let token = w.start_quantity_dialog(a).unwrap();
                assert!(w.resume_quantity_dialog(a, token).unwrap());
                if denial == 2 {
                    w.set_membership(a, false).unwrap();
                }
                w.complete_tanning(a, request, index, 28).unwrap();
                let after = w.data(a).unwrap();
                assert_eq!(after.stats, before.stats);
                if denial == 0 {
                    assert_eq!(after.inventory.get(2).unwrap().count, row.cost - 1);
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
                } else {
                    assert_eq!(after.inventory, before.inventory);
                }
            }
        }
    }
}
