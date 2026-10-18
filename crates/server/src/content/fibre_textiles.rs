//! Newer fibre items are present in the target cache but absent from its older
//! symbol file. Require exact ordinary definitions; reject any conflicting alias.
use super::{Definitions, Symbols};
use gielinor_world::facility_crafting::Recipe;

pub struct FibreTextiles {
    pub spinning: Vec<Recipe>,
    pub weaving: Vec<Recipe>,
}
pub fn fibre_textiles(objects: &Symbols, defs: &Definitions) -> Result<FibreTextiles, String> {
    if objects.get("flax") != Some(&1779) {
        return Err("Missing/aliased flax".into());
    }
    for (symbol, id, name) in [
        ("flax", 1779, "Flax"),
        ("hemp", 31457, "Hemp"),
        ("cotton_boll", 31460, "Cotton boll"),
        ("linen_yarn", 31463, "Linen yarn"),
        ("hemp_yarn", 31466, "Hemp yarn"),
        ("cotton_yarn", 31469, "Cotton yarn"),
        ("bolt_of_linen", 31472, "Bolt of linen"),
        ("bolt_of_canvas", 31475, "Bolt of canvas"),
        ("bolt_of_cotton", 31478, "Bolt of cotton"),
    ] {
        if objects.get(symbol).is_some_and(|&actual| actual != id) {
            return Err(format!("Conflicting textile alias {symbol}"));
        }
        let item = defs.items.get(&(id as u32)).ok_or("Missing textile item")?;
        if item.name != name
            || !item.members
            || item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.transform.template != 0
        {
            return Err(format!("Invalid textile item {id}"));
        }
    }
    let profile = |input, input_count, output, level, xp_tenths, first_ticks, label| Recipe {
        input,
        secondary: None,
        byproduct: None,
        input_count,
        output,
        output_count: 1,
        level,
        xp_tenths,
        members: true,
        first_ticks,
        cycle_ticks: 3,
        failure: None,
        label,
    };
    Ok(FibreTextiles {
        spinning: vec![
            profile(1779, 1, 31463, 12, 160, 3, "linen yarn"),
            profile(31457, 1, 31466, 39, 600, 3, "hemp yarn"),
            profile(31460, 1, 31469, 73, 1050, 3, "cotton yarn"),
        ],
        weaving: vec![
            // Wiki Make-all explicitly starts at four ticks, then repeats at three.
            profile(31463, 2, 31472, 12, 200, 4, "bolt of linen"),
            profile(31466, 2, 31475, 39, 750, 4, "bolt of canvas"),
            profile(31469, 2, 31478, 73, 1320, 4, "bolt of cotton"),
        ],
    })
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
    fn fibre_textiles_missing_aliases_require_exact_types_and_atomic_per_product_rewards() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (id, name) in [
            (1779, "Flax"),
            (31457, "Hemp"),
            (31460, "Cotton boll"),
            (31463, "Linen yarn"),
            (31466, "Hemp yarn"),
            (31469, "Cotton yarn"),
            (31472, "Bolt of linen"),
            (31475, "Bolt of canvas"),
            (31478, "Bolt of cotton"),
        ] {
            let mut bytes = vec![2];
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(&[0, 16, 0]);
            defs.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&bytes).unwrap(),
            );
        }
        let mut objects: Symbols = [("flax".into(), 1779)].into();
        let profiles = fibre_textiles(&objects, &defs).unwrap();
        assert_eq!((profiles.spinning.len(), profiles.weaving.len()), (3, 3));
        let cat = defs.inventory_catalog();
        for (index, (recipe, fine)) in profiles
            .spinning
            .iter()
            .chain(&profiles.weaving)
            .copied()
            .zip([200, 750, 1312, 250, 937, 1650])
            .enumerate()
        {
            assert_eq!(
                (recipe.first_ticks, recipe.cycle_ticks),
                (if index < 3 { 3 } else { 4 }, 3)
            );
            assert!(recipe.failure.is_none());
            let mut inv = Inventory::new(28, StackType::Normal, false);
            for slot in 0..28 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: 1511,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let slots: Vec<_> = (0..usize::from(recipe.input_count)).collect();
            for &slot in &slots {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id: recipe.input,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            let mut stats = Stats::default();
            stats.set_base(12, recipe.level as i8);
            stats.set_current(12, recipe.level as i8);
            let (next, reward, _) =
                facility_crafting::prepare_group(recipe, &slots, &inv, &cat, &stats, 125, true)
                    .unwrap();
            assert_eq!(reward.fine_xp(12), fine);
            assert_eq!(next.free_space(), usize::from(recipe.input_count) - 1);
            assert_eq!(
                next.slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == recipe.output)
                    .count(),
                1
            );
            let before = (inv.clone(), stats.clone());
            assert!(facility_crafting::prepare_group(
                recipe, &slots, &inv, &cat, &stats, 125, false
            )
            .is_err());
            stats.set_current(12, recipe.level as i8 - 1);
            assert!(facility_crafting::prepare_group(
                recipe, &slots, &inv, &cat, &stats, 125, true
            )
            .is_err());
            stats = before.1.clone();
            let mut bad = defs.inventory_catalog();
            bad.dummy.insert(recipe.output);
            assert!(facility_crafting::prepare_group(
                recipe, &slots, &inv, &bad, &stats, 125, true
            )
            .is_err());
            assert!(facility_crafting::prepare_group(
                recipe,
                &slots[..slots.len() - 1],
                &inv,
                &cat,
                &stats,
                125,
                true
            )
            .is_err());
            assert_eq!((inv, stats), before);
        }
        objects.insert("hemp".into(), 31457);
        assert!(fibre_textiles(&objects, &defs).is_ok());
        objects.insert("hemp".into(), 1);
        assert!(fibre_textiles(&objects, &defs).is_err());
        objects.remove("hemp");
        defs.items.get_mut(&31475).unwrap().cert.template = 799;
        assert!(fibre_textiles(&objects, &defs).is_err());
        defs.items.get_mut(&31475).unwrap().cert.template = 0;
        defs.items.remove(&31478);
        assert!(fibre_textiles(&objects, &defs).is_err());
    }
}
