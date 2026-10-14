use crate::skill_multi::Presentation;
use gielinor_world::smelting::{ItemType, Recipe};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Config {
    pub presentation: Presentation,
    pub recipes: Vec<Recipe>,
    pub types: BTreeMap<i32, ItemType>,
    pub quantity_varp: u32,
    pub skill: u8,
    pub animation: u16,
}

/// Enriched item templates use zero for absence, as Kotlin ObjType does.
pub(crate) fn item_types(
    items: &BTreeMap<u32, gielinor_cache::definitions::ItemType>,
) -> BTreeMap<i32, ItemType> {
    items
        .iter()
        .map(|(&id, item)| {
            (
                id as i32,
                ItemType {
                    name: item.name.clone(),
                    certificate: item.cert.template != 0,
                    placeholder: item.placeholder.template != 0,
                },
            )
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoded_template_defaults_allow_real_ore_aliases_and_skip_notes_and_placeholders() {
        use gielinor_world::inventory::{Catalog, Inventory, Item, StackType};
        let named = |extra: &[u8]| {
            let mut bytes = vec![2];
            bytes.extend(b"Copper ore");
            bytes.push(0);
            bytes.extend(extra);
            bytes.push(0);
            gielinor_cache::definitions::ItemType::decode(&bytes).unwrap()
        };
        let definitions = BTreeMap::from([
            (1, named(&[])),
            (2, named(&[98, 3, 31])),
            (3, named(&[149, 0, 1])),
            (4, named(&[])),
        ]);
        let types = item_types(&definitions);
        assert!(!types[&1].certificate && !types[&1].placeholder);
        assert!(types[&2].certificate && types[&3].placeholder);
        let mut inv = Inventory::new(3, StackType::Normal, false);
        for (slot, id) in [2, 3, 4].into_iter().enumerate() {
            inv.restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        assert_eq!(
            gielinor_world::smelting::resolve_ingredient(&inv, &Catalog::default(), &types, 1)
                .unwrap(),
            4
        );
    }
}
