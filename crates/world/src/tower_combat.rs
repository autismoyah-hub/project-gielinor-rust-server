//! FloorOneScript Modify hooks; resolved packed identities, independent of map bounds.
use std::collections::BTreeSet;
#[derive(Clone, Debug)]
pub struct Config {
    pub bosses: BTreeSet<u16>,
    pub chicken: u16,
    pub scimitars: BTreeSet<i32>,
    pub shortbows: BTreeSet<i32>,
    pub bronze_arrows: BTreeSet<i32>,
    pub skills: [u8; 3],
    pub bones: u16,
}

/// FloorOneRules.armedScimitar; canonical identity wins even without a definition.
pub fn scimitar(id: i32, name: Option<&str>) -> bool {
    id == 1321 || name.is_some_and(|s| crate::tower_chest::name_matches(s, "Bronze scimitar"))
}
pub fn shortbow(name: &str) -> bool {
    crate::tower_chest::name_matches(name, "Shortbow")
}
pub fn bronze_arrow(name: &str) -> bool {
    let chars: Vec<_> = name.chars().collect();
    chars
        .windows(12)
        .any(|s| crate::tower_chest::name_matches(&s.iter().collect::<String>(), "bronze arrow"))
}
