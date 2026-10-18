//! Normal-furnace cupronickel profile; no nickel mining or shipbuilding engine implied.
use super::{Definitions, Symbols};
use gielinor_world::smelting::Recipe;
pub fn cupronickel_smelting(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Recipe, String> {
    if stats.get("smithing") != Some(&13) {
        return Err("Missing Smithing stat".into());
    }
    let stat = defs
        .stats
        .get(&13)
        .ok_or("Missing Smithing stat definition")?;
    if stat.name != "Smithing" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Smithing stat".into());
    }
    for (symbol, id, name, members) in [
        ("nickel_ore", 31719, "Nickel ore", true),
        ("copper_ore", 436, "Copper ore", false),
        ("cupronickel_bar", 32892, "Cupronickel bar", true),
    ] {
        if objects.get(symbol).is_some_and(|v| *v != id) {
            return Err(format!("Conflicting cupronickel alias {symbol}"));
        }
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing cupronickel material definition")?;
        if item.name != name
            || item.members != members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid cupronickel material {id}"));
        }
    }
    Ok(Recipe {
        bar: 32892,
        label: "Cupronickel".into(),
        level: 74,
        xp: 42.0,
        ingredients: vec![(31719, 1), (436, 2)],
        impurity: false,
        members: true,
        repeat_delay: Some(6),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Inventory, Item, StackType},
        smithing_make::Outcome,
        stats::Stats,
    };
    struct NoDraw;
    impl gielinor_world::npc::RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Not iron")
        }
    }
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
        d.stats.insert(
            13,
            gielinor_cache::definitions::StatType::decode(b"\x01Smithing\0\0").unwrap(),
        );
        for (id, name, member) in [
            (31719, "Nickel ore", true),
            (436, "Copper ore", false),
            (32892, "Cupronickel bar", true),
        ] {
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            b.push(0);
            d.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        (d, [("smithing".into(), 13)].into())
    }
    #[test]
    fn cupronickel_smelting_definitions_and_boosted_current_two_material_exchange() {
        let (mut d, n) = fixture();
        let recipe = cupronickel_smelting(&Symbols::new(), &n, &d).unwrap();
        assert_eq!(
            (recipe.bar, recipe.level, recipe.xp, recipe.repeat_delay),
            (32892, 74, 42.0, Some(6))
        );
        let types = crate::smelting::item_types(&d.items);
        let cat = d.inventory_catalog();
        for refusal in 0..5 {
            let mut inv = Inventory::new(28, StackType::Normal, false);
            for slot in 0..28 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: match slot {
                            0 | 3 => 31719,
                            1 | 2 | 4 | 5 => 436,
                            _ => 1511,
                        },
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = Stats::default();
            stats.set_current(13, if refusal == 1 { 73 } else { 74 });
            let selected = recipe
                .select(&inv, &cat, &types, &stats, 13, 899, 2)
                .unwrap();
            if refusal == 1 {
                assert!(matches!(
                    selected,
                    gielinor_world::smelting::Selection::Denied(_)
                ));
                continue;
            }
            let gielinor_world::smelting::Selection::Start { request, .. } = selected else {
                panic!()
            };
            if refusal == 3 {
                inv.restore_slot(1, None).unwrap();
                inv.restore_slot(2, None).unwrap();
                inv.restore_slot(4, None).unwrap();
            }
            let mut credit = d.inventory_catalog();
            if refusal == 4 {
                credit.dummy.insert(32892);
            }
            let before = inv.clone();
            let mut job = request.start(2).unwrap();
            let mut made = 0;
            loop {
                match job
                    .complete_for_member(
                        &mut inv,
                        &credit,
                        &mut stats,
                        13,
                        1.25,
                        99,
                        refusal != 2,
                        &mut NoDraw,
                    )
                    .unwrap()
                {
                    Outcome::Made {
                        product: 32892,
                        count: 1,
                        next,
                        ..
                    } => {
                        made += 1;
                        if let Some(n) = next {
                            job = n;
                        } else {
                            break;
                        }
                    }
                    Outcome::RequirementsChanged | Outcome::MissingInputs | Outcome::NoSpace => {
                        break
                    }
                    other => panic!("Unexpected {other:?}"),
                }
            }
            if refusal == 0 {
                assert_eq!(made, 2);
                assert_eq!(stats.fine_xp(13), 1050);
                assert_eq!(
                    inv.slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 32892)
                        .count(),
                    2
                );
                assert!(!inv
                    .slots()
                    .iter()
                    .flatten()
                    .any(|i| i.id == 31719 || i.id == 436));
            } else {
                assert_eq!(made, 0);
                assert_eq!(stats.fine_xp(13), 0);
                // Legacy refunds may relocate slots, but must conserve ordinary counts.
                for id in [31719, 436, 1511, 32892] {
                    let total = |v: &Inventory| {
                        v.slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == id)
                            .map(|i| i.count)
                            .sum::<i32>()
                    };
                    assert_eq!(total(&inv), total(&before));
                }
            }
        }
        let wrong = [("cupronickel_bar".into(), 31719)].into();
        assert!(cupronickel_smelting(&wrong, &n, &d).is_err());
        d.items.get_mut(&32892).unwrap().cert.template = 799;
        assert!(cupronickel_smelting(&Symbols::new(), &n, &d).is_err());
    }
}
