//! Cache-derived food operations, after the runtime validates held component/UI context.
use gielinor_cache::{definitions::ItemType, game::Definitions};
use gielinor_protocol::buttons::Button;
use gielinor_world::{food::Edible, simulation::Action};
use std::collections::BTreeMap;
/// The held component reserves operation1 for Use; cache options1..4 map to2..5.
pub(crate) fn button_operation(item: &ItemType) -> Option<u8> {
    item.inventory_ops
        .iter()
        .take(4)
        .position(|op| matches!(op.as_deref(), Some("Eat" | "Drink")))
        .map(|index| index as u8 + 2)
}
pub(crate) fn held_action(
    button: &Button,
    slot: usize,
    current: i32,
    foods: &BTreeMap<i32, Edible>,
    defs: &Definitions,
    native: bool,
) -> Option<Action> {
    if usize::try_from(button.slot).ok() != Some(slot) {
        return None;
    }
    let registered = |id: i32| {
        foods.contains_key(&id)
            && defs.items.get(&(id as u32)).and_then(button_operation) == Some(button.op)
    };
    let matches = if native {
        registered(current) || registered(button.object)
    } else {
        button.op == 2 && (foods.contains_key(&current) || foods.contains_key(&button.object))
    };
    // Keep the claim. The authoritative transaction must reject stale/wrong IDs.
    matches.then_some(Action::EatItemSlot {
        slot,
        item: button.object,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Definitions, BTreeMap<i32, Edible>) {
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
        for (id, index) in [(1, 0), (2152, 3)] {
            let mut d = ItemType::decode(&[0]).unwrap();
            d.inventory_ops[index] = Some("Eat".into());
            defs.items.insert(id as u32, d);
            foods.insert(
                id,
                Edible {
                    id,
                    heal: 1,
                    delay: 3,
                    replacement: None,
                    message: "Eat".into(),
                },
            );
        }
        (defs, foods)
    }
    fn packet(id: i32, op: u8) -> Button {
        let mut p = ((149u32 << 16) | 0).to_be_bytes().to_vec();
        p.extend(27u16.to_be_bytes());
        p.extend((id as u16).to_be_bytes());
        p.push(op);
        gielinor_protocol::buttons::button(&p).unwrap()
    }
    #[test]
    fn native_food_operation_selects_first_consume_within_four_cache_slots() {
        for index in 0..5 {
            let mut d = ItemType::decode(&[0]).unwrap();
            d.inventory_ops[index] = Some("Drink".into());
            assert_eq!(button_operation(&d), (index < 4).then_some(index as u8 + 2));
        }
        let mut d = ItemType::decode(&[0]).unwrap();
        d.inventory_ops[3] = Some("Eat".into());
        d.inventory_ops[0] = Some("Drink".into());
        assert_eq!(button_operation(&d), Some(2));
        d.inventory_ops = [None, None, None, None, None];
        assert_eq!(button_operation(&d), None);
    }
    #[test]
    fn native_food_first_and_fourth_decoded_buttons_require_correct_operation_and_preserve_stale_claim(
    ) {
        let (d, f) = fixture();
        for (id, want) in [(1, 2), (2152, 5)] {
            for op in 0..=7 {
                let a = held_action(&packet(id, op), 27, id, &f, &d, true);
                assert_eq!(a.is_some(), op == want);
                if let Some(Action::EatItemSlot { slot, item }) = a {
                    assert_eq!((slot, item), (27, id));
                }
            }
        }
        assert!(matches!(
            held_action(&packet(2152, 5), 27, 1, &f, &d, true),
            Some(Action::EatItemSlot {
                slot: 27,
                item: 2152
            })
        ));
        assert!(matches!(
            held_action(&packet(1, 2), 27, 2152, &f, &d, true),
            Some(Action::EatItemSlot { slot: 27, item: 1 })
        ));
    }
    #[test]
    fn native_food_missing_definition_unknown_identity_or_wrong_slot_does_not_create_an_action() {
        let (mut d, f) = fixture();
        d.items.remove(&2152);
        assert!(held_action(&packet(2152, 5), 27, 2152, &f, &d, true).is_none());
        assert!(held_action(&packet(999, 2), 27, 999, &f, &d, true).is_none());
        assert!(held_action(&packet(1, 2), 26, 1, &f, &d, true).is_none());
    }
    #[test]
    fn legacy_food_reference_mode_retains_first_operation_without_native_cache_routing() {
        let (mut d, f) = fixture();
        d.items.clear();
        assert!(matches!(
            held_action(&packet(1, 2), 27, 2152, &f, &d, false),
            Some(Action::EatItemSlot { slot: 27, item: 1 })
        ));
        assert!(held_action(&packet(2152, 5), 27, 2152, &f, &d, false).is_none());
    }
}
