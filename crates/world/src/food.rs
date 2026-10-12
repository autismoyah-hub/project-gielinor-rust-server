//! EdibleFoods and Eating.bite, including this server's cake/pizza consumption behavior.
use crate::{
    inventory::{Catalog, Delete, Insert, Inventory},
    stat_ops::{self, Operation, Update},
    stats::Stats,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Food {
    pub symbol: &'static str,
    pub heal: i32,
    pub delay: i32,
    pub replacement: Option<&'static str>,
    pub message: Option<String>,
}
pub fn catalog() -> Vec<Food> {
    let bite = |symbol, heal, delay, replacement, message| Food {
        symbol,
        heal,
        delay,
        replacement,
        message,
    };
    let mut foods = vec![
        bite(
            "potato",
            1,
            3,
            None,
            Some("You eat the potato. Yuck!".into()),
        ),
        bite(
            "cabbage",
            1,
            3,
            None,
            Some("You eat the cabbage. Yuck!".into()),
        ),
    ];
    for (symbol, heal) in [
        ("banana", 2),
        ("king_worm", 2),
        ("peach", 8),
        ("ugthanki_kebab_bad", 19),
        ("bread", 5),
    ] {
        foods.push(bite(symbol, heal, 3, None, None));
    }
    for (whole, part, slice, heal, name) in [
        ("cake", "partial_cake", "cake_slice", 4, "cake"),
        (
            "chocolate_cake",
            "partial_chocolate_cake",
            "chocolate_slice",
            5,
            "chocolate cake",
        ),
    ] {
        // Kotlin's helpers supply chat/delays, but no replacement item for these entries.
        foods.push(bite(
            whole,
            heal,
            2,
            None,
            Some(format!("You eat part of the {name}.")),
        ));
        foods.push(bite(
            part,
            heal,
            2,
            None,
            Some(format!("You eat some more of the {name}.")),
        ));
        foods.push(bite(
            slice,
            heal,
            3,
            None,
            Some(format!("You eat the slice of {name}.")),
        ));
    }
    for (whole, half, heal, name) in [
        ("plain_pizza", "half_plain_pizza", 7, "pizza"),
        ("meat_pizza", "half_meat_pizza", 8, "meat pizza"),
        ("anchovie_pizza", "half_anchovie_pizza", 9, "anchovy pizza"),
        (
            "pineapple_pizza",
            "half_pineapple_pizza",
            11,
            "pineapple pizza",
        ),
        ("redberry_pie", "half_a_redberry_pie", 5, "redberry pie"),
        ("meat_pie", "half_a_meat_pie", 6, "meat pie"),
        ("apple_pie", "half_an_apple_pie", 7, "apple pie"),
    ] {
        foods.push(bite(
            whole,
            heal,
            1,
            None,
            Some(format!("You eat half of the {name}.")),
        ));
        foods.push(bite(
            half,
            heal,
            2,
            None,
            Some(format!("You eat the remaining {name}.")),
        ));
    }
    foods.push(bite(
        "bowl_mushroom_fried",
        5,
        3,
        Some("bowl_empty"),
        Some("You eat the fried mushrooms.".into()),
    ));
    foods.push(bite(
        "curry",
        19,
        3,
        Some("bowl_empty"),
        Some("You eat the curry.".into()),
    ));
    for (symbol, heal) in [
        ("potato_chilli+carne", 14),
        ("potato_mushroom+onion", 20),
        ("potato_cheese", 16),
        ("potato_tuna+sweetcorn", 22),
        ("cooked_chicken", 3),
        ("cooked_meat", 3),
        ("shrimp", 3),
        ("sardine", 4),
        ("herring", 5),
        ("mackerel", 6),
        ("trout", 7),
        ("cod", 7),
        ("pike", 8),
        ("salmon", 9),
        ("tuna", 10),
        ("lobster", 12),
        ("bass", 13),
        ("swordfish", 14),
        ("monkfish", 16),
        ("shark", 20),
        ("seaturtle", 21),
        ("mantaray", 22),
        ("dark_crab", 22),
        ("edible_seaweed", 4),
    ] {
        foods.push(bite(symbol, heal, 3, None, None));
    }
    foods
}
#[derive(Clone, Debug)]
pub struct Edible {
    pub id: i32,
    pub heal: i32,
    pub delay: i32,
    pub replacement: Option<i32>,
    pub message: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Animation(u16),
    Stat(Update),
    Message(String),
    Delay(i32),
}
/// Invoke from a protected inventory-op handler. The original consumes even at full HP,
/// ignores transaction failure, and awards no XP. Events preserve the script's ordering.
pub fn eat(
    food: &Edible,
    inventory: &mut Inventory,
    stats: &mut Stats,
    items: &Catalog,
    hitpoints: u8,
    animation: u16,
    mut emit: impl FnMut(Event),
) -> Result<bool, stat_ops::Error> {
    if !inventory
        .slots()
        .iter()
        .flatten()
        .any(|i| i.id == food.id && i.count > 0)
    {
        return Ok(false);
    }
    emit(Event::Animation(animation));
    let mut transaction = inventory.clone();
    let result = transaction
        .delete(items, Delete::new(food.id, 1))
        .and_then(|_| {
            if let Some(replacement) = food.replacement {
                transaction.insert(items, Insert::new(replacement, 1))
            } else {
                Ok(crate::inventory::Success {
                    requested: 1,
                    completed: 1,
                })
            }
        });
    if result.is_ok() {
        *inventory = transaction;
    }
    let update = stats.apply(hitpoints, Operation::Heal, food.heal, 0, true)?;
    emit(Event::Stat(update));
    emit(Event::Message(food.message.clone()));
    emit(Event::Delay(food.delay));
    Ok(true)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    #[test]
    fn catalog_preserves_diary_heals_bowls_and_source_cake_quirk() {
        let foods = catalog();
        assert_eq!(foods.len(), 53);
        let by_name: std::collections::BTreeMap<_, _> =
            foods.iter().map(|f| (f.symbol, f)).collect();
        assert_eq!(by_name.len(), 53);
        assert_eq!(by_name["cooked_chicken"].heal, 3);
        assert_eq!(by_name["curry"].replacement, Some("bowl_empty"));
        assert_eq!(by_name["cake"].replacement, None);
        assert_eq!(by_name["plain_pizza"].delay, 1);
        assert_eq!(by_name["shark"].heal, 20);
    }
    #[test]
    fn full_health_consumes_and_replacement_uses_first_free_slot() {
        let mut inventory = Inventory::new(3, StackType::Normal, false);
        inventory
            .restore_slot(
                2,
                Some(Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let food = Edible {
            id: 1,
            heal: 19,
            delay: 3,
            replacement: Some(2),
            message: "You eat the curry.".into(),
        };
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        let xp = stats.fine_xp(3);
        let mut events = vec![];
        assert_eq!(
            eat(
                &food,
                &mut inventory,
                &mut stats,
                &Catalog::default(),
                3,
                829,
                |e| events.push(e)
            ),
            Ok(true)
        );
        assert_eq!(inventory.get(0).unwrap().id, 2);
        assert!(inventory.get(2).is_none());
        assert_eq!(stats.current(3), 10);
        assert_eq!(stats.fine_xp(3), xp);
        assert_eq!(events.first(), Some(&Event::Animation(829)));
        assert_eq!(events.last(), Some(&Event::Delay(3)));
        assert_eq!(
            eat(
                &food,
                &mut inventory,
                &mut stats,
                &Catalog::default(),
                3,
                829,
                |_| panic!()
            ),
            Ok(false)
        );
    }
}
