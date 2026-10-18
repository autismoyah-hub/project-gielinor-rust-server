//! Held Imcando hammer swaps. Worn swapping and acquisition are separate.
use super::{Definitions, Symbols};
use gielinor_world::held_item_conversion::Rule;
pub fn imcando_held_swaps(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<i32, Rule>, String> {
    for (symbol, id, name, wear) in [
        ("imcando_hammer", 25644, "Imcando hammer", 3),
        (
            "imcando_hammer_offhand",
            29775,
            "Imcando hammer (off-hand)",
            5,
        ),
    ] {
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing Imcando hammer")?;
        if objects.get(symbol) != Some(&id)
            || item.name != name
            || !item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
            || item.inventory_ops[2].as_deref() != Some("Swap")
            || item.wear[0] != wear
        {
            return Err(format!("Invalid Imcando hammer Swap definition {id}"));
        }
    }
    // The source held handlers add no entitlement or skill gate. Item membership
    // and the separate equipment permission must not become a fabricated swap gate.
    Ok([(25644, 29775), (29775, 25644)]
        .into_iter()
        .map(|(input, output)| {
            (
                input,
                Rule {
                    output,
                    members: false,
                    button_operation: 4,
                    message: None,
                },
            )
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
        for (id, name, wear) in [
            (25644, "Imcando hammer", 3),
            (29775, "Imcando hammer (off-hand)", 5),
        ] {
            let mut raw = vec![2];
            raw.extend(name.bytes());
            raw.extend([0, 16, 13, wear, 36]);
            raw.extend(b"Wield\0\x25Swap\0\0");
            d.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        (
            d,
            [
                ("imcando_hammer".into(), 25644),
                ("imcando_hammer_offhand".into(), 29775),
            ]
            .into(),
        )
    }
    #[test]
    fn imcando_held_swaps_validate_both_forms_and_exact_third_operation() {
        let (mut d, o) = fixture();
        let rules = imcando_held_swaps(&o, &d).unwrap();
        assert_eq!(rules[&25644].output, 29775);
        assert_eq!(rules[&29775].output, 25644);
        assert!(rules
            .values()
            .all(|r| r.button_operation == 4 && !r.members));
        d.items.get_mut(&29775).unwrap().inventory_ops[2] = None;
        assert!(imcando_held_swaps(&o, &d).is_err());
    }
    #[test]
    fn imcando_held_swaps_full_bag_selected_roundtrip_and_atomic_refusals() {
        use gielinor_world::{
            inventory::Item,
            simulation::{Action, PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
        let (d, o) = fixture();
        let rules = imcando_held_swaps(&o, &d).unwrap();
        for member in [false, true] {
            for (input, output) in [(25644, 29775), (29775, 25644)] {
                for denial in 0..3 {
                    let mut w = World::default();
                    w.collision.allocate(3200, 3200, 0);
                    w.object_types = d.inventory_catalog();
                    w.empty_containers = rules.clone();
                    if denial == 2 {
                        w.object_types.dummy.insert(output);
                    }
                    let mut data = PlayerData::new(
                        1,
                        "Hammer swap".into(),
                        CoordGrid::new(3200, 3200, 0).unwrap(),
                        Stats::default(),
                    );
                    for slot in 0..28 {
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
                    for slot in [4, 19] {
                        data.inventory
                            .restore_slot(
                                slot,
                                Some(Item {
                                    id: input,
                                    count: 1,
                                    vars: if denial == 1 && slot == 19 { 7 } else { 0 },
                                }),
                            )
                            .unwrap();
                    }
                    w.queue_login(1, data).unwrap();
                    w.tick().unwrap();
                    let a = w.actors().next().unwrap();
                    w.set_membership(a, member).unwrap();
                    let before = w.data(a).unwrap().clone();
                    w.drain_events();
                    w.submit(
                        a,
                        Action::ConvertHeldItem {
                            slot: 19,
                            item: input,
                        },
                    )
                    .unwrap();
                    w.tick().unwrap();
                    let after = w.data(a).unwrap();
                    assert_eq!(after.stats, before.stats);
                    assert_eq!(after.equipment, before.equipment);
                    if denial == 0 {
                        assert_eq!(after.inventory.get(19).unwrap().id, output);
                        assert_eq!(after.inventory.get(4).unwrap().id, input);
                        assert_eq!(after.inventory.slots().iter().flatten().count(), 28);
                    } else {
                        assert_eq!(after.inventory, before.inventory);
                    }
                    assert!(!w
                        .drain_events()
                        .iter()
                        .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
                    if denial == 0 {
                        w.submit(
                            a,
                            Action::ConvertHeldItem {
                                slot: 19,
                                item: output,
                            },
                        )
                        .unwrap();
                        w.tick().unwrap();
                        assert_eq!(
                            w.data(a).unwrap().inventory.slots(),
                            before.inventory.slots()
                        );
                    }
                }
            }
        }
    }
}
