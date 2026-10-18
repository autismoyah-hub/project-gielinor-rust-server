//! Worn hammer mode swaps use staged equipment transactions and existing hooks.
use super::{Definitions, Symbols};
use gielinor_world::worn_item_swap::Rule;
pub fn imcando_worn_swaps(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<i32, Rule>, String> {
    let held = super::imcando_held_swaps(objects, defs)?;
    for &id in held.keys() {
        let d = &defs.items[&(id as u32)];
        if d.wear[1..] != [-1, -1]
            || d.params.get(&453)
                != Some(&gielinor_cache::definitions::Param::String("Swap".into()))
        {
            return Err(format!("Invalid Imcando worn Swap definition {id}"));
        }
    }
    Ok(held
        .into_iter()
        .map(|(input, r)| {
            let from_slot = if input == 25644 { 3 } else { 5 };
            (
                input,
                Rule {
                    output: r.output,
                    from_slot,
                    to_slot: if from_slot == 3 { 5 } else { 3 },
                    button_operation: 2,
                },
            )
        })
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imcando_worn_swaps_validate_both_worn4_labels_and_hand_mapping() {
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
            raw.extend([0, 16, 13, wear, 37]);
            raw.extend(b"Swap\0\xf9\x01\x01\0\x01\xc5Swap\0\0");
            d.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        let o = [
            ("imcando_hammer".into(), 25644),
            ("imcando_hammer_offhand".into(), 29775),
        ]
        .into();
        let rules = imcando_worn_swaps(&o, &d).unwrap();
        assert_eq!((rules[&25644].from_slot, rules[&25644].to_slot), (3, 5));
        assert_eq!((rules[&29775].from_slot, rules[&29775].to_slot), (5, 3));
        d.items.get_mut(&29775).unwrap().params.remove(&453);
        assert!(imcando_worn_swaps(&o, &d).is_err());
    }
}
