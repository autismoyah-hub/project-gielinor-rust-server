//! Ordinary silver manufacture; quest-specific silver rows and enchanting are separate.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::Recipe;
const ITEMS: [(&str, i32, &str, bool, bool); 28] = [
    ("silver_bar", 2355, "Silver bar", false, false),
    ("nostringstar", 1714, "Unstrung symbol", false, false),
    ("holy_symbol_mould", 1599, "Holy mould", false, false),
    ("silver_sickle", 2961, "Silver sickle", true, false),
    ("sickle_mould", 2976, "Sickle mould", true, false),
    (
        "xbows_crossbow_bolts_silver_unfeathered",
        9382,
        "Silver bolts (unf)",
        true,
        true,
    ),
    ("xbows_silver_bolt_mould", 9434, "Bolt mould", true, false),
    ("tiara", 5525, "Tiara", false, false),
    ("tiara_mould", 5523, "Tiara mould", false, false),
    ("opal", 1609, "Opal", true, false),
    ("jade", 1611, "Jade", true, false),
    ("red_topaz", 1613, "Red topaz", true, false),
    ("ring_mould", 1592, "Ring mould", false, false),
    ("necklace_mould", 1597, "Necklace mould", false, false),
    ("jewl_bracelet_mould", 11065, "Bracelet mould", true, false),
    ("amulet_mould", 1595, "Amulet mould", false, false),
    ("opal_ring", 21081, "Opal ring", true, false),
    ("opal_necklace", 21090, "Opal necklace", true, false),
    ("opal_bracelet", 21117, "Opal bracelet", true, false),
    (
        "unstrung_opal_amulet",
        21099,
        "Opal amulet (u)",
        true,
        false,
    ),
    ("jade_ring", 21084, "Jade ring", true, false),
    ("jade_necklace", 21093, "Jade necklace", true, false),
    ("jade_bracelet", 21120, "Jade bracelet", true, false),
    (
        "unstrung_jade_amulet",
        21102,
        "Jade amulet (u)",
        true,
        false,
    ),
    ("topaz_ring", 21087, "Topaz ring", true, false),
    ("topaz_necklace", 21096, "Topaz necklace", true, false),
    ("topaz_bracelet", 21123, "Topaz bracelet", true, false),
    (
        "unstrung_topaz_amulet",
        21105,
        "Topaz amulet (u)",
        true,
        false,
    ),
];
pub fn silver_jewellery(
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
    for (symbol, id, name, member, stack) in ITEMS {
        if objects.get(symbol) != Some(&id) {
            return Err(format!("Missing/aliased silver jewellery item {symbol}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing silver jewellery item")?;
        if d.name != name
            || d.members != member
            || d.stackable != stack
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid silver jewellery item {id}"));
        }
    }
    Ok([
        (
            1714,
            None,
            &[1599][..],
            16,
            500,
            1,
            false,
            "unstrung symbol",
        ),
        (2961, None, &[2976][..], 18, 500, 1, true, "silver sickle"),
        (
            9382,
            None,
            &[9434][..],
            21,
            500,
            10,
            true,
            "unfinished silver bolts",
        ),
        (5525, None, &[5523][..], 23, 525, 1, false, "tiara"),
        (21081, Some(1609), &[1592][..], 1, 100, 1, true, "opal ring"),
        (
            21090,
            Some(1609),
            &[1597][..],
            16,
            350,
            1,
            true,
            "opal necklace",
        ),
        (
            21117,
            Some(1609),
            &[11065][..],
            22,
            450,
            1,
            true,
            "opal bracelet",
        ),
        (
            21099,
            Some(1609),
            &[1595][..],
            27,
            550,
            1,
            true,
            "opal amulet (u)",
        ),
        (
            21084,
            Some(1611),
            &[1592][..],
            13,
            320,
            1,
            true,
            "jade ring",
        ),
        (
            21093,
            Some(1611),
            &[1597][..],
            25,
            540,
            1,
            true,
            "jade necklace",
        ),
        (
            21120,
            Some(1611),
            &[11065][..],
            29,
            600,
            1,
            true,
            "jade bracelet",
        ),
        (
            21102,
            Some(1611),
            &[1595][..],
            34,
            700,
            1,
            true,
            "jade amulet (u)",
        ),
        (
            21087,
            Some(1613),
            &[1592][..],
            16,
            350,
            1,
            true,
            "topaz ring",
        ),
        (
            21096,
            Some(1613),
            &[1597][..],
            32,
            700,
            1,
            true,
            "topaz necklace",
        ),
        (
            21123,
            Some(1613),
            &[11065][..],
            38,
            750,
            1,
            true,
            "topaz bracelet",
        ),
        (
            21105,
            Some(1613),
            &[1595][..],
            45,
            800,
            1,
            true,
            "topaz amulet (u)",
        ),
    ]
    .into_iter()
    .map(
        |(
            output,
            secondary,
            required_inventory_tools,
            level,
            xp_tenths,
            output_count,
            members,
            label,
        )| Recipe {
            input: 2355,
            input_count: 1,
            secondary,
            byproduct: None,
            required_inventory_tools,
            output,
            output_count,
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
    fn silver_jewellery_sixteen_rewards_atomic_materials_and_durable_moulds() {
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
        for (symbol, id, name, member, stack) in ITEMS {
            objects.insert(symbol.into(), id);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.push(0);
            if member {
                b.push(16);
            }
            if stack {
                b.push(11);
            }
            b.push(0);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        let names = [("crafting".into(), 12)].into();
        let rows = silver_jewellery(&objects, &names, &defs).unwrap();
        assert_eq!(rows.len(), 16);
        assert!(rows.len() <= 18);
        for recipe in rows {
            let tool = recipe.required_inventory_tools[0];
            let mut inv = Inventory::new(28, StackType::Normal, false);
            for slot in 0..28 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: 2355,
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
            let mut slots = vec![19];
            if let Some(gem) = recipe.secondary {
                inv.restore_slot(
                    4,
                    Some(Item {
                        id: gem,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                slots.push(4);
            }
            let mut stats = Stats::default();
            stats.set_base(12, 1);
            stats.set_current(12, i8::try_from(recipe.level).unwrap());
            let cat = defs.inventory_catalog();
            let (next, ns, _) =
                facility_crafting::prepare_group(recipe, &slots, &inv, &cat, &stats, 125, true)
                    .unwrap();
            assert_eq!(next.get(3), inv.get(3));
            assert_eq!(
                next.slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == recipe.output)
                    .map(|i| i.count)
                    .sum::<i32>(),
                i32::from(recipe.output_count)
            );
            assert_eq!(ns.fine_xp(12), i32::from(recipe.xp_tenths) * 125 / 100);
            if recipe.members {
                assert!(facility_crafting::prepare_group(
                    recipe, &slots, &inv, &cat, &stats, 125, false
                )
                .is_err());
            } else {
                assert!(facility_crafting::prepare_group(
                    recipe, &slots, &inv, &cat, &stats, 125, false
                )
                .is_ok());
            }
            let before = (inv.clone(), stats.clone());
            let mut bad = defs.inventory_catalog();
            bad.dummy.insert(recipe.output);
            assert!(facility_crafting::prepare_group(
                recipe, &slots, &inv, &bad, &stats, 125, true
            )
            .is_err());
            assert_eq!((inv.clone(), stats.clone()), before);
            if let Some(gem) = recipe.secondary {
                inv.restore_slot(
                    4,
                    Some(Item {
                        id: gem,
                        count: 1,
                        vars: 9,
                    }),
                )
                .unwrap();
                assert!(facility_crafting::prepare_group(
                    recipe, &slots, &inv, &cat, &stats, 125, true
                )
                .is_err());
                inv = before.0.clone();
            }
            inv.restore_slot(3, None).unwrap();
            assert_eq!(recipe.available(&inv), 0);
            assert!(facility_crafting::prepare_group(
                recipe, &slots, &inv, &cat, &stats, 125, true
            )
            .is_err());
        }
        defs.items.get_mut(&9382).unwrap().stackable = false;
        assert!(silver_jewellery(&objects, &names, &defs).is_err());
    }
}
