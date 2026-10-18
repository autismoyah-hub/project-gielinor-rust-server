//! SmithingActions.smelt/resolveInvIngredient. Definition IDs and names are
//! supplied from the host's cache; recipes preserve Kotlin's linked-map order.
use crate::{
    inventory::{Catalog, Inventory},
    smithing_make::{self, Request},
    stats::Stats,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct ItemType {
    pub name: String,
    pub certificate: bool,
    pub placeholder: bool,
}
#[derive(Clone, Debug)]
pub struct Recipe {
    pub bar: i32,
    pub label: String,
    pub level: i32,
    pub xp: f64,
    pub ingredients: Vec<(i32, i32)>,
    pub impurity: bool,
    /// Requires trusted membership at admission and each delayed completion.
    pub members: bool,
    /// Queue delay for repeats; None retains the characterized legacy cadence.
    pub repeat_delay: Option<i32>,
}
#[derive(Debug)]
pub enum Selection {
    Denied(String),
    Start { request: Arc<Request>, count: i32 },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    MissingType(i32),
    ZeroIngredientAmount,
}
#[derive(Clone, Copy, Debug)]
pub struct Furnace {
    pub operations: u8,
    pub item_use: bool,
    /// Specific tutorial bindings override the generic ore content group.
    pub ore_ids: Option<[i32; 2]>,
    pub tutorial: Option<crate::tutorial_progression::Config>,
}
impl Furnace {
    pub fn accepts_ore(self, object: i32, generic: &std::collections::HashSet<i32>) -> bool {
        self.item_use
            && self
                .ore_ids
                .map_or_else(|| generic.contains(&object), |ids| ids.contains(&object))
    }
    pub fn supports(self, operation: u8) -> bool {
        (1..=5).contains(&operation) && self.operations & (1 << operation) != 0
    }
}

// Java String.equalsIgnoreCase compares Unicode code points using simple case
// conversion, without expanding a character into multiple characters. Rust's
// full mappings need an explicit single-character fallback for expansions.
fn upper(c: char) -> char {
    // CharacterData00.toUpperCase uses these simple mappings where Rust's
    // full uppercase expands the Greek prosgegrammeni into two code points.
    match c as u32 {
        0x1f80..=0x1f87 | 0x1f90..=0x1f97 | 0x1fa0..=0x1fa7 => {
            return char::from_u32(c as u32 + 8).unwrap()
        }
        0x1fb3 => return '\u{1fbc}',
        0x1fc3 => return '\u{1fcc}',
        0x1ff3 => return '\u{1ffc}',
        _ => (),
    }
    let mut chars = c.to_uppercase();
    let first = chars.next().unwrap();
    if chars.next().is_none() {
        first
    } else {
        c
    }
}
fn lower(c: char) -> char {
    if c == '\u{0130}' {
        return 'i';
    } // Character.toLowerCase, not full string mapping.
    let mut chars = c.to_lowercase();
    let first = chars.next().unwrap();
    if chars.next().is_none() {
        first
    } else {
        c
    }
}
fn same_name(a: &str, b: &str) -> bool {
    if a.encode_utf16().count() != b.encode_utf16().count() {
        return false;
    }
    let mut a = a.chars();
    let mut b = b.chars();
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (Some(a), Some(b))
                if a == b || upper(a) == upper(b) || lower(upper(a)) == lower(upper(b)) =>
            {
                ()
            }
            _ => return false,
        }
    }
}
pub fn resolve_ingredient(
    inv: &Inventory,
    catalog: &Catalog,
    types: &BTreeMap<i32, ItemType>,
    wanted: i32,
) -> Result<i32, Error> {
    let want = types.get(&wanted).ok_or(Error::MissingType(wanted))?;
    if smithing_make::total(inv, catalog, wanted) > 0 {
        return Ok(wanted);
    }
    for item in inv.slots().iter().flatten() {
        let ty = types.get(&item.id).ok_or(Error::MissingType(item.id))?;
        if !ty.certificate && !ty.placeholder && same_name(&ty.name, &want.name) {
            return Ok(item.id);
        }
    }
    Ok(wanted)
}
impl Recipe {
    pub fn select(
        &self,
        inv: &Inventory,
        catalog: &Catalog,
        types: &BTreeMap<i32, ItemType>,
        stats: &Stats,
        skill: u8,
        animation: u16,
        count: i32,
    ) -> Result<Selection, Error> {
        // The overload resolves all ingredients before the level check. Kotlin's
        // associate retains the first key position but replaces duplicate values.
        let mut consumed: Vec<(i32, i32)> = vec![];
        for &(wanted, amount) in &self.ingredients {
            let resolved = resolve_ingredient(inv, catalog, types, wanted)?;
            if let Some(previous) = consumed.iter_mut().find(|(id, _)| *id == resolved) {
                previous.1 = amount;
            } else {
                consumed.push((resolved, amount));
            }
        }
        if (stats.current(skill) as u8 as i32) < self.level {
            return Ok(Selection::Denied(format!(
                "You need a Smithing level of {} to smelt {} bars.",
                self.level,
                self.label.to_lowercase()
            )));
        }
        if !types.contains_key(&self.bar) {
            return Err(Error::MissingType(self.bar));
        }
        let request = Arc::new(Request {
            product: self.bar,
            produced: 1,
            consumed,
            xp: self.xp,
            animation,
            cycles: 5,
            impurity: self.impurity,
            members: self.members,
            repeat_delay: self.repeat_delay,
            required_level: self.level,
            required_tools: vec![],
        });
        if request
            .affordable_count(inv, catalog, count)
            .map_err(|_| Error::ZeroIngredientAmount)?
            <= 0
        {
            return Ok(Selection::Denied(format!(
                "You do not have enough ore to smelt {}.",
                self.label.to_lowercase()
            )));
        }
        Ok(Selection::Start { request, count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    fn types() -> BTreeMap<i32, ItemType> {
        [
            (436, "Copper ore"),
            (438, "Tin ore"),
            (2349, "Bronze bar"),
            (100, "COPPER ORE"),
            (101, "Copper ore"),
        ]
        .into_iter()
        .map(|(id, name)| {
            (
                id,
                ItemType {
                    name: name.into(),
                    certificate: id == 100,
                    placeholder: false,
                },
            )
        })
        .collect()
    }
    fn inv(ids: &[i32]) -> Inventory {
        let mut inv = Inventory::new(ids.len(), StackType::Normal, false);
        for (slot, &id) in ids.iter().enumerate() {
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
        inv
    }
    fn recipe() -> Recipe {
        Recipe {
            impurity: false,
            members: false,
            repeat_delay: None,
            bar: 2349,
            label: "Bronze".into(),
            level: 1,
            xp: 6.2,
            ingredients: vec![(436, 1), (438, 1)],
        }
    }
    #[test]
    fn exact_id_beats_first_named_stack_and_notes_are_not_substitutes() {
        let catalog = Catalog::default();
        let types = types();
        assert_eq!(
            resolve_ingredient(&inv(&[101, 436]), &catalog, &types, 436),
            Ok(436)
        );
        assert_eq!(
            resolve_ingredient(&inv(&[100, 101]), &catalog, &types, 436),
            Ok(101)
        );
        assert_eq!(
            resolve_ingredient(&inv(&[100]), &catalog, &types, 436),
            Ok(436)
        );
    }
    #[test]
    fn current_level_precedes_ore_failure_and_requested_count_is_not_affordability() {
        let mut recipe = recipe();
        recipe.level = 15;
        let mut stats = Stats::default();
        stats.set_base(13, 99);
        let catalog = Catalog::default();
        let types = types();
        assert!(
            matches!(recipe.select(&inv(&[]), &catalog, &types, &stats, 13, 899, 28).unwrap(), Selection::Denied(text) if text == "You need a Smithing level of 15 to smelt bronze bars.")
        );
        stats.set_current(13, 15);
        assert!(
            matches!(recipe.select(&inv(&[]), &catalog, &types, &stats, 13, 899, 28).unwrap(), Selection::Denied(text) if text == "You do not have enough ore to smelt bronze.")
        );
        let Selection::Start { request, count } = recipe
            .select(&inv(&[101, 438]), &catalog, &types, &stats, 13, 899, 28)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(request.consumed, vec![(101, 1), (438, 1)]);
        assert_eq!(count, 28);
    }
    #[test]
    fn aliased_ingredients_replace_values_without_reordering_keys() {
        let mut recipe = recipe();
        recipe.ingredients = vec![(436, 1), (438, 1), (101, 2)];
        let Selection::Start { request, .. } = recipe
            .select(
                &inv(&[101, 101, 438]),
                &Catalog::default(),
                &types(),
                &Stats::default(),
                13,
                899,
                1,
            )
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(request.consumed, vec![(101, 2), (438, 1)]);
    }
    #[test]
    fn names_use_simple_case_comparison_without_expanding_sharp_s() {
        assert!(same_name("COPPER ORE", "Copper ore"));
        assert!(same_name("İ", "i"));
        assert!(same_name("ı", "I"));
        assert!(same_name("Σ", "ς"));
        assert!(same_name("ß", "ẞ"));
        assert!(same_name("\u{1f80}", "\u{1f88}"));
        assert!(!same_name("ß", "SS"));
    }
}
