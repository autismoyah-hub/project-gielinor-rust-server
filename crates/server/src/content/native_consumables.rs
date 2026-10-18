//! Native category clocks for existing registered items. No new foods are registered.
use super::{Definitions, Edible, Symbols};
use gielinor_world::{
    consumable_clocks::{Kind, Policy, Rules},
    potions::{Potion, DRINK_DELAY},
};
use std::collections::BTreeMap;
// Binding facts independently checked in native239 symbols and raw definitions.
// Toads' legs uses its independently attested fourth Eat option; routing is cache-derived.
const SPECIAL: &[(&str, i32, &str, i32, i32, bool)] = &[
    ("toads_legs", 2152, "Toad's legs", 3, 2, true),
    ("king_worm", 2162, "King worm", 3, 2, true),
    ("toad_crunchies", 2217, "Toad crunchies", 3, 2, true),
    (
        "premade_toad_crunchies",
        2243,
        "Premade t'd crunch",
        3,
        2,
        true,
    ),
    ("spicy_crunchies", 2213, "Spicy crunchies", 3, 2, true),
    (
        "premade_spicy_crunchies",
        2241,
        "Premade s'y crunch",
        3,
        2,
        true,
    ),
    ("worm_crunchies", 2205, "Worm crunchies", 3, 2, true),
    (
        "premade_worm_crunchies",
        2237,
        "Premade w'm crun'",
        3,
        2,
        true,
    ),
    ("chocchip_crunchies", 2209, "Chocchip crunchies", 3, 2, true),
    (
        "premade_chocchip_crunchies",
        2239,
        "Premade ch' crunch",
        3,
        2,
        true,
    ),
    ("fruit_batta", 2277, "Fruit batta", 3, 2, true),
    (
        "premade_fruit_batta",
        2225,
        "Premade fr't batta",
        3,
        2,
        true,
    ),
    ("toad_batta", 2255, "Toad batta", 3, 2, true),
    ("premade_toad_batta", 2221, "Premade t'd batta", 3, 2, true),
    ("worm_batta", 2253, "Worm batta", 3, 2, true),
    ("premade_worm_batta", 2219, "Premade w'm batta", 3, 2, true),
    ("vegetable_batta", 2281, "Vegetable batta", 3, 2, true),
    (
        "premade_vegetable_batta",
        2227,
        "Premade veg batta",
        3,
        2,
        true,
    ),
    ("cheese+tom_batta", 2259, "Cheese+tom batta", 3, 2, true),
    (
        "premade_cheese+tom_batta",
        2223,
        "Premade c+t batta",
        3,
        2,
        true,
    ),
    ("worm_hole", 2191, "Worm hole", 3, 2, true),
    ("premade_worm_hole", 2233, "Premade worm hole", 3, 2, true),
    ("veg_ball", 2195, "Veg ball", 3, 2, true),
    ("premade_veg_ball", 2235, "Premade veg ball", 3, 2, true),
    ("chocolate_bomb", 2185, "Chocolate bomb", 3, 2, true),
    (
        "premade_chocolate_bomb",
        2229,
        "Premade choc bomb",
        3,
        2,
        true,
    ),
    (
        "tangled_toads_legs",
        2187,
        "Tangled toad's legs",
        3,
        2,
        true,
    ),
    (
        "premade_tangled_toads_legs",
        2231,
        "Premade ttl",
        3,
        2,
        true,
    ),
    (
        "gauntlet_combo_food",
        25960,
        "Crystal paddlefish",
        3,
        2,
        true,
    ),
    (
        "gauntlet_combo_food_hm",
        25958,
        "Corrupted paddlefish",
        3,
        2,
        true,
    ),
    (
        "tbwt_cooked_karambwan",
        3144,
        "Cooked karambwan",
        3,
        2,
        true,
    ),
    (
        "blighted_karambwan",
        24595,
        "Blighted karambwan",
        3,
        2,
        true,
    ),
    ("aluft_fruit_batta", 9527, "Fruit batta", 3, 2, true),
    ("aluft_toad_batta", 9529, "Toad batta", 3, 2, true),
    ("aluft_worm_batta", 9531, "Worm batta", 3, 2, true),
    ("aluft_vegetable_batta", 9533, "Vegetable batta", 3, 2, true),
    (
        "aluft_cheese+tom_batta",
        9535,
        "Cheese+tom batta",
        3,
        2,
        true,
    ),
    ("aluft_toad_crunchies", 9538, "Toad crunchies", 3, 2, true),
    ("aluft_spicy_crunchies", 9540, "Spicy crunchies", 3, 2, true),
    ("aluft_worm_crunchies", 9542, "Worm crunchies", 3, 2, true),
    (
        "aluft_chocchip_crunchies",
        9544,
        "Chocchip crunchies",
        3,
        2,
        true,
    ),
    ("aluft_worm_hole", 9547, "Worm hole", 3, 2, true),
    ("aluft_veg_ball", 9549, "Veg ball", 3, 2, true),
    (
        "aluft_tangled_toads_legs",
        9551,
        "Tangled toads' legs",
        3,
        2,
        true,
    ),
    ("aluft_chocolate_bomb", 9553, "Chocolate bomb", 3, 2, true),
    (
        "hundred_pirate_giant_crab_meat_5",
        7521,
        "Cooked giant crab meat",
        2,
        2,
        false,
    ),
    (
        "hundred_pirate_giant_crab_meat_4",
        7523,
        "Cooked giant crab meat",
        2,
        2,
        false,
    ),
    (
        "hundred_pirate_giant_crab_meat_3",
        7524,
        "Cooked giant crab meat",
        2,
        2,
        false,
    ),
    (
        "hundred_pirate_giant_crab_meat_2",
        7525,
        "Cooked giant crab meat",
        2,
        2,
        false,
    ),
    (
        "hundred_pirate_giant_crab_meat_1",
        7526,
        "Cooked giant crab meat",
        2,
        2,
        false,
    ),
];
pub fn native_consumables(
    objects: &Symbols,
    defs: &Definitions,
    foods: &BTreeMap<i32, Edible>,
    potions: &BTreeMap<i32, Potion>,
) -> Result<Rules, String> {
    let mut rules = Rules::default();
    for (&id, food) in foods {
        let def = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing native food {id}"))?;
        if id < 0
            || food.id != id
            || food.delay <= 0
            || def.dummy == 1
            || def.cert.template != 0
            || def.placeholder.template != 0
            || def.transform.template != 0
            || crate::food_operations::button_operation(def).is_none()
        {
            return Err(format!("Invalid native food clock definition {id}"));
        }
        rules.foods.insert(
            id,
            Policy {
                kind: Kind::Food,
                consume_delay: food.delay,
                combat_delay: 3,
                members: def.members,
            },
        );
    }
    for (&id, potion) in potions {
        let def = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing native potion {id}"))?;
        // Some ordinary dose definitions omit visible labels; keep existing dispatch.
        if id < 0
            || potion.id != id
            || foods.contains_key(&id)
            || def.stackable
            || def.dummy == 1
            || def.cert.template != 0
            || def.placeholder.template != 0
            || def.transform.template != 0
        {
            return Err(format!("Invalid native potion clock definition {id}"));
        }
        rules.potions.insert(
            id,
            Policy {
                kind: Kind::Potion,
                consume_delay: DRINK_DELAY,
                combat_delay: 0,
                members: def.members,
            },
        );
    }
    for &(symbol, id, name, consume, combat, combo) in SPECIAL {
        let Some(policy) = rules.foods.get_mut(&id) else {
            continue;
        };
        let def = &defs.items[&(id as u32)];
        if objects.get(symbol) != Some(&id)
            || def.name != name
            || !def.members
            || def.stackable
            || def.inventory_ops[if id == 2152 { 3 } else { 0 }].as_deref() != Some("Eat")
            || policy.consume_delay != consume
        {
            return Err(format!(
                "Unattested native consumable clock binding {symbol}"
            ));
        }
        policy.kind = if combo { Kind::ComboFood } else { Kind::Food };
        policy.combat_delay = combat;
    }
    Ok(rules)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (
        Symbols,
        Definitions,
        BTreeMap<i32, Edible>,
        BTreeMap<i32, Potion>,
    ) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut foods = BTreeMap::new();
        for &(symbol, id, name, consume, _, _) in SPECIAL {
            symbols.insert(symbol.into(), id);
            let mut def = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            def.name = name.into();
            def.members = true;
            def.inventory_ops[if id == 2152 { 3 } else { 0 }] = Some("Eat".into());
            defs.items.insert(id as u32, def);
            foods.insert(
                id,
                Edible {
                    id,
                    heal: 1,
                    delay: consume,
                    replacement: None,
                    message: "Eat".into(),
                },
            );
        }
        let mut def = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        def.inventory_ops[0] = Some("Drink".into());
        defs.items.insert(1, def);
        foods.insert(
            1,
            Edible {
                id: 1,
                heal: 1,
                delay: 3,
                replacement: None,
                message: "Drink".into(),
            },
        );
        let mut def = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        def.members = true;
        defs.items.insert(2, def);
        let potions = BTreeMap::from([(
            2,
            Potion {
                id: 2,
                next: 3,
                vial: 3,
                sip: "Sip".into(),
                remaining: "Left".into(),
                effects: vec![],
            },
        )]);
        (symbols, defs, foods, potions)
    }
    #[test]
    fn native_consumable_policies_classify_only_registered_target_bindings_and_member_flags() {
        let (s, d, mut f, p) = fixture();
        let rules = native_consumables(&s, &d, &f, &p).unwrap();
        assert_eq!(rules.foods.len(), 51);
        assert_eq!(rules.foods[&2152].kind, Kind::ComboFood);
        assert_eq!(rules.potions.len(), 1);
        assert_eq!(
            rules.foods[&2162],
            Policy {
                kind: Kind::ComboFood,
                consume_delay: 3,
                combat_delay: 2,
                members: true
            }
        );
        assert_eq!(
            rules.foods[&7521],
            Policy {
                kind: Kind::Food,
                consume_delay: 2,
                combat_delay: 2,
                members: true
            }
        );
        assert_eq!(rules.foods[&1].kind, Kind::Food);
        assert!(!rules.foods[&1].members);
        assert_eq!(
            rules.potions[&2],
            Policy {
                kind: Kind::Potion,
                consume_delay: 3,
                combat_delay: 0,
                members: true
            }
        );
        f.remove(&2162);
        f.remove(&2152);
        let rules = native_consumables(&s, &d, &f, &p).unwrap();
        assert!(!rules.foods.contains_key(&2162));
        assert!(!rules.foods.contains_key(&2152));
    }
    #[test]
    fn native_consumable_policy_refuses_conflicts_and_invalid_cache_or_timing_without_mutating_catalogs(
    ) {
        for mode in 0..9 {
            let (mut s, mut d, mut f, mut p) = fixture();
            match mode {
                0 => {
                    s.insert("king_worm".into(), 1);
                }
                1 => d.items.get_mut(&2162).unwrap().members = false,
                2 => d.items.get_mut(&2162).unwrap().inventory_ops[0] = Some("Drop".into()),
                3 => d.items.get_mut(&2162).unwrap().name = "Wrong".into(),
                4 => f.get_mut(&2162).unwrap().delay = 2,
                5 => d.items.get_mut(&2).unwrap().stackable = true,
                6 => {
                    p.insert(
                        1,
                        Potion {
                            id: 1,
                            ..p[&2].clone()
                        },
                    );
                }
                7 => d.items.get_mut(&2).unwrap().transform.template = 65534,
                _ => f.get_mut(&1).unwrap().id = 99,
            }
            let before = format!("{f:?}");
            assert!(native_consumables(&s, &d, &f, &p).is_err());
            assert_eq!(format!("{f:?}"), before);
        }
    }
}
