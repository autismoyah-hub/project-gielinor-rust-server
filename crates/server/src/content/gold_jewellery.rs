//! Plain gold jewellery through the existing material-use furnace production queue.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::Recipe;
const ITEMS: [(&str, i32, &str, bool); 9] = [
    ("gold_bar", 2357, "Gold bar", false),
    ("gold_ring", 1635, "Gold ring", false),
    ("gold_necklace", 1654, "Gold necklace", false),
    ("jewl_gold_bracelet", 11069, "Gold bracelet", true),
    ("unstrung_gold_amulet", 1673, "Gold amulet (u)", false),
    ("ring_mould", 1592, "Ring mould", false),
    ("necklace_mould", 1597, "Necklace mould", false),
    ("jewl_bracelet_mould", 11065, "Bracelet mould", true),
    ("amulet_mould", 1595, "Amulet mould", false),
];
pub fn gold_jewellery(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<Recipe>, String> {
    if stats.get("crafting") != Some(&12) {
        return Err("Missing/aliased Crafting stat".into());
    }
    let stat = defs.stats.get(&12).ok_or("Missing Crafting stat")?;
    if stat.name != "Crafting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Crafting stat".into());
    }
    for (symbol, id, name, member) in ITEMS {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased gold jewellery item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing gold jewellery item")?;
        if d.name != name
            || d.members != member
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid gold jewellery item {id}"));
        }
    }
    Ok([
        (1635, 5, 150, false, &[1592][..], "gold ring"),
        (1654, 6, 200, false, &[1597][..], "gold necklace"),
        (11069, 7, 250, true, &[11065][..], "gold bracelet"),
        (1673, 8, 300, false, &[1595][..], "unstrung gold amulet"),
    ]
    .into_iter()
    .map(
        |(output, level, xp_tenths, members, required_inventory_tools, label)| Recipe {
            input: 2357,
            input_count: 1,
            secondary: None,
            byproduct: None,
            required_inventory_tools,
            output,
            output_count: 1,
            level,
            xp_tenths,
            members,
            first_ticks: 2,
            cycle_ticks: 3,
            failure: None,
            label,
        },
    )
    .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        facility_crafting,
        inventory::{Inventory, Item, StackType},
        stats::Stats,
    };
    #[test]
    fn gold_jewellery_four_rewards_full_bag_and_retained_mould_refusals() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            12,
            gielinor_cache::definitions::StatType::decode(b"\x01Crafting\0\0").unwrap(),
        );
        let mut objects = Symbols::new();
        for (symbol, id, name, member) in ITEMS {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        let names = [("crafting".into(), 12)].into();
        for recipe in gold_jewellery(&objects, &names, &defs).unwrap() {
            let tool = recipe.required_inventory_tools[0];
            let mut inv = Inventory::new(28, StackType::Normal, false);
            for slot in 0..28 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: 2357,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            inv.restore_slot(
                3,
                Some(Item {
                    id: tool,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_base(12, 1);
            stats.set_current(12, i8::try_from(recipe.level).unwrap());
            let cat = defs.inventory_catalog();
            let (next, ns, _) =
                facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, true).unwrap();
            assert_eq!(next.get(3), inv.get(3));
            assert_eq!(next.free_space(), 0);
            assert_eq!(
                next.slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == recipe.output)
                    .count(),
                1
            );
            assert_eq!(ns.fine_xp(12), i32::from(recipe.xp_tenths) * 125 / 100);
            if recipe.members {
                assert!(
                    facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, false).is_err()
                );
            } else {
                assert!(
                    facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, false).is_ok()
                );
            }
            let before = (inv.clone(), stats.clone());
            let mut bad = defs.inventory_catalog();
            bad.dummy.insert(recipe.output);
            assert!(facility_crafting::prepare(recipe, 19, &inv, &bad, &stats, 125, true).is_err());
            assert_eq!((inv.clone(), stats.clone()), before);
            inv.restore_slot(
                3,
                Some(Item {
                    id: tool,
                    count: 1,
                    vars: 1,
                }),
            )
            .unwrap();
            assert_eq!(recipe.available(&inv), 0);
            assert!(facility_crafting::prepare(recipe, 19, &inv, &cat, &stats, 125, true).is_err());
        }
        defs.items.get_mut(&1592).unwrap().cert.template = 799;
        assert!(gold_jewellery(&objects, &names, &defs).is_err());
    }
}
